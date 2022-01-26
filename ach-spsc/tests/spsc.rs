use ach_spsc::Spsc;
use std::collections::BTreeSet;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static SPSC: Spsc<usize, 100> = Spsc::new();
    let mut sender = SPSC.take_sender().unwrap();
    let mut recver = SPSC.take_recver().unwrap();
    let mut data_set: BTreeSet<usize> = TEST_DATA.collect();
    thread::spawn(move || {
        for i in TEST_DATA {
            loop {
                let result = sender.send(i);
                if result.is_ok() {
                    break;
                } else {
                    thread::yield_now();
                }
            }
        }
    });

    let h = thread::spawn(move || {
        for _ in TEST_DATA {
            loop {
                let result = recver.recv();
                if let Some(i) = result {
                    assert!(data_set.remove(&i));
                    break;
                } else {
                    thread::yield_now();
                }
            }
        }
        assert!(data_set.is_empty());
    });
    let _ = h.join();
    assert!(SPSC.is_empty());
}
