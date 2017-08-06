//! Provides the necessary types to handle CPU local values.

use super::get_cpu_id;
use alloc::Vec;
use core::cell::UnsafeCell;
use core::ops::Deref;

/// A helper type to wrap a CPU local value.
pub struct CPULocal<T>(Vec<T>);

impl<T> Deref for CPULocal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0[get_cpu_id()]
    }
}

impl<T> CPULocal<T> {
    /// Creates a new `CPULocal` from the underlying vector.
    ///
    /// # Safety
    /// - Make sure that the vector has as many elements as the CPU number is.
    /// - Should only be called by a macro and not directly.
    pub unsafe fn new(vec: Vec<T>) -> CPULocal<T> {
        CPULocal(vec)
    }
}

/// A helper type to wrap a mutable CPU local value.
pub struct CPULocalMut<T>(UnsafeCell<Vec<T>>);

impl<T> Deref for CPULocalMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.0.get())[get_cpu_id()] }
    }
}

unsafe impl<T: Sync> Sync for CPULocalMut<T> {}

impl<T> CPULocalMut<T> {
    /// Creates a new `CPULocal` from the underlying vector.
    ///
    /// # Safety
    /// - Make sure that the vector has as many elements as the CPU number is.
    /// - Should only be called by a macro and not directly.
    /// - There should be some kind of synchronization for the contained type.
    pub unsafe fn new(vec: Vec<T>) -> CPULocalMut<T> {
        CPULocalMut(UnsafeCell::new(vec))
    }

    /// Sets the value to the specified type.
    ///
    /// # Safety
    /// - Make sure there are no references relying on the value.
    pub unsafe fn set(&self, value: T) {
        (*self.0.get())[get_cpu_id()] = value;
    }

    /// Returns a mutable reference to the contained type.
    ///
    /// # Safety
    /// - Make sure there is only one mutable reference at a time.
    pub unsafe fn as_mut(&self) -> &mut T {
        &mut (*self.0.get())[get_cpu_id()]
    }
}
