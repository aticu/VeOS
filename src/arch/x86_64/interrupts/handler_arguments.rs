//! This module contains the code  of the arguments for the interrupt handlers.

use core::fmt;
use x86_64::registers::flags::Flags;

/// Represents the stack frame of an exception handler.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64
}

impl fmt::Debug for ExceptionStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "RIP: {:x}, RSP: {:x}, RFLAGS: {:?}, Code Segment: {:x}, Stack Segment: {:x}",
               self.instruction_pointer,
               self.stack_pointer,
               Flags::from_bits_truncate(self.cpu_flags as usize),
               self.code_segment,
               self.stack_segment)
    }
}

/// Represents the registers saved on the stack of the exception handler.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SavedRegisters {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64
}

impl fmt::Debug for SavedRegisters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Register state:
rax = {:>016x},
rbx = {:>016x},
rcx = {:>016x},
rdx = {:>016x},
rsi = {:>016x},
rdi = {:>016x},
rbp = {:>016x},
r8  = {:>016x},
r9  = {:>016x},
r10 = {:>016x},
r11 = {:>016x},
r12 = {:>016x},
r13 = {:>016x},
r14 = {:>016x},
r15 = {:>016x}",
               self.rax,
               self.rbx,
               self.rcx,
               self.rdx,
               self.rsi,
               self.rdi,
               self.rbp,
               self.r8,
               self.r9,
               self.r10,
               self.r11,
               self.r12,
               self.r13,
               self.r14,
               self.r15)
    }
}

bitflags! {
    /// Reads the error code of a page fault.
    pub flags PageFaultErrorCode: u64 {
        /// Indicates that the page was present.
        const PRESENT = 1 << 0,
        /// Indicates that the page was written to.
        const WRITE = 1 << 1,
        /// Indicates that the CPL was 3.
        const USERMODE = 1 << 2,
        /// Indicates that invalid values were set in reserved fields.
        const RESERVED_SET = 1 << 3,
        /// Indicates that the fault occured during an instruction fetch.
        const INSTRUCTION_FETCH = 1 << 4
    }
}
