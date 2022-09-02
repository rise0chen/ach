use ach_linked::{LinkedList, Node};

#[test]
fn test() {
    let list = LinkedList::new();

    let mut node1 = Node::new(1);
    unsafe { list.push(&mut node1) };
    list.remove(&mut node1);
    assert!(list.is_empty());
    let mut node2 = Node::new(2);
    unsafe { list.push(&mut node2) };
    let mut node3 = Node::new(3);
    unsafe { list.push(&mut node3) };
    let nodes = list.take_all().unwrap();
    let mut nodes = nodes.into_iter();
    assert_eq!(&**nodes.next().unwrap(), &*node3);
    assert_eq!(&**nodes.next().unwrap(), &*node2);
    assert!(nodes.next().is_none());
}
