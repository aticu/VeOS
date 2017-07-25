//! Provides saving and restoring of architecture specific execution context.

use super::gdt::{USER_CODE_SEGMENT, USER_DATA_SEGMENT};
use super::interrupts::lapic;
use core::mem::size_of;
use memory::VirtualAddress;
use multitasking::scheduler::{after_context_switch, idle};
use x86_64::structures::idt::ExceptionStackFrame;

// TODO: Floating point state is not saved yet.
/// Saves the an execution context.
pub struct Context {
    pub kernel_stack_pointer: VirtualAddress,
    base_pointer: VirtualAddress
}

impl Context {
    // TODO: Remove me, I'm only for testing.
    pub fn test(function: u64,
                arg1: u64,
                arg2: u64,
                arg3: u64,
                arg4: u64,
                arg5: u64,
                arg6: u64,
                stack_pointer: u64,
                kernel_stack_pointer: usize)
                -> Context {
        use x86_64::registers::flags::Flags;

        let stack_frame = ExceptionStackFrame {
            instruction_pointer: ::x86_64::VirtualAddress(function as usize),
            code_segment: USER_CODE_SEGMENT.0 as u64,
            cpu_flags: (Flags::IF | Flags::A1).bits() as u64,
            stack_pointer: ::x86_64::VirtualAddress(stack_pointer as usize),
            stack_segment: USER_DATA_SEGMENT.0 as u64
        };

        let kernel_stack_pointer = unsafe {
            set_initial_stack(kernel_stack_pointer,
                              stack_frame,
                              arg1,
                              arg2,
                              arg3,
                              arg4,
                              arg5,
                              arg6)
        };

        Context {
            kernel_stack_pointer,
            base_pointer: kernel_stack_pointer
        }
    }

    pub fn idle_context(stack_pointer: u64) -> Context {
        let stack_pointer = unsafe { set_idle_stack(stack_pointer) };

        Context {
            kernel_stack_pointer: stack_pointer as usize,
            base_pointer: stack_pointer as usize
        }
    }
}

/// This is the first thing that's called by every new process.
#[naked]
unsafe fn enter_thread() -> ! {
    after_context_switch();
    lapic::set_priority(0x0);
    asm!("xor r15, r15
          xor r14, r14
          xor r13, r13
          xor r12, r12
          xor r11, r11
          xor r10, r10
          xor r9, r9
          xor r8, r8
          xor rbp, rbp
          xor rdi, rdi
          xor rsi, rsi
          xor rdx, rdx
          xor rcx, rcx
          xor rbx, rbx
          xor rax, rax
          pop r9
          pop r8
          pop rcx
          pop rdx
          pop rsi
          pop rdi
          iretq" : : : : "intel", "volatile");
    unreachable!();
}

/// Sets the initial idle thread stack.
unsafe fn set_idle_stack(stack_pointer: u64) -> u64 {
    let mut stack_pointer = stack_pointer;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = idle as u64;
    stack_pointer
}

/// Sets the initial kernel stack of a thread, so that it can properly start.
unsafe fn set_initial_stack(stack_pointer: usize,
                            stack_frame: ExceptionStackFrame,
                            arg1: u64,
                            arg2: u64,
                            arg3: u64,
                            arg4: u64,
                            arg5: u64,
                            arg6: u64)
                            -> usize {
    let mut stack_pointer = stack_pointer;
    stack_pointer -= size_of::<ExceptionStackFrame>();
    *(stack_pointer as *mut ExceptionStackFrame) = stack_frame;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg1;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg2;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg3;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg4;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg5;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = arg6;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = enter_thread as u64;
    stack_pointer
}

/// Switches the context from the old thread to the current thread.
///
/// # Safety
/// - To make sure that everything is properly cleaned up after switching the
/// context this should
/// only be called by the scheduler.
#[naked]
pub unsafe fn switch_context(old_context: &mut Context, new_context: &Context) {
    println!("");

    let old_sp;
    let old_bp;
    let new_sp = new_context.kernel_stack_pointer;
    let new_bp = new_context.base_pointer;
    let base_sp = ::multitasking::CURRENT_THREAD
        .lock()
        .kernel_stack
        .base_stack_pointer;
    super::gdt::TSS.as_mut().privilege_stack_table[0] = ::x86_64::VirtualAddress(base_sp);

    asm!("" : "={rsp}"(old_sp), "={rbp}"(old_bp));
    old_context.kernel_stack_pointer = old_sp;
    old_context.base_pointer = old_bp;
    asm!("" : : "{rsp}"(new_sp), "{rbp}"(new_bp));

}
