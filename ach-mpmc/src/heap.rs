use crate::heapless::Mpmc;
use alloc::sync::Arc;
use util::Error;

#[derive(Clone)]
pub struct Sender<T, const N: usize> {
    tx: Arc<Mpmc<T, N>>,
}
impl<T, const N: usize> Sender<T, N> {
    pub fn try_send(&self, val: T) -> Result<(), Error<T>> {
        self.tx.sender().try_send(val)
    }
}

#[derive(Clone)]
pub struct Receiver<T, const N: usize> {
    rx: Arc<Mpmc<T, N>>,
}
impl<T, const N: usize> Receiver<T, N> {
    pub fn try_recv(&self) -> Result<T, Error<()>> {
        self.rx.recver().try_recv()
    }
}

pub fn channel<T, const N: usize>() -> (Sender<T, N>, Receiver<T, N>) {
    let tx = Arc::new(Mpmc::new());
    let rx = tx.clone();
    (Sender { tx }, Receiver { rx })
}
