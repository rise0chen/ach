#![no_std]
use ach_cell::Cell;
use core::marker::PhantomPinned;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};

pub struct Node<T: 'static> {
    val: Cell<T>,
    next: AtomicPtr<Node<T>>,
    _pin: PhantomPinned,
}
impl<T> Node<T> {
    pub const fn new() -> Self {
        Self {
            val: Cell::new(),
            next: AtomicPtr::new(ptr::null_mut()),
            _pin: PhantomPinned,
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            val: Cell::new_with(val),
            next: AtomicPtr::new(ptr::null_mut()),
            _pin: PhantomPinned,
        }
    }
}
impl<T> Deref for Node<T> {
    type Target = Cell<T>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

pub struct LinkedList<T: 'static> {
    root: AtomicPtr<Node<T>>,
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
        }
    }
    /// Adds a node to the LinkedList.
    ///
    /// Call this method multiple times is safe.
    pub fn push(&self, node: &'static mut Node<T>) {
        if !node.next.load(SeqCst).is_null() {
            return;
        }
        let _ = self.root.fetch_update(SeqCst, SeqCst, |old| {
            node.next.store(old, SeqCst);
            Some(node)
        });
    }

    /// Remove all node and returns a iter.
    pub fn iter(&self) -> LinkedIterator<T> {
        LinkedIterator {
            next: unsafe { self.root.swap(ptr::null_mut(), SeqCst).as_mut() },
        }
    }
}

pub struct LinkedIterator<T: 'static> {
    next: Option<&'static mut Node<T>>,
}
impl<T> Iterator for LinkedIterator<T> {
    type Item = &'static mut Node<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next.take()?;
        self.next = unsafe { ret.next.swap(ptr::null_mut(), SeqCst).as_mut() };
        Some(ret)
    }
}
