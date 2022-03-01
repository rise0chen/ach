use ach_pool::Pool;
use on_drop::OnDrop;

#[test]
fn test() {
    let vec: Pool<_, 3> = Pool::new();
    let (item, token) = OnDrop::token(1);
    assert!(vec.push(item).is_ok());
    drop(vec.pop().unwrap());
    assert!(token.is_droped());

    let (item, token) = OnDrop::token(1);
    assert!(vec.push(item).is_ok());
    drop(vec);
    assert!(token.is_droped());
}
