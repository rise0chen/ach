#![no_std]
use ach_cell::{Cell, Ref};
use core::marker::PhantomPinned;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering::SeqCst};
use util::unwrap;

pub struct Node<T: 'static> {
    root: AtomicPtr<LinkedList<T>>,
    val: Cell<T>,
    next: Cell<&'static Node<T>>,
    _pin: PhantomPinned,
}
impl<T> Node<T> {
    pub const fn new() -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
            val: Cell::new(),
            next: Cell::new(),
            _pin: PhantomPinned,
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            root: AtomicPtr::new(ptr::null_mut()),
            val: Cell::new_with(val),
            next: Cell::new(),
            _pin: PhantomPinned,
        }
    }
    pub fn drop(&mut self) {
        if let Ok(root) = self.root.fetch_update(SeqCst, SeqCst, |x| {
            if x.is_null() {
                None
            } else {
                Some(ptr::null_mut())
            }
        }) {
            unsafe { (*root).remove(self) };
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

pub struct LinkedList<T: 'static> {
    root: Node<T>,
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            root: Node {
                root: AtomicPtr::new(ptr::null_mut()),
                val: Cell::new(),
                next: Cell::new(),
                _pin: PhantomPinned,
            },
        }
    }
    /// Adds a node to the LinkedList.
    /// 
    /// Safety: This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    /// 
    /// In addition to this `node` may not be added to another other list before
    /// it is removed from the current one.
    pub unsafe fn push(&self, node: &mut Node<T>) {
        if let Err(root) = node.root.fetch_update(SeqCst, SeqCst, |x| {
            if x.is_null() {
                Some(self as *const _ as *mut _)
            } else {
                None
            }
        }) {
            assert_eq!(root as *const _, self as *const _);
            return;
        }
        let old = unwrap(|node| self.root.next.replace(node), &*node);
        if let Some(old) = old {
            node.next.replace(old).unwrap();
        }
    }

    /// Removes the given node from the list.
    /// 
    /// The node must be a member of this list, and not a member of any other
    /// heap.
    pub unsafe fn remove(&self, node: &mut Node<T>) {
        let mut now: &Node<T> = &self.root;
        let mut next_node = now.next.get();
        loop {
            let next: &Node<T> = if let Ok(next) = next_node {
                *next
            } else {
                break;
            };
            if next as *const _ == node as *const _ {
                let new = node.next.get();
                let new_val = if let Ok(v) = &new { Some(**v) } else { None };
                let ret = now.next.fetch_update(|_| Some(new_val));
                assert!(ret.is_ok());
                return;
            }
            now = next;
            next_node = now.next.get();
        }
        unreachable!()
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the pool is empty.
    pub fn iter(&self) -> LinkedIterator<T> {
        LinkedIterator {
            next: self.root.next.get().map_or_else(|_| None, |v| Some(v)),
        }
    }
}

pub struct LinkedIterator<'a, T: 'static> {
    next: Option<Ref<'a, &'static Node<T>>>,
}
impl<'a, T> Iterator for LinkedIterator<'a, T> {
    type Item = Ref<'a, &'static Node<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next.take()?;
        self.next = ret.next.get().map_or_else(|_| None, |v| Some(v));
        Some(ret)
    }
}
