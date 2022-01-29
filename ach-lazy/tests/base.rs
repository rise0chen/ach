use ach_lazy::Lazy;

#[test]
fn base() {
    static CELL: Lazy<usize> = Lazy::new(|| 1);
    assert_eq!(*CELL, 1);
}
