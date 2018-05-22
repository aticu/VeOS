//! This module defines a process control block (PCB).

use alloc::BTreeMap;
use arch::schedule;
use core::cmp::max;
use core::ops::{Deref, DerefMut};
use memory::address_space::AddressSpace;
use multitasking::{get_cpu_num, ProcessID, ThreadID, PROCESS_LIST};
use multitasking::thread_management::CURRENT_THREAD;
use sync::mutex::MutexGuard;

/// Represents the states a process can have.
#[derive(Debug, PartialEq)]
enum ProcessState {
    /// The process is currently active.
    Active,
    /// The process is dead.
    Dead
}

/// A process control block (PCB) holds all data required to manage a process.
pub struct PCB {
    /// The address space of the process.
    pub address_space: AddressSpace,
    /// The amount of currently existing threads within this process.
    pub thread_count: usize,
    /// The state of the process.
    state: ProcessState,
    /// The highest ID of a thread within this process.
    highest_thread_id: ThreadID
}

impl Drop for PCB {
    fn drop(&mut self) {
        assert!(self.is_droppable());
    }
}

impl PCB {
    /// Creates a new PCB with the given parameters.
    pub fn new(address_space: AddressSpace) -> PCB {
        PCB {
            address_space,
            thread_count: 1,
            highest_thread_id: 0.into(),
            state: ProcessState::Active
        }
    }

    /// Creates a pcb for the idle threads.
    pub fn idle_pcb() -> PCB {
        assert_has_not_been_called!("There should only be one idle PCB.");
        PCB {
            address_space: AddressSpace::idle_address_space(),
            thread_count: get_cpu_num(),
            highest_thread_id: (get_cpu_num() - 1).into(),
            state: ProcessState::Active
        }
    }

    /// Finds an ID for a new thread in this process.
    pub fn find_thread_id(&self) -> Option<ThreadID> {
        // UNOPTIMIZED
        self.highest_thread_id.0.checked_add(1).map(|new_id| new_id.into())
    }

    /// Adds a thread to the process.
    pub fn add_thread(&mut self, id: ThreadID) {
        self.highest_thread_id = max(self.highest_thread_id, id);

        self.thread_count += 1;
    }

    /// Returns true if the process is dead.
    pub fn is_dead(&self) -> bool {
        self.state == ProcessState::Dead
    }

    /// Marks this process as dead.
    ///
    /// This will cause the scheduler to not schedule any threads of this
    /// process anymore.
    pub fn kill(&mut self) {
        self.state = ProcessState::Dead;
    }

    /// Marks this process as dead.
    ///
    /// This will cause the scheduler to not schedule any threads of this
    /// process anymore. The scheduler will be invoked immediately.
    pub fn kill_immediately(&mut self) -> ! {
        self.state = ProcessState::Dead;
        schedule();
        unreachable!();
    }

    /// Determines if this process can be dropped.
    pub fn is_droppable(&self) -> bool {
        self.thread_count == 0
    }
}

/// Represents a lock on the process list.
pub struct ProcessLock<'a> {
    /// The mutex guard that keeps the lock on the list.
    guard: MutexGuard<'a, BTreeMap<ProcessID, PCB>>,
    /// The key to get the proccess out of the list.
    key: ProcessID
}

impl<'a> Deref for ProcessLock<'a> {
    type Target = PCB;

    fn deref(&self) -> &PCB {
        self.guard.get(&self.key).expect("Process not existing.")
    }
}

impl<'a> DerefMut for ProcessLock<'a> {
    fn deref_mut(&mut self) -> &mut PCB {
        self.guard
            .get_mut(&self.key)
            .expect("Process not existing.")
    }
}

/// Returns a lock of the current process.
pub fn get_current_process<'a>() -> ProcessLock<'a> {
    let pid = CURRENT_THREAD.lock().pid;
    ProcessLock {
        guard: PROCESS_LIST.lock(),
        key: pid
    }
}
