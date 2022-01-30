use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::{ptr, slice};
use interrupt::CriticalSection;
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
    const fn wrap_max(&self) -> usize {
        MemoryRing::max_idx(Self::CAPACITY)
    }
    fn wrap_len(&self, start: usize, end: usize) -> usize {
        if end >= start {
            end - start
        } else {
            self.wrap_max() - start + end
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
    fn add_ptr_end(&self, old: usize) {
        let new = self.next_idx(old);
        let _ = self
            .end
            .compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed);
    }
    fn add_ptr_start(&self, old: usize) {
        let new = self.next_idx(old);
        let _ = self
            .start
            .compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed);
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

    pub fn pop(&self) -> Option<T> {
        loop {
            match self.try_pop() {
                Ok(val) => return Some(val),
                Err(err) if err.retry => {
                    continue;
                }
                Err(_) => return None,
            }
        }
    }
    pub fn try_pop(&self) -> Result<T, Error<()>> {
        let _cs = CriticalSection::new();
        let end = self.end.load(Ordering::Relaxed);
        let start = self.start.load(Ordering::Relaxed);
        let len = self.wrap_len(start, end);
        if len == 0 || len > self.capacity() {
            return Err(Error {
                state: MemoryState::Unknown,
                input: (),
                retry: false,
            });
        }
        let cycle = MemoryRing::cycle_of_idx(start, Self::CAPACITY);
        let index = self.index(start);
        let expect = MemoryRing::new(cycle, MemoryState::Initialized);
        if let Err(op) = self.ops[index].fetch_update(Ordering::Relaxed, Ordering::Relaxed, |op| {
            if op == expect {
                Some(op.next())
            } else {
                None
            }
        }) {
            let state = op.state();
            if op > expect {
                // retry next cell, needn't `spin`
                self.add_ptr_start(start);
                self.try_pop()
            } else if op.next() == expect {
                // initializing
                Err(Error {
                    state,
                    input: (),
                    retry: true,
                })
            } else {
                Err(Error {
                    state,
                    input: (),
                    retry: false,
                })
            }
        } else {
            self.add_ptr_start(start);
            let ret = unsafe { self.buffer_read(index) };
            let op = MemoryRing::new(cycle + 1, MemoryState::Uninitialized);
            self.ops[index].store(op, Ordering::Relaxed);
            Ok(ret)
        }
    }
    pub fn push(&self, mut value: T) -> Result<(), T> {
        loop {
            match self.try_push(value) {
                Ok(val) => return Ok(val),
                Err(err) if err.retry => {
                    value = err.input;
                    continue;
                }
                Err(err) => return Err(err.input),
            }
        }
    }
    pub fn try_push(&self, value: T) -> Result<(), Error<T>> {
        let _cs = CriticalSection::new();
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);
        let len = self.wrap_len(start, end);
        if len >= self.capacity() {
            return Err(Error {
                state: MemoryState::Unknown,
                input: value,
                retry: false,
            });
        }
        let cycle = MemoryRing::cycle_of_idx(end, Self::CAPACITY);
        let index = self.index(end);
        let expect = MemoryRing::new(cycle, MemoryState::Uninitialized);
        if let Err(op) = self.ops[index].fetch_update(Ordering::Relaxed, Ordering::Relaxed, |op| {
            if op == expect {
                Some(op.next())
            } else {
                None
            }
        }) {
            let state = op.state();
            if op > expect {
                // retry next cell, needn't `spin`
                self.add_ptr_end(end);
                self.try_push(value)
            } else if op.next() == expect {
                // erasing
                Err(Error {
                    state,
                    input: value,
                    retry: true,
                })
            } else {
                Err(Error {
                    state,
                    input: value,
                    retry: false,
                })
            }
        } else {
            self.add_ptr_end(end);
            unsafe { self.buffer_write(index, value) };
            let op = MemoryRing::new(cycle, MemoryState::Initialized);
            self.ops[index].store(op, Ordering::Relaxed);
            Ok(())
        }
    }
}
impl<T, const N: usize> Drop for Ring<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}
