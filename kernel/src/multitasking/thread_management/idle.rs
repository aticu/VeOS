//! This module contains functions related to the idle process.

use super::SLEEPING_LIST;
use arch::{self, Architecture, schedule};
use sync::{cpu_halt, enable_preemption};
use sync::time::Timestamp;

/// This function gets executed whenever there is nothing else to execute.
///
/// It can perform various tasks, such as cleaning up unused resources.
///
/// Once it's done performing it's initial cleanup, it sleeps in a loop,
/// performing periodic cleanup. It should also be interruptable as often as
/// possible.
pub fn idle() -> ! {
    // TODO: Peform initial cleanup here.
    unsafe {
        enable_preemption();
        schedule();
    }
    loop {
        // TODO: Perform periodic cleanup here.
        unsafe {
            {
                if let Some(next_wake_thread) = SLEEPING_LIST.lock().peek() {
                    let current_time = Timestamp::get_current();
                    let wake_time = next_wake_thread.get_wake_time();
                    if let Some(sleep_duration) = wake_time.checked_sub(current_time) {
                        arch::Current::interrupt_in(sleep_duration);
                    } else {
                        schedule();
                    }
                }
            }
            cpu_halt();
        }
    }
}