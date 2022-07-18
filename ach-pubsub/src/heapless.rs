use ach_array::Array;
pub use ach_array::Ref;
use ach_ring::Ring;
use util::Error;

pub struct Subscriber<'a, T, const N: usize> {
    ch: Ref<'a, Ring<T, N>>,
}
impl<'a, T, const N: usize> Subscriber<'a, T, N> {
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.ch.pop()
    }
}
impl<'a, T, const N: usize> Drop for Subscriber<'a, T, N> {
    fn drop(&mut self) {
        self.ch.remove();
    }
}

pub struct Publisher<T, const NT: usize, const NS: usize> {
    subscribers: Array<Ring<T, NT>, NS>,
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
        let subscriber = Ring::new();
        if let Ok(i) = self.subscribers.push(subscriber) {
            let sub = self.subscribers[i].get().unwrap();
            Some(Subscriber { ch: sub })
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
