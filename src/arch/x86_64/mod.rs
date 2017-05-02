//! The x86_64 architecture.
//!
//! This module does all the architecture specific things for x86_64.

pub mod vga_buffer;
pub mod memory;
pub mod sync;

use x86_64::instructions::{rdmsr, wrmsr};
use x86_64::registers::msr::IA32_EFER;

/// Initializes the machine state for the x86_64 architecture.
pub fn init() {
    unsafe {
        // enable syscall/sysret instructions and the NXE bit in the page table
        wrmsr(IA32_EFER, rdmsr(IA32_EFER) | 1 << 11 | 1);
    }
}
