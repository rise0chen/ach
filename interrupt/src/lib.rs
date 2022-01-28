#![no_std]

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "none"))] {
        // Disable interrupt if using an OS
        mod mock;
        pub use crate::mock::*;
    } else if #[cfg(feature = "custom")] {
        mod custom;
        pub use crate::custom::*;
    } else if #[cfg(target_arch = "arm")] {
        mod cortex_m;
        pub use crate::cortex_m::*;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        pub use crate::riscv::*;
    } else if #[cfg(target_arch = "msp430")] {
        mod msp430;
        pub use crate::msp430::*;
    } else if #[cfg(target_arch = "xtensa")] {
        mod xtensa;
        pub use crate::xtensa::*;
    } else {
        compile_error!("not support");
    }
}

/// Disables specific interrupts and returns the previous settings
#[inline]
pub unsafe fn disable_mask(mask: u32) -> u32 {
    let prev = get_mask();
    set_mask(prev & !mask);
    prev
}

/// Enables specific interrupts and returns the previous setting
#[inline]
pub unsafe fn enable_mask(mask: u32) -> u32 {
    let prev = get_mask();
    set_mask(prev | mask);
    prev
}
