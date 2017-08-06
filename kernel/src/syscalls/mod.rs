//! This module handles system calls.

use arch::schedule;
use multitasking::{CURRENT_THREAD, ThreadState};

/// This function accepts the syscalls and calls the corresponding handlers.
pub fn syscall_handler(num: u64, arg1: u64, _: u64, _: u64, _: u64, _: u64, _: u64) -> u64 {
    match num {
        0 => print_char(arg1 as u8 as char),
        1 => kill_thread(),
        _ => unknown_syscall(num),
    }
}

fn print_char(character: char) -> u64 {
    print!("{}", character);
    0
}

fn kill_thread() -> u64 {
    CURRENT_THREAD.lock().state = ThreadState::Dead;
    schedule();
    0
}

fn unknown_syscall(num: u64) -> u64 {
    println!("The syscall {} is not known.", num);
    panic!("");
}
