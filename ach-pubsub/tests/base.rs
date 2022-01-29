use ach_pubsub::Publisher;

#[test]
fn base() {
    static PUB: Publisher<usize, 3, 2> = Publisher::new(false);
    let sub1 = PUB.subscribe().unwrap();
    let sub2 = PUB.subscribe().unwrap();
    assert!(PUB.subscribe().is_none());

    assert_eq!(PUB.send(1), 2);
    assert_eq!(PUB.send(2), 2);
    assert_eq!(PUB.send(3), 2);
    assert_eq!(PUB.send(4), 0); // full
    assert_eq!(sub1.recv().unwrap(), 1);
    assert_eq!(sub2.recv().unwrap(), 1);
    assert_eq!(PUB.send(5), 2);
    assert_eq!(sub1.recv().unwrap(), 2);
    assert_eq!(sub2.recv().unwrap(), 2);
    drop(sub2);
    assert_eq!(PUB.send(6), 1);
    assert_eq!(sub1.recv().unwrap(), 3);
    assert_eq!(sub1.recv().unwrap(), 5);
    assert_eq!(sub1.recv().unwrap(), 6);
}
