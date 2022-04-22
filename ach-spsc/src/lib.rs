#![no_std]

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use core::{ptr, slice};

pub struct Sender<'a, T, const N: usize> {
    spsc: &'a Spsc<T, N>,
}
impl<'a, T, const N: usize> Sender<'a, T, N> {
    const fn new(spsc: &'a Spsc<T, N>) -> Self {
        Sender { spsc }
    }
    pub fn send(&mut self, t: T) -> Result<(), T> {
        self.spsc.push(t)
    }
}
impl<'a, T, const N: usize> Drop for Sender<'a, T, N> {
    fn drop(&mut self) {
        unsafe { self.spsc.free_sender() };
    }
}

pub struct Receiver<'a, T, const N: usize> {
    spsc: &'a Spsc<T, N>,
}
impl<'a, T, const N: usize> Receiver<'a, T, N> {
    const fn new(spsc: &'a Spsc<T, N>) -> Self {
        Receiver { spsc }
    }
    pub fn recv(&mut self) -> Option<T> {
        self.spsc.pop()
    }
}
impl<'a, T, const N: usize> Drop for Receiver<'a, T, N> {
    fn drop(&mut self) {
        unsafe { self.spsc.free_recver() };
    }
}

pub struct Spsc<T, const N: usize> {
    buf: MaybeUninit<[T; N]>,
    /// always points to the first element
    start: AtomicUsize,
    end: AtomicUsize,
    has_sender: AtomicBool,
    has_receiver: AtomicBool,
}
impl<T, const N: usize> Spsc<T, N> {
    const CAPACITY: usize = N;
    pub const fn new() -> Self {
        Spsc {
            buf: MaybeUninit::uninit(),
            end: AtomicUsize::new(0),
            start: AtomicUsize::new(0),
            has_sender: AtomicBool::new(true),
            has_receiver: AtomicBool::new(true),
        }
    }
    pub fn take_sender(&self) -> Option<Sender<T, N>> {
        match self
            .has_sender
            .compare_exchange(true, false, SeqCst, SeqCst)
        {
            Ok(_) => Some(Sender::new(self)),
            Err(_) => None,
        }
    }
    pub(crate) unsafe fn free_sender(&self) {
        self.has_sender.store(true, SeqCst)
    }
    pub fn take_recver(&self) -> Option<Receiver<T, N>> {
        match self
            .has_receiver
            .compare_exchange(true, false, SeqCst, SeqCst)
        {
            Ok(_) => Some(Receiver::new(self)),
            Err(_) => None,
        }
    }
    pub(crate) unsafe fn free_recver(&self) {
        self.has_receiver.store(true, SeqCst)
    }
    fn ptr(&self) -> *mut T {
        self.buf.as_ptr() as *mut T
    }
    pub fn capacity(&self) -> usize {
        Self::CAPACITY
    }
    fn wrap_max(&self) -> usize {
        usize::MAX / Self::CAPACITY * Self::CAPACITY
    }
    fn wrap_len(&self, start: usize, end: usize) -> usize {
        if end >= start {
            end - start
        } else {
            self.wrap_max() - start + end
        }
    }
    pub fn len(&self) -> usize {
        let start = self.start.load(SeqCst);
        let end = self.end.load(SeqCst);
        self.wrap_len(start, end)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }
    #[inline]
    unsafe fn buffer_read(&self, off: usize) -> T {
        ptr::read(self.ptr().add(off))
    }
    #[inline]
    unsafe fn buffer_write(&self, off: usize, value: T) {
        ptr::write(self.ptr().add(off), value);
    }
    #[inline]
    fn index(&self, idx: usize) -> usize {
        idx % Self::CAPACITY
    }
    #[inline]
    fn next_idx(&self, old: usize) -> usize {
        if old == self.wrap_max() - 1 {
            0
        } else {
            old + 1
        }
    }
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let ptr = self.ptr();
        let start = self.start.load(SeqCst);
        let end = self.end.load(SeqCst);
        if start == end {
            return (&mut [], &mut []);
        }
        let start = self.index(start);
        let end = self.index(end);
        if end > start {
            (
                unsafe { slice::from_raw_parts_mut(ptr.add(start), end - start) },
                &mut [],
            )
        } else {
            (
                unsafe { slice::from_raw_parts_mut(ptr.add(start), N - start) },
                unsafe { slice::from_raw_parts_mut(ptr, end) },
            )
        }
    }
    pub fn clear(&mut self) {
        let (a, b) = self.as_mut_slices();
        unsafe { ptr::drop_in_place(a) };
        unsafe { ptr::drop_in_place(b) };
        self.end.store(0, SeqCst);
        self.start.store(0, SeqCst);
    }
    fn pop(&self) -> Option<T> {
        let end = self.end.load(SeqCst);
        let start = self.start.load(SeqCst);
        let len = self.wrap_len(start, end);
        if len == 0 || len > self.capacity() {
            return None;
        }

        let index = self.index(start);
        let ret = unsafe { Some(self.buffer_read(index)) };
        self.start
            .compare_exchange(start, self.next_idx(start), SeqCst, SeqCst)
            .unwrap();
        ret
    }
    fn push(&self, value: T) -> Result<(), T> {
        let start = self.start.load(SeqCst);
        let end = self.end.load(SeqCst);
        let len = self.wrap_len(start, end);
        if len >= self.capacity() {
            return Err(value);
        }

        let index = self.index(end);
        unsafe { self.buffer_write(index, value) };
        self.end
            .compare_exchange(end, self.next_idx(end), SeqCst, SeqCst)
            .unwrap();
        Ok(())
    }
}
impl<T, const N: usize> Drop for Spsc<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}
