//! Handles architecture specific synchronization.

use x86_64::instructions::interrupts;
use x86_64::registers::flags::*;

/// Called while spinning (name borrowed from Linux). Can be implemented to call
/// a platform-specific method of lightening CPU load in spinlocks.
#[inline(always)]
pub fn cpu_relax() {
    // This instruction is meant for usage in spinlock loops
    // (see Intel x86 manual, III, 4.2)
    unsafe {
        asm!("pause" :::: "volatile");
    }
}

/// Disables interrupts.
#[inline(always)]
pub unsafe fn disable_interrupts() {
    interrupts::disable();
}

/// Enables interrupts.
#[inline(always)]
pub unsafe fn enable_interrupts() {
    interrupts::enable();
}

/// Checks whether interrupts are enabled.
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    flags().contains(Flags::IF)
}
