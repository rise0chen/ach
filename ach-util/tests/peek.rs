use ach_util::{AtomicMemoryPeek, MemoryPeek, MemoryState};
use core::sync::atomic::Ordering::Relaxed;

#[test]
fn test() {
    assert_eq!(
        AtomicMemoryPeek::ZERO.load(Relaxed).state(),
        MemoryState::Uninitialized
    );

    assert_eq!(
        MemoryPeek::from(MemoryState::Initialized).state(),
        MemoryState::Initialized
    );

    assert_eq!(MemoryPeek::PEEK1.peek_num(), Ok(1));

    let mut peek = MemoryPeek::new();
    assert!(!peek.is_peekable());
    assert!(peek.peek_num().is_err());

    peek.set_state(MemoryState::Initialized).unwrap();
    assert!(peek.state().is_initialized());
    assert!(peek.is_peekable());
    assert_eq!(peek.peek_num(), Ok(0));

    peek.peek_add().unwrap();
    assert!(peek.set_state(MemoryState::Initialized).is_err());
    assert!(peek.state().is_peeking());
    assert_eq!(peek.peek_num(), Ok(1));

    peek.peek_sub().unwrap();
    assert!(peek.state().is_initialized());
    assert_eq!(peek.peek_num(), Ok(0));
    peek.peek_sub().unwrap();
    assert!(peek.state().is_initialized());
    assert_eq!(peek.peek_num(), Ok(0));
}
