use ach_once::Once;

#[test]
fn base() {
    static CELL: Once<usize> = Once::new();

    assert_eq!(CELL.get(), None);
    assert!(CELL.set(1).is_ok());
    assert!(CELL.is_initialized());
    assert!(CELL.set(2).is_err());
    assert_eq!(CELL.get(), Some(&1));
}
