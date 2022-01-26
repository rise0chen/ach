use ach_pubsub::Publisher;

#[test]
fn base() {
    static PUB: Publisher<usize, 3, 2> = Publisher::new();
    let sub1 = PUB.subscribe().unwrap();
    let sub2 = PUB.subscribe().unwrap();
    assert!(PUB.subscribe().is_none());

    PUB.send(1);
    PUB.send(2);
    PUB.send(3);
    PUB.send(4); // full
    assert_eq!(sub1.recv().unwrap(), 1);
    assert_eq!(sub2.recv().unwrap(), 1);
    PUB.send(5);
    assert_eq!(sub1.recv().unwrap(), 2);
    assert_eq!(sub2.recv().unwrap(), 2);
    drop(sub2);
    assert_eq!(sub1.recv().unwrap(), 3);
    assert_eq!(sub1.recv().unwrap(), 5);
}
