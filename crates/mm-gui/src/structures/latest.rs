//! This container holds an atomic pointer to some heap memory, and uses
//! `compare_exchange` to swap the pointer to the new value during sending or
//! `null` during receiving.
//!
//! The main safety invariant is that the shared pointer is always swapped out
//! with some other (possibly `null`) pointer. This means that at most a single
//! thread will ever see the current value, and it can therefore safely take
//! ownership.

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// Implements a "latest message" container. A receiving thread can regularly
/// poll this for values, where it will remove the value from this container and
/// take ownership of it, while a sending thread can overwrite the current value
/// with a new one. Sending a value may block if another thread is sending or
/// receiving, but receiving a value is always non-blocking.
pub struct Latest<T> {
    ptr: Arc<AtomicPtr<T>>,
}

impl<T> Latest<T> {
    /// Create a new, empty `Latest`.
    pub fn new() -> Self {
        Self {
            ptr: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
        }
    }

    /// Send a value, overwriting and returning the previous one held by this
    /// container if it has not been taken yet.
    pub fn send(&self, value: T) -> Option<T> {
        let new = Box::into_raw(Box::new(value));
        let current = self.ptr.swap(new, Ordering::SeqCst);
        if current.is_null() {
            None
        } else {
            // SAFETY: The pointer is not null, which means it must originate
            // from a call to `Box::new()` in `send`. Because `send` and `take`
            // both atomically swap the pointer before reading it, this thread
            // has unique access to this pointer.
            let b = unsafe { Box::from_raw(current) };
            Some(*b)
        }
    }

    /// Take the value currently stored in this container.
    pub fn take(&self) -> Option<T> {
        let current = self.ptr.swap(std::ptr::null_mut(), Ordering::SeqCst);

        if current.is_null() {
            None
        } else {
            // SAFETY: The pointer is not null, which means it must originate
            // from a call to `Box::new()` in `send`. Because `send` and `take`
            // both atomically swap the pointer before reading it, this thread
            // has unique access to this pointer.
            let b = unsafe { Box::from_raw(current) };
            Some(*b)
        }
    }
}

impl<T> Clone for Latest<T> {
    /// Create another `Latest` referencing the same underlying pointer.
    fn clone(&self) -> Self {
        Self {
            ptr: Arc::clone(&self.ptr),
        }
    }
}

impl<T> Drop for Latest<T> {
    fn drop(&mut self) {
        if let Some(ptr) = Arc::get_mut(&mut self.ptr) {
            // If we're here, then all other threads and containers are gone and
            // we're about to disappear, so we can deallocate the value.

            let p = ptr.get_mut();
            if !p.is_null() {
                // SAFETY: The pointer is not null, which means it must
                // originate from a call to `Box::new()` in `send`. Because
                // `send` and `take` both atomically swap the pointer out before
                // reading it, this thread has unique access to this pointer.
                let _ = unsafe { Box::from_raw(*p) };
            }
        }
    }
}
