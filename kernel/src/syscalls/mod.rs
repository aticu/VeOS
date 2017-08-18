//! This module handles system calls.

mod delta_queue;

use arch::schedule;
use elf;
use core::str;
use memory::VirtualAddress;
use multitasking::{CURRENT_THREAD, get_current_process};

/// This function accepts the syscalls and calls the corresponding handlers.
pub fn syscall_handler(num: u64, arg1: u64, arg2: u64, _: u64, _: u64, _: u64, _: u64) -> i64 {
    match num {
        0 => print_char(arg1 as u8 as char),
        1 => kill_process(),
        2 => return_pid(),
        3 => exec(arg1 as VirtualAddress, arg2 as usize),
        _ => unknown_syscall(num),
    }
}

fn print_char(character: char) -> i64 {
    print!("{}", character);
    0
}

fn kill_process() -> i64 {
    get_current_process().kill();

    schedule();
    0
}

fn return_pid() -> i64 {
    let pid = CURRENT_THREAD.lock().pid;
    pid as i64
}

fn exec(name_ptr: VirtualAddress, name_length: usize) -> i64 {
    let name_ptr_valid = {
        let pcb = get_current_process();

        pcb.address_space.contains_range(name_ptr, name_length)
    };

    if name_ptr_valid {
        let name = from_raw_str!(name_ptr, name_length);

        if let Ok(name) = name {
            let process_id = elf::process_from_initramfs_file(name);

            if let Ok(process_id) = process_id {
                assert!(process_id as i64 > 0, "Process ID too large.");

                process_id as i64
            } else {
                -1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

fn sleep(ms: usize) -> i64 {
    0
}

fn unknown_syscall(num: u64) -> i64 {
    println!("The syscall {} is not known.", num);
    panic!("");
}
