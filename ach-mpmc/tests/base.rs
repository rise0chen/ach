use ach_mpmc::Mpmc;

#[test]
fn base() {
    static VEC: Mpmc<usize, 3> = Mpmc::new();
    assert_eq!(VEC.capacity(), 3);
    assert!(VEC.is_empty());

    assert!(VEC.push(1).is_ok());
    assert!(!VEC.is_empty());
    assert!(VEC.push(2).is_ok());
    assert!(VEC.push(3).is_ok());
    assert!(VEC.push(4).is_err());
    assert_eq!(VEC.pop().unwrap(), 1);
    assert!(VEC.push(5).is_ok());
    assert_eq!(VEC.pop().unwrap(), 2);
    assert_eq!(VEC.pop().unwrap(), 3);
    assert_eq!(VEC.pop().unwrap(), 5);
    assert!(VEC.pop().is_none());
    assert!(VEC.push(6).is_ok());
}
