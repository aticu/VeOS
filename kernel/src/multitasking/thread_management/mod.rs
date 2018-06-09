//! This module handles all scheduling related things.

use super::tcb::SleepTimeSortedTCB;
use super::TCB;
use alloc::binary_heap::BinaryHeap;
use core::ops::{Deref, DerefMut};
use sync::mutex::MutexGuard;
use sync::Mutex;

mod idle;
pub mod scheduler;

pub use self::idle::idle;
pub use self::scheduler::{after_context_switch, schedule_next_thread};

cpu_local! {
    pub static ref READY_LIST: Mutex<BinaryHeap<TCB>> = |_| Mutex::new(BinaryHeap::new());
}

lazy_static! {
    pub static ref SLEEPING_LIST: Mutex<BinaryHeap<SleepTimeSortedTCB>> =
        Mutex::new(BinaryHeap::new());
}

cpu_local! {
    /// Holds the TCB of the currently running thread.
    static ref CURRENT_THREAD: Mutex<TCB> = |cpu_id| Mutex::new(TCB::idle_tcb(cpu_id));
}

cpu_local! {
    /// Holds the TCB of the previously running thread during context switches.
    static mut ref OLD_THREAD: Option<TCB> = |_| None;
}

/// References a TCB.
///
/// Values of this type hold a lock that allows access to a TCB.
pub struct ThreadLock<'a> {
    /// The underlying mutex guard of the thread lock.
    guard: MutexGuard<'a, TCB>
}

impl<'a> Deref for ThreadLock<'a> {
    type Target = TCB;

    fn deref(&self) -> &TCB {
        self.guard.deref()
    }
}

impl<'a> DerefMut for ThreadLock<'a> {
    fn deref_mut(&mut self) -> &mut TCB {
        self.guard.deref_mut()
    }
}

/// Returns a reference to the running thread's TCB.
pub fn get_current_thread<'a>() -> ThreadLock<'a> {
    ThreadLock {
        guard: CURRENT_THREAD.lock()
    }
}
