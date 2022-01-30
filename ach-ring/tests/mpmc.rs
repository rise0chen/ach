use ach_ring::Ring;
use core::sync::atomic::Ordering::Relaxed;
use std::ops::Range;
use std::thread;
use util::{MemoryRing, MemoryState};

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static ARRAY: Ring<usize, 100> = Ring::new();
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

    let mut handle = Vec::new();
    for _ in TEST_DATA {
        handle.push(thread::spawn(move || loop {
            let result = ARRAY.pop();
            if let Some(_) = result {
                break;
            } else {
                thread::yield_now();
            }
        }));
    }
    for h in handle {
        let _ = h.join();
    }
    assert!(ARRAY
        .ops
        .iter()
        .all(|x| { x.load(Relaxed) == MemoryRing::new(10, MemoryState::Uninitialized) }));
    assert!(ARRAY.is_empty());
}
