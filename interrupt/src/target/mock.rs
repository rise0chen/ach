static mut MOCK: u32 = 0;

/// Get specific interrupts and returns the current setting
#[inline]
pub fn get_mask() -> u32 {
    unsafe { MOCK }
}

/// Set specific interrupts
/// # Safety
#[inline]
pub unsafe fn set_mask(mask: u32) {
    MOCK = mask
}
