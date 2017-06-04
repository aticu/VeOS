//! The x86_64 architecture.
//!
//! This module does all the architecture specific things for x86_64.

pub mod vga_buffer;
pub mod memory;
pub mod sync;

use x86_64::instructions::{rdmsr, wrmsr};
use x86_64::registers::*;

/// Initializes the machine state for the x86_64 architecture.
pub fn init() {
    unsafe {
        // Enable syscall/sysret instructions and the NXE bit in the page table.
        wrmsr(msr::IA32_EFER, rdmsr(msr::IA32_EFER) | 1 << 11 | 1);
        // Enable global pages.
        let cr4_flags = control_regs::cr4() | control_regs::Cr4::ENABLE_GLOBAL_PAGES;
        control_regs::cr4_write(cr4_flags);
    }
}
