use ach_option::AchOption;
use on_drop::OnDrop;

#[test]
fn test() {
    let cell = AchOption::new();

    let (item, token) = OnDrop::token(1);
    assert!(cell.set(item).is_ok());
    drop(cell.take().unwrap());
    assert!(token.is_droped());

    let (item, token) = OnDrop::token(1);
    assert!(cell.set(item).is_ok());
    drop(cell);
    assert!(token.is_droped());
}
