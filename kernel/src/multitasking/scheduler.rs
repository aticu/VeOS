//! This module implements a scheduler.

use super::tcb::SleepTimeSortedTCB;
use super::{ThreadState, TCB};
use alloc::binary_heap::BinaryHeap;
use arch::schedule;
use arch::switch_context;
use arch::interrupt_in;
use core::mem::swap;
use sync::Mutex;
use sync::{disable_preemption, enable_preemption, restore_preemption_state};
use sync::time::Timestamp;
use x86_64::instructions::halt;

cpu_local! {
    pub static ref READY_LIST: Mutex<BinaryHeap<TCB>> = |_| Mutex::new(BinaryHeap::new());
}

lazy_static! {
    pub static ref SLEEPING_LIST: Mutex<BinaryHeap<SleepTimeSortedTCB>> =
        Mutex::new(BinaryHeap::new());
}

cpu_local! {
    /// Holds the TCB of the currently running thread.
    pub static ref CURRENT_THREAD: Mutex<TCB> = |cpu_id| Mutex::new(TCB::idle_tcb(cpu_id));
}

cpu_local! {
    /// Holds the TCB of the previously running thread during context switches.
    static mut ref OLD_THREAD: Option<TCB> = |_| None;
}

/// Schedules the next thread to run and dispatches it.
///
/// # Safety
/// - This function should not be called directly. Rather call `arch::schedule`.
pub unsafe fn schedule_next_thread() {
    check_sleeping_processes();

    // No interrupts during scheduling (this essentially locks OLD_THREAD).
    let preemption_state = disable_preemption();

    debug_assert!(OLD_THREAD.is_none());

    let mut ready_list = READY_LIST.lock();

    // Scheduling is needed if:
    // There is another thread to schedule.
    let schedule_needed = ready_list.peek().is_some();
    // And it has at least the same priority.
    let schedule_needed = schedule_needed && ready_list.peek().unwrap() >= &CURRENT_THREAD.lock();
    // Or the current thread can't run anymore.
    let schedule_needed =
        schedule_needed || !CURRENT_THREAD.lock().is_running() || CURRENT_THREAD.lock().is_dead();

    // Only switch if actually needed.
    if schedule_needed {
        // Move the new thread to the temporary spot for old threads.
        (*OLD_THREAD).set(Some(ready_list.pop().unwrap()));

        // Make sure no locks are held when switching.
        drop(ready_list);

        trace!("Switching from {:?} to {:?}", **CURRENT_THREAD, **OLD_THREAD);

        // Now swap the references.
        swap(
            &mut *CURRENT_THREAD.lock(),
            OLD_THREAD.as_mut().as_mut().unwrap()
        );

        // OLD_THREAD holds the thread that was previously running.
        // CURRENT_THREAD now holds the thread that is to run now.

        if OLD_THREAD.as_ref().unwrap().is_running() {
            // If the thread was running, set it's state to ready.
            OLD_THREAD.as_mut().as_mut().unwrap().set_ready();
        }
        CURRENT_THREAD.lock().set_running();

        // This is where the actual switch happens.
        switch_context(
            &mut OLD_THREAD.as_mut().as_mut().unwrap().context,
            &CURRENT_THREAD.without_locking().context
        );

        after_context_switch();
    } else {
        // Ensure that the correct drop order is used.
        drop(ready_list);
    }

    restore_preemption_state(&preemption_state);
}

/// This function should get called after calling `context_switch` to perform
/// clean up.
pub fn after_context_switch() {
    if OLD_THREAD.is_some() {
        if OLD_THREAD.as_ref().unwrap().is_dead() {
            unsafe {
                // Drop the old thread.
                OLD_THREAD.as_mut().take();
            }
        } else {
            let old_thread = unsafe { OLD_THREAD.as_mut().take().unwrap() };
            return_old_thread_to_queue(old_thread);
        }
    }
    interrupt_in(CURRENT_THREAD.lock().get_quantum());
}

/// Returns the old thread to the corresponding queue after switching the
/// context.
fn return_old_thread_to_queue(thread: TCB) {
    match thread.state {
        ThreadState::Ready => READY_LIST.lock().push(thread),
        ThreadState::Sleeping(_) => SLEEPING_LIST.lock().push(SleepTimeSortedTCB(thread)),
        _ => panic!("Running or dead thread is being returned to a queue.")
    }
}

/// Updates the status for processes that were sleeping.
fn check_sleeping_processes() {
    {
        let mut sleeping_list = SLEEPING_LIST.lock();
        loop {
            let wake_first = {
                if let Some(first_to_wake) = sleeping_list.peek() {
                    first_to_wake.get_wake_time() <= Timestamp::get_current()
                }
                else {
                    false
                }
            };
            if wake_first {
                READY_LIST.lock().push(sleeping_list.pop().unwrap().0);
            } else {
                break;
            }
        }
    }
}

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
                        interrupt_in(sleep_duration);
                    } else {
                        schedule();
                    }
                }
            }
            halt();
        }
    }
}
