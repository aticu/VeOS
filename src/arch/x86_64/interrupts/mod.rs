//! Handles interrupts on the x86_64 architecture.

mod idt;
#[macro_use]
mod idt_entry;
mod handler_arguments;

use self::idt::Idt;
use self::idt_entry::IdtEntry;
use self::handler_arguments::*;
use x86_64::registers::control_regs;

lazy_static! {
    /// The interrupt descriptor table used by the kernel.
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        
        idt.divide_by_zero = handler_without_error_code!(divide_by_zero_handler);
        idt.breakpoint = handler_without_error_code!(breakpoint_handler);
        idt.page_fault = handler_with_error_code!(page_fault_handler);
        idt.page_fault.set_interrupt_gate();

        idt
    };
}

/// Initializes interrupts on the x86_64 architecture.
pub fn init() {
    assert_has_not_been_called!("Interrupts should only be initialized once.");

    unsafe {
        IDT.load();
    }
}

/// The divide by zero exception handler of the kernel.
extern "C" fn divide_by_zero_handler(stack_frame: &mut ExceptionStackFrame, regs: &mut SavedRegisters) {
    println!("DIVIDE BY ZERO!");
    println!("{:?}", stack_frame);
    println!("{:?}", regs);
    loop {}
}

/// The breakpoint exception handler of the kernel.
extern "C" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame, regs: &mut SavedRegisters) {
    println!("BREAKPOINT!");
    println!("{:?}", stack_frame);
    println!("{:?}", regs);
    loop {}
}

/// The page fault handler of the kernel.
extern "C" fn page_fault_handler(stack_frame: &mut ExceptionStackFrame, regs: &mut SavedRegisters, error_code: u64) {
    println!("PAGE FAULT!");
    println!("Address: {:x}", control_regs::cr2());
    println!("Error code: {:?}", PageFaultErrorCode::from_bits_truncate(error_code));
    println!("{:?}", stack_frame);
    println!("{:?}", regs);
    loop {}
}
