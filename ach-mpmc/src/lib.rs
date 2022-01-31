use ach_ring::Ring;
use core::ops::Deref;
use util::*;

pub struct Sender<'a, T, const N: usize> {
    mpmc: &'a Mpmc<T, N>,
}
impl<'a, T, const N: usize> Sender<'a, T, N> {
    const fn new(mpmc: &'a Mpmc<T, N>) -> Self {
        Sender { mpmc }
    }
    pub fn send(&self, t: T) -> Result<(), Error<T>> {
        self.mpmc.push(t)
    }
}

pub struct Receiver<'a, T, const N: usize> {
    mpmc: &'a Mpmc<T, N>,
}
impl<'a, T, const N: usize> Receiver<'a, T, N> {
    const fn new(mpmc: &'a Mpmc<T, N>) -> Self {
        Receiver { mpmc }
    }
    pub fn recv(&self) -> Result<T,Error<()>> {
        self.mpmc.pop()
    }
}

pub struct Mpmc<T, const N: usize> {
    ring: Ring<T, N>,
}
impl<T, const N: usize> Mpmc<T, N> {
    pub const fn new() -> Self {
        Mpmc { ring: Ring::new() }
    }
    pub const fn sender(&self) -> Sender<T, N> {
        Sender::new(self)
    }
    pub const fn recver(&self) -> Receiver<T, N> {
        Receiver::new(self)
    }
}
impl<T, const N: usize> Deref for Mpmc<T, N> {
    type Target = Ring<T, N>;
    fn deref(&self) -> &Self::Target {
        &self.ring
    }
}
