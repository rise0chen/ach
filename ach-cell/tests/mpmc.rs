use ach_cell::Cell;
use std::ops::Range;
use std::thread;

const TEST_DATA: Range<usize> = 0..1000;

#[test]
fn test() {
    static CELL: Cell<usize> = Cell::new();
    for i in TEST_DATA {
        thread::spawn(move || loop {
            let result = CELL.set(i);
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
            let result = CELL.take();
            if let Ok(Some(_)) = result {
                break;
            } else {
                thread::yield_now();
            }
        }));
    }
    for h in handle {
        let _ = h.join();
    }
    assert!(!CELL.is_initialized());
}
