use super::state::MemoryState;

const REF1: u32 = u8::MAX as u32 + 1;
const REF_MAX: usize = (0x00FF_FFFF - REF1 + 1) as usize;

#[atomic_macro::atomic(32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryRefer(u32);
impl MemoryRefer {
    pub const REF1: Self = Self(REF1);
    /// Uninitialized
    pub const fn new() -> MemoryRefer {
        Self(MemoryState::Uninitialized as u32)
    }
    pub fn state(&self) -> MemoryState {
        if self.0 < REF1 {
            (self.0 as u8).into()
        } else {
            MemoryState::Referred
        }
    }
    pub fn set_state(&mut self, val: MemoryState) -> Result<(), MemoryState> {
        if self.0 < REF1 {
            self.0 = u8::from(val) as u32;
            Ok(())
        } else {
            Err(MemoryState::Referred)
        }
    }
    pub fn can_refer(&self) -> bool {
        let state = self.state();
        if state == MemoryState::Initialized || state == MemoryState::Referred {
            true
        } else {
            false
        }
    }
    pub fn ref_num(&self) -> Result<usize, MemoryState> {
        if self.0 >= REF1 {
            Ok((self.0 - REF1 + 1) as usize)
        } else if self.state().is_initialized() {
            Ok(0)
        } else {
            Err((self.0 as u8).into())
        }
    }
    pub fn ref_add(&mut self) -> Result<(), MemoryState> {
        let ref_num = self.ref_num()?;
        if ref_num == REF_MAX {
            return Ok(());
        }
        if ref_num == 0 {
            self.0 = REF1;
        } else {
            self.0 += 1;
        }
        Ok(())
    }
    pub fn ref_sub(&mut self) -> Result<(), MemoryState> {
        let ref_num = self.ref_num()?;
        if ref_num == REF_MAX || ref_num == 0 {
            return Ok(());
        }
        if ref_num == 1 {
            self.0 = MemoryState::Initialized as u32;
        } else {
            self.0 -= 1;
        }
        Ok(())
    }
}
impl From<MemoryState> for MemoryRefer {
    fn from(s: MemoryState) -> Self {
        Self(s as u32)
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
