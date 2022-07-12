use ach_spsc::Spsc;

#[test]
fn base() {
    static SPSC: Spsc<usize, 3> = Spsc::new();
    assert_eq!(SPSC.capacity(), 3);
    let mut sender = SPSC.take_sender().unwrap();
    let mut recver = SPSC.take_recver().unwrap();
    assert!(SPSC.take_sender().is_none());

    assert!(sender.try_send(1).is_ok());
    assert!(sender.try_send(2).is_ok());
    assert!(sender.try_send(3).is_ok());
    assert!(sender.try_send(4).is_err());
    assert_eq!(recver.try_recv().unwrap(), 1);
    assert!(sender.try_send(5).is_ok());

    drop(recver);
    let mut recver = SPSC.take_recver().unwrap();
    assert_eq!(recver.try_recv().unwrap(), 2);
    assert_eq!(recver.try_recv().unwrap(), 3);
    assert_eq!(recver.try_recv().unwrap(), 5);
    assert!(recver.try_recv().is_none());
    assert!(sender.try_send(6).is_ok());
}
