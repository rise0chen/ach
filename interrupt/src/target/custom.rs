/// Register interrupt function on custom targets.
///
/// # Usage
///
/// To register the function, we first depend on `interrupt` in `Cargo.toml`:
/// ```toml
/// [dependencies]
/// interrupt = { version = "0.1", features = ["custom"] }
/// ```
///
/// Then, we register the function in `src/main.rs`:
/// ```rust
/// fn interrupt_get_mask() -> u32 {
///     register.read()
/// }
/// fn interrupt_set_mask(mask: u32) {
///     register.write(mask)
/// }
/// register_interrupt!(interrupt_get_mask, interrupt_set_mask);
/// ```
///
/// # Addition
/// You can also register `fn rust_interrupt_get() -> u32;` and `fn rust_interrupt_set(mask: u32);`
/// in the Static-link Library or Dynamic-link Library.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "custom")))]
macro_rules! register_interrupt {
    ($get:path, $set:path) => {
        #[no_mangle]
        extern "C" fn rust_interrupt_get() -> u32 {
            unsafe { $get() }
        }
        #[no_mangle]
        extern "C" fn rust_interrupt_set(mask: u32) {
            let _ = unsafe { $set(mask) };
        }
    };
}

extern "C" {
    fn rust_interrupt_get() -> u32;
    fn rust_interrupt_set(mask: u32);
}

/// Get specific interrupts and returns the current setting
#[inline]
pub fn get_mask() -> u32 {
    unsafe { rust_interrupt_get() }
}

/// Set specific interrupts
#[inline]
pub unsafe fn set_mask(mask: u32) {
    rust_interrupt_set(mask)
}
