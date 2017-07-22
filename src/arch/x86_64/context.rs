//! Provides saving and restoring of architecture specific execution context.

use core::mem::size_of;
use memory::VirtualAddress;
use multitasking::scheduler::{idle, after_context_switch};
use super::gdt::{USER_CODE_SEGMENT, USER_DATA_SEGMENT};
use x86_64::structures::idt::ExceptionStackFrame;

// TODO: Floating point state is not saved yet.
/// Saves the an execution context.
pub struct Context {
    pub kernel_stack_pointer: VirtualAddress,
    base_pointer: VirtualAddress
}

struct SavedRegisters {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64
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
        let regs = SavedRegisters {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: arg6,
            r8: arg5,
            rbp: 0,
            rdi: arg1,
            rsi: arg2,
            rdx: arg3,
            rcx: arg4,
            rbx: 0,
            rax: 0
        };
        use x86_64::registers::flags::Flags;

        let stack_frame = ExceptionStackFrame {
            instruction_pointer: ::x86_64::VirtualAddress(function as usize),
            code_segment: USER_CODE_SEGMENT.0 as u64,
            cpu_flags: (Flags::IF | Flags::A1).bits() as u64,
            stack_pointer: ::x86_64::VirtualAddress(stack_pointer as usize),
            stack_segment: USER_DATA_SEGMENT.0 as u64
        };

        let kernel_stack_pointer = unsafe { set_initial_stack(kernel_stack_pointer, stack_frame, regs) };

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

#[naked]
unsafe fn enter_thread() -> ! {
    after_context_switch();
    asm!("pop r15
          pop r14
          pop r13
          pop r12
          pop r11
          pop r10
          pop r9
          pop r8
          pop rbp
          pop rdi
          pop rsi
          pop rdx
          pop rcx
          pop rbx
          pop rax
          iretq" : : : : "intel", "volatile");
    unreachable!();
}

unsafe fn set_idle_stack(stack_pointer: u64) -> u64 {
    let mut stack_pointer = stack_pointer;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = idle as u64;
    stack_pointer
}

unsafe fn set_initial_stack(stack_pointer: usize, stack_frame: ExceptionStackFrame, saved_registers: SavedRegisters) -> usize {
    let mut stack_pointer = stack_pointer;
    stack_pointer -= size_of::<ExceptionStackFrame>();
    *(stack_pointer as *mut ExceptionStackFrame) = stack_frame;
    stack_pointer -= size_of::<SavedRegisters>();
    *(stack_pointer as *mut SavedRegisters) = saved_registers;
    stack_pointer -= 8;
    *(stack_pointer as *mut u64) = enter_thread as u64;
    stack_pointer
}

/// Switches the context from the old thread to the current thread.
///
/// # Safety
/// - To make sure that everything is properly cleaned up after switching the context this should
/// only be called by the scheduler.
#[naked]
pub unsafe fn switch_context(old_context: &mut Context, new_context: &Context) {
    println!("");

    let old_sp;
    let old_bp;
    let new_sp = new_context.kernel_stack_pointer;
    let new_bp = new_context.base_pointer;
    let base_sp = ::multitasking::CURRENT_THREAD.lock().syscall_stack.base_stack_pointer;
    super::gdt::TSS.as_mut().privilege_stack_table[0] = ::x86_64::VirtualAddress(base_sp);

    asm!("" : "={rsp}"(old_sp), "={rbp}"(old_bp));
    old_context.kernel_stack_pointer = old_sp;
    old_context.base_pointer = old_bp;
    asm!("" : : "{rsp}"(new_sp), "{rbp}"(new_bp));

}
