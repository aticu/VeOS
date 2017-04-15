//!Abstracts architecture details.
//!
//!The job of this module is to have submodules for each architecture and to provide interfaces
///to them.

use core::fmt;

pub mod x86_64;

///Writes the formatted arguments.
///
///This takes arguments as dictated by core::fmt and prints the to the screen using the
///printing method relevant for the current architecture.
pub fn write_fmt(args: fmt::Arguments) {
    if cfg!(target_arch = "x86_64") {
        use core::fmt::Write;
        x86_64::vga_buffer::WRITER
            .lock()
            .write_fmt(args)
            .unwrap();
    }
}
