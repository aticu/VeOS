use core::fmt;
use core::ptr::Unique;
use volatile::Volatile;
use spin::Mutex;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black       = 0,
    Blue        = 1,
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Magenta     = 5,
    Brown       = 6,
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    Pink        = 13,
    Yellow      = 14,
    White       = 15
}

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    character: u8,
    color_code: ColorCode
}

const BUFFER_HEIGHT: usize = 25; //TODO read these from multiboot header
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>
}

impl Writer {
    pub fn write_char(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let column_position = self.column_position;
                let row_position = self.row_position;
                let color_code = self.color_code;

                self.get_buffer().chars[row_position][column_position].write(ScreenChar {
                    character: byte,
                    color_code: color_code
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.write_char(byte);
        }
    }

    fn get_buffer(&mut self) -> &mut Buffer {
        unsafe {
            self.buffer.get_mut()
        }
    }

    fn new_line(&mut self) {
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for i in 1..BUFFER_HEIGHT {
                self.shift_line(i);
            }
            self.row_position = BUFFER_HEIGHT - 2;
            self.clear_line(BUFFER_HEIGHT - 1);
        }

        self.row_position += 1;
        self.column_position = 0;
    }

    fn shift_line(&mut self, line: usize) {
        let buffer = self.get_buffer();

        for i in 0..BUFFER_HEIGHT {
            let char_below = buffer.chars[line][i].read();

            buffer.chars[line - 1][i].write(char_below);
        }
    }

    fn clear_line(&mut self, line: usize) {
        let color_code = self.color_code;
        let buffer = self.get_buffer();
        let space = ScreenChar {
            character: b' ',
            color_code: color_code
        };

        for i in 0..BUFFER_WIDTH {
            buffer.chars[line][i].write(space);
        }
    }

    fn clear_screen(&mut self)
    {
        for _ in 0..BUFFER_HEIGHT {
            self.new_line();
        }

        self.column_position = 0;
        self.row_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_string(string);

        Ok(())
    }
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    row_position: 0,
    color_code: ColorCode::new(Color::LightGray, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) }
});

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}
