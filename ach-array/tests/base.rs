use ach_array::Array;

#[test]
fn base() {
    static VEC: Array<usize, 3> = Array::new();
    assert_eq!(VEC.capacity(), 3);
    assert!(VEC.is_empty());

    assert_eq!(VEC[0].replace(1).unwrap(), None);
    assert_eq!(VEC[0].replace(2).unwrap(), Some(1));
    let refer = VEC[0].get();
    assert_eq!(VEC[0].replace(3).unwrap_err().input, 3);
    drop(refer);
    assert_eq!(VEC[0].replace(4).unwrap(), Some(2));
    assert_eq!(VEC.pop().unwrap(), 4);

    assert!(VEC.push(1).is_ok());
    assert!(!VEC.is_empty());
    assert_eq!(*VEC[0].get().unwrap(), 1);

    assert!(VEC.push(2).is_ok());
    assert!(VEC.push(3).is_ok());
    assert!(VEC.push(4).is_err());
    assert_eq!(VEC.pop().unwrap(), 1);
    assert!(VEC.push(5).is_ok());
    assert_eq!(VEC.pop().unwrap(), 5);
    assert_eq!(VEC.pop().unwrap(), 2);
    assert_eq!(VEC.pop().unwrap(), 3);
    assert!(VEC.pop().is_none());
    assert!(VEC.push(6).is_ok());
}
