#[test]
fn test() {
    use interrupt::*;

    unsafe { set_mask(1) };
    assert_eq!(get_mask(), 1);
    let cs = CriticalSection::new();
    assert_eq!(get_mask(), 0);
    drop(cs);
    assert_eq!(get_mask(), 1);
}
