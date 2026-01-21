use ach_mpmc::heapless::Mpmc;
use std::collections::BTreeSet;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static ARRAY: Mpmc<usize, 100> = Mpmc::new();
    let mut data_set: BTreeSet<usize> = TEST_DATA.collect();
    for i in TEST_DATA {
        thread::spawn(move || loop {
            let result = ARRAY.push(i);
            if result.is_ok() {
                break;
            } else {
                thread::yield_now();
            }
        });
    }

    let h = thread::spawn(move || {
        for _ in TEST_DATA {
            loop {
                let result = ARRAY.pop();
                if let Ok(i) = result {
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
    assert!(ARRAY.is_empty());
}
