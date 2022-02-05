use ach_linked::{LinkedList, Node};
use on_drop::OnDrop;

#[test]
fn test() {
    let list = LinkedList::new();

    let (item, token) = OnDrop::token(1);
    let mut node1 = Node::new_with(item);
    unsafe { list.push(&mut node1) };

    node1.drop(); //must drop it before move it.
    drop(node1);
    assert!(token.is_droped());
}
