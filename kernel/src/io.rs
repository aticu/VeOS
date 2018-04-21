//! This module deals with all in-kernel IO.
//!
//! It handles all the IO that kernel code needs to perform.

/// Initializes all IO devices.
pub fn init() {
    assert_has_not_been_called!("IO components should only be initialized once");
    ::arch::init_io();
}

/// Prints the given line to the screen.
///
/// It uses the arguments passed to it and prints the string with the
/// formatting arguments.
/// Then a new line is started.
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Prints the given string to the screen.
///
/// It uses the arguments passed to it and prints the string with the
/// formatting arguments.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::arch::write_fmt(format_args!($($arg)*));
    });
}
