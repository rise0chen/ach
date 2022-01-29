use crate::*;

/// Disables specific interrupts and returns the previous settings
#[inline]
pub fn disable_mask(mask: u32) -> u32 {
    let prev = get_mask();
    unsafe { set_mask(prev & !mask) };
    prev
}

/// Enables specific interrupts and returns the previous setting
#[inline]
pub unsafe fn enable_mask(mask: u32) -> u32 {
    let prev = get_mask();
    set_mask(prev | mask);
    prev
}

/// Disables all interrupts and returns the previous settings
#[inline]
pub fn disable() -> u32 {
    disable_mask(MASK_ALL)
}

/// Enables all interrupts and returns the previous setting
#[inline]
pub unsafe fn enable() -> u32 {
    enable_mask(MASK_ALL)
}

/// Execute closure `f` in an interrupt-free context.
///
/// This as also known as a "critical section".
#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&bare_metal::CriticalSection) -> R,
{
    let cs = CriticalSection::new();
    f(&cs)
}
