use ach_util::state::{AtomicMemoryState, MemoryState};
use core::sync::atomic::Ordering::Relaxed;

#[test]
fn test() {
    assert_eq!(
        AtomicMemoryState::ZERO.load(Relaxed),
        MemoryState::Uninitialized
    );

    assert_eq!(
        MemoryState::Uninitialized,
        u8::from(MemoryState::Uninitialized).into()
    );
    assert_eq!(
        MemoryState::Initializing,
        u8::from(MemoryState::Initializing).into()
    );
    assert_eq!(
        MemoryState::Initialized,
        u8::from(MemoryState::Initialized).into()
    );
    assert_eq!(MemoryState::Erasing, u8::from(MemoryState::Erasing).into());
    assert_eq!(MemoryState::Peeking, u8::from(MemoryState::Peeking).into());

    assert!(MemoryState::Uninitialized.is_uninitialized());
    assert!(MemoryState::Initializing.is_initializing());
    assert!(MemoryState::Initialized.is_initialized());
    assert!(MemoryState::Erasing.is_erasing());
    assert!(MemoryState::Peeking.is_peeking());
}
