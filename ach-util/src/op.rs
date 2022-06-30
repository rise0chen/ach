#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    None = 0,
    Write,
    Take,
    Replace,
    Update,
    Remove,
}
impl From<u8> for Op {
    fn from(s: u8) -> Self {
        match s {
            s if s == Op::None as u8 => Op::None,
            s if s == Op::Write as u8 => Op::Write,
            s if s == Op::Take as u8 => Op::Take,
            s if s == Op::Replace as u8 => Op::Replace,
            s if s == Op::Remove as u8 => Op::Remove,
            _ => Op::None,
        }
    }
}
impl From<Op> for u8 {
    fn from(s: Op) -> Self {
        s as u8
    }
}

pub type AtomicMemoryOp = atomic::Atomic<MemoryOp>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryOp(u32);
impl MemoryOp {
    pub const fn new() -> MemoryOp {
        Self(0)
    }
    pub fn op(&self) -> Op {
        Op::from((self.0 & 0xFF) as u8)
    }
    pub fn cur_version(&self) -> u16 {
        ((self.0 & 0x000FFF00) >> 8) as u16
    }
    pub fn next_version(&mut self) -> u16 {
        ((self.0 & 0xFFF00000) >> 20) as u16
    }
    pub fn finish(&mut self) {
        self.0 = (self.0 & 0xFFF00000) | ((self.next_version() as u32) << 8)
    }
    pub fn is_finished(&mut self) -> bool {
        self.cur_version() == self.next_version()
    }
    pub fn set_op(&mut self, op: Op) -> u16 {
        self.0 = (self.0.wrapping_add(0x00100000) & 0xFFFFFF00) | (op as u32);
        self.next_version()
    }
}
impl From<u32> for MemoryOp {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
impl From<MemoryOp> for u32 {
    fn from(s: MemoryOp) -> Self {
        s.0
    }
}
