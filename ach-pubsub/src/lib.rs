use ach_array::{Array, Ref};
use ach_ring::Ring;

pub struct Subscriber<T, const N: usize>(Ring<T, N>);
impl<T, const N: usize> Subscriber<T, N> {
    pub fn recv(&self) -> Option<T> {
        self.0.pop()
    }
}

pub struct Publisher<T, const NT: usize, const NS: usize> {
    subscribers: Array<Subscriber<T, NT>, NS>,
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
    pub fn subscribe(&self) -> Option<Ref<Subscriber<T, NT>>> {
        let subscriber = Subscriber(Ring::new());
        if let Ok(i) = self.subscribers.push(subscriber) {
            let sub = self.subscribers[i].get().unwrap();
            Some(sub)
        } else {
            None
        }
    }
}
impl<T: Clone, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    /// return success times
    /// Notice: `Spin` if strict
    pub fn send(&self, val: T) -> usize {
        let mut success: usize = 0;
        let mut send = None;
        for sub in self.subscribers.iter(self.strict) {
            if sub.ref_num() == Ok(1) {
                // No subscriber
                sub.remove();
                continue;
            }
            let value = if let Some(v) = send.take() {
                v
            } else {
                val.clone()
            };
            if let Err(v) = sub.0.push(value) {
                send = Some(v);
            } else {
                success += 1
            }
        }
        success
    }
}
