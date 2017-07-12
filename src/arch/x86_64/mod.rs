//! The x86_64 architecture.
//!
//! This module does all the architecture specific things for x86_64.

pub mod vga_buffer;
pub mod memory;
pub mod sync;
pub mod interrupts;
pub mod context;
mod syscalls;
mod gdt;

pub use self::context::Context;
use self::gdt::GDT;
use multitasking::StackType;
use raw_cpuid::CpuId;
use x86_64::instructions::{rdmsr, wrmsr};
use x86_64::registers::*;

/// The stack type used for the x86_64 architecture.
pub const STACK_TYPE: StackType = StackType::FullDescending;

/// Initializes the machine state for the x86_64 architecture to a bare minimum.
pub fn early_init() {
    assert_has_not_been_called!("Early x86_64 specific initialization should only be called once.");

    let cpuid = CpuId::new();
    let mut supported = true;

    if let Some(features) = cpuid.get_feature_info() {
        supported &= features.has_apic();
    } else {
        supported = false;
    }

    if let Some(function_info) = cpuid.get_extended_function_info() {
        supported &= function_info.has_syscall_sysret();
        supported &= function_info.has_execute_disable();
    } else {
        supported = false;
    }

    if !supported {
        panic!("Your hardware unfortunately does not supported VeOS.");
    }

    unsafe {
        // Enable syscall/sysret instructions and the NXE bit in the page table.
        wrmsr(msr::IA32_EFER, rdmsr(msr::IA32_EFER) | 1 << 11 | 1);

        // Enable global pages.
        let cr4_flags = control_regs::cr4() | control_regs::Cr4::ENABLE_GLOBAL_PAGES;
        control_regs::cr4_write(cr4_flags);

        // Enable read only pages.
        let cr0_flags = control_regs::cr0() | control_regs::Cr0::WRITE_PROTECT;
        control_regs::cr0_write(cr0_flags);
    }
}

/// Initializes the machine state for the x86_64 architecture to the final
/// state.
pub fn init() {
    assert_has_not_been_called!("x86_64 specific initialization code should only be called once.");

    unsafe {
        GDT.load();
    }

    syscalls::init();
    interrupts::init();
}

/// Returns the ID of the currently running CPU.
pub fn get_cpu_id() -> usize {
    CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize
}

/// Returns the number of addressable CPUs.
pub fn get_cpu_num() -> usize {
    CpuId::new()
        .get_feature_info()
        .unwrap()
        .max_logical_processor_ids() as usize
}

/// Switches from the old context to the next context.
pub fn switch_context(_: Context, _: Context) {}
