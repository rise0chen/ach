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
    pub fn take_next(&mut self) -> Option<&mut Node<T>> {
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
    pub fn push(&mut self, node: &mut Node<T>) {
        self.last().next = Some(node.into());
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
    will_remove: [AtomicPtr<Node<T>>; 4],
}
impl<T> LinkedList<T> {
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
                let next = node.take_next().map(|x| x as *mut Node<T>);
                if self
                    .will_remove
                    .iter()
                    .find(|x| {
                        x.compare_exchange((*node) as *mut _, ptr::null_mut(), Relaxed, Relaxed)
                            .is_ok()
                    })
                    .is_none()
                {
                    unsafe { self.push(*node) };
                }
                drop(node);
                root = next.map(|x| unsafe { &mut *x });
            }
            if my_node.load(Relaxed) != node as *mut _ {
                return;
            }
            spin_loop::spin();
            let new_root = self.take_all();
            root = new_root;
        }
    }
}
