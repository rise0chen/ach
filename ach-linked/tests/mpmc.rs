use ach_linked::{LinkedList, Node};
use std::collections::BTreeSet;
use std::thread;

const TEST_TIMES: usize = 100;
const NODE: Node<usize> = Node::new();

#[test]
fn base() {
    let mut handle=Vec::with_capacity(TEST_TIMES);
    let mut handle1=Vec::with_capacity(TEST_TIMES);
    static LIST: LinkedList<usize> = LinkedList::new();
    let mut data_set: BTreeSet<usize> = (0..TEST_TIMES).collect();
    let mut data_set1: BTreeSet<usize> = (0..TEST_TIMES).collect();

    let mut nodes = [NODE; TEST_TIMES];
    for i in 0..TEST_TIMES {
        nodes[i].try_replace(i).unwrap();
        let node = unsafe{(&mut nodes[i] as *mut Node<usize>).as_mut().unwrap()};
        handle.push(thread::spawn(move || {
            unsafe { LIST.push( node) };
        }));
    }
    while let Some(h)=handle.pop() {
        h.join().unwrap();
    }

    for node in LIST.iter() {
        assert!(data_set.remove(&node.try_get().unwrap()));
    }
    assert!(data_set.is_empty());

    for i in 0..TEST_TIMES {
        let node = unsafe{(&mut nodes[i] as *mut Node<usize>).as_mut().unwrap()};
        handle1.push( thread::spawn(move || {
            let val = *node.get().unwrap();
            node.drop();
            val
        }));
    }
    while let Some(h)=handle1.pop() {
        assert!(data_set1.remove(&h.join().unwrap()));
    }
    assert!(data_set1.is_empty());

    println!("fin");
    drop(nodes);
    for node in LIST.iter(){
        println!("{:?}", node.get());
    }
}
