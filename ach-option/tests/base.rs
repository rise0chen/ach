use ach_option::AchOption;

#[test]
fn base() {
    static CELL: AchOption<usize> = AchOption::new();

    assert!(CELL.set(1).is_ok());
    assert!(CELL.is_some());
    assert!(CELL.set(2).is_err());
    assert_eq!(CELL.take(), Some(1));
    assert!(CELL.is_none());
    assert!(CELL.take().is_none());

    assert_eq!(CELL.replace(3), None);
    assert_eq!(CELL.replace(4), Some(3));
}
