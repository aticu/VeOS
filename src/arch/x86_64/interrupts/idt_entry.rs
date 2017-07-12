//! Contains the code to handle an IDT-entry.

use super::handler_arguments::{ExceptionStackFrame, SavedRegisters};
use super::super::gdt::KERNEL_CODE_SEGMENT;
use core::fmt;
use core::marker::PhantomData;

/// A trait to differentiate between IDT entries with and without error codes
/// at the type level.
pub trait ErrorCode {}

/// Represents a handler function for an interrupt with an error code.
pub type WithErrorCode = extern "C" fn(&mut ExceptionStackFrame, &mut SavedRegisters, u64);

impl ErrorCode for WithErrorCode {}

/// Represents a handler function for an interrupt without an error code.
pub type WithoutErrorCode = extern "C" fn(&mut ExceptionStackFrame, &mut SavedRegisters);

impl ErrorCode for WithoutErrorCode {}

/// Represents an entry in the IDT.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry<T: ErrorCode> {
    /// Bits 0-15 of the handler function address.
    fn_ptr_low: u16,
    /// The code segment selector in the GDT.
    gdt_selector: u16,
    /// The options of the IDT entry.
    options: IdtEntryOptions,
    /// Bits 16-31 of the handler function address.
    fn_ptr_mid: u16,
    /// Bits 32-63 of the handler function address.
    fn_ptr_high: u32,
    /// Reserved. Set to zero.
    reserved: u32,
    error_code: PhantomData<T>
}

impl<T: ErrorCode> fmt::Debug for IdtEntry<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ptr: u64 = self.fn_ptr_low as u64 | (self.fn_ptr_mid as u64) << 16 |
                       (self.fn_ptr_high as u64) << 32;
        write!(f,
               "IdtEntry: (Function: {:x}, GDT selector: {}, {:?})",
               ptr,
               self.gdt_selector,
               self.options)
    }
}

/// Models the available options for an IDT entry.
#[derive(Clone, Copy)]
struct IdtEntryOptions(u16);

bitflags! {
    /// The possible options for the `IdtEntryOptions`.
    flags Options: u16 {
        /// The index into the interrupt stack table.
        const STACK_TABLE_INDEX = 0b0000000000000111,
        /// These fields are reserved. They should be set to 0.
        const RESERVED          = 0b0000000011111000,
        /// This is a trap gate.
        ///
        /// That means that interrupts are enabled while the handler function is called.
        /// If this bit is 0 interrupts will be disabled.
        const TRAP_GATE         = 0b0000000100000000,
        /// These bits must be one.
        const MUST_BE_ONE       = 0b0000111000000000,
        /// This bit must be zero.
        const MUST_BE_ZERO      = 0b0001000000000000,
        /// The minimum privilege level required to run the handler function.
        const DPL               = 0b0110000000000000,
        /// Indicates whether or not this IDT entry is present.
        const PRESENT           = 0b1000000000000000
    }
}

impl fmt::Debug for IdtEntryOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let gate_type = if self.0 & TRAP_GATE.bits() > 0 {
            "Trap gate"
        } else {
            "Interrupt gate"
        };
        let present = if self.0 & PRESENT.bits() > 0 {
            "Present"
        } else {
            "Not present"
        };
        write!(f,
               "Stack: {}, {}, DPL: {}, {}",
               self.0 & STACK_TABLE_INDEX.bits(),
               gate_type,
               (self.0 & DPL.bits()) >> 13,
               present)
    }
}

impl IdtEntryOptions {
    /// Creates minimal `IdtEntryOptions`.
    fn new() -> IdtEntryOptions {
        IdtEntryOptions(MUST_BE_ONE.bits())
    }

    /// Sets the interrupt stack table index.
    fn set_stack_table_index(&mut self, index: u16) -> &mut IdtEntryOptions {
        self.0 &= !STACK_TABLE_INDEX.bits();
        self.0 |= (index << 0) & STACK_TABLE_INDEX.bits();

        self
    }

    /// Sets the descriptor privilege level.
    fn set_dpl(&mut self, dpl: u16) {
        self.0 &= !DPL.bits();
        self.0 |= (dpl << 13) & DPL.bits();
    }

    /// Sets this entry as used.
    fn set_used(&mut self) {
        self.0 |= PRESENT.bits();
    }

    /// Sets this entry as a trap gate.
    fn set_trap_gate(&mut self) {
        self.0 |= TRAP_GATE.bits();
    }

    /// Sets this entry as an interrupt gate.
    fn set_interrupt_gate(&mut self) {
        self.0 &= !TRAP_GATE.bits();
    }
}

/// Saves the general purpose registers onto the stack.
macro_rules! save_registers {
    () => {
        asm!("push rax
        push rbx
        push rcx
        push rdx
        push rsi
        push rdi
        push rbp
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15"
        : : : : "intel", "volatile");
    }
}

/// Restores the general purpose registers from the stack.
macro_rules! restore_registers {
    () => {
        asm!("pop r15
        pop r14
        pop r13
        pop r12
        pop r11
        pop r10
        pop r9
        pop r8
        pop rbp
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rbx
        pop rax"
        : : : : "intel", "volatile");
    }
}

/// Make an interrupt handler from the given function.
macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() {
            unsafe {
                save_registers!();

                asm!("mov rdi, rsp
                      mov rsi, rdi // rsi = &mut SavedRegisters
                      add rdi, 8 * 15 // rdi = &mut ExceptionStackFrame
                      call $0"
                      : : "i"($name as $crate::arch::interrupts::idt_entry::WithoutErrorCode)
                      : "rdi", "rsi" : "intel", "volatile");

                restore_registers!();

                asm!("iretq" : : : : "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }};
    (Errorcode: $name: ident) => {{
        #[naked]
        extern "C" fn wrapper() {
            unsafe {
                save_registers!();

                asm!("mov rdx, [rsp + 8 * 15] // rdx = error_code
                      mov rdi, rsp
                      mov rsi, rdi // rsi = &mut SavedRegisters
                      add rdi, 8 * 16 // rdi = &mut ExceptionStackFrame
                      sub rsp, 8 // Align the stack pointer to 16 bytes.
                      call $0
                      add rsp, 8 // Undo the alignment."
                      : : "i"($name as $crate::arch::interrupts::idt_entry::WithErrorCode)
                      : "rdi", "rsi" : "intel", "volatile");

                restore_registers!();

                asm!("add rsp, 8 // Pop the error code.
                      iretq" : : : "rsp" : "intel", "volatile");

                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

/// Create a new interrupt handler without an error code.
macro_rules! handler_without_error_code {
    ($handler: ident) => {{
        IdtEntry::<$crate::arch::interrupts::idt_entry::WithoutErrorCode>::new(handler!($handler))
    }}
}

/// Create a new interrupt handler with an error code.
macro_rules! handler_with_error_code {
    ($handler: ident) => {{
        IdtEntry::<$crate::arch::interrupts::idt_entry::WithErrorCode>
            ::new(handler!(Errorcode: $handler))
    }}
}

impl<T: ErrorCode> IdtEntry<T> {
    /// Creates a new IDT-entry.
    pub fn new(handler: extern "C" fn()) -> IdtEntry<T> {
        let fn_ptr = handler as u64;
        let mut options = IdtEntryOptions::new();
        let segment_selector = KERNEL_CODE_SEGMENT.0;
        options.set_used();
        let mut entry = IdtEntry {
            fn_ptr_low: fn_ptr as u16,
            gdt_selector: segment_selector,
            options,
            fn_ptr_mid: (fn_ptr >> 16) as u16,
            fn_ptr_high: (fn_ptr >> 32) as u32,
            reserved: 0,
            error_code: PhantomData
        };

        entry
            .set_stack_table_index(0)
            .set_dpl(0)
            .set_trap_gate();

        entry
    }

    /// Represents an unused IDT-entry.
    pub fn unused() -> IdtEntry<T> {
        IdtEntry {
            fn_ptr_low: 0,
            gdt_selector: 0,
            options: IdtEntryOptions::new(),
            fn_ptr_mid: 0,
            fn_ptr_high: 0,
            reserved: 0,
            error_code: PhantomData
        }
    }

    /// Sets the interrupt stack table index.
    pub fn set_stack_table_index(&mut self, index: u16) -> &mut IdtEntry<T> {
        self.options.set_stack_table_index(index);

        self
    }

    /// Sets the descriptor privilege level.
    pub fn set_dpl(&mut self, dpl: u16) -> &mut IdtEntry<T> {
        self.options.set_dpl(dpl);

        self
    }

    /// Sets this entry as a trap gate.
    pub fn set_trap_gate(&mut self) -> &mut IdtEntry<T> {
        self.options.set_trap_gate();

        self
    }

    /// Sets this entry as an interrupt gate.
    pub fn set_interrupt_gate(&mut self) -> &mut IdtEntry<T> {
        self.options.set_interrupt_gate();

        self
    }
}
