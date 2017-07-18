//! This module implements a scheduler.

use super::{READY_LIST, TCB};
use arch::context::switch_context;
use core::mem::swap;
use sync::Mutex;
use sync::{disable_preemption, restore_preemption_state};
use x86_64::instructions::halt;

lazy_static! {
    pub static ref CURRENT_THREAD: Mutex<TCB> = Mutex::new(TCB::idle_tcb());
}
cpu_local! {
    static mut ref OLD_THREAD: Option<TCB> = None;
}

pub static mut THREAD_ID: u16 = 0;

/// Schedules the next thread to run and dispatches it.
pub fn schedule() {
    // No interrupts during scheduling (this essentially locks OLD_THREAD).
    let preemption_state = unsafe { disable_preemption() };

    assert!(OLD_THREAD.is_none());

    let mut ready_list = READY_LIST.lock();

    // Only switch if actually needed.
    if ready_list.peek().unwrap() >= &CURRENT_THREAD.lock() {
        // Move the new thread to the temporary spot for old threads.
        unsafe {
            (*OLD_THREAD).set(Some(ready_list.pop().unwrap()));
        }

        // Make sure no locks are held when switching.
        drop(ready_list);

        // Now swap the references.
        unsafe {
            swap(&mut *CURRENT_THREAD.lock(), OLD_THREAD.as_mut().as_mut().unwrap());
        }

        // OLD_THREAD holds the thread that was previously running.
        // CURRENT_THREAD now holds the thread that is to run now.

        unsafe {
            THREAD_ID = CURRENT_THREAD.lock().id;
        }

        if OLD_THREAD.as_ref().unwrap().is_dead() {
            // Kill the old thread by dropping it.
            unsafe {
                OLD_THREAD.as_mut().take();
            }
        } else {
            // Otherwise it must be ready again.
            unsafe {
                OLD_THREAD.as_mut().as_mut().unwrap().set_ready();
            }
        }
        CURRENT_THREAD.lock().set_running();

        // This is where the actual switch happens.
        unsafe {
            switch_context(&mut OLD_THREAD.as_mut().as_mut().unwrap().context, &CURRENT_THREAD.without_locking().context);
        }

        if OLD_THREAD.is_some() {
            unsafe {
                READY_LIST.lock().push(OLD_THREAD.as_mut().take().unwrap());
            }
        }
    } else {
        // Ensure that the correct drop order is used.
        drop(ready_list);
    }

    unsafe {
        restore_preemption_state(&preemption_state);
    }
}

/// This function gets executed whenever there is nothing else to execute.
///
/// It can perform various tasks, such as cleaning up unused resources.
///
/// Once it's done performing it's tasks, it halts.
pub fn idle() {
    loop {
        println!("IDLE");
        unsafe {
            halt();
        }
    }
}
