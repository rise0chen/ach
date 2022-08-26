#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod heap;
pub mod heapless;
