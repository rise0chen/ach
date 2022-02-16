use ach_linked::{LinkedList, Node};
use std::collections::BTreeSet;
use std::thread;

const TEST_TIMES: usize = 1000;
const NODE: Node<usize> = Node::new();

#[test]
fn base() {
    let mut handle = Vec::with_capacity(TEST_TIMES);
    static LIST: LinkedList<usize> = LinkedList::new();
    let mut data_set: BTreeSet<usize> = (0..TEST_TIMES).collect();

    let mut nodes = [NODE; TEST_TIMES];
    for i in 0..TEST_TIMES {
        nodes[i].try_replace(i).unwrap();
        let node = unsafe { (&mut nodes[i] as *mut Node<usize>).as_mut().unwrap() };
        handle.push(thread::spawn(move || {
            LIST.insert(node);
        }));
    }
    while let Some(h) = handle.pop() {
        h.join().unwrap();
    }

    // for node in LIST.iter() {
    //     assert!(data_set.remove(&node.try_get().unwrap()));
    // }
    // assert!(data_set.is_empty());
}
