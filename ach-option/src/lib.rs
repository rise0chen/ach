#![no_std]
use core::fmt;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use interrupt::CriticalSection;
use util::*;

pub struct AchOption<T> {
    val: MaybeUninit<T>,
    state: AtomicMemoryState,
}
impl<T> Default for AchOption<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> AchOption<T> {
    pub const fn new() -> Self {
        Self {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryState::new(MemoryState::Uninitialized),
        }
    }
    pub const fn new_with(init: T) -> Self {
        Self {
            val: MaybeUninit::new(init),
            state: AtomicMemoryState::new(MemoryState::Initialized),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn into_inner(self) -> Option<T> {
        self.take()
    }
    pub fn is_some(&self) -> bool {
        let state = self.state.load(SeqCst);
        state.is_initialized()
    }
    pub fn is_none(&self) -> bool {
        let state = self.state.load(SeqCst);
        state.is_uninitialized()
    }

    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the cell is in critical section.
    pub fn try_take(&self) -> Result<Option<T>, Error<()>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.fetch_update(SeqCst, Relaxed, |x| {
            if x.is_initialized() {
                Some(MemoryState::Erasing)
            } else {
                None
            }
        }) {
            if state.is_uninitialized() {
                Ok(None)
            } else {
                Err(Error {
                    state,
                    input: (),
                    retry: state.is_transient(),
                })
            }
        } else {
            let ret = unsafe { ptr::read(self.ptr()) };
            self.state.store(MemoryState::Uninitialized, SeqCst);
            Ok(Some(ret))
        }
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Notice: `Spin`
    pub fn take(&self) -> Option<T> {
        unwrap(|_| self.try_take(), ())
    }

    /// Sets the value of the Option to the argument value.
    ///
    /// Returns Err if the value is initialized or in critical section.
    pub fn try_set(&self, value: T) -> Result<(), Error<T>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.compare_exchange(
            MemoryState::Uninitialized,
            MemoryState::Initializing,
            SeqCst,
            Relaxed,
        ) {
            Err(Error {
                state,
                input: value,
                retry: state.is_erasing(),
            })
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized, SeqCst);
            Ok(())
        }
    }
    /// Sets the value of the Option to the argument value.
    ///
    /// Returns Err if the value is initialized.
    /// Notice: `Spin`
    pub fn set(&self, value: T) -> Result<(), Error<T>> {
        retry(|val| self.try_set(val), value)
    }

    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the value is in critical section.
    pub fn try_replace(&self, value: T) -> Result<Option<T>, Error<T>> {
        let _cs = CriticalSection::new();
        match self.state.fetch_update(SeqCst, Relaxed, |x| {
            if x.is_uninitialized() || x.is_initialized() {
                Some(MemoryState::Initializing)
            } else {
                None
            }
        }) {
            Ok(state) => {
                let ret = if state.is_uninitialized() {
                    None
                } else {
                    Some(unsafe { ptr::read(self.ptr()) })
                };
                unsafe { ptr::write(self.ptr(), value) };
                self.state.store(MemoryState::Initialized, SeqCst);
                Ok(ret)
            }
            Err(state) => Err(Error {
                state,
                input: value,
                retry: state.is_transient(),
            }),
        }
    }
    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Notice: `Spin`
    pub fn replace(&self, value: T) -> Option<T> {
        unwrap(|val| self.try_replace(val), value)
    }
}
impl<T: fmt::Debug> fmt::Debug for AchOption<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = if self.is_some() {
            Some(unsafe { self.val.assume_init_ref() })
        } else {
            None
        };
        fmt::Debug::fmt(&v, f)
    }
}
impl<T> Drop for AchOption<T> {
    fn drop(&mut self) {
        let _ = self.take();
    }
}
