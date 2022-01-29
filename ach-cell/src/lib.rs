#![no_std]
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use interrupt::CriticalSection;
use util::*;

pub struct Ref<'a, T>(&'a Cell<T>);
impl<'a, T> Ref<'a, T> {
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        self.0.ref_num()
    }
    pub fn remove(self) {
        self.0.will_drop.store(true, Relaxed);
    }
}
impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.val.assume_init_ref() }
    }
}
impl<'a, T: fmt::Debug> fmt::Debug for Ref<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}
impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let _cs = CriticalSection::new();
        let will_drop = self.0.will_drop.load(Relaxed);
        let old = self.0.state.fetch_update(Relaxed, Relaxed, |mut x| {
            if x == MemoryRefer::REF1 {
                if will_drop {
                    return Some(MemoryState::Erasing.into());
                }
            }
            if x.ref_sub().is_ok() {
                Some(x)
            } else {
                None
            }
        });
        match old {
            Ok(MemoryRefer::REF1) => {
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
    state: AtomicMemoryRefer,
    will_drop: AtomicBool,
}
impl<T> Cell<T> {
    pub const fn new() -> Self {
        Cell {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryRefer::new(MemoryRefer::UNINITIALIZED),
            will_drop: AtomicBool::new(false),
        }
    }
    pub const fn new_with(init: T) -> Self {
        Cell {
            val: MaybeUninit::new(init),
            state: AtomicMemoryRefer::new(MemoryRefer::INITIALIZED),
            will_drop: AtomicBool::new(false),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn is_initialized(&self) -> bool {
        let state = self.state.load(Relaxed);
        state.can_refer()
    }
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        self.state.load(Relaxed).ref_num()
    }
    pub fn try_take(&self) -> Result<T, Error<()>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.fetch_update(Relaxed, Relaxed, |x| {
            if x.ref_num() == Ok(0) {
                Some(MemoryState::Erasing.into())
            } else {
                None
            }
        }) {
            let state = state.state();
            Err(Error {
                state,
                input: (),
                retry: state.is_transient(),
            })
        } else {
            let ret = unsafe { ptr::read(self.ptr()) };
            self.will_drop.store(false, Relaxed);
            self.state.store(MemoryState::Uninitialized.into(), Relaxed);
            Ok(ret)
        }
    }
    /// Notice: `Spin`
    pub fn take(&self) -> Option<T> {
        loop {
            match self.try_take() {
                Ok(val) => return Some(val),
                Err(err) if err.retry => {
                    spin_loop::spin();
                    continue;
                }
                Err(_) => return None,
            }
        }
    }
    pub fn try_get(&self) -> Result<Ref<T>, Error<()>> {
        if self.will_drop.load(Relaxed) {
            return Err(Error {
                state: MemoryState::Erasing,
                input: (),
                retry: false,
            });
        }
        if let Err(state) = self.state.fetch_update(Relaxed, Relaxed, |mut x| {
            if x.ref_add().is_ok() {
                Some(x)
            } else {
                None
            }
        }) {
            let state = state.state();
            Err(Error {
                state,
                input: (),
                retry: state.is_transient(),
            })
        } else {
            Ok(Ref(self))
        }
    }
    /// Notice: `Spin`
    pub fn get(&self) -> Option<Ref<T>> {
        loop {
            match self.try_get() {
                Ok(val) => return Some(val),
                Err(err) if err.retry => {
                    spin_loop::spin();
                    continue;
                }
                Err(_) => return None,
            }
        }
    }
    pub fn try_set(&self, value: T) -> Result<(), Error<T>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            Relaxed,
            Relaxed,
        ) {
            let state = state.state();
            Err(Error {
                state,
                input: value,
                retry: state.is_transient(),
            })
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized.into(), Relaxed);
            Ok(())
        }
    }
    /// Notice: `Spin`
    pub fn set(&self, mut value: T) -> Result<(), T> {
        loop {
            match self.try_set(value) {
                Ok(val) => return Ok(val),
                Err(err) if err.retry => {
                    value = err.input;
                    spin_loop::spin();
                    continue;
                }
                Err(err) => return Err(err.input),
            }
        }
    }
    pub fn try_swap(&self, value: T) -> Result<Option<T>, Error<T>> {
        let _cs = CriticalSection::new();
        match self.state.fetch_update(Relaxed, Relaxed, |x| {
            if x.state().is_uninitialized() {
                Some(MemoryState::Initializing.into())
            } else if x.ref_num() == Ok(0) {
                Some(MemoryState::Erasing.into())
            } else {
                None
            }
        }) {
            Ok(state) => {
                let ret = if state.state().is_uninitialized() {
                    None
                } else {
                    Some(unsafe { ptr::read(self.ptr()) })
                };
                self.will_drop.store(false, Relaxed);
                unsafe { ptr::write(self.ptr(), value) };
                self.state.store(MemoryState::Initialized.into(), Relaxed);
                Ok(ret)
            }
            Err(state) => {
                let state = state.state();
                Err(Error {
                    state,
                    input: value,
                    retry: state.is_transient(),
                })
            }
        }
    }
    /// Notice: `Spin`
    pub fn swap(&self, mut value: T) -> Result<Option<T>, T> {
        loop {
            match self.try_swap(value) {
                Ok(val) => return Ok(val),
                Err(err) if err.retry => {
                    value = err.input;
                    spin_loop::spin();
                    continue;
                }
                Err(err) => return Err(err.input),
            }
        }
    }
    pub fn get_or_try_init(&self, value: T) -> Result<Ref<T>, Error<T>> {
        let _cs = CriticalSection::new();
        if let Err(_) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            Relaxed,
            Relaxed,
        ) {
            self.try_get().map_err(
                |Error {
                     state,
                     input: _,
                     retry,
                 }| Error {
                    state,
                    input: value,
                    retry,
                },
            )
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryRefer::REF1, Relaxed);
            Ok(Ref(self))
        }
    }
    /// Notice: `Spin`
    pub fn get_or_init(&self, mut value: T) -> Ref<T> {
        loop {
            match self.get_or_try_init(value) {
                Ok(val) => return val,
                Err(err) if err.retry => {
                    value = err.input;
                    spin_loop::spin();
                    continue;
                }
                Err(_) => unreachable!(),
            }
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
