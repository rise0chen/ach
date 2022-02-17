#![no_std]
use ach_cell::Cell;
use core::marker::PhantomPinned;
use core::ops::Deref;

pub struct Node<T: 'static> {
    val: Cell<T>,
    prev: Option<&'static Node<T>>,
    next: Option<&'static Node<T>>,
    _pin: PhantomPinned,
}
impl<T> Node<T> {
    pub const fn new() -> Self {
        Self {
            val: Cell::new(),
            prev: None,
            next: None,
            _pin: PhantomPinned,
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            val: Cell::new_with(val),
            prev: None,
            next: None,
            _pin: PhantomPinned,
        }
    }
    unsafe fn set_prev(&self, node: Option<&'static Node<T>>) {
        (*(self as *const Self as *mut Self)).prev = node;
    }
    unsafe fn take_prev(&self) -> Option<&'static Node<T>> {
        (*(self as *const Self as *mut Self)).prev.take()
    }
    unsafe fn set_next(&self, node: Option<&'static Node<T>>) {
        (*(self as *const Self as *mut Self)).next = node;
    }
    unsafe fn take_next(&self) -> Option<&'static Node<T>> {
        (*(self as *const Self as *mut Self)).next.take()
    }
}
impl<T> Deref for Node<T> {
    type Target = Cell<T>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

pub struct LinkedList<T: 'static> {
    head: Option<&'static Node<T>>,
    tail: Option<&'static Node<T>>,
}
impl<T: 'static> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    /// Adds a node to the LinkedList.
    ///
    /// Call this method multiple times is safe.
    pub fn push(&mut self, node: &'static Node<T>) {
        if node.prev.is_some() || node.next.is_some() {
            return;
        }
        if let Some(head) = self.head {
            if head as *const _ == node as *const _ {
                return;
            }
            self.head = Some(node);
            unsafe { head.set_prev(Some(node)) };
            unsafe { node.set_next(Some(head)) };
        } else {
            self.head = Some(node);
            self.tail = Some(node);
        }
    }

    /// Removes a node from the LinkedList.
    pub fn remove(&mut self, node: &'static Node<T>) {
        match unsafe { (node.take_prev(), node.take_next()) } {
            (None, None) => {
                if let Some(head) = self.head {
                    if head as *const _ == node as *const _ {
                        self.head = None;
                        self.tail = None;
                    }
                }
            }
            (None, Some(next)) => {
                self.head = Some(next);
                unsafe { next.set_prev(None) };
            }
            (Some(prev), None) => {
                self.tail = Some(prev);
                unsafe { prev.set_next(None) };
            }
            (Some(prev), Some(next)) => {
                unsafe { prev.set_next(Some(next)) };
                unsafe { next.set_prev(Some(prev)) };
            }
        }
    }

    /// Removes the new element and returns it.
    pub fn pop(&mut self) -> Option<&'static Node<T>> {
        if let Some(tail) = self.tail {
            let prev = unsafe { tail.take_prev() };
            if let Some(prev) = prev {
                unsafe { prev.set_next(None) };
                self.tail = Some(prev);
            } else {
                self.head = None;
                self.tail = None;
            }
            Some(tail)
        } else {
            None
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that f(&e) returns false.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// but not preserves the order of the retained elements.
    pub fn retain(&mut self, mut f: impl FnMut(&'static Node<T>) -> bool) {
        let mut now = self.head;
        while let Some(now_node) = now {
            now = now_node.next;
            if !f(now_node) {
                // Remove it
                self.remove(now_node);
            }
        }
    }
}
