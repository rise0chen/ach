#![no_std]

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{ptr, slice};
use util::*;

pub struct Ring<T, const N: usize> {
    buf: MaybeUninit<[T; N]>,
    /// always points to the first element
    start: AtomicUsize,
    end: AtomicUsize,
    pub ops: [AtomicMemoryRing; N],
}
impl<T, const N: usize> Ring<T, N> {
    const CAPACITY: usize = N;
    const WRAP_MAX: usize = MemoryRing::max_idx(Self::CAPACITY);
    const INIT_STATE: AtomicMemoryRing = AtomicMemoryRing::new(MemoryRing::INIT);
    pub const fn new() -> Self {
        Ring {
            buf: MaybeUninit::uninit(),
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
            ops: [Self::INIT_STATE; N],
        }
    }
    fn ptr(&self) -> *mut T {
        self.buf.as_ptr() as *mut T
    }
    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    fn wrap_len(&self, start: usize, end: usize) -> usize {
        if end >= start {
            end - start
        } else {
            Self::WRAP_MAX - start + end
        }
    }
    pub fn len(&self) -> usize {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);
        self.wrap_len(start, end)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_full(&self) -> bool {
        self.len() >= Self::CAPACITY
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
        if old == Self::WRAP_MAX - 1 {
            0
        } else {
            old + 1
        }
    }
    fn add_ptr_end(&self, old: usize) -> Result<usize, usize> {
        let new = self.next_idx(old);
        self.end
            .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed)
    }
    fn add_ptr_start(&self, old: usize) -> Result<usize, usize> {
        let new = self.next_idx(old);
        self.start
            .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed)
    }
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let ptr = self.ptr();
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);
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
        self.end.store(0, Ordering::Relaxed);
        self.start.store(0, Ordering::Relaxed);
        self.ops = [Self::INIT_STATE; N];
    }

    /// Removes the first element and returns it.
    ///
    /// Returns Err if the Ring is empty.
    pub fn pop(&self) -> Result<T, Error<()>> {
        #[cfg(target_os = "none")]
        let _cs = interrupt::CriticalSection::new();
        let mut start = self.start.load(Ordering::Relaxed);
        loop {
            let cycle = MemoryRing::cycle_of_idx(start, Self::CAPACITY);
            let index = self.index(start);
            let expect = MemoryRing::new(cycle, MemoryState::Initialized);
            let op = self.ops[index].load(Ordering::Acquire);
            let state = op.state();
            if op >= expect {
                if let Err(i) = self.add_ptr_start(start) {
                    start = i;
                    continue;
                } else {
                    let ret = unsafe { self.buffer_read(index) };
                    let op = MemoryRing::new(cycle + 1, MemoryState::Uninitialized);
                    self.ops[index].store(op, Ordering::Release);
                    return Ok(ret);
                }
            } else {
                return Err(Error {
                    state,
                    input: (),
                    retry: false,
                });
            }
        }
    }

    /// Appends an element to the back of the Ring.
    ///
    /// Returns Err if the Ring is full.
    pub fn push(&self, value: T) -> Result<(), Error<T>> {
        #[cfg(target_os = "none")]
        let _cs = interrupt::CriticalSection::new();
        let mut end = self.end.load(Ordering::Relaxed);
        loop {
            let cycle = MemoryRing::cycle_of_idx(end, Self::CAPACITY);
            let index = self.index(end);
            let expect = MemoryRing::new(cycle, MemoryState::Uninitialized);
            let op = self.ops[index].load(Ordering::Acquire);
            let state = op.state();
            if op >= expect {
                if let Err(i) = self.add_ptr_end(end) {
                    end = i;
                    continue;
                } else {
                    unsafe { self.buffer_write(index, value) };
                    let op = MemoryRing::new(cycle, MemoryState::Initialized);
                    self.ops[index].store(op, Ordering::Release);
                    return Ok(());
                }
            } else {
                return Err(Error {
                    state,
                    input: value,
                    retry: false,
                });
            }
        }
    }
}
impl<T, const N: usize> Drop for Ring<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}
