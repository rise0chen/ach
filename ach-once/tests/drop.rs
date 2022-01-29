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
