use super::state::MemoryState;
use core::cmp::Ordering;

pub type AtomicMemoryGroup = atomic::Atomic<MemoryGroup>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryGroup(u32);
impl MemoryGroup {
    pub const INIT: Self = Self::new(0, MemoryState::Uninitialized);
    pub const fn new(group: usize, state: MemoryState) -> Self {
        Self((state as u32) << 24 | group as u32)
    }
    pub const fn max_group() -> usize {
        0x00FF_FFFF + 1
    }
    pub fn group(&self) -> usize {
        (self.0 as usize) & 0x00FF_FFFF
    }
    pub fn set_group(&mut self, val: usize) {
        self.0 = (self.0 & 0xFF00_0000) | val as u32;
    }
    pub fn state(&self) -> MemoryState {
        ((self.0 >> 24) as u8).into()
    }
    pub fn set_state(&mut self, val: MemoryState) {
        self.0 = (self.0 & 0x00FF_FFFF) | ((u8::from(val) as u32) << 24);
    }
    pub fn next(&self) -> Self {
        let mut ret = *self;
        match self.state() {
            MemoryState::Uninitialized => ret.set_state(MemoryState::Initializing),
            MemoryState::Initializing => ret.set_state(MemoryState::Initialized),
            MemoryState::Initialized => ret.set_state(MemoryState::Erasing),
            MemoryState::Erasing => {
                let group = self.group() + 1;
                ret.set_group(if group >= Self::max_group() { 0 } else { group });
                ret.set_state(MemoryState::Uninitialized);
            }
            _ => unreachable!(),
        }
        ret
    }

    pub const fn max_idx(size: usize) -> usize {
        let group_max = Self::max_group();
        usize::MAX / size / group_max * size * group_max
    }
    pub const fn group_of_idx(idx: usize, size: usize) -> usize {
        idx / size
    }
}
impl From<u32> for MemoryGroup {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
impl From<MemoryGroup> for u32 {
    fn from(s: MemoryGroup) -> Self {
        s.0
    }
}
impl PartialOrd for MemoryGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_group = self.group();
        let other_group = other.group();
        let max_group = Self::max_group();
        let ord = self_group.partial_cmp(&other_group);
        if ord == Some(Ordering::Equal) {
            self.state().partial_cmp(&other.state())
        } else {
            if self_group < max_group / 4 && other_group > max_group / 4 * 3 {
                // self overflow
                Some(Ordering::Greater)
            } else if self_group > max_group / 4 * 3 && other_group < max_group / 4 {
                // other overflow
                Some(Ordering::Less)
            } else {
                ord
            }
        }
    }
}
