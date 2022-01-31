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

    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized or in critical section.
    pub fn try_get(&self) -> Result<&T, Error<()>> {
        let state = self.state.load(Relaxed);
        if state.is_initialized() {
            let ret = unsafe { self.val.assume_init_ref() };
            Ok(ret)
        } else {
            Err(Error {
                state,
                input: (),
                retry: state.is_initializing(),
            })
        }
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized.
    ///
    /// Notice: `Spin`
    pub fn get(&self) -> Result<&T, Error<()>> {
        retry(|_| self.try_get(), ())
    }
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            let ret = unsafe { self.val.assume_init_mut() };
            Some(ret)
        } else {
            None
        }
    }

    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is initialized or in critical section.
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
                retry: state.is_erasing(),
            })
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized.into(), Relaxed);
            Ok(())
        }
    }
    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is initialized.
    /// 
    /// Notice: `Spin`
    pub fn set(&self,  value: T) -> Result<(), Error<T>> {
        retry(|val|self.try_set(val), value)
    }

    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is in critical section.
    pub fn get_or_try_init(&self, value: T) -> Result<&T, Error<T>> {
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
            self.state.store(MemoryState::Initialized.into(), Relaxed);
            Ok(unsafe { self.val.assume_init_ref() })
        }
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Notice: `Spin`
    pub fn get_or_init(&self, mut value: T) -> &T {
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
