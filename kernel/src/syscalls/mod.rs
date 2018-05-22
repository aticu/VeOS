//! This module handles system calls.

use arch::schedule;
use core::time::Duration;
use elf;
use memory::{Address, MemoryArea, VirtualAddress};
use multitasking::thread_management::READY_LIST;
use multitasking::{get_current_process, get_current_thread, TCB};
use sync::time::Timestamp;

/// This function accepts the syscalls and calls the corresponding handlers.
pub fn syscall_handler(
    num: u16,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize
) -> isize {
    match num {
        0 => print_char(arg1 as u8 as char),
        1 => kill_process(),
        2 => return_pid(),
        3 => exec(VirtualAddress::from_usize(arg1), arg2),
        4 => sleep(arg1, arg2),
        5 => create_thread(
            VirtualAddress::from_usize(arg1),
            arg2,
            arg3,
            arg4,
            arg5,
            arg6
        ),
        6 => kill_thread(),
        _ => unknown_syscall(num)
    }
}

fn print_char(character: char) -> isize {
    print!("{}", character);
    0
}

fn kill_process() -> isize {
    get_current_process().kill();

    schedule();
    0
}

fn return_pid() -> isize {
    let pid = get_current_thread().pid;
    let pid: usize = pid.into();

    pid as isize
}

fn exec(name_ptr: VirtualAddress, name_length: usize) -> isize {
    let name_ptr_valid = {
        let pcb = get_current_process();

        pcb.address_space
            .contains_area(MemoryArea::new(name_ptr, name_length))
    };

    if name_ptr_valid {
        let name = from_raw_str!(name_ptr, name_length);

        if let Ok(name) = name {
            let process_id = elf::process_from_initramfs_file(name);

            if let Ok(process_id) = process_id {
                let pid: usize = process_id.into();

                assert!(pid as isize > 0, "Process ID too large.");

                pid as isize
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

fn create_thread(
    start_address: VirtualAddress,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize
) -> isize {
    let pid = get_current_thread().pid;
    let mut pcb = get_current_process();
    let id = pcb.find_thread_id();

    match id {
        Some(id) => {
            let thread = TCB::in_process_with_arguments(
                pid,
                id,
                start_address,
                &mut pcb,
                arg1,
                arg2,
                arg3,
                arg4,
                arg5
            );

            pcb.add_thread(id);

            READY_LIST.lock().push(thread);

            let tid: usize = id.into();

            tid as isize
        },
        None => -1
    }
}

fn kill_thread() -> isize {
    get_current_thread().kill();

    schedule();

    0
}

fn sleep(seconds: usize, nanoseconds: usize) -> isize {
    // Check if the duration is valid
    let seconds = seconds as u64;
    let nanoseconds = nanoseconds as u32;
    let duration = if seconds
        .checked_add((nanoseconds / 1000_000_000).into())
        .is_none()
    {
        // The wake time overflowed
        // TODO: handle this in a more useful way
        get_current_process().kill_immediately();
    } else {
        // If the duration was valid, return it
        Duration::new(seconds, nanoseconds)
    };

    let wake_time = if let Some(time) = Timestamp::get_current().offset(duration) {
        time
    } else {
        // The wake time overflowed
        // TODO: handle this in a more useful way
        get_current_process().kill_immediately();
    };

    get_current_thread().state = ::multitasking::ThreadState::Sleeping(wake_time);
    schedule();
    0
}

fn unknown_syscall(num: u16) -> ! {
    if cfg!(debug) {
        panic!("The syscall {} is not known.", num);
    } else {
        // TODO: Handle this better
        get_current_process().kill_immediately();
    }
}
