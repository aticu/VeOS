//! Abstracts architecture details.
//!
//! The job of this module is to have submodules for each architecture and to
//! provide interfaces to them.


#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

use core::fmt;
#[cfg(target_arch = "x86_64")]
mod x86_64;

/// Writes the formatted arguments.
///
/// This takes arguments as dictated by `core::fmt` and prints the to the
/// screen using the printing method relevant for the current architecture.
pub fn write_fmt(args: fmt::Arguments) {
    if cfg!(target_arch = "x86_64") {
        use core::fmt::Write;
        x86_64::vga_buffer::WRITER
            .lock()
            .write_fmt(args)
            .unwrap();
    }
}

/// Sets the state of being interruptable to the given state.
///
/// # Safety
/// - Don't use this function directly, rather use the sync module.
pub unsafe fn set_interrupt_state(state: bool) {
    if state {
        sync::enable_interrupts();
    } else {
        sync::disable_interrupts();
    }
}
