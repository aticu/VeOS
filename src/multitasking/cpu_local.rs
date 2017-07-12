//! Provides the necessary types to handle CPU local values.

use super::get_cpu_id;
use alloc::Vec;
use core::ops::{Deref, DerefMut};

/// A helper type to wrap a CPU local value.
pub struct CPULocal<T>(Vec<T>);

impl<T> Deref for CPULocal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0[get_cpu_id()]
    }
}

impl<T> DerefMut for CPULocal<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0[get_cpu_id()]
    }
}

impl<T> CPULocal<T> {
    /// Creates a new `CPULocal` from the underlying vector.
    ///
    /// # Safety
    /// - Make sure that the vector has as many elements as the CPU number is.
    pub unsafe fn new(vec: Vec<T>) -> CPULocal<T> {
        CPULocal(vec)
    }
}
