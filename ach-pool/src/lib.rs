use std::ops::Index;
use ach_option::AchOption;

pub struct Pool<T, const N: usize> {
    buf: [AchOption<T>; N],
}
impl<T, const N: usize> Pool<T, N> {
    const CAPACITY: usize = N;
    const INIT_ITEM: AchOption<T> = AchOption::new();
    pub const fn new() -> Self {
        Pool {
            buf: [Self::INIT_ITEM; N],
        }
    }
    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }
    pub fn is_empty(&self) -> bool {
        self.buf.iter().all(|x| x.is_none())
    }
    pub fn is_full(&self) -> bool {
        self.buf.iter().all(|x| x.is_some())
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
}
impl<T, const N: usize> Index<usize> for Pool<T, N> {
    type Output = AchOption<T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}
