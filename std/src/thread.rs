//! Handles thread related syscalls.

/// The number of the exit syscall.
const SLEEP_SYSCALL_NUM: u64 = 4;

/// Lets the current thread sleep for `ms` milliseconds.
pub fn sleep(ms: u64) {
    unsafe {
        syscall!(SLEEP_SYSCALL_NUM, ms);
    }
}
