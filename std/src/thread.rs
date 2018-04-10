//! Handles thread related syscalls.

/// The number of the exit syscall.
const SLEEP_SYSCALL_NUM: u64 = 4;

/// The number of the syscall to create a new thread.
const NEW_THREAD_SYSCALL_NUM: u64 = 5;

/// Kills the current thread.
const KILL_THREAD_SYSCALL_NUM: u64 = 6;

/// Lets the current thread sleep for `ms` milliseconds.
pub fn sleep(ms: u64) {
    unsafe {
        syscall!(SLEEP_SYSCALL_NUM, ms);
    }
}

/// Creates a new thread passing it the given arguments.
pub fn new_thread(function: fn(u64, u64, u64, u64), arg1: u64, arg2: u64, arg3: u64, arg4: u64) {
    unsafe {
        syscall!(
            NEW_THREAD_SYSCALL_NUM,
            new_thread_creator as u64,
            function as u64,
            arg1,
            arg2,
            arg3,
            arg4
        );
    }
}

/// Kills the current thread.
pub fn kill_thread() {
    unsafe {
        syscall!(KILL_THREAD_SYSCALL_NUM);
    }
}

/// Used internally to create and exit new threads.
extern "C" fn new_thread_creator(
    function: fn(u64, u64, u64, u64),
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) {
    function(arg1, arg2, arg3, arg4);

    kill_thread();
}
