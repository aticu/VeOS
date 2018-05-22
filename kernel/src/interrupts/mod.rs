//! This module contains general interrupt handlers.
//!
//! None of the contained interrupt handlers should be architecture specific.
//! They should instead
//! be called by the architecture specific interrupt handlers.

use arch::{self, schedule, Architecture};
use memory::VirtualAddress;
use multitasking::get_current_thread;

/// The timer interrupt handler for the system.
pub fn timer_interrupt() {
    schedule();
}

/// The keyboard interrupt handler.
pub fn keyboard_interrupt(scancode: u8) {
    if scancode == 1 {
        unsafe { ::sync::disable_preemption() };
        loop {}
    }
    info!("Key: <{}>", scancode);
}

/// The page fault handler.
pub fn page_fault_handler(address: VirtualAddress, program_counter: VirtualAddress) {
    unsafe { ::sync::disable_preemption() };
    let current_thread = get_current_thread();

    error!(
        "Page fault in {:?} {:?} at address {:?} (PC: {:?})",
        current_thread.pid, current_thread.id, address, program_counter
    );

    error!("Page flags: {:?}", arch::Current::get_page_flags(address));
    loop {}
}
