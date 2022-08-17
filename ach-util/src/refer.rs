use super::state::MemoryState;

pub type AtomicMemoryRefer = atomic::Atomic<MemoryRefer>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryRefer(u32);
impl MemoryRefer {
    /// Uninitialized
    pub const fn new() -> Self {
        Self(0)
    }
    pub fn state(&self) -> MemoryState {
        ((self.0 >> 24) as u8).into()
    }
    pub fn set_state(&mut self, val: MemoryState) {
        self.0 = (self.0 & 0x00FF_FFFF) | ((u8::from(val) as u32) << 24);
    }
    pub const fn max_refer() -> usize {
        0x00FF_FFFF
    }
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        let state = self.state();
        if state.is_initialized() || state.is_regaining() || state.is_erasing() {
            Ok((self.0 as usize) & 0x00FF_FFFF)
        } else {
            Err(state)
        }
    }
    pub fn ref_add(&mut self) -> Result<(), MemoryState> {
        let ref_num = self.ref_num()?;
        if ref_num == Self::max_refer() {
            return Ok(());
        }
        let state = self.state();
        if state.is_initialized() {
            self.0 += 1;
            Ok(())
        } else {
            Err(state)
        }
    }
    pub fn ref_sub(&mut self) -> Result<(), MemoryState> {
        let ref_num = self.ref_num()?;
        if ref_num == Self::max_refer() || ref_num == 0 {
            return Ok(());
        }
        let state = self.state();
        if state.is_initialized() || state.is_regaining() {
            self.0 -= 1;
            Ok(())
        } else {
            Err(state)
        }
    }
}
impl From<MemoryState> for MemoryRefer {
    fn from(s: MemoryState) -> Self {
        let mut refer = MemoryRefer::new();
        refer.set_state(s);
        refer
    }
}
impl From<u32> for MemoryRefer {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
impl From<MemoryRefer> for u32 {
    fn from(s: MemoryRefer) -> Self {
        s.0
    }
}
