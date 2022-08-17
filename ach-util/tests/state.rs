use ach_util::state::{AtomicMemoryState, MemoryState};

#[test]
fn test() {
    assert!(AtomicMemoryState::is_lock_free());

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
    assert_eq!(
        MemoryState::Regaining,
        u8::from(MemoryState::Regaining).into()
    );

    assert!(MemoryState::Uninitialized.is_uninitialized());
    assert!(MemoryState::Initializing.is_initializing());
    assert!(MemoryState::Initialized.is_initialized());
    assert!(MemoryState::Erasing.is_erasing());
    assert!(MemoryState::Regaining.is_regaining());
}
