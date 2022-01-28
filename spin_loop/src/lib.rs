#![no_std]

#[cfg(not(target_os = "none"))]
extern crate std;

pub fn spin() {
    #[cfg(target_os = "none")]
    core::hint::spin_loop();
    #[cfg(not(target_os = "none"))]
    std::thread::yield_now();
}
