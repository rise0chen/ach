#![no_std]
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use util::*;

pub struct Peek<'a, T>(&'a Cell<T>);
impl<'a, T> Peek<'a, T> {
    pub fn peek_num(&self) -> Result<usize, MemoryState> {
        self.0.peek_num()
    }
    pub fn remove(self) {
        self.0.will_drop.store(true, Relaxed);
    }
}
impl<'a, T> Deref for Peek<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.val.assume_init_ref() }
    }
}
impl<'a, T: fmt::Debug> fmt::Debug for Peek<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}
impl<'a, T> Drop for Peek<'a, T> {
    fn drop(&mut self) {
        let will_drop = self.0.will_drop.load(Relaxed);
        let old = self.0.state.fetch_update(Relaxed, Relaxed, |mut x| {
            if x == MemoryPeek::PEEK1 {
                if will_drop {
                    return Some(MemoryState::Erasing.into());
                }
            }
            if x.peek_sub().is_ok() {
                Some(x)
            } else {
                None
            }
        });
        match old {
            Ok(MemoryPeek::PEEK1) => {
                if will_drop {
                    unsafe { ptr::drop_in_place(self.0.val.as_ptr() as *mut T) };
                    self.0.will_drop.store(false, Relaxed);
                    self.0
                        .state
                        .store(MemoryState::Uninitialized.into(), Relaxed);
                }
            }
            _ => {}
        }
    }
}

pub struct Cell<T> {
    val: MaybeUninit<T>,
    state: AtomicMemoryPeek,
    will_drop: AtomicBool,
}
impl<T> Cell<T> {
    pub const fn new() -> Self {
        Cell {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryPeek::ZERO,
            will_drop: AtomicBool::new(false),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn is_initialized(&self) -> bool {
        let state = self.state.load(Relaxed);
        state.is_peekable()
    }
    pub fn peek_num(&self) -> Result<usize, MemoryState> {
        self.state.load(Relaxed).peek_num()
    }
    pub fn take(&self) -> Option<T> {
        if let Err(_) = self.state.fetch_update(Relaxed, Relaxed, |x| {
            if x.peek_num() == Ok(0) {
                Some(MemoryState::Erasing.into())
            } else {
                None
            }
        }) {
            None
        } else {
            let ret = Some(unsafe { ptr::read(self.ptr()) });
            self.will_drop.store(false,Relaxed);
            self.state.store(MemoryState::Uninitialized.into(), Relaxed);
            ret
        }
    }
    pub fn get(&self) -> Option<Peek<T>> {
        if self.will_drop.load(Relaxed) {
            return None;
        }
        if let Err(_) = self.state.fetch_update(Relaxed, Relaxed, |mut x| {
            if x.peek_add().is_ok() {
                Some(x)
            } else {
                None
            }
        }) {
            None
        } else {
            Some(Peek(self))
        }
    }
    pub fn set(&self, value: T) -> Result<(), T> {
        if let Err(_) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            Relaxed,
            Relaxed,
        ) {
            Err(value)
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized.into(), Relaxed);
            Ok(())
        }
    }
    pub fn get_or_try_init(&self, value: T) -> Result<Peek<T>, T> {
        if let Err(_) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            Relaxed,
            Relaxed,
        ) {
            self.get().ok_or_else(|| value)
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryPeek::PEEK1, Relaxed);
            Ok(Peek(self))
        }
    }
    /// Notice: Maybe spin
    pub fn get_or_init(&self, mut value: T) -> Peek<T> {
        loop {
            match self.get_or_try_init(value) {
                Ok(peek) => return peek,
                Err(val) => value = val,
            }
            spin();
        }
    }
}
impl<T: fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.get(), f)
    }
}
impl<T> Drop for Cell<T> {
    fn drop(&mut self) {
        self.take();
    }
}
