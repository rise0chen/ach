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
        self.0.set_op(Op::Remove);
    }
    pub fn will_remove(&self) -> bool {
        self.0.op.load(SeqCst).op() == Op::Remove
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
        let v: &T = &*self;
        fmt::Debug::fmt(&v, f)
    }
}
impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        let _cs = CriticalSection::new();
        let will_drop = self.will_remove();
        let old = self.0.state.fetch_update(SeqCst, Relaxed, |mut x| {
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
                    self.0.finish_op();
                    self.0
                        .state
                        .store(MemoryState::Uninitialized.into(), SeqCst);
                }
            }
            _ => {}
        }
    }
}

pub struct Cell<T> {
    val: MaybeUninit<T>,
    state: AtomicMemoryRefer,
    op: AtomicMemoryOp,
}
impl<T> Cell<T> {
    pub const fn new() -> Self {
        Cell {
            val: MaybeUninit::uninit(),
            state: AtomicMemoryRefer::new(MemoryRefer::UNINITIALIZED),
            op: AtomicMemoryOp::new(MemoryOp::new()),
        }
    }
    pub const fn new_with(init: T) -> Self {
        Cell {
            val: MaybeUninit::new(init),
            state: AtomicMemoryRefer::new(MemoryRefer::INITIALIZED),
            op: AtomicMemoryOp::new(MemoryOp::new()),
        }
    }
    fn ptr(&self) -> *mut T {
        self.val.as_ptr() as *mut T
    }
    pub fn is_initialized(&self) -> bool {
        let state = self.state.load(SeqCst);
        state.can_refer()
    }
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        self.state.load(SeqCst).ref_num()
    }
    pub fn set_op(&self, next: Op) -> u16 {
        let version = self.op.fetch_update(SeqCst, Relaxed, |mut op| {
            op.set_op(next);
            Some(op)
        });
        version.unwrap().set_op(next)
    }
    pub fn finish_op(&self) {
        let _ = self.op.fetch_update(SeqCst, Relaxed, |mut op| {
            op.finish();
            Some(op)
        });
    }

    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the cell is refered or in critical section.
    pub fn try_take(&self) -> Result<Option<T>, Error<()>> {
        let _cs = CriticalSection::new();
        if let Err(state) = self.state.fetch_update(SeqCst, Relaxed, |x| {
            if x.ref_num() == Ok(0) {
                Some(MemoryState::Erasing.into())
            } else {
                None
            }
        }) {
            let state = state.state();
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
            self.state.store(MemoryState::Uninitialized.into(), SeqCst);
            Ok(Some(ret))
        }
    }
    /// Takes ownership of the current value, leaving the cell uninitialized.
    ///
    /// Returns Err if the operation is outdated.
    ///
    /// Notice: `Spin`
    pub fn take(&self) -> Result<Option<T>, Error<()>> {
        let version = self.set_op(Op::Take);
        while self.op.load(Relaxed).next_version() == version {
            if let Ok(val) = self.try_take() {
                self.finish_op();
                return Ok(val);
            }
            spin_loop::spin();
        }
        self.finish_op();
        Err(Error::new(()))
    }

    pub unsafe fn peek(&self) -> &T {
        self.val.assume_init_ref()
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is uninitialized, in operation or in critical section.
    pub fn try_get(&self) -> Result<Ref<T>, Error<()>> {
        if !self.op.load(Relaxed).is_finished() {
            return Err(Error {
                state: MemoryState::Unknown,
                input: (),
                retry: true,
            });
        }
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
    pub fn get(&self) -> Result<Ref<T>, Error<()>> {
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
    /// Returns Err if the value is refered, initialized or outdated.
    ///
    /// Notice: `Spin`
    pub fn set(&self, mut value: T) -> Result<(), Error<T>> {
        let version = self.set_op(Op::Write);
        while self.op.load(Relaxed).next_version() == version {
            if let Err(err) = self.try_set(value) {
                if !err.retry {
                    self.finish_op();
                    return Err(err);
                } else {
                    value = err.input;
                }
            } else {
                self.finish_op();
                return Ok(());
            }
            spin_loop::spin();
        }
        self.finish_op();
        Err(Error::new(value))
    }

    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the value is refered or in critical section.
    pub fn try_replace(&self, value: T) -> Result<Option<T>, Error<T>> {
        let _cs = CriticalSection::new();
        match self.state.fetch_update(SeqCst, Relaxed, |x| {
            if x.state().is_uninitialized() || x.ref_num() == Ok(0) {
                Some(MemoryState::Initializing.into())
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
                unsafe { ptr::write(self.ptr(), value) };
                self.state.store(MemoryState::Initialized.into(), SeqCst);
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
    /// Replaces the contained value with value, and returns the old contained value.
    ///
    /// Returns Err if the operation is outdated.
    ///
    /// Notice: `Spin`
    pub fn replace(&self, mut value: T) -> Result<Option<T>, Error<T>> {
        let version = self.set_op(Op::Replace);
        while self.op.load(Relaxed).next_version() == version {
            match self.try_replace(value) {
                Ok(val) => {
                    self.finish_op();
                    return Ok(val);
                }
                Err(err) => {
                    value = err.input;
                }
            }
            spin_loop::spin();
        }
        self.finish_op();
        Err(Error::new(value))
    }

    /// Tries to get a reference to the value of the Cell.
    ///
    /// Returns Err if the cell is in critical section.
    pub fn get_or_try_init(&self, value: T) -> Result<Ref<T>, Error<T>> {
        let _cs = CriticalSection::new();
        if let Err(_) = self.state.compare_exchange(
            MemoryState::Uninitialized.into(),
            MemoryState::Initializing.into(),
            SeqCst,
            SeqCst,
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
            self.state.store(MemoryRefer::REF1, SeqCst);
            Ok(Ref(self))
        }
    }
    /// Tries to get a reference to the value of the Cell.
    ///
    /// Notice: `Spin`
    pub fn get_or_init(&self, mut value: T) -> Ref<T> {
        loop {
            match self.get_or_try_init(value) {
                Ok(val) => return val,
                Err(err) => {
                    value = err.input;
                    spin_loop::spin();
                    continue;
                }
            }
        }
    }
}
impl<'a, T: fmt::Debug> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = if let Ok(v) = self.try_get() {
            Some(v)
        } else {
            None
        };
        fmt::Debug::fmt(&v, f)
    }
}
impl<T> Drop for Cell<T> {
    fn drop(&mut self) {
        let _ = self.take();
    }
}
