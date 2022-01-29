use cortex_m::interrupt;
use cortex_m::register::primask;

/// Get specific interrupts and returns the current setting
#[inline]
pub fn get_mask() -> u32 {
    primask::read().is_active() as u32
}

/// Set specific interrupts
#[inline]
pub unsafe fn set_mask(mask: u32) {
    if mask == 0 {
        interrupt::disable();
    } else {
        interrupt::enable();
    }
}
