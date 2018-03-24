//! Handles IO using VGA.
//!
//! This module is used to handle IO with the basic VGA interface usually
//! located at 0xb8000;

use boot;
use core::fmt;
use core::ptr::Unique;
use sync::Mutex;
use volatile::Volatile;
use memory::VirtualAddress;

/// Represents a color in the buffer.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
}

/// Represents a color code in the buffer.
///
/// A color code includes both information about the foreground and the
/// background color.
#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    /// Creates a color code.
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Represents a character in the buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    /// The ascii character represented.
    character: u8,
    /// The color code of the character represented.
    color_code: ColorCode
}

/// Represents the buffer.
struct Buffer {
    address: Unique<Volatile<ScreenChar>>,
    width: usize,
    height: usize
}

impl Buffer {
    /// Creates a new buffer.
    const fn new(address: usize, width: usize, height: usize) -> Buffer {
        Buffer {
            address: unsafe { Unique::new_unchecked(address as *mut _) },
            width,
            height
        }
    }

    /// Writes a character to this buffer.
    fn write_char(&mut self, row_position: usize, column_position: usize, character: ScreenChar) {
        let start = self.address.as_ptr();
        // TODO better safety check here
        unsafe {
            let position_ptr = start.offset((row_position * self.width + column_position) as isize);
            (&mut *position_ptr).write(character);
        }
    }

    /// Reads a character from this buffer.
    fn read_char(&self, row_position: usize, column_position: usize) -> ScreenChar {
        let start = unsafe { self.address.as_ref() as *const Volatile<ScreenChar> };
        unsafe {
            let position_ptr = start.offset((row_position * self.width + column_position) as isize);
            (&*position_ptr).read()
        }
    }
}

/// The writer is used to write to a legacy VGA display buffer.
pub struct Writer {
    /// The current column position.
    column_position: usize,
    /// The current row position.
    row_position: usize,
    /// The color code used throughout the buffer.
    color_code: ColorCode,
    /// Access to the buffer itself.
    buffer: Buffer
}

impl Writer {
    /// Writes the given character to the buffer.
    pub fn write_char(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= self.buffer.width {
                    self.new_line();
                }

                let column_position = self.column_position;
                let row_position = self.row_position;
                let color_code = self.color_code;

                self.buffer
                    .write_char(row_position,
                                column_position,
                                ScreenChar {
                                    character: byte,
                                    color_code: color_code
                                });

                self.column_position += 1;
            },
        }
    }

    /// Writes the given string to the buffer.
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_char(byte);
        }
    }

    /// Inserts a new line character.
    fn new_line(&mut self) {
        let height = self.buffer.height;
        if self.row_position >= self.buffer.height - 1 {
            for i in 1..height {
                self.shift_line(i);
            }
            self.row_position = height - 2;
            self.clear_line(height - 1);
        }

        self.row_position += 1;
        self.column_position = 0;
    }

    /// Shifts the given line upwards.
    fn shift_line(&mut self, line: usize) {
        for i in 0..self.buffer.width {
            let char_below = self.buffer.read_char(line, i);

            self.buffer.write_char((line - 1), i, char_below);
        }
    }

    /// Clears the given line.
    fn clear_line(&mut self, line: usize) {
        let color_code = self.color_code;
        let width = self.buffer.width;
        let space = ScreenChar {
            character: b' ',
            color_code: color_code
        };

        for i in 0..width {
            self.buffer.write_char(line, i, space);
        }
    }

    /// Clears the whole screen.
    fn clear_screen(&mut self) {
        for i in 0..self.buffer.height {
            self.clear_line(i);
        }

        self.column_position = 0;
        self.row_position = 0;
    }

    /// Initializes the buffer.
    fn init(&mut self, info: Info) {
        assert_has_not_been_called!("The VGA buffer should only be initialized once.");

        self.buffer.height = info.height;
        self.buffer.width = info.width;
        self.buffer.address = unsafe { Unique::new_unchecked(info.address as *mut _) };
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);

        Ok(())
    }
}

/// The Writer that is used to print to the screen.
pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
                                                  column_position: 0,
                                                  row_position: 0,
                                                  color_code: ColorCode::new(Color::LightGray,
                                                                             Color::Black),
                                                  buffer: Buffer::new(to_virtual!(0xb8000), 25, 80)
                                              });

/// Contains basic buffer information.
///
/// This is what is used to convey information about the buffer from the
/// outside to this module.
pub struct Info {
    pub height: usize,
    pub width: usize,
    pub address: VirtualAddress
}

/// Initializes the buffer for use.
pub fn init() {
    let info = boot::get_vga_info();
    WRITER.lock().init(info);
    clear_screen();
}

/// Clears the screen.
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}
