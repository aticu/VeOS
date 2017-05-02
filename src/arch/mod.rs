//! Abstracts architecture details.
//!
//! The job of this module is to have submodules for each architecture and to
//! provide interfaces


#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;
/// to them.

use core::fmt;
#[cfg(target_arch = "x86_64")]
mod x86_64;

/// Writes the formatted arguments.
///
/// This takes arguments as dictated by core::fmt and prints the to the screen
/// using the
/// printing method relevant for the current architecture.
pub fn write_fmt(args: fmt::Arguments) {
    if cfg!(target_arch = "x86_64") {
        use core::fmt::Write;
        x86_64::vga_buffer::WRITER
            .lock()
            .write_fmt(args)
            .unwrap();
    }
}

/// //Initializes architecture specific things.
/// pub fn init() {
/// if cfg!(target_arch = "x86_64") {
/// x86_64::init();
///
///

/// //Returns whether interrupts are enabled.
/// pub fn interrupts_enabled() -> bool {
/// if cfg!(target_arch = "x86_64") {
/// x86_64::sync::interrupts_enabled()
/// else {
/// unimplemented!();
///
///

/// //Enables interrupts.
/// pub unsafe fn enable_interrupts() {
/// if cfg!(target_arch = "x86_64") {
/// x86_64::sync::enable_interrupts();
///
///

/// //Disables interrupts.
/// pub unsafe fn disable_interrupts() {
/// if cfg!(target_arch = "x86_64") {
/// x86_64::sync::disable_interrupts();
///
///

/// Sets the state of being interruptable to the given state.
pub unsafe fn set_interrupt_state(state: bool) {
    if state {
        sync::enable_interrupts();
    } else {
        sync::disable_interrupts();
    }
}
