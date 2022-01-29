#[cfg(feature = "custom")]
#[test]
fn test() {
    use interrupt::*;

    static mut MOCK: u32 = 0;
    pub fn my_get_mask() -> u32 {
        unsafe { MOCK }
    }
    pub fn my_set_mask(mask: u32) {
        unsafe { MOCK = mask }
    }
    register_interrupt!(my_get_mask, my_set_mask);

    unsafe { set_mask(1) };
    assert_eq!(get_mask(), 1);
    disable_mask(1);
    assert_eq!(get_mask(), 0);
    unsafe { enable_mask(1) };
    assert_eq!(get_mask(), 1);
}
