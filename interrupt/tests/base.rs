#[cfg(not(feature = "custom"))]
#[test]
fn test() {
    use interrupt::*;

    unsafe { set_mask(1) };
    assert_eq!(get_mask(), 1);
    unsafe { disable_mask(1) };
    assert_eq!(get_mask(), 0);
    unsafe { enable_mask(1) };
    assert_eq!(get_mask(), 1);
}
