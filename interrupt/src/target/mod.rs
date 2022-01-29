cfg_if::cfg_if! {
    if #[cfg(feature = "custom")] {
        #[path = "custom.rs"]
        mod target;
    } else if #[cfg(not(target_os = "none"))] {
        // Disable interrupt if using an OS
        #[path = "mock.rs"]
        mod target;
    } else if #[cfg(target_arch = "arm")] {
        #[path = "cortex_m.rs"]
        mod target;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        #[path = "riscv.rs"]
        mod target;
    } else if #[cfg(target_arch = "msp430")] {
        #[path = "msp430.rs"]
        mod target;
    } else if #[cfg(target_arch = "xtensa")] {
        #[path = "xtensa.rs"]
        mod target;
    } else {
        compile_error!("not support");
    }
}

pub use target::{get_mask, set_mask};
