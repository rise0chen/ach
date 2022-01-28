#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub fn spin() {
    #[cfg(not(feature = "std"))]
    core::hint::spin_loop();
    #[cfg(feature = "std")]
    std::thread::yield_now();
}
