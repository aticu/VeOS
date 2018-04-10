//! This module defines IO functions.

use core::fmt;
use core::fmt::Write;

/// The number of the print char syscall.
const PRINT_CHAR_SYSCALL: u64 = 0;

/// A dummy struct to implement fmt::Write on.
struct StdOut;

impl fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for character in s.chars() {
            print_char(character);
        }
        Ok(())
    }
}

/// Prints a line to the standard output.
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Prints to the standard output.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::io::print(format_args!($($arg)*));
    });
}

/// Prints the given format arguments.
pub fn print(args: fmt::Arguments) {
    StdOut.write_fmt(args).unwrap();
}

/// Prints a character to the screen.
fn print_char(character: char) {
    unsafe {
        syscall!(PRINT_CHAR_SYSCALL, character as u64);
    }
}
