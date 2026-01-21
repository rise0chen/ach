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
    pub fn next<'b>(&mut self) -> Option<&'b mut Node<T>> {
        unsafe { Some(self.next?.as_mut()) }
    }
    pub fn take_next<'b>(&mut self) -> Option<&'b mut Node<T>> {
        unsafe { Some(self.next.take()?.as_mut()) }
    }
    pub fn last(&mut self) -> &mut Node<T> {
        let mut now = self;
        loop {
            if let Some(next) = &mut now.next {
                now = unsafe { next.as_mut() };
            } else {
                return now;
            }
        }
    }
    /// Adds a node to the LinkedList.
    ///  
    /// # Safety
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn push(&mut self, node: &mut Node<T>) {
        assert!(node.next.is_none());
        node.next = self.next;
        self.next = Some(node.into());
    }
    /// Adds a list to the LinkedList.
    ///
    /// # Safety
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn push_list(&mut self, node: &mut Node<T>) {
        let last = node.last() as *mut Node<T>;
        (*last).next = self.next;
        self.next = Some(node.into());
    }
    /// remove child which eq node.
    ///
    /// Notice: can't remove head node.
    pub fn remove_node(&mut self, node: &mut Node<T>) -> bool {
        let mut now = self;
        loop {
            if let Some(next) = &mut now.next {
                let next = unsafe { next.as_mut() };
                if core::ptr::eq(next, node) {
                    now.next = next.next;
                    return true;
                }
                now = next;
            } else {
                return false;
            }
        }
    }
    pub fn into_iter(&mut self) -> NodeIter<'_, T> {
        NodeIter { node: Some(self) }
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

pub struct NodeIter<'a, T> {
    node: Option<&'a mut Node<T>>,
}
impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a mut Node<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node.as_mut().map(|x| *x as *mut Node<T>);
        if let Some(node) = node.map(|x| unsafe { &mut *x }) {
            let next = node.take_next();
            self.node = next;
            Some(node)
        } else {
            None
        }
    }
}

pub struct LinkedList<T> {
    head: AtomicPtr<Node<T>>,
    will_remove: [AtomicPtr<Node<T>>; 4],
}
impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> LinkedList<T> {
    #[allow(clippy::declare_interior_mutable_const)]
    const NONE_NODE: AtomicPtr<Node<T>> = AtomicPtr::new(ptr::null_mut());
    pub const fn new() -> Self {
        Self {
            head: Self::NONE_NODE,
            will_remove: [Self::NONE_NODE; 4],
        }
    }

    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Relaxed);
        head.is_null()
    }

    /// delete all entries from LinkedList
    ///
    /// If list is empty, return NULL, otherwise, delete all entries and return the pointer to the first entry.
    #[allow(clippy::mut_from_ref)]
    pub fn take_all(&self) -> Option<&mut Node<T>> {
        unsafe { self.head.swap(ptr::null_mut(), Relaxed).as_mut() }
    }

    /// Adds a node to the LinkedList.
    ///  
    /// # Safety
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn push(&self, node: &mut Node<T>) {
        assert!(node.next.is_none());
        self.head
            .fetch_update(Relaxed, Relaxed, |p| {
                node.next = p.as_mut().map(|x| x.into());
                Some(node)
            })
            .unwrap();
    }
    /// Adds a list to the LinkedList.
    ///
    /// # Safety
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
        let my_node = loop {
            let node = self.will_remove.iter().find(|x| {
                x.compare_exchange(ptr::null_mut(), node as *mut _, Relaxed, Relaxed)
                    .is_ok()
            });
            if let Some(node) = node {
                break node;
            }
            spin_loop::spin();
        };

        let mut root = self.take_all();
        loop {
            while let Some(node) = &mut root {
                let next = node.take_next();
                if !self.will_remove.iter().any(|x| {
                    x.compare_exchange((*node) as *mut _, ptr::null_mut(), Relaxed, Relaxed)
                        .is_ok()
                }) {
                    unsafe { self.push(*node) };
                }
                root = next;
            }
            if !core::ptr::eq(my_node.load(Relaxed), node) {
                return;
            }
            spin_loop::spin();
            let new_root = self.take_all();
            root = new_root;
        }
    }
}
