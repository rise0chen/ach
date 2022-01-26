use ach_ring::Ring;
use on_drop::OnDrop;

#[test]
fn test() {
    let vec: Ring<_, 3> = Ring::new();
    let (item, token) = OnDrop::token(1);
    assert!(vec.push(item).is_ok());
    drop(vec.pop().unwrap());
    assert!(token.is_droped());

    let (item, token) = OnDrop::token(1);
    assert!(vec.push(item).is_ok());
    drop(vec);
    assert!(token.is_droped());
}
