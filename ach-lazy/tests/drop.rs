use ach_lazy::Lazy;
use on_drop::{OnDrop, OnDropToken};

#[test]
fn test() {
    let cell: Lazy<(OnDrop<usize>, OnDropToken)> = Lazy::new(|| OnDrop::token(1));
    let token = cell.1.clone();
    drop(cell);
    assert!(token.is_droped());
}
