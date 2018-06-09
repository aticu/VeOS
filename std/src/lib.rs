#![feature(start)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(panic_implementation)]
#![feature(naked_functions)]
#![no_std]

/// Makes a syscall with the given arguments.
macro_rules! syscall {
    ($num:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr, $arg2:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1),
                                "{rsi}"($arg2)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1),
                                "{rsi}"($arg2),
                                "{rdx}"($arg3)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1),
                                "{rsi}"($arg2),
                                "{rdx}"($arg3),
                                "{r10}"($arg4)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1),
                                "{rsi}"($arg2),
                                "{rdx}"($arg3),
                                "{r10}"($arg4),
                                "{r8}"($arg5)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
    ($num:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {{
        let result: u64;
        asm!("syscall" :
                                "={rax}"(result) :
                                "{rax}"($num),
                                "{rdi}"($arg1),
                                "{rsi}"($arg2),
                                "{rdx}"($arg3),
                                "{r10}"($arg4),
                                "{r8}"($arg5),
                                "{r9}"($arg6)
                                : "rax", "rdi", "rsi", "rdx", "r10", "r8", "r9", "r12", "r11", "rcx"
                                : "intel", "volatile");
        result
    }};
}

#[macro_use]
pub mod io;
pub mod process;
pub mod thread;

use core::panic::PanicInfo;
use process::exit;

extern "Rust" {
    /// The function that the program provides as a start.
    fn main();
}

/// The start of the application.
///
/// This should perform initialization and call main. After main returns, it should exit.
#[start]
#[no_mangle]
pub fn _start(_: isize, _: *const *const u8) -> isize {
    unsafe {
        main();
    }
    exit();
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {
    unimplemented!();
}

/// The panic handler of the program.
///
/// This exits after printing some debug information.
#[panic_implementation]
#[no_mangle]
pub extern "C" fn panic_fmt(info: &PanicInfo) -> ! {
    println!("{}", info);
    exit();
}
