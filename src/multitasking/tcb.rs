//! Provides and manages thread control blocks.

use super::Stack;
use super::stack::AccessType;
use arch::Context;
use core::cmp::Ordering;

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
    pub id: u16,
    pub syscall_stack: Stack,
    pub user_stack: Stack,
    pub state: ThreadState,
    pub priority: i32,
    pub context: Context
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

        let syscall_stack = Stack::new(0x3000,
                                       STACK_MAX_SIZE,
                                       SYSCALL_STACK_AREA_BASE + STACK_OFFSET * (id as usize + 1),
                                       AccessType::KernelOnly);
        let user_stack = Stack::new(0x1000,
                                    0x1000,
                                    0x1000 * (id as usize + 1),
                                    AccessType::UserAccessible);
        let stack_pointer = user_stack.current_stack_pointer as u64;

        TCB {
            id,
            syscall_stack,
            user_stack,
            state: ThreadState::Ready,
            priority: 1,
            context: Context::test(id as usize,
                                   (0x10000 * (id as usize + 1) + function as usize % 0x1000) as
                                   u64,
                                   arg1,
                                   arg2,
                                   arg3,
                                   arg4,
                                   arg5,
                                   arg6,
                                   stack_pointer)
        }
    }

    pub fn idle_tcb() -> TCB {
        let user_stack = Stack::new(0x3000, 0x10000, 0x1000000, AccessType::KernelOnly);
        let stack_pointer = user_stack.current_stack_pointer as u64;

        TCB {
            id: 0,
            syscall_stack: Stack::new(0, 0, 0, AccessType::KernelOnly),
            user_stack,
            state: ThreadState::Ready,
            priority: i32::min_value(),
            context: Context::idle_context(stack_pointer)
        }
    }
}
