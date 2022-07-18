use ach_array::Array;
pub use ach_array::Ref;
use ach_ring::Ring;
use alloc::sync::Arc;
use util::Error;

pub struct Subscriber<T, const N: usize> {
    ch: Arc<Ring<T, N>>,
}
impl<T, const N: usize> Subscriber<T, N> {
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.ch.pop()
    }
}

pub struct Publisher<T, const NT: usize, const NS: usize> {
    subscribers: Array<Arc<Ring<T, NT>>, NS>,
    strict: bool,
}
impl<T, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    /// It will wait all subscriber ready when `send`, if strict is `true`.
    pub const fn new(strict: bool) -> Publisher<T, NT, NS> {
        Self {
            subscribers: Array::new(),
            strict,
        }
    }
    pub fn subscribe(&self) -> Option<Subscriber<T, NT>> {
        let subscriber = Arc::new(Ring::new());
        if let Ok(i) = self.subscribers.push(subscriber) {
            let sub = self.subscribers[i].get().unwrap();
            Some(Subscriber { ch: sub.clone() })
        } else {
            None
        }
    }
}
impl<T: Clone, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    /// return success times
    ///
    /// Notice: `Spin` if strict
    pub fn send(&self, val: T) -> usize {
        let mut success: usize = 0;
        let mut send = None;
        for sub in self.subscribers.iter(self.strict) {
            if Arc::strong_count(&sub) <= 1 {
                sub.remove();
                continue;
            }
            let value = if let Some(v) = send.take() {
                v
            } else {
                val.clone()
            };
            if let Err(v) = sub.push(value) {
                send = Some(v.input);
            } else {
                success += 1
            }
        }
        success
    }
}
