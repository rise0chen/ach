use ach_cell::Cell;
use std::collections::BTreeSet;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static CELL: Cell<usize> = Cell::new();
    let mut data_set: BTreeSet<usize> = TEST_DATA.collect();
    thread::spawn(move || {
        for i in TEST_DATA {
            loop {
                let result = CELL.set(i);
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
                let result = CELL.take();
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
    assert!(!CELL.is_initialized());
}
