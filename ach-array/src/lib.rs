use ach_cell::Cell;
pub use ach_cell::Ref;
use util::Error;

#[derive(Debug)]
pub struct Array<T, const N: usize> {
    buf: [Cell<T>; N],
}
impl<T, const N: usize> Array<T, N> {
    const CAPACITY: usize = N;
    const INIT_ITEM: Cell<T> = Cell::new();
    pub const fn new() -> Self {
        Array {
            buf: [Self::INIT_ITEM; N],
        }
    }
    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }
    pub fn is_empty(&self) -> bool {
        self.buf.iter().all(|x| !x.is_initialized())
    }
    pub fn is_full(&self) -> bool {
        self.buf.iter().all(|x| x.is_initialized())
    }
    pub fn clear(&mut self) {
        self.buf = [Self::INIT_ITEM; N];
    }
    /// pop a value from random position
    pub fn pop(&self) -> Option<T> {
        for index in 0..self.capacity() {
            if let Ok(x) = self.buf[index].try_take() {
                return Some(x);
            }
        }
        None
    }
    /// push a value to random position, return index
    pub fn push(&self, mut value: T) -> Result<usize, T> {
        for index in 0..self.capacity() {
            if let Err(v) = self.buf[index].try_set(value) {
                value = v.input;
            } else {
                return Ok(index);
            }
        }
        Err(value)
    }
    pub fn try_get(&self, index: usize) -> Result<Ref<T>, Error<()>> {
        self.buf[index].try_get()
    }
    /// Notice: `Spin`
    pub fn get(&self, index: usize) -> Option<Ref<T>> {
        self.buf[index].get()
    }
    pub fn try_take(&self, index: usize) -> Result<T, Error<()>> {
        self.buf[index].try_take()
    }
    /// Notice: `Spin`
    pub fn take(&self, index: usize) -> Option<T> {
        self.buf[index].take()
    }
    pub fn try_swap(&self, index: usize, value: T) -> Result<Option<T>, Error<T>> {
        self.buf[index].try_swap(value)
    }
    /// Notice: `Spin`
    pub fn swap(&self, index: usize, value: T) -> Result<Option<T>, T> {
        self.buf[index].swap(value)
    }
    /// It will ignore values which is transient if strict is `false`
    /// Notice: `Spin` if strict
    pub fn iter(&self, strict: bool) -> ArrayIterator<T, N> {
        ArrayIterator {
            vec: self,
            index: 0,
            strict,
        }
    }
}
impl<T, const N: usize> Drop for Array<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}

pub struct ArrayIterator<'a, T, const N: usize> {
    vec: &'a Array<T, N>,
    index: usize,
    strict: bool,
}
impl<'a, T, const N: usize> Iterator for ArrayIterator<'a, T, N> {
    type Item = Ref<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.capacity() {
            return None;
        }
        if self.strict {
            let ret = self.vec.get(self.index);
            self.index += 1;
            if let Some(ret) = ret {
                Some(ret)
            } else {
                self.next()
            }
        } else {
            let ret = self.vec.try_get(self.index);
            self.index += 1;
            if let Ok(ret) = ret {
                Some(ret)
            } else {
                self.next()
            }
        }
    }
}
