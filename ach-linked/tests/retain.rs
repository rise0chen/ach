use ach_linked::{LinkedList, Node};
use std::collections::BTreeSet;

const TEST_TIMES: usize = 10;
const NODE: Node<usize> = Node::new();

#[test]
fn base() {
    let mut list: LinkedList<usize> = LinkedList::new();
    let mut data_set: BTreeSet<usize> = (0..TEST_TIMES).collect();

    let mut nodes = [NODE; TEST_TIMES];
    for i in 0..TEST_TIMES {
        nodes[i].try_replace(i).unwrap();
        let node = unsafe { (&mut nodes[i] as *mut Node<usize>).as_mut().unwrap() };
        list.push(node);
    }

    list.retain(|node| {
        assert!(data_set.remove(&node.try_get().unwrap()));
        false
    });

    assert!(list.is_empty());
    assert!(data_set.is_empty());
}
