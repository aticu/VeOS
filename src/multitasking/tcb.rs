//! Provides and manages thread control blocks.

use arch::Context;
use core::cmp::Ordering;
use core::fmt;
use super::Stack;
use super::stack::AccessType;

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

/// A structure representing a thread control block.
pub struct TCB {
    /// The thread ID within the process.
    pub id: u16,
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

impl TCB {
    // TODO: Remove me, I'm only for testing.
    pub fn test(id: u16,
                function: u64,
                arg1: u64,
                arg2: u64,
                arg3: u64,
                arg4: u64,
                arg5: u64,
                arg6: u64)
                -> TCB {
        use memory::*;
        map_page_at(0x10000 * (id as usize + 1),
                    (function - 0xffff800000000000) as usize,
                    READABLE | WRITABLE | EXECUTABLE | USER_ACCESSIBLE);
        map_page_at(0x10000 * (id as usize + 1) + 0x1000,
                    (function - 0xffff800000000000 + 0x1000) as usize,
                    READABLE | WRITABLE | EXECUTABLE | USER_ACCESSIBLE);

        let syscall_stack = Stack::new(0x5000,
                                       STACK_MAX_SIZE,
                                       KERNEL_STACK_AREA_BASE + STACK_OFFSET * (id as usize + 1),
                                       AccessType::KernelOnly);
        let user_stack = Stack::new(0x1000,
                                    0x1000,
                                    0x1000 * (id as usize + 1),
                                    AccessType::UserAccessible);
        let stack_pointer = user_stack.base_stack_pointer as u64;
        let syscall_stack_pointer = syscall_stack.base_stack_pointer;

        TCB {
            id,
            kernel_stack: syscall_stack,
            user_stack,
            state: ThreadState::Ready,
            priority: 1,
            context: Context::test((0x10000 * (id as usize + 1) + function as usize % 0x1000) as
                                   u64,
                                   arg1,
                                   arg2,
                                   arg3,
                                   arg4,
                                   arg5,
                                   arg6,
                                   stack_pointer,
                                   syscall_stack_pointer)
        }
    }

    pub fn idle_tcb() -> TCB {
        let user_stack = Stack::new(0x3000, 0x10000, 0x1000000, AccessType::KernelOnly);
        let stack_pointer = user_stack.base_stack_pointer as u64;

        TCB {
            id: 0,
            kernel_stack: Stack::new(0, 0, 0, AccessType::KernelOnly),
            user_stack,
            state: ThreadState::Ready,
            priority: i32::min_value(),
            context: Context::idle_context(stack_pointer)
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
