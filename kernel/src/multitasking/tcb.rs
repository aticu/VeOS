//! This module defines thread control blocks (TCBs).

use super::{PCB, PROCESS_LIST, ProcessID, Stack, ThreadID, get_cpu_id};
use super::stack::AccessType;
use arch::Context;
use core::cmp::Ordering;
use core::fmt;
use memory::{KERNEL_STACK_AREA_BASE, STACK_MAX_SIZE, STACK_OFFSET, VirtualAddress};
use x86_64::registers::control_regs::cr3;

/// Represents the possible states a thread can have.
#[derive(PartialEq)]
pub enum ThreadState {
    /// The thread is currently running.
    Running,
    /// The thread is ready to run.
    Ready,
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
    pub context: Context
}

impl fmt::Debug for TCB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Thread <ID: {}>", self.id)
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

        let pcb = process_list
            .get_mut(&self.pid)
            .expect("Process of the thread doesn't exist.");

        self.kernel_stack.resize(0, Some(&mut pcb.address_space));
        self.user_stack.resize(0, Some(&mut pcb.address_space));
    }
}

impl TCB {
    /// Creates a new thread in the given process at the given start address.
    pub fn in_process(pid: ProcessID, id: ThreadID, pc: VirtualAddress, pcb: &mut PCB) -> TCB {
        let kernel_stack = Stack::new(0x2000,
                                      STACK_MAX_SIZE,
                                      KERNEL_STACK_AREA_BASE + STACK_OFFSET * (id as usize),
                                      AccessType::KernelOnly,
                                      Some(&mut pcb.address_space));

        let user_stack = Stack::new(0x1000,
                                    0x1000,
                                    0x1000 * (id as usize), /* TODO: find a more suitable
                                                             * position. */
                                    AccessType::UserAccessible,
                                    Some(&mut pcb.address_space));

        let stack_pointer = user_stack.base_stack_pointer as u64;
        let kernel_stack_pointer = kernel_stack.base_stack_pointer;

        TCB {
            id,
            pid,
            kernel_stack,
            user_stack,
            state: ThreadState::Ready,
            priority: 1,
            context: Context::test(pc as u64,
                                   0,
                                   0,
                                   0,
                                   0,
                                   0,
                                   0,
                                   stack_pointer,
                                   kernel_stack_pointer,
                                   &mut pcb.address_space)
        }
    }

    /// Creates a new TCB for an idle thread.
    pub fn idle_tcb() -> TCB {
        let id = get_cpu_id() as ThreadID;


        // NOTE: This assumes that the idle address space is currently active.
        let user_stack = Stack::new(0x2000,
                                    STACK_MAX_SIZE,
                                    KERNEL_STACK_AREA_BASE + STACK_OFFSET * (id as usize),
                                    AccessType::KernelOnly,
                                    None);

        let stack_pointer = user_stack.base_stack_pointer as u64;

        TCB {
            id,
            pid: 0,
            kernel_stack: Stack::new(0, 0, 0, AccessType::KernelOnly, None),
            user_stack,
            state: ThreadState::Ready,
            priority: i32::min_value(),
            context: Context::idle_context(stack_pointer, cr3().0 as usize)
        }
    }

    /// Returns true if the thread state is dead.
    pub fn is_dead(&self) -> bool {
        self.state == ThreadState::Dead
    }

    /// Sets the thread state to ready if applicable.
    pub fn set_ready(&mut self) {
        if !self.is_dead() {
            self.state == ThreadState::Ready;
        }
    }

    /// Sets the thread state to running.
    pub fn set_running(&mut self) {
        assert!(!self.is_dead(), "Trying to run a dead thread: {:?}", self);

        self.state == ThreadState::Running;
    }
}
