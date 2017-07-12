//! Serves to accept syscalls.

use super::gdt::{KERNEL_CODE_SEGMENT, USER_32BIT_CODE_SEGMENT};
use arch::memory::{STACK_OFFSET, SYSCALL_STACK_AREA_BASE};
use multitasking::THREAD_ID;
use syscalls::syscall_handler;
use x86_64::registers::flags::Flags;
use x86_64::registers::msr::{IA32_FMASK, IA32_LSTAR, IA32_STAR, wrmsr};

/// Initializes the system to be able to accept syscalls.
pub fn init() {
    let sysret_cs = USER_32BIT_CODE_SEGMENT.0 as u64;
    let syscall_cs = KERNEL_CODE_SEGMENT.0 as u64;

    let star_value = sysret_cs << 48 | syscall_cs << 32;
    let lstar_value = syscall_entry as u64;
    let fmask_value = Flags::IF.bits() as u64;

    unsafe {
        wrmsr(IA32_LSTAR, lstar_value);
        wrmsr(IA32_STAR, star_value);
        wrmsr(IA32_FMASK, fmask_value);
    }
}

/// The entry point for all syscalls.
#[naked]
extern "C" fn syscall_entry() {
    extern "C" fn syscall_inner() -> u64 {
        let num;
        let arg1;
        let arg2;
        let arg3;
        let arg4;
        let arg5;
        let arg6;
        unsafe {
            asm!("" :
                 "={rdi}"(num),
                 "={rsi}"(arg1),
                 "={rdx}"(arg2),
                 "={r8}"(arg3),
                 "={r9}"(arg4),
                 "={r12}"(arg5),
                 "={r13}"(arg6)
                 : : : "intel", "volatile");
        }

        syscall_handler(num, arg1, arg2, arg3, arg4, arg5, arg6)
    }

    unsafe {
        asm!("// Calculate the stack pointer and set it
              // rax is the thread ID.
              // Increase it, because the stack starts at the top.
              inc rax
              // r10 is the distance between stack tops.
              mov r10, $1
              mul r10
              // rax is now the offset from the stack area base."
              : : "{rax}"(THREAD_ID), "i"(STACK_OFFSET) : : "intel", "volatile");

        asm!("movabs r10, $0
              // r10 is now the base address of the stack area.
              add rax, r10
              // rax is now the desired stack pointer.
              mov r10, rsp
              // r10 is the old stack pointer.
              mov rsp, rax

              // Now that the stack pointer is a kernel stack pointer, enable interrupts.
              sti

              push r10 //The old stack pointer
              push r11 //The flags register
              push rcx //The program counter"
              : : "i"(SYSCALL_STACK_AREA_BASE) : : "intel", "volatile");

        asm!("call $0
              // Pop the rest context.
              pop rcx
              pop r11
              pop r10

              // Restore the old stack pointer.
              cli
              mov rsp, r10
              sysret"
              : : "i"(syscall_inner as extern "C" fn() -> u64) : : "intel", "volatile");
    }
}
