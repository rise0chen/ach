use ach_cell::Cell;
pub use ach_cell::Peek;

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
            if let Some(x) = self.buf[index].take() {
                return Some(x);
            }
        }
        None
    }
    /// push a value to random position, return index
    pub fn push(&self, mut value: T) -> Result<usize, T> {
        for index in 0..self.capacity() {
            if let Err(v) = self.buf[index].set(value) {
                value = v;
            } else {
                return Ok(index);
            }
        }
        Err(value)
    }
    pub fn get(&self, index: usize) -> Option<Peek<T>> {
        self.buf[index].get()
    }
    pub fn swap(&self, index: usize, value: T) -> Result<Option<T>, T> {
        self.buf[index].swap(value)
    }
    pub fn iter(&self) -> ArrayIterator<T, N> {
        ArrayIterator {
            vec: self,
            index: 0,
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
}
impl<'a, T, const N: usize> Iterator for ArrayIterator<'a, T, N> {
    type Item = Option<Peek<'a, T>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.capacity() {
            return None;
        }
        let ret = self.vec.get(self.index);
        self.index += 1;
        Some(ret)
    }
}
