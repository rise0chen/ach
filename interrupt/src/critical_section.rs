use crate::{disable_mask, set_mask, MASK_ALL};
use core::ops::Deref;

/// Critical section token.
///
/// The current core is executing code within a critical section.
pub struct CriticalSection {
    inner: bare_metal::CriticalSection,
    mask: u32,
}
impl Default for CriticalSection {
    fn default() -> Self {
        Self::new()
    }
}
impl CriticalSection {
    /// Into a critical section. And exit the critical section, when it drop.
    pub fn new() -> Self {
        let mask = disable_mask(MASK_ALL);
        Self {
            inner: unsafe { bare_metal::CriticalSection::new() },
            mask,
        }
    }
}
impl Deref for CriticalSection {
    type Target = bare_metal::CriticalSection;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl Drop for CriticalSection {
    fn drop(&mut self) {
        unsafe { set_mask(self.mask) };
    }
}
