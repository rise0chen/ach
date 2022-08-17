use ach_util::{AtomicMemoryRefer, MemoryRefer, MemoryState};

#[test]
fn test() {
    assert!(AtomicMemoryRefer::is_lock_free());

    assert_eq!(
        MemoryRefer::from(MemoryState::Initialized).state(),
        MemoryState::Initialized
    );

    let mut refer = MemoryRefer::new();
    assert!(!refer.state().is_initialized());
    assert!(refer.ref_num().is_err());

    refer.set_state(MemoryState::Initialized);
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(0));

    refer.ref_add().unwrap();
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(1));

    refer.ref_sub().unwrap();
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(0));
    refer.ref_sub().unwrap();
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(0));
}
