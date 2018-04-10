//! Handles process related system calls.

/// The number of the exit syscall.
const EXIT_SYSCALL_NUM: u64 = 1;

/// The number of the get_pid syscall.
const GET_PID_SYSCALL_NUM: u64 = 2;

/// The number of the exec syscall.
const EXEC_SYSCALL_NUM: u64 = 3;

/// The possible types of errors that are process related.
#[derive(Debug)]
pub enum ProcessError {
    /// The error is not further specified.
    Unspecified,
}

/// Exits the current process.
pub fn exit() -> ! {
    unsafe {
        syscall!(EXIT_SYSCALL_NUM);
    }
    unreachable!();
}

/// Returns the ID of the current process.
pub fn get_pid() -> u64 {
    unsafe { syscall!(GET_PID_SYSCALL_NUM) as u64 }
}

/// Creates a new process from the given executable.
pub fn exec(name: &str) -> Result<u64, ProcessError> {
    let name_ptr = name as *const str as *const usize as u64;
    let result = unsafe { syscall!(EXEC_SYSCALL_NUM, name_ptr, name.len() as u64) as i64 };
    if result < 0 {
        Err(ProcessError::Unspecified)
    } else {
        Ok(result as u64)
    }
}
