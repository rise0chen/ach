use riscv::interrupt;
use riscv::register::mstatus;

/// Get specific interrupts and returns the current setting
#[inline]
pub fn get_mask() -> u32 {
    mstatus::read().mie() as u32
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
