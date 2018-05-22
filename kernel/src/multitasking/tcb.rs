//! This module defines thread control blocks (TCBs).

use super::stack::AccessType;
use super::{ProcessID, Stack, ThreadID, PCB, PROCESS_LIST};
use arch::{self, Architecture};
use core::cmp::Ordering;
use core::fmt;
use core::time::Duration;
use memory::{AddressSpaceManager, VirtualAddress};
use sync::time::Timestamp;

/// Represents the possible states a thread can have.
#[derive(Debug, PartialEq)]
pub enum ThreadState {
    /// The thread is currently running.
    Running,
    /// The thread is ready to run.
    Ready,
    /// The thread is sleeping for a specified amount of time.
    ///
    /// The timestamp corresponds to the time the thread should wake up.
    Sleeping(Timestamp),
    /// The thread is dead.
    Dead
}

/// A structure representing a thread control block (TCB).
pub struct TCB {
    /// The thread ID within the process.
    pub id: ThreadID,
    /// The ID of the process that the thread belongs to.
    pub pid: ProcessID,
    /// The stack used during kernel operations.
    pub kernel_stack: Stack,
    /// The usermode stack.
    pub user_stack: Stack,
    /// The state of the thread.
    pub state: ThreadState,
    /// The priority of the thread.
    pub priority: i32,
    /// The architecture specific context of this thread.
    pub context: <arch::Current as Architecture>::Context
}

impl fmt::Debug for TCB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.pid == 0.into() {
            write!(f, "Thread <IDLE on CPU {}> ({:?})", self.id.0, self.state)
        } else {
            write!(
                f,
                "Thread <{:?}, {:?}> ({:?})",
                self.id, self.pid, self.state
            )
        }
    }
}

impl PartialEq for TCB {
    fn eq(&self, other: &TCB) -> bool {
        // This assumes that thread IDs are unique.
        self.id == other.id
    }
}

impl Eq for TCB {}

impl Ord for TCB {
    fn cmp(&self, other: &TCB) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for TCB {
    fn partial_cmp(&self, other: &TCB) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for TCB {
    fn drop(&mut self) {
        let mut process_list = PROCESS_LIST.lock();

        let drop_pcb = {
            let pcb = process_list
                .get_mut(&self.pid)
                .expect("Process of the thread doesn't exist.");

            pcb.thread_count -= 1;

            self.kernel_stack.resize(0, Some(&mut pcb.address_space));
            self.user_stack.resize(0, Some(&mut pcb.address_space));

            pcb.is_droppable()
        };

        if drop_pcb {
            process_list.remove(&self.pid);
        }
    }
}

impl TCB {
    /// Creates a new thread in the given process at the given start address.
    pub fn in_process(pid: ProcessID, id: ThreadID, pc: VirtualAddress, pcb: &mut PCB) -> TCB {
        TCB::in_process_with_arguments(pid, id, pc, pcb, 0, 0, 0, 0, 0)
    }

    /// Creates a new thread in the given process at the given start address
    /// with the given arguments.
    pub fn in_process_with_arguments(
        pid: ProcessID,
        id: ThreadID,
        pc: VirtualAddress,
        pcb: &mut PCB,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
        arg5: usize
    ) -> TCB {
        let kernel_stack = pcb.address_space.create_kernel_stack(id);

        let user_stack = pcb.address_space.create_user_stack(id);

        let stack_pointer = user_stack.base_stack_pointer;
        let kernel_stack_pointer = kernel_stack.base_stack_pointer;

        TCB {
            id,
            pid,
            kernel_stack,
            user_stack,
            state: ThreadState::Ready,
            priority: 1,
            context: <<arch::Current as Architecture>::Context as arch::Context>::new(
                pc,
                stack_pointer,
                kernel_stack_pointer,
                &mut pcb.address_space,
                arg1,
                arg2,
                arg3,
                arg4,
                arg5
            )
        }
    }

    /// Creates a new TCB for an idle thread.
    pub fn idle_tcb(cpu_id: usize) -> TCB {
        let id: ThreadID = cpu_id.into();

        // NOTE: This assumes that the idle address space is currently active.
        let kernel_stack = <<arch::Current as Architecture>::AddressSpaceManager as AddressSpaceManager>::create_idle_stack(cpu_id);

        let stack_pointer = kernel_stack.base_stack_pointer;

        TCB {
            id,
            pid: 0.into(),
            kernel_stack,
            user_stack: Stack::new(
                0,
                0,
                VirtualAddress::default(),
                AccessType::KernelOnly,
                None
            ),
            state: ThreadState::Ready,
            priority: i32::min_value(),
            context: <<arch::Current as Architecture>::Context as arch::Context>::idle(
                stack_pointer
            )
        }
    }

    /// Returns true if the thread state is dead.
    pub fn is_dead(&self) -> bool {
        let process_list = PROCESS_LIST.lock();
        let process = process_list
            .get(&self.pid)
            .expect("Process of the thread doesn't exist.");

        self.state == ThreadState::Dead || process.is_dead()
    }

    /// Returns true if the thread state is running.
    pub fn is_running(&self) -> bool {
        self.state == ThreadState::Running
    }

    /// Sets the thread state to ready if applicable.
    pub fn set_ready(&mut self) {
        if !self.is_dead() {
            self.state = ThreadState::Ready;
        }
    }

    /// Sets the thread state to running.
    pub fn set_running(&mut self) {
        debug_assert!(!self.is_dead(), "Trying to run a dead thread: {:?}", self);

        self.state = ThreadState::Running;
    }

    /// Marks this thread as dead.
    ///
    /// This will cause the scheduler to not schedule it anymore and drop it.
    pub fn kill(&mut self) {
        self.state = ThreadState::Dead;
    }

    /// Returns the time quantum this process should run.
    pub fn get_quantum(&self) -> Duration {
        Duration::from_millis(150)
    }
}

/// A TCB that is sorted by its sleep time (shortest first).
pub struct SleepTimeSortedTCB(pub TCB);

impl SleepTimeSortedTCB {
    /// Returns the sleep time for this TCB.
    pub fn get_wake_time(&self) -> Timestamp {
        match self.0.state {
            ThreadState::Sleeping(time) => time,
            _ => unreachable!()
        }
    }
}

impl PartialEq for SleepTimeSortedTCB {
    fn eq(&self, other: &SleepTimeSortedTCB) -> bool {
        self.get_wake_time() == other.get_wake_time()
    }
}

impl Eq for SleepTimeSortedTCB {}

impl Ord for SleepTimeSortedTCB {
    fn cmp(&self, other: &SleepTimeSortedTCB) -> Ordering {
        other.get_wake_time().cmp(&self.get_wake_time())
    }
}

impl PartialOrd for SleepTimeSortedTCB {
    fn partial_cmp(&self, other: &SleepTimeSortedTCB) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
