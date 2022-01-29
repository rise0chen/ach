pub type AtomicMemoryState = atomic::Atomic<MemoryState>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryState {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
    Erasing = 3,
    Referred = 4,
}
impl MemoryState {
    pub fn is_uninitialized(&self) -> bool {
        self == &Self::Uninitialized
    }
    pub fn is_initializing(&self) -> bool {
        self == &Self::Initializing
    }
    pub fn is_initialized(&self) -> bool {
        self == &Self::Initialized
    }
    pub fn is_erasing(&self) -> bool {
        self == &Self::Erasing
    }
    pub fn is_referred(&self) -> bool {
        self == &Self::Referred
    }
    pub fn is_transient(&self) -> bool {
        self == &Self::Initializing || self == &Self::Erasing
    }
}
impl From<u8> for MemoryState {
    fn from(s: u8) -> Self {
        match s {
            s if s == MemoryState::Initializing as u8 => MemoryState::Initializing,
            s if s == MemoryState::Initialized as u8 => MemoryState::Initialized,
            s if s == MemoryState::Erasing as u8 => MemoryState::Erasing,
            s if s == MemoryState::Referred as u8 => MemoryState::Referred,
            _ => MemoryState::Uninitialized,
        }
    }
}
impl From<MemoryState> for u8 {
    fn from(s: MemoryState) -> Self {
        s as u8
    }
}
