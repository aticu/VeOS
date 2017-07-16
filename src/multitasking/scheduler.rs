//! This module implements a scheduler.

use super::{READY_LIST, TCB, ThreadState};
use arch::switch_context;
use core::mem::swap;
use sync::Mutex;
use x86_64::instructions::halt;

lazy_static! {
    pub static ref CURRENT_THREAD: Mutex<TCB> = Mutex::new(TCB::idle_tcb());
}

pub static mut THREAD_ID: u16 = 0;

/// Schedules the next thread to run and switches the context to it.
pub fn schedule() {
    let mut ready_list = READY_LIST.lock();
    let mut new_thread = ready_list.pop().expect("No thread in the ready list.");
    let mut old_thread = CURRENT_THREAD.lock();

    unsafe {
        THREAD_ID = new_thread.id;
    }

    let kill_old = old_thread.state == ThreadState::Dead;

    old_thread.state = ThreadState::Ready;
    new_thread.state = ThreadState::Running;

    if new_thread >= *old_thread {
        swap(&mut *old_thread, &mut new_thread);

        // The references are swapped now, so old_thread points to the new thread and
        // vice versa.
        let (new_thread, old_thread) = (old_thread, new_thread);

        let (new_thread_context, old_thread_context) = (new_thread.context.clone(),
                                                        old_thread.context.clone());

        if !kill_old {
            // Return the old thread to the ready list.
            ready_list.push(old_thread);
        } else {
            // Kill the old thread by dropping it.
            drop(old_thread);
        }

        // Make sure that no locks are held when actually switching the context.
        drop(ready_list);
        drop(new_thread);

        switch_context(new_thread_context, old_thread_context);
    } else {
        ready_list.push(new_thread);
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
