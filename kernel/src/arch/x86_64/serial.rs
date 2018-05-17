//! This module handles communication over serial ports.

use core::fmt;
use x86_64::instructions::port::{inb, outb};

/// Represents a serial port that can be read from and written to.
pub struct SerialPort {
    /// The IO-Port that the serial port is located at.
    port: u16
}

impl SerialPort {
    /// Creates a new serial port.
    pub const fn new(port: u16) -> SerialPort {
        SerialPort { port }
    }

    /// Initializes the serial port.
    ///
    /// According to the [OS-dev wiki](https://wiki.osdev.org/Serial_ports).
    pub fn init(&mut self) {
        unsafe {
            outb(self.port + 1, 0x00); // Disable all interrupts
            outb(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(self.port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(self.port + 1, 0x00); //                  (hi byte)
            outb(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    /// Checks if the last trasmission is fully finished.
    fn transmission_ready(&self) -> bool {
        unsafe { inb(self.port + 5) & 0x20 != 1 }
    }

    /// Transmits a character on the serial port.
    pub fn transmit(&mut self, data: u8) {
        while !self.transmission_ready() {}

        unsafe {
            outb(self.port, data);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.bytes() {
            self.transmit(byte);
        }

        Ok(())
    }
}

/// Prints the given line to the serial port.
///
/// It uses the arguments passed to it and prints the string with the
/// formatting arguments.
/// Then a new line is started.
#[macro_export]
macro_rules! serial_println {
    ($fmt:expr) => (serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (serial_print!(concat!($fmt, "\n"), $($arg)*));
}

/// Prints the given string to the serial port.
///
/// It uses the arguments passed to it and prints the string with the
/// formatting arguments.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ({
        $crate::arch::x86_64::COM1.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}
