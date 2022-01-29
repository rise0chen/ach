use ach_array::{Array, Peek};
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
    pub fn subscribe(&self) -> Option<Peek<Subscriber<T, NT>>> {
        let subscriber = Subscriber(Ring::new());
        if let Ok(i) = self.subscribers.push(subscriber) {
            let peek = self.subscribers.get(i).unwrap();
            Some(peek)
        } else {
            None
        }
    }
}
impl<T: Clone, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    /// return success times
    /// Notice: `Spin`
    pub fn send(&self, val: T) -> usize {
        let mut success: usize = 0;
        let mut send = None;
        for peek in self.subscribers.iter(self.strict) {
            if peek.peek_num() == Ok(1) {
                // No subscriber
                peek.remove();
                continue;
            }
            let value = if let Some(v) = send.take() {
                v
            } else {
                val.clone()
            };
            if let Err(v) = peek.0.push(value) {
                send = Some(v);
            } else {
                success += 1
            }
        }
        success
    }
}
