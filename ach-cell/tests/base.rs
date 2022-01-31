use ach_cell::Cell;

#[test]
fn base() {
    static CELL: Cell<usize> = Cell::new();

    assert!(CELL.set(1).is_ok());
    assert!(CELL.is_initialized());
    assert!(CELL.set(2).is_err());
    assert_eq!(CELL.take().unwrap(), Some(1));
    assert!(!CELL.is_initialized());
    assert!(CELL.take().unwrap().is_none());

    assert!(CELL.ref_num().is_err());
    assert!(CELL.set(3).is_ok());
    assert_eq!(CELL.ref_num(), Ok(0));
    let refer = CELL.get().unwrap();
    assert_eq!(CELL.ref_num(), Ok(1));
    assert_eq!(*refer, 3);
    assert!(CELL.take().is_err());
    drop(refer);
    assert_eq!(CELL.take().unwrap(), Some(3));

    assert!(CELL.set(4).is_ok());
    assert_eq!(*CELL.get().unwrap(), 4);
    assert_eq!(CELL.take().unwrap(), Some(4));

    assert_eq!(CELL.replace(5).unwrap(), None);
    assert_eq!(CELL.replace(6).unwrap(), Some(5));
    let refer = CELL.get();
    assert_eq!(CELL.replace(7).unwrap_err().input, 7);
    drop(refer);
    assert_eq!(CELL.replace(8).unwrap(), Some(6));
    assert_eq!(CELL.take().unwrap(), Some(8));
}
