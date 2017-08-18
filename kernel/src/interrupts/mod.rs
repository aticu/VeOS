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
    if scancode == 1 {
        unsafe { ::sync::disable_preemption() };
        loop {}
    }
    println!("Key: <{}>", scancode);
}

/// The page fault handler.
pub fn page_fault_handler(address: VirtualAddress, program_counter: VirtualAddress) {
    unsafe { ::sync::disable_preemption() };
    let current_thread = CURRENT_THREAD.lock();
    println!("Page fault in process {} (thread {}) at address {:x} (PC: {:x})",
             current_thread.pid,
             current_thread.id,
             address,
             program_counter);
    //let current_process = PROCESS_LIST.lock().get(current_thread.pid).expect("Thread without process running.");
    println!("Page flags: {:?}", ::memory::get_page_flags(address));
    loop {}
}
