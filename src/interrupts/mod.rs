//! This module contains general interrupt handlers.
//!
//! None of the contained interrupt handlers should be architecture specific.
//! They should instead
//! be called by the architecture specific interrupt handlers.

use arch::schedule;
use memory::VirtualAddress;
use multitasking::CURRENT_THREAD;

/// The timer interrupt handler for the system.
pub fn timer_interrupt() {
    print!("!");
    schedule();
}

/// The keyboard interrupt handler.
pub fn keyboard_interrupt(scancode: u8) {
    println!("Key: <{}>", scancode);
}

/// The page fault handler.
pub fn page_fault_handler(address: VirtualAddress) {
    println!("Page fault in thread {} at address {:x}",
             CURRENT_THREAD.lock().id,
             address);
    println!("Page flags: {:?}", ::memory::get_page_flags(address));
    unsafe { ::sync::disable_preemption() };
    loop {}
}
