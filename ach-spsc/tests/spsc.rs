use ach_spsc::Spsc;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..10000;

#[test]
fn test() {
    static SPSC: Spsc<usize, 2> = Spsc::new();
    let mut sender = SPSC.take_sender().unwrap();
    let mut recver = SPSC.take_recver().unwrap();
    thread::spawn(move || {
        for i in TEST_DATA {
            loop {
                let result = sender.try_send(i);
                if result.is_ok() {
                    break;
                } else {
                    thread::yield_now();
                }
            }
        }
    });

    let h = thread::spawn(move || {
        for i in TEST_DATA {
            loop {
                let result = recver.try_recv();
                if let Some(result) = result {
                    assert_eq!(result, i);
                    break;
                } else {
                    thread::yield_now();
                }
            }
        }
    });
    let _ = h.join();
    assert!(SPSC.is_empty());
}
