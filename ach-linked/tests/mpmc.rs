use ach_linked::{LinkedList, Node};
use std::sync::Arc;
use std::thread;

const TEST_TIMES: usize = 10000;

#[test]
fn base() {
    let list: Arc<LinkedList<usize>> = Arc::new(LinkedList::new());
    let mut node = Node::new(TEST_TIMES);
    unsafe { list.push(&mut node) };
    thread::scope(|s| {
        for i in 0..TEST_TIMES {
            let list = list.clone();
            s.spawn(move || {
                let mut node = Node::new(i);
                unsafe { list.push(&mut node) };
                thread::yield_now();
                list.remove(&mut node);
            });
        }
    });

    let last = list.take_all().unwrap();
    assert_eq!(**last, TEST_TIMES);
    assert!(last.next().is_none())
}
