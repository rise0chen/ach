use crate::{disable_mask, set_mask, MASK_ALL};
use core::ops::Deref;

/// Critical section token.
///
/// The current core is executing code within a critical section.
pub struct CriticalSection<'cs> {
    inner: bare_metal::CriticalSection<'cs>,
    mask: u32,
}
impl<'cs> CriticalSection<'cs> {
    /// Into a critical section. And exit the critical section, when it drop.
    pub fn new() -> Self {
        let mask = disable_mask(MASK_ALL);
        Self {
            inner: unsafe { bare_metal::CriticalSection::new() },
            mask,
        }
    }
}
impl<'cs> Deref for CriticalSection<'cs> {
    type Target = bare_metal::CriticalSection<'cs>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<'cs> Drop for CriticalSection<'cs> {
    fn drop(&mut self) {
        unsafe { set_mask(self.mask) };
    }
}
