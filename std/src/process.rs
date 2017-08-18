//! Handles process related system calls.

use ::syscall;

/// The number of the exit syscall.
const EXIT_SYSCALL_NUM: u64 = 1;

/// The number of the get_pid syscall.
const GET_PID_SYSCALL_NUM: u64 = 2;

/// The number of the exec syscall.
const EXEC_SYSCALL_NUM: u64 = 3;

/// Exits the current process.
pub fn exit() -> ! {
    unsafe {
        syscall(EXIT_SYSCALL_NUM, 0, 0, 0, 0, 0, 0);
    }
    unreachable!();
}

/// Returns the ID of the current process.
pub fn get_pid() -> u64 {
    unsafe {
        syscall(GET_PID_SYSCALL_NUM, 0, 0, 0, 0, 0, 0) as u64
    }
}

/// Creates a new process from the given executable.
pub fn exec(name: &str) -> i64 {
    let name_ptr = name as *const str as *const usize as u64;
    unsafe {
        syscall(EXEC_SYSCALL_NUM, name_ptr, name.len() as u64, 0, 0, 0, 0)
    }
}
