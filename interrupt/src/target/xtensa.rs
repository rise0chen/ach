use xtensa_lx::interrupt;

/// Get specific interrupts and returns the current setting
#[inline]
pub fn get_mask() -> u32 {
    interrupt::get_mask()
}

/// Set specific interrupts
#[inline]
pub unsafe fn set_mask(mask: u32) {
    interrupt::set_mask(mask);
}
