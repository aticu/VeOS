//!Handles IO using VGA.
//!
//!This module is used to handle IO with the basic VGA interface usually located at 0xb8000;

use core::fmt;
use core::ptr::Unique;
use volatile::Volatile;
use spin::Mutex;
use boot;

///Represents a color in the buffer.
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
    White = 15,
}

///Represents a color code in the buffer.
///
///A color code includes both information about the foreground and the background color.
#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    ///Creates a color code.
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

///Represents a character in the buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    ///The ascii character represented.
    character: u8,
    ///The color code of the character represented.
    color_code: ColorCode,
}

///Represents a character on this screen.
struct Buffer {
    //TODO: this works for arbitrary screen sizes up to 10000000, but it isn't nice
    chars: [Volatile<ScreenChar>; 10000000],
}

///The writer is used to write to a legacy VGA display buffer.
pub struct Writer {
    ///The height of the buffer.
    buffer_height: usize,
    ///The width of the buffer.
    buffer_width: usize,
    ///The current column position.
    column_position: usize,
    ///The current row position.
    row_position: usize,
    ///The color code used throughout the buffer.
    color_code: ColorCode,
    ///Access to the buffer itself.
    buffer: Unique<Buffer>,
}

impl Writer {
    ///Writes the given character to the buffer.
    pub fn write_char(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= self.buffer_width {
                    self.new_line();
                }

                let column_position = self.column_position;
                let row_position = self.row_position;
                let color_code = self.color_code;
                let width = self.buffer_width;

                self.get_buffer().chars[row_position * width + column_position]
                    .write(ScreenChar {
                               character: byte,
                               color_code: color_code,
                           });

                self.column_position += 1;
            }
        }
    }

    ///Writes the given string to the buffer.
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_char(byte);
        }
    }

    ///Returns a reference to the buffer.
    fn get_buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.get_mut() }
    }

    ///Inserts a new line character.
    fn new_line(&mut self) {
        let height = self.buffer_height;
        if self.row_position >= self.buffer_height - 1 {
            for i in 1..height {
                self.shift_line(i);
            }
            self.row_position = height - 2;
            self.clear_line(height - 1);
        }

        self.row_position += 1;
        self.column_position = 0;
    }

    ///Shifts the given line upwards.
    fn shift_line(&mut self, line: usize) {
        let height = self.buffer_height;
        let width = self.buffer_width;
        let buffer = self.get_buffer();

        for i in 0..height {
            let char_below = buffer.chars[line * width + i].read();

            buffer.chars[(line - 1) * width + i].write(char_below);
        }
    }

    ///Clears the given line.
    fn clear_line(&mut self, line: usize) {
        let color_code = self.color_code;
        let width = self.buffer_width;
        let buffer = self.get_buffer();
        let space = ScreenChar {
            character: b' ',
            color_code: color_code,
        };

        for i in 0..width {
            buffer.chars[line * width + i].write(space);
        }
    }

    ///Clears the whole screen.
    fn clear_screen(&mut self) {
        for i in 0..self.buffer_height {
            self.clear_line(i);
        }

        self.column_position = 0;
        self.row_position = 0;
    }

    ///Initializes the buffer.
    fn init(&mut self, info: Info) {
        self.buffer_height = info.height;
        self.buffer_width = info.width;
        self.buffer = unsafe { Unique::new(info.address as *mut _) };
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);

        Ok(())
    }
}

///The Writer that is used to print to the screen.
pub static WRITER: Mutex<Writer> =
    Mutex::new(Writer {
                   buffer_height: 25,
                   buffer_width: 80,
                   column_position: 0,
                   row_position: 0,
                   color_code: ColorCode::new(Color::LightGray, Color::Black),
                   buffer: unsafe { Unique::new(to_virtual!(0xb8000) as *mut _) },
               });

///Contains basic buffer information.
///
///This is what is used to convey information about the buffer from the outside to this module.
pub struct Info {
    pub height: usize,
    pub width: usize,
    pub address: usize,
}

///Initializes the buffer for use.
pub fn init() {
    let info = boot::get_vga_info();
    WRITER.lock().init(info);
    clear_screen();
}

///Clears the screen.
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}
