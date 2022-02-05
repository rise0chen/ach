use ach_linked::{LinkedList, Node};

#[test]
fn base() {
    static LIST: LinkedList<usize> = LinkedList::new();
    fn lists() -> Vec<usize> {
        LIST.iter().map(|x| *x.get().unwrap()).collect()
    }

    let mut node1 = Node::new_with(1);
    unsafe { LIST.push(&mut node1) };
    assert_eq!(lists(), vec![1]);

    {
        let mut node2 = Node::new_with(2);
        unsafe { LIST.push(&mut node2) };
        assert_eq!(lists(), vec![2, 1]);
    }
    assert_eq!(lists(), vec![1]);

    unsafe { LIST.push(&mut Node::new_with(3)) };
    assert_eq!(lists(), vec![1]);
}
