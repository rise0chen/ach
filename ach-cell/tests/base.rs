use ach_cell::Cell;

#[test]
fn base() {
    static CELL: Cell<usize> = Cell::new();

    assert!(CELL.set(1).is_ok());
    assert!(CELL.is_initialized());
    assert!(CELL.set(2).is_err());
    assert_eq!(CELL.take().unwrap(), 1);
    assert!(!CELL.is_initialized());
    assert!(CELL.take().is_none());

    assert!(CELL.peek_num().is_err());
    assert!(CELL.set(3).is_ok());
    assert_eq!(CELL.peek_num(), Ok(0));
    let peek = CELL.get().unwrap();
    assert_eq!(CELL.peek_num(), Ok(1));
    assert_eq!(*peek, 3);
    assert!(CELL.take().is_none());
    drop(peek);
    assert_eq!(CELL.take().unwrap(), 3);

    assert!(CELL.set(4).is_ok());
    assert_eq!(*CELL.get().unwrap(), 4);
    assert_eq!(CELL.take().unwrap(), 4);

    assert_eq!(CELL.swap(5), Ok(None));
    assert_eq!(CELL.swap(6), Ok(Some(5)));
    let peek = CELL.get();
    assert_eq!(CELL.swap(7), Err(7));
    drop(peek);
    assert_eq!(CELL.swap(8), Ok(Some(6)));
    assert_eq!(CELL.take().unwrap(), 8);
}
