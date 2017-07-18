//! Provides saving and restoring of architecture specific execution context.

use super::gdt::{KERNEL_CODE_SEGMENT, KERNEL_DATA_SEGMENT, USER_CODE_SEGMENT, USER_DATA_SEGMENT};
use super::interrupts::handler_arguments::{ExceptionStackFrame, SavedRegisters};

// TODO: Floating point state is not saved yet.
/// Saves the an execution context.
pub struct Context {
    exception_stack_frame: ExceptionStackFrame,
    saved_registers: SavedRegisters
}

impl Context {
    /// Creates a new context from the saved registers and the exception stack
    /// frame.
    pub fn new(saved_registers: SavedRegisters,
               exception_stack_frame: ExceptionStackFrame)
               -> Context {
        Context {
            exception_stack_frame,
            saved_registers
        }
    }

    // TODO: Remove me, I'm only for testing.
    pub fn test(function: u64,
                arg1: u64,
                arg2: u64,
                arg3: u64,
                arg4: u64,
                arg5: u64,
                arg6: u64,
                stack_pointer: u64)
                -> Context {
        let regs = SavedRegisters {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: arg6,
            r8: arg5,
            rbp: 0,
            rdi: arg1,
            rsi: arg2,
            rdx: arg3,
            rcx: arg4,
            rbx: 0,
            rax: 0
        };
        use x86_64::registers::flags::Flags;

        let stack_frame = ExceptionStackFrame {
            instruction_pointer: function,
            code_segment: USER_CODE_SEGMENT.0 as u64,
            cpu_flags: (Flags::IF | Flags::A1).bits() as u64,
            stack_pointer,
            stack_segment: USER_DATA_SEGMENT.0 as u64
        };
        Context {
            exception_stack_frame: stack_frame,
            saved_registers: regs
        }
    }

    pub fn idle_context(stack_pointer: u64) -> Context {
        let regs = SavedRegisters {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rbp: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0
        };
        use x86_64::registers::flags::Flags;

        let stack_frame = ExceptionStackFrame {
            instruction_pointer: ::multitasking::scheduler::idle as u64,
            code_segment: KERNEL_CODE_SEGMENT.0 as u64,
            cpu_flags: (Flags::IF | Flags::A1).bits() as u64,
            stack_pointer,
            stack_segment: KERNEL_DATA_SEGMENT.0 as u64
        };
        Context {
            exception_stack_frame: stack_frame,
            saved_registers: regs
        }
    }

    /// Returns the parts that the context is made from.
    pub fn get_parts(&self) -> (SavedRegisters, ExceptionStackFrame) {
        (self.saved_registers, self.exception_stack_frame)
    }
}

/// Switches the context from the old thread to the current thread.
pub fn switch_context(old_context: &mut Context, new_context: &Context) {
    println!("");
}
