#![no_std]
use ach_once::Once;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

pub struct Lazy<T, F = fn() -> T> {
    val: Once<T>,
    has_val: AtomicBool,
    init: MaybeUninit<F>,
}
impl<T, F> Lazy<T, F> {
    pub const fn new(f: F) -> Lazy<T, F> {
        Lazy {
            val: Once::new(),
            has_val: AtomicBool::new(true),
            init: MaybeUninit::new(f),
        }
    }
}
impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub fn force(this: &Lazy<T, F>) -> &T {
        if this
            .has_val
            .compare_exchange(true, false, Relaxed, Relaxed)
            .is_ok()
        {
            let val = unsafe { ptr::read(this.init.as_ptr()) };
            let value = val();
            this.val.get_or_init(value)
        } else {
            this.val
                .get()
                .expect("Lazy instance has previously been poisoned")
        }
    }
}
impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        Self::force(self)
    }
}
impl<T, F> Drop for Lazy<T, F> {
    fn drop(&mut self) {
        if self
            .has_val
            .compare_exchange(true, false, Relaxed, Relaxed)
            .is_ok()
        {
            unsafe { ptr::drop_in_place(self.init.as_mut_ptr()) };
        }
    }
}
