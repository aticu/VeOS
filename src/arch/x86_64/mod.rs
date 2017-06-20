//! The x86_64 architecture.
//!
//! This module does all the architecture specific things for x86_64.

pub mod vga_buffer;
pub mod memory;
pub mod sync;
pub mod interrupts;

use raw_cpuid::CpuId;
use x86_64::instructions::{rdmsr, wrmsr};
use x86_64::registers::*;

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

    if !supported {
        panic!("Your hardware unfortunately does not supported VeOS.");
    }

    unsafe {
        // Enable syscall/sysret instructions and the NXE bit in the page table.
        wrmsr(msr::IA32_EFER, rdmsr(msr::IA32_EFER) | 1 << 11 | 1);

        // Enable global pages.
        let cr4_flags = control_regs::cr4() | control_regs::Cr4::ENABLE_GLOBAL_PAGES;
        control_regs::cr4_write(cr4_flags);

        let cr0_flags = control_regs::cr0() | control_regs::Cr0::WRITE_PROTECT;
        control_regs::cr0_write(cr0_flags);
    }
}

/// Initializes the machine state for the x86_64 architecture to the final
/// state.
pub fn init() {
    assert_has_not_been_called!("x86_64 specific initialization code should only be called once.");

    interrupts::init();
}
