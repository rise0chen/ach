#![no_std]
use ach_cell::Cell;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};

pub struct Node<T> {
    root: AtomicPtr<Node<T>>,
    val: Cell<T>,
    next: AtomicPtr<Node<T>>,
}
impl<T> Node<T> {
    pub const fn new() -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
            val: Cell::new(),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
            val: Cell::new_with(val),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
    fn remove(&self, node: *mut Node<T>) -> bool {
        if let Err(next) = self.next.fetch_update(SeqCst, SeqCst, |p| {
            if p == node {
                Some(unsafe { (*node).next.load(SeqCst) })
            } else {
                None
            }
        }) {
            if !next.is_null() {
                unsafe { (*next).remove(node) }
            } else {
                false
            }
        } else {
            true
        }
    }
    pub fn drop(&mut self) {
        let root = self
            .root
            .fetch_update(SeqCst, SeqCst, |_| Some(ptr::null_mut()))
            .unwrap();
        if !root.is_null() {
            assert!(unsafe { (*root).remove(self) });
        }
    }
}
impl<T> Deref for Node<T> {
    type Target = Cell<T>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        self.drop()
    }
}

pub struct LinkedList<T> {
    root: Node<T>,
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self { root: Node::new() }
    }
    /// Adds a node to the LinkedList.
    /// Safety: This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    /// In addition to this `node` may not be added to another other list before
    /// it is removed from the current one.
    pub unsafe fn push(&self, node: &mut Node<T>) {
        if node
            .root
            .fetch_update(SeqCst, SeqCst, |p| {
                let root = &self.root as *const _ as *mut _;
                if p.is_null() {
                    Some(root)
                } else {
                    assert_eq!(p, root);
                    None
                }
            })
            .is_err()
        {
            return;
        }
        self.root
            .next
            .fetch_update(SeqCst, SeqCst, |p| {
                node.next.store(p, SeqCst);
                Some(node)
            })
            .unwrap();
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the pool is empty.
    pub fn iter(&self) -> LinkedIterator<T> {
        LinkedIterator { node: &self.root }
    }
}

pub struct LinkedIterator<'a, T> {
    node: &'a Node<T>,
}
impl<'a, T> Iterator for LinkedIterator<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node.next.load(SeqCst);
        let next = unsafe { next.as_ref() };
        if let Some(next) = next {
            self.node = next;
        }
        next
    }
}
