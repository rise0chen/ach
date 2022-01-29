use ach_util::{AtomicMemoryRefer, MemoryRefer, MemoryState};
use core::sync::atomic::Ordering::Relaxed;

#[test]
fn test() {
    assert_eq!(
        AtomicMemoryRefer::ZERO.load(Relaxed).state(),
        MemoryState::Uninitialized
    );

    assert_eq!(
        MemoryRefer::from(MemoryState::Initialized).state(),
        MemoryState::Initialized
    );

    assert_eq!(MemoryRefer::REF1.ref_num(), Ok(1));

    let mut refer = MemoryRefer::new();
    assert!(!refer.can_refer());
    assert!(refer.ref_num().is_err());

    refer.set_state(MemoryState::Initialized).unwrap();
    assert!(refer.state().is_initialized());
    assert!(refer.can_refer());
    assert_eq!(refer.ref_num(), Ok(0));

    refer.ref_add().unwrap();
    assert!(refer.set_state(MemoryState::Initialized).is_err());
    assert!(refer.state().is_referred());
    assert_eq!(refer.ref_num(), Ok(1));

    refer.ref_sub().unwrap();
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(0));
    refer.ref_sub().unwrap();
    assert!(refer.state().is_initialized());
    assert_eq!(refer.ref_num(), Ok(0));
}
