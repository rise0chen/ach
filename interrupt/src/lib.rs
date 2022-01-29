#![no_std]

mod critical_section;
mod ext;
mod target;

pub use bare_metal::Mutex;
pub use critical_section::CriticalSection;
pub use ext::*;
pub use target::{get_mask, set_mask};

pub const MASK_ALL: u32 = u32::MAX;
