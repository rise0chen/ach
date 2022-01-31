use std::ops::Index;

use ach_cell::Cell;
pub use ach_cell::Ref;

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
            if let Ok(Some(x)) = self.buf[index].try_take() {
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
impl<T, const N: usize> Index<usize> for Array<T, N> {
    type Output = Cell<T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
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
        let ret = if self.strict {
            self.vec[self.index].get()
        } else {
            self.vec[self.index].try_get()
        };
        self.index += 1;
        if let Ok(ret) = ret {
            Some(ret)
        } else {
            self.next()
        }
    }
}
