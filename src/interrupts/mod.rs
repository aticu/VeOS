//! This module contains general interrupt handlers.
//!
//! None of the contained interrupt handlers should be architecture specific. They should instead
//! be called by the architecture specific interrupt handlers.

use arch::schedule;

/// The timer interrupt handler for the system.
pub fn timer_interrupt() {
    print!("!");
    schedule();
}

/// The keyboard interrupt handler.
pub fn keyboard_interrupt(scancode: u8) {
    println!("Key: <{}>", scancode);
}

pub fn page_fault_handler(address: VirtualAddress) {
}
