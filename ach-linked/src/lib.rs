#![no_std]
use core::marker::PhantomPinned;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, Ordering::Relaxed};

pub struct Node<T> {
    val: T,
    next: Option<NonNull<Node<T>>>,
    _pin: PhantomPinned,
}
impl<T> Node<T> {
    pub const fn new(val: T) -> Self {
        Self {
            val,
            next: None,
            _pin: PhantomPinned,
        }
    }
    pub fn next(&mut self) -> Option<&mut Node<T>> {
        unsafe { Some(self.next?.as_mut()) }
    }
    fn last(&mut self) -> &mut Node<T> {
        let mut now = self;
        loop {
            if let Some(next) = &mut now.next {
                now = unsafe { next.as_mut() };
            } else {
                return now;
            }
        }
    }
    /// remove child which eq node.
    ///
    /// Notice: can't remove head node.
    fn remove_node(&mut self, node: &mut Node<T>) -> bool {
        let mut now = self;
        loop {
            if let Some(next) = &mut now.next {
                let next = unsafe { next.as_mut() };
                if next as *const _ == node as *const _ {
                    now.next = next.next;
                    return true;
                }
                now = next;
            } else {
                return false;
            }
        }
    }
}
impl<T> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

pub struct LinkedList<T> {
    head: AtomicPtr<Node<T>>,
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Relaxed);
        head.is_null()
    }

    /// delete all entries from LinkedList
    ///
    /// If list is empty, return NULL, otherwise, delete all entries and return the pointer to the first entry.
    pub fn take_all(&self) -> Option<&mut Node<T>> {
        unsafe { self.head.swap(ptr::null_mut(), Relaxed).as_mut() }
    }

    /// Adds a node to the LinkedList.
    ///  
    /// Safety:
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn push(&self, node: &mut Node<T>) {
        if node.next.is_some() {
            return;
        }
        self.head
            .fetch_update(Relaxed, Relaxed, |p| {
                node.next = p.as_mut().map(|x| x.into());
                Some(node)
            })
            .unwrap();
    }
    /// Adds a list to the LinkedList.
    ///
    /// Safety:
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn push_list(&self, node: &mut Node<T>) {
        let last = node.last() as *mut Node<T>;
        self.head
            .fetch_update(Relaxed, Relaxed, |p| {
                (*last).next = p.as_mut().map(|x| x.into());
                Some(node)
            })
            .unwrap();
    }

    /// Removes a node from the LinkedList.
    pub fn remove(&self, node: &mut Node<T>) {
        let mut root = self.take_all();
        loop {
            if let Some(root) = &mut root {
                if (*root) as *const _ == node as *const _ {
                    if let Some(next) = root.next() {
                        unsafe { self.push_list(next) };
                    }
                    return;
                } else {
                    if root.remove_node(node) {
                        unsafe { self.push_list(*root) };
                        return;
                    }
                }
            }
            let new_root = self.take_all();
            if let Some(root) = root {
                unsafe { self.push_list(root) };
            }
            spin_loop::spin();
            root = new_root;
        }
    }
}
