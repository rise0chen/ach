use ach_pool::Pool;

#[test]
fn base() {
    static VEC: Pool<usize, 3> = Pool::new();
    assert_eq!(VEC.capacity(), 3);
    assert!(VEC.is_empty());

    assert_eq!(VEC[0].replace(1), None);
    assert_eq!(VEC[0].replace(2), Some(1));
    assert_eq!(VEC.pop().unwrap(), 2);

    assert!(VEC.push(1).is_ok());
    assert!(!VEC.is_empty());
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
