use ach_spsc::Spsc;
use on_drop::OnDrop;

#[test]
fn test() {
    let spsc: Spsc<_, 3> = Spsc::new();
    let mut sender = spsc.take_sender().unwrap();
    let mut receiver = spsc.take_recver().unwrap();
    let (item, token) = OnDrop::token(1);
    assert!(sender.send(item).is_ok());
    drop(receiver.recv().unwrap());
    assert!(token.is_droped());

    let (item, token) = OnDrop::token(1);
    assert!(sender.send(item).is_ok());
    drop(sender);
    drop(receiver);
    drop(spsc);
    assert!(token.is_droped());
}
