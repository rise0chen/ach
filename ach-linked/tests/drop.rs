use ach_linked::{LinkedList, Node};
use on_drop::OnDrop;

#[test]
fn test() {
    let list = LinkedList::new();

    let (item, token) = OnDrop::token(1);
    let mut node1 = Node::new(item);
    unsafe { list.push(&mut node1) };
    list.remove(&mut node1);
    assert!(list.is_empty());

    drop(node1);
    assert!(token.is_droped());
}
