#![no_std]
use ach_cell::{Cell, Ref};
use core::marker::PhantomPinned;
use core::ops::Deref;
use util::*;

pub struct Node<T: 'static> {
    val: Cell<T>,
    next: Cell<&'static Node<T>>,
    _pin: PhantomPinned,
}
impl<T> Node<T> {
    pub const fn new() -> Self {
        Self {
            val: Cell::new(),
            next: Cell::new(),
            _pin: PhantomPinned,
        }
    }
    pub const fn new_with(val: T) -> Self {
        Self {
            val: Cell::new_with(val),
            next: Cell::new(),
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
    head: Cell<&'static Node<T>>,
}
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self { head: Cell::new() }
    }
    pub fn head(&self) -> Result<Ref<&'static Node<T>>, Error<()>> {
        self.head.get()
    }
    pub fn tail(&self) -> Result<Ref<&'static Node<T>>, Error<()>> {
        let mut tail = Err(Error {
            state: MemoryState::Uninitialized,
            input: (),
            retry: false,
        });
        let mut now = self.head();
        while let Ok(now_node) = now {
            now = now_node.next.get();
            tail = Ok(now_node);
        }
        tail
    }

    /// Adds a node to the LinkedList.
    ///
    /// Call this method multiple times is safe.
    pub fn insert(&self, node: &'static Node<T>) {
        if node.next.get().is_ok() {
            return;
        }
        let _ = self.head.fetch_update(|old| {
            if let Some(old) = old {
                node.next.set(old).unwrap();
            }
            Some(Some(node))
        });
    }

    /// append a list to the LinkedList.
    pub fn append(&self, list: LinkedList<T>) {
        let tail = if let Ok(tail) = list.tail() {
            tail
        } else {
            return;
        };
        let head = list.head.take().unwrap().unwrap();
        let _ = self.head.fetch_update(|old| {
            if let Some(old) = old {
                tail.next.set(old).unwrap();
            }
            Some(Some(head))
        });
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that f(&e) returns false.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// but not preserves the order of the retained elements.
    pub fn retain(&self, mut f: impl FnMut(&'static Node<T>) -> bool) {
        let mut prev = None;
        let mut now = self.head.get();
        while let Ok(now_node) = now {
            if f(&*now_node) {
                // Skip to next
                now = now_node.next.get();
                prev = Some(now_node);
            } else {
                // Remove it
                if let Some(p) = prev {
                    let _ = p.next.fetch_update(|_| {
                        let next = unwrap(|_| now_node.next.take(), ());
                        Some(next)
                    });
                    now = p.next.get();
                    prev = Some(p);
                } else {
                    let _ = self.head.fetch_update(|_| {
                        let next = unwrap(|_| now_node.next.take(), ());
                        Some(next)
                    });
                    now = self.head.get();
                    prev = None;
                };
            }
        }
    }

    /// Remove all node and returns a iter.
    pub fn iter(&self) -> LinkedIterator<T> {
        LinkedIterator {
            next: unwrap(|_| self.head.take(),()),
        }
    }
}

pub struct LinkedIterator<T: 'static> {
    next: Option<&'static Node<T>>,
}
impl<T> Iterator for LinkedIterator<T> {
    type Item = &'static Node<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next.take()?;
        self.next = unwrap(|_| ret.next.take(),());
        Some(ret)
    }
}
