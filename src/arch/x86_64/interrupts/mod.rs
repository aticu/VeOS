//! Handles interrupts on the x86_64 architecture.

mod idt;
#[macro_use]
mod idt_entry;
pub mod handler_arguments;
mod lapic;
mod ioapic;

use self::handler_arguments::*;
use self::idt::Idt;
use self::idt_entry::IdtEntry;
use x86_64::registers::control_regs;

// TODO: When using a lazy static here, the system slows down a lot. Check out
// why.
cpu_local! {
    /// The interrupt descriptor table used by the kernel.
    static ref IDT: Idt = {
        let mut idt = Idt::new();

        idt.divide_by_zero = handler_without_error_code!(divide_by_zero_handler);
        idt.breakpoint = handler_without_error_code!(breakpoint_handler);
        idt.page_fault = handler_with_error_code!(page_fault_handler);
        idt.page_fault.set_interrupt_gate();
        idt.interrupts[0] = handler_without_error_code!(timer_handler);
        idt.interrupts[0].set_interrupt_gate();
        idt.interrupts[1] = handler_without_error_code!(irq1_handler);
        idt.interrupts[1].set_interrupt_gate();
        // The spurious interrupt handler.
        idt.interrupts[0xf] = handler_without_error_code!(empty_handler);

        idt
    };
}

/// Initializes interrupts on the x86_64 architecture.
pub fn init() {
    assert_has_not_been_called!("Interrupts should only be initialized once.");

    unsafe {
        IDT.load();
    }

    lapic::init();
    lapic::set_periodic_timer(150);

    ioapic::init();
}

/// The divide by zero exception handler of the kernel.
extern "C" fn divide_by_zero_handler(stack_frame: &mut ExceptionStackFrame,
                                     regs: &mut SavedRegisters) {
    println!("DIVIDE BY ZERO!");
    println!("{:?}", stack_frame);
    println!("{:?}", regs);
    loop {}
}

/// The breakpoint exception handler of the kernel.
extern "C" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame,
                                 regs: &mut SavedRegisters) {
    use multitasking::CURRENT_THREAD;
    use multitasking::schedule;
    schedule();
    let context = &CURRENT_THREAD.lock().context;
    let (registers, exception_stack_frame) = context.get_parts();
    *regs = registers;
    *stack_frame = exception_stack_frame;
}

/// The page fault handler of the kernel.
extern "C" fn page_fault_handler(stack_frame: &mut ExceptionStackFrame,
                                 regs: &mut SavedRegisters,
                                 error_code: u64) {
    println!("PAGE FAULT!");
    println!("Address: {:x}", control_regs::cr2());
    println!("Error code: {:?}",
             PageFaultErrorCode::from_bits_truncate(error_code));
    println!("Page flags: {:?}",
             super::memory::get_page_flags(control_regs::cr2().0));
    println!("{:?}", stack_frame);
    println!("{:?}", regs);
    loop {}
}

extern "C" fn empty_handler(_: &mut ExceptionStackFrame, _: &mut SavedRegisters) {}

extern "C" fn timer_handler(stack_frame: &mut ExceptionStackFrame, regs: &mut SavedRegisters) {
    use multitasking::CURRENT_THREAD;
    use multitasking::schedule;
    use arch::Context;

    let context = Context::new(*regs, *stack_frame);
    let mut current_thread = CURRENT_THREAD.lock();
    current_thread.context = context;
    drop(current_thread);

    schedule();

    let (registers, exception_stack_frame) = CURRENT_THREAD.lock().context.get_parts();
    *regs = registers;
    *stack_frame = exception_stack_frame;

    println!("!");
    lapic::signal_eoi();
}

extern "C" fn irq1_handler(_: &mut ExceptionStackFrame, _: &mut SavedRegisters) {
    let scancode = unsafe { ::x86_64::instructions::port::inb(0x60) };
    println!("Scancode: {}", scancode);
    lapic::signal_eoi();
}
