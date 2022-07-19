use ach_array::Array;
pub use ach_array::Ref;
use ach_ring::Ring;
use alloc::sync::Arc;
use util::Error;

pub struct Subscriber<T, const NT: usize, const NS: usize> {
    index: usize,
    parent: Publisher<T, NT, NS>,
}
impl<T, const NT: usize, const NS: usize> Subscriber<T, NT, NS> {
    fn ch(&self) -> Ref<Ring<T, NT>> {
        self.parent.subscribers[self.index].try_get().unwrap()
    }
    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.ch().pop()
    }
}
impl<T, const NT: usize, const NS: usize> Drop for Subscriber<T, NT, NS> {
    fn drop(&mut self) {
        self.ch().remove();
    }
}

pub struct Publisher<T, const NT: usize, const NS: usize> {
    subscribers: Arc<Array<Ring<T, NT>, NS>>,
    strict: bool,
}
impl<T, const NT: usize, const NS: usize> Publisher<T, NT, NS> {
    /// It will wait all subscriber ready when `send`, if strict is `true`.
    pub fn new(strict: bool) -> Publisher<T, NT, NS> {
        Self {
            subscribers: Arc::new(Array::new()),
            strict,
        }
    }
    pub fn subscribe(&self) -> Option<Subscriber<T, NT, NS>> {
        let subscriber = Ring::new();
        if let Ok(i) = self.subscribers.push(subscriber) {
            Some(Subscriber {
                index: i,
                parent: self.clone(),
            })
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
impl<T, const NT: usize, const NS: usize> Clone for Publisher<T, NT, NS> {
    fn clone(&self) -> Self {
        Self { subscribers: self.subscribers.clone(), strict: self.strict }
    }
}
