use ach_util::{AtomicMemoryRing, MemoryRing, MemoryState};

#[test]
fn test() {
    assert!(AtomicMemoryRing::is_lock_free());

    let mut cycle = MemoryRing::new(0, MemoryState::Uninitialized);
    assert_eq!(cycle.cycle(), 0);
    assert!(cycle.state().is_uninitialized());

    cycle.set_cycle(1);
    assert_eq!(cycle.cycle(), 1);
    cycle.set_cycle(MemoryRing::max_cycle());
    assert_eq!(cycle.cycle(), 0);
    cycle.set_state(MemoryState::Initialized);
    assert!(cycle.state().is_initialized());

    cycle.set_cycle(MemoryRing::max_cycle() - 1);
    cycle.set_state(MemoryState::Erasing);
    let cycle_next = cycle.next();
    assert_eq!(cycle_next.cycle(), 0);
    assert!(cycle_next.state().is_uninitialized());
    assert!(cycle_next > cycle);

    assert!(
        MemoryRing::new(0, MemoryState::Uninitialized)
            < MemoryRing::new(0, MemoryState::Initializing)
    );
    assert!(
        MemoryRing::new(0, MemoryState::Initializing)
            < MemoryRing::new(0, MemoryState::Initialized)
    );
    assert!(
        MemoryRing::new(0, MemoryState::Initialized) < MemoryRing::new(0, MemoryState::Erasing)
    );
    assert!(
        MemoryRing::new(0, MemoryState::Erasing) < MemoryRing::new(1, MemoryState::Uninitialized)
    );
}
