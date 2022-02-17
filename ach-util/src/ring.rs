use super::state::MemoryState;
use core::cmp::Ordering;

pub type AtomicMemoryRing = atomic::Atomic<MemoryRing>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRing(u32);
impl MemoryRing {
    pub const INIT: Self = Self::new(0, MemoryState::Uninitialized);
    pub const fn new(cycle: usize, state: MemoryState) -> Self {
        let cycle = cycle & 0x00FF_FFFF;
        Self((state as u32) << 24 | cycle as u32)
    }
    pub const fn max_cycle() -> usize {
        0x00FF_FFFF + 1
    }
    pub fn cycle(&self) -> usize {
        (self.0 as usize) & 0x00FF_FFFF
    }
    pub fn set_cycle(&mut self, val: usize) {
        let val = val & 0x00FF_FFFF;
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
                let cycle = self.cycle() + 1;
                ret.set_cycle(if cycle >= Self::max_cycle() { 0 } else { cycle });
                ret.set_state(MemoryState::Uninitialized);
            }
            _ => unreachable!(),
        }
        ret
    }

    pub const fn max_idx(size: usize) -> usize {
        let cycle_max = Self::max_cycle();
        usize::MAX / size / cycle_max * size * cycle_max
    }
    pub const fn cycle_of_idx(idx: usize, size: usize) -> usize {
        idx / size
    }
}
impl From<u32> for MemoryRing {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
impl From<MemoryRing> for u32 {
    fn from(s: MemoryRing) -> Self {
        s.0
    }
}
impl PartialOrd for MemoryRing {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_cycle = self.cycle();
        let other_cycle = other.cycle();
        let max_cycle = Self::max_cycle();
        let ord = self_cycle.partial_cmp(&other_cycle);
        if ord == Some(Ordering::Equal) {
            self.state().partial_cmp(&other.state())
        } else {
            if self_cycle < max_cycle / 4 && other_cycle > max_cycle / 4 * 3 {
                // self overflow
                Some(Ordering::Greater)
            } else if self_cycle > max_cycle / 4 * 3 && other_cycle < max_cycle / 4 {
                // other overflow
                Some(Ordering::Less)
            } else {
                ord
            }
        }
    }
}
