#![no_std]
use core::fmt;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::Ordering::Relaxed;
use interrupt::CriticalSection;
use util::*;

pub struct Once<T> {
    val: MaybeUninit<T>,
    state: AtomicMemoryState,
}
impl<T> Once<T> {
    pub const fn new() -> Self {
        Once {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryState::new(MemoryState::Uninitialized),
        }
    }
    pub const fn new_with(init: T) -> Self {
        Once {
            val: MaybeUninit::new(init),
            state: AtomicMemoryState::new(MemoryState::Initialized),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn is_initialized(&self) -> bool {
        let state = self.state.load(Relaxed);
        state.is_initialized()
    }
    pub fn take(&mut self) -> Option<T> {
        if self.is_initialized() {
            let ret = unsafe { ptr::read(self.ptr()) };
            self.state.store(MemoryState::Uninitialized.into(), Relaxed);
            Some(ret)
        } else {
            None
        }
    }
    pub fn into_inner(self) -> Option<T> {
        if self.is_initialized() {
            let ret = unsafe { ptr::read(self.ptr()) };
            Some(ret)
        } else {
            None
        }
    }
    pub fn try_get(&self) -> Result<&T, Error<()>> {
        let state = self.state.load(Relaxed);
        if state.is_initialized() {
            let ret = unsafe { self.val.assume_init_ref() };
            Ok(ret)
        } else {
            Err(Error {
                state,
                input: (),
                retry: state.is_transient(),
            })
        }
    }
    /// Notice: `Spin`
    pub fn get(&self) -> Option<&T> {
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
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            let ret = unsafe { self.val.assume_init_mut() };
            Some(ret)
        } else {
            None
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
    pub fn get_or_try_init(&self, value: T) -> Result<&T, T> {
        let _cs = CriticalSection::new();
        if let Err(_) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            Relaxed,
            Relaxed,
        ) {
            self.get().ok_or_else(|| value)
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized.into(), Relaxed);
            Ok(unsafe { self.val.assume_init_ref() })
        }
    }
    /// Notice: `Spin`
    pub fn get_or_init(&self, mut value: T) -> &T {
        loop {
            match self.get_or_try_init(value) {
                Ok(val) => return val,
                Err(err) => {
                    value = err;
                    spin_loop::spin();
                    continue;
                }
            }
        }
    }
}
impl<T: fmt::Debug> fmt::Debug for Once<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.get(), f)
    }
}
impl<T> Drop for Once<T> {
    fn drop(&mut self) {
        self.take();
    }
}
