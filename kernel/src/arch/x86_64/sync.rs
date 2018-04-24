//! Handles architecture specific synchronization.

use core::time::Duration;
use sync::time::Timestamp;
use x86_64::instructions::interrupts;
use x86_64::registers::flags::*;

/// The number of milliseconds since boot.
pub static mut CLOCK: Duration = Duration::from_secs(0);

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

/// Halts the cpu, until it is woken again.
///
/// # Safety
/// - Don't use this function directly, rather use the interface through the
/// sync module.
#[inline(always)]
pub unsafe fn cpu_halt() {
    asm!("hlt" :::: "volatile");
}

/// Disables interrupts.
///
/// # Safety
/// - Don't use this function directly, rather use the interface through the
/// sync module.
#[inline(always)]
pub unsafe fn disable_interrupts() {
    interrupts::disable();
}

/// Enables interrupts.
///
/// # Safety
/// - Don't use this function directly, rather use the interface through the
/// sync module.
#[inline(always)]
pub unsafe fn enable_interrupts() {
    interrupts::enable();
}

/// Checks whether interrupts are enabled.
#[inline(always)]
pub fn interrupts_enabled() -> bool {
    flags().contains(Flags::IF)
}

/// Returns the current timestamp.
pub fn get_current_timestamp() -> Timestamp {
    Timestamp::from_duration(unsafe { CLOCK })
}
