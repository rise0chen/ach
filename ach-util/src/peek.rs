use super::state::MemoryState;

const PEEK1: u32 = u8::MAX as u32 + 1;
const PEEK_MAX: usize = (0x00FF_FFFF - PEEK1 + 1) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[atomic_macro::atomic(32)]
pub struct MemoryPeek(u32);
impl MemoryPeek {
    pub const PEEK1: Self = Self(PEEK1);
    /// Uninitialized
    pub const fn new() -> MemoryPeek {
        Self(MemoryState::Uninitialized as u32)
    }
    pub fn state(&self) -> MemoryState {
        if self.0 < PEEK1 {
            (self.0 as u8).into()
        } else {
            MemoryState::Peeking
        }
    }
    pub fn set_state(&mut self, val: MemoryState) -> Result<(), MemoryState> {
        if self.0 < PEEK1 {
            self.0 = u8::from(val) as u32;
            Ok(())
        } else {
            Err(MemoryState::Peeking)
        }
    }
    pub fn is_peekable(&self) -> bool {
        let state = self.state();
        if state == MemoryState::Initialized || state == MemoryState::Peeking {
            true
        } else {
            false
        }
    }
    pub fn peek_num(&self) -> Result<usize, MemoryState> {
        if self.0 >= PEEK1 {
            Ok((self.0 - PEEK1 + 1) as usize)
        } else if self.state().is_initialized() {
            Ok(0)
        } else {
            Err((self.0 as u8).into())
        }
    }
    pub fn peek_add(&mut self) -> Result<(), MemoryState> {
        let peek_num = self.peek_num()?;
        if peek_num == PEEK_MAX {
            return Ok(());
        }
        if peek_num == 0 {
            self.0 = PEEK1;
        } else {
            self.0 += 1;
        }
        Ok(())
    }
    pub fn peek_sub(&mut self) -> Result<(), MemoryState> {
        let peek_num = self.peek_num()?;
        if peek_num == PEEK_MAX || peek_num == 0 {
            return Ok(());
        }
        if peek_num == 1 {
            self.0 = MemoryState::Initialized as u32;
        } else {
            self.0 -= 1;
        }
        Ok(())
    }
}
impl From<MemoryState> for MemoryPeek {
    fn from(s: MemoryState) -> Self {
        Self(s as u32)
    }
}
impl From<u32> for MemoryPeek {
    fn from(s: u32) -> Self {
        Self(s)
    }
}
impl From<MemoryPeek> for u32 {
    fn from(s: MemoryPeek) -> Self {
        s.0
    }
}
