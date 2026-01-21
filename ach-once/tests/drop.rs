use ach_once::Once;
use on_drop::OnDrop;

#[test]
fn test() {
    let cell = Once::new();

    let (item, token) = OnDrop::token(1);
    assert!(cell.set(item).is_ok());
    drop(cell);
    assert!(token.is_droped());
}

#[test]
fn test_into_inner() {
    let once = Once::new_with(Box::new(123u64));
    let v= once.into_inner();
    assert_eq!(v.as_deref(), Some(&123u64));
    drop(v);
}

#[test]
fn test_take() {
    let mut once = Once::new_with(Box::new(123u64));
    let v = once.take();
    assert_eq!(v.as_deref(), Some(&123u64));
    drop(v);
    drop(once);
}
