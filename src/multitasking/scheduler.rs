//! This module implements a scheduler.

use super::{READY_LIST, TCB};
use arch::context::switch_context;
use core::mem::swap;
use sync::Mutex;
use sync::{disable_preemption, restore_preemption_state, enable_preemption};
use x86_64::instructions::halt;

lazy_static! {
    /// Holds the TCB of the currently running thread.
    pub static ref CURRENT_THREAD: Mutex<TCB> = Mutex::new(TCB::idle_tcb());
}
cpu_local! {
    /// Holds the TCB of the previously running thread during context switches.
    static mut ref OLD_THREAD: Option<TCB> = None;
}

/// Schedules the next thread to run and dispatches it.
///
/// # Safety
/// - This function should not be called directly. Rather call `arch::schedule`.
pub unsafe fn schedule_next_thread() {
    // No interrupts during scheduling (this essentially locks OLD_THREAD).
    let preemption_state = disable_preemption();

    debug_assert!(OLD_THREAD.is_none());

    let mut ready_list = READY_LIST.lock();

    // TODO: Start the timer again here to ensure fairness.

    // TODO: Rework this part.
    let schedule_needed = ready_list.peek().is_some();
    let schedule_needed = schedule_needed && ready_list.peek().unwrap() >= &CURRENT_THREAD.lock();
    let schedule_needed = schedule_needed || CURRENT_THREAD.lock().is_dead();

    // Only switch if actually needed.
    if schedule_needed {
        // Move the new thread to the temporary spot for old threads.
        (*OLD_THREAD).set(Some(ready_list.pop().unwrap()));

        // Make sure no locks are held when switching.
        drop(ready_list);

        // Now swap the references.
        swap(&mut *CURRENT_THREAD.lock(), OLD_THREAD.as_mut().as_mut().unwrap());

        // OLD_THREAD holds the thread that was previously running.
        // CURRENT_THREAD now holds the thread that is to run now.

        if !OLD_THREAD.as_ref().unwrap().is_dead() {
            // If the thread isn't dead, set it's state to ready.
            OLD_THREAD.as_mut().as_mut().unwrap().set_ready();
        }
        CURRENT_THREAD.lock().set_running();

        // This is where the actual switch happens.
        switch_context(&mut OLD_THREAD.as_mut().as_mut().unwrap().context, &CURRENT_THREAD.without_locking().context);

        after_context_switch();
    } else {
        // Ensure that the correct drop order is used.
        drop(ready_list);
    }

    restore_preemption_state(&preemption_state);
}

/// This function should get called after calling `context_switch` to perform clean up.
pub fn after_context_switch() {
    if OLD_THREAD.is_some() {
        if OLD_THREAD.as_ref().unwrap().is_dead() {
            unsafe {
                OLD_THREAD.as_mut().take();
            }
        }
        else {
            unsafe {
                READY_LIST.lock().push(OLD_THREAD.as_mut().take().unwrap());
            }
        }
    }
}

/// This function gets executed whenever there is nothing else to execute.
///
/// It can perform various tasks, such as cleaning up unused resources.
///
/// Once it's done performing it's tasks, it halts.
pub fn idle() -> ! {
    // TODO: Peform initial cleanup here.
    unsafe {
        enable_preemption();
        schedule_next_thread();
    }
    loop {
        // TODO: Perform periodic cleanup here.
        unsafe {
            halt();
        }
    }
}
