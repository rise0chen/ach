#![no_std]
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering::{Relaxed, SeqCst};
use interrupt::CriticalSection;
use util::*;

pub struct Ref<'a, T>(&'a Cell<T>);
impl<'a, T> Ref<'a, T> {
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        self.0.ref_num()
    }
    /// Will remove the val of cell, after drop all Ref.
    pub fn remove(&self) {
        let _ = self.0.state.fetch_update(SeqCst, Relaxed, |mut x| {
            x.set_state(MemoryState::Erasing);
            Some(x)
        });
    }
    pub fn will_remove(&self) -> bool {
        self.0.state.load(SeqCst).state().is_erasing()
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
        let v: &T = self;
        fmt::Debug::fmt(&v, f)
    }
}
impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let _cs = CriticalSection::new();
        let old = match self.0.state.fetch_update(SeqCst, Relaxed, |mut x| {
            if x.ref_sub().is_ok() {
                Some(x)
            } else {
                None
            }
        }) {
            Ok(v) => v,
            Err(v) => v,
        };
        let will_drop = self.will_remove();
        if old.ref_num() == Ok(1) && will_drop {
            unsafe { ptr::drop_in_place(self.0.val.as_ptr() as *mut T) };
            self.0
                .state
                .store(MemoryState::Uninitialized.into(), SeqCst);
        }
    }
}

pub struct Cell<T> {
    val: MaybeUninit<T>,
    state: AtomicMemoryRefer,
}
impl<T> Default for Cell<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Cell<T> {
    pub const fn new() -> Self {
        Cell {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryRefer::new(MemoryRefer::new()),
        }
    }
    pub const fn new_with(init: T) -> Self {
        Cell {
            val: MaybeUninit::new(init),
            state: AtomicMemoryRefer::new(MemoryRefer::new()),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn is_initialized(&self) -> bool {
        let state = self.state.load(SeqCst);
        state.state().is_initialized()
    }
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        self.state.load(SeqCst).ref_num()
    }

    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the cell is refered or in critical section.
    pub fn try_take(&self) -> Result<Option<T>, Error<()>> {
        let _cs = CriticalSection::new();
        let refer = match self.state.fetch_update(SeqCst, Relaxed, |mut x| {
            let state = x.state();
            if state.is_initialized() || state.is_regaining() {
                if x.ref_num() == Ok(0) {
                    x.set_state(MemoryState::Erasing);
                } else {
                    x.set_state(MemoryState::Regaining);
                }
                Some(x)
            } else {
                None
            }
        }) {
            Ok(v) => v,
            Err(v) => v,
        };

        match refer.state() {
            MemoryState::Uninitialized => Ok(None),
            MemoryState::Initialized | MemoryState::Regaining => {
                if refer.ref_num() == Ok(0) {
                    let ret = unsafe { ptr::read(self.ptr()) };
                    self.state.store(MemoryState::Uninitialized.into(), SeqCst);
                    Ok(Some(ret))
                } else {
                    Err(Error {
                        state: MemoryState::Regaining,
                        input: (),
                        retry: true,
                    })
                }
            }
            state => Err(Error {
                state,
                input: (),
                retry: state.is_transient(),
            }),
        }
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err is unreachable.
    ///
    /// Notice: `Spin`
    pub fn take(&self) -> Result<Option<T>, Error<()>> {
        retry(|_| self.try_take(), ())
    }

    /// # Safety
    /// Calling this when the content is not yet fully initialized causes undefined behavior: it is up to the caller to guarantee that the MaybeUninit<T> really is in an initialized state.
    pub unsafe fn peek(&self) -> &T {
        self.val.assume_init_ref()
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized, in operation or in critical section.
    pub fn try_get(&self) -> Result<Ref<'_, T>, Error<()>> {
        if let Err(state) = self.state.fetch_update(SeqCst, Relaxed, |mut x| {
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
                retry: state.is_initializing(),
            })
        } else {
            Ok(Ref(self))
        }
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized.
    ///
    /// Notice: `Spin`
    pub fn get(&self) -> Result<Ref<'_, T>, Error<()>> {
        retry(|_| self.try_get(), ())
    }

    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is refered, initialized or in critical section.
    pub fn try_set(&self, value: T) -> Result<(), Error<T>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            SeqCst,
            SeqCst,
        ) {
            let state = state.state();
            Err(Error {
                state,
                input: value,
                retry: state.is_erasing(),
            })
        } else {
            unsafe { ptr::write(self.ptr(), value) };
            self.state.store(MemoryState::Initialized.into(), SeqCst);
            Ok(())
        }
    }
    /// Sets the value of the Cell to the argument value.
    ///
    /// Returns Err if the value is refered, initialized.
    ///
    /// Notice: `Spin`
    pub fn set(&self, value: T) -> Result<(), Error<T>> {
        retry(|v| self.try_set(v), value)
    }

    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the value is refered or in critical section.
    pub fn try_replace(&self, value: T) -> Result<Option<T>, Error<T>> {
        let _cs = CriticalSection::new();
        let refer = match self.state.fetch_update(SeqCst, Relaxed, |mut x| {
            let state = x.state();
            if state.is_initialized() || state.is_regaining() {
                if x.ref_num() == Ok(0) {
                    x.set_state(MemoryState::Initializing);
                } else {
                    x.set_state(MemoryState::Regaining);
                }
                Some(x)
            } else if state.is_uninitialized() {
                x.set_state(MemoryState::Initializing);
                Some(x)
            } else {
                None
            }
        }) {
            Ok(v) => v,
            Err(v) => v,
        };

        match refer.state() {
            MemoryState::Uninitialized => {
                unsafe { ptr::write(self.ptr(), value) };
                self.state.store(MemoryState::Initialized.into(), SeqCst);
                Ok(None)
            }
            MemoryState::Initialized | MemoryState::Regaining => {
                if refer.ref_num() == Ok(0) {
                    let ret = unsafe { ptr::read(self.ptr()) };
                    unsafe { ptr::write(self.ptr(), value) };
                    self.state.store(MemoryState::Initialized.into(), SeqCst);
                    Ok(Some(ret))
                } else {
                    Err(Error {
                        state: MemoryState::Regaining,
                        input: value,
                        retry: true,
                    })
                }
            }
            state => Err(Error {
                state,
                input: value,
                retry: state.is_transient(),
            }),
        }
    }
    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err is unreachable.
    ///
    /// Notice: `Spin`
    pub fn replace(&self, value: T) -> Result<Option<T>, Error<T>> {
        retry(|v| self.try_replace(v), value)
    }

    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is in critical section.
    pub fn get_or_try_init(&self, value: T) -> Result<Ref<'_, T>, Error<T>> {
        let ret = self.try_set(value);
        if let Ok(v) = self.try_get() {
            Ok(v)
        } else {
            Err(ret.unwrap_err())
        }
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Notice: `Spin`
    pub fn get_or_init(&self, value: T) -> Ref<'_, T> {
        retry(|v| self.get_or_try_init(v), value).unwrap()
    }
}
impl<T: fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.try_get().ok();
        fmt::Debug::fmt(&v, f)
    }
}
impl<T> Drop for Cell<T> {
    fn drop(&mut self) {
        let _ = self.take();
    }
}
