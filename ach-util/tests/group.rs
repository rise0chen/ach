use ach_util::{AtomicMemoryGroup, MemoryGroup, MemoryState};

#[test]
fn test() {
    assert!(AtomicMemoryGroup::is_lock_free());

    let mut group = MemoryGroup::new(0, MemoryState::Uninitialized);
    assert_eq!(group.group(), 0);
    assert!(group.state().is_uninitialized());

    group.set_group(1);
    assert_eq!(group.group(), 1);
    group.set_group(MemoryGroup::max_group());
    assert_eq!(group.group(), 0);
    group.set_state(MemoryState::Initialized);
    assert!(group.state().is_initialized());

    group.set_group(MemoryGroup::max_group() - 1);
    group.set_state(MemoryState::Erasing);
    let group_next = group.next();
    assert_eq!(group_next.group(), 0);
    assert!(group_next.state().is_uninitialized());
    assert!(group_next > group);

    assert!(
        MemoryGroup::new(0, MemoryState::Uninitialized)
            < MemoryGroup::new(0, MemoryState::Initializing)
    );
    assert!(
        MemoryGroup::new(0, MemoryState::Initializing)
            < MemoryGroup::new(0, MemoryState::Initialized)
    );
    assert!(
        MemoryGroup::new(0, MemoryState::Initialized) < MemoryGroup::new(0, MemoryState::Erasing)
    );
    assert!(
        MemoryGroup::new(0, MemoryState::Erasing) < MemoryGroup::new(1, MemoryState::Uninitialized)
    );
}
