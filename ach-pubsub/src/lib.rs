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
}
impl<T, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    pub const fn new() -> Publisher<T, NT, NS> {
        Self {
            subscribers: Array::new(),
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
    pub fn send(&self, val: T) {
        let mut send = None;
        for sub in self.subscribers.iter() {
            if let Some(peek) = sub {
                if peek.peek_num() == Ok(0) {
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
                }
            }
        }
    }
}