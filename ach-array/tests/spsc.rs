use ach_array::Array;
use std::collections::BTreeSet;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static ARRAY: Array<usize, 100> = Array::new();
    let mut data_set: BTreeSet<usize> = TEST_DATA.collect();
    thread::spawn(move || {
        for i in TEST_DATA {
            loop {
                let result = ARRAY.push(i);
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
                let result = ARRAY.pop();
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
    assert!(ARRAY.is_empty());
}
