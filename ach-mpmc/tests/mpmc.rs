use ach_mpmc::heapless::Mpmc;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static ARRAY: Mpmc<usize, 100> = Mpmc::new();
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
            if let Ok(_) = result {
                break;
            } else {
                thread::yield_now();
            }
        }));
    }
    for h in handle {
        let _ = h.join();
    }
    assert!(ARRAY.is_empty());
}
