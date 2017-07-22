//! Serves to accept syscalls.

use super::gdt::{KERNEL_CODE_SEGMENT, USER_32BIT_CODE_SEGMENT, TSS};
use arch::memory::{STACK_OFFSET, SYSCALL_STACK_AREA_BASE};
use syscalls::syscall_handler;
use x86_64::registers::flags::Flags;
use x86_64::registers::msr::{IA32_FMASK, IA32_LSTAR, IA32_STAR, IA32_KERNEL_GS_BASE, wrmsr};

/// Initializes the system to be able to accept syscalls.
pub fn init() {
    let sysret_cs = USER_32BIT_CODE_SEGMENT.0 as u64;
    let syscall_cs = KERNEL_CODE_SEGMENT.0 as u64;

    let star_value = sysret_cs << 48 | syscall_cs << 32;
    let lstar_value = syscall_entry as u64;
    let fmask_value = Flags::IF.bits() as u64;
    let gs_base_value = &TSS.privilege_stack_table[0] as *const _ as u64;

    unsafe {
        wrmsr(IA32_LSTAR, lstar_value);
        wrmsr(IA32_STAR, star_value);
        wrmsr(IA32_FMASK, fmask_value);
        wrmsr(IA32_KERNEL_GS_BASE, gs_base_value);
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
                 "={rax}"(num),
                 "={rdi}"(arg1),
                 "={rsi}"(arg2),
                 "={rdx}"(arg3),
                 "={r8}"(arg4),
                 "={r9}"(arg5),
                 "={r12}"(arg6)
                 : : : "intel", "volatile");
        }

        syscall_handler(num, arg1, arg2, arg3, arg4, arg5, arg6)
    }

    unsafe {
        asm!("// Load the gs base to point to the stack pointer.
              swapgs

              // Save the old stack pointer.
              mov r10, rsp
              // Load the new stack pointer.
              mov rsp, gs:[0]

              // Restore the gs base.
              swapgs

              // Now that the stack pointer is a kernel stack pointer, enable interrupts.
              sti

              // Save some context.
              push r10 //The old stack pointer
              push r11 //The flags register
              push rcx //The program counter

              // Call the actual handler.
              call $0

              // Restore the context.
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
