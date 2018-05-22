//! This module handles all scheduling related things.

use super::tcb::SleepTimeSortedTCB;
use super::TCB;
use alloc::binary_heap::BinaryHeap;
use sync::Mutex;

pub mod scheduler;
mod idle;

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
    pub static ref CURRENT_THREAD: Mutex<TCB> = |cpu_id| Mutex::new(TCB::idle_tcb(cpu_id));
}

cpu_local! {
    /// Holds the TCB of the previously running thread during context switches.
    static mut ref OLD_THREAD: Option<TCB> = |_| None;
}

pub struct ActiveThreadReference;

/// Returns a reference to the running thread's TCB.
pub fn get_current_thread() -> ActiveThreadReference {
    ActiveThreadReference
}