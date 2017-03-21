use core::fmt;

pub fn init() {
    if cfg!(target_arch = "x86_64") {
        ::arch::x86_64::vga_buffer::init();
    }
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::io::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    if cfg!(target_arch = "x86_64") {
        use core::fmt::Write;
        ::arch::x86_64::vga_buffer::WRITER.lock().write_fmt(args).unwrap();
    }
}
