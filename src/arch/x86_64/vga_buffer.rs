use core::fmt;
use core::ptr::Unique;
use volatile::Volatile;
use spin::Mutex;
use boot;

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

struct Buffer {
    chars: [[Volatile<ScreenChar>; 80]; 25] //TODO this doesn't work for bigger displays
}

///The writer is used to write to a legacy VGA display buffer.
pub struct Writer {
    buffer_height: usize,
    buffer_width: usize,
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>
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

                self.get_buffer().chars[row_position][column_position].write(ScreenChar {
                    character: byte,
                    color_code: color_code
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

    fn get_buffer(&mut self) -> &mut Buffer {
        unsafe {
            self.buffer.get_mut()
        }
    }

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

    fn shift_line(&mut self, line: usize) {
        let height = self.buffer_height;
        let buffer = self.get_buffer();

        for i in 0..height {
            let char_below = buffer.chars[line][i].read();

            buffer.chars[line - 1][i].write(char_below);
        }
    }

    fn clear_line(&mut self, line: usize) {
        let color_code = self.color_code;
        let width = self.buffer_width;
        let buffer = self.get_buffer();
        let space = ScreenChar {
            character: b' ',
            color_code: color_code
        };

        for i in 0..width {
            buffer.chars[line][i].write(space);
        }
    }

    fn clear_screen(&mut self)
    {
        for i in 0..self.buffer_height {
            self.clear_line(i);
        }

        self.column_position = 0;
        self.row_position = 0;
    }

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

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    buffer_height: 25,
    buffer_width: 80,
    column_position: 0,
    row_position: 0,
    color_code: ColorCode::new(Color::LightGray, Color::Black),
    buffer: unsafe { Unique::new(0xb8000 as *mut _) }
});

pub struct Info {
    pub height: usize,
    pub width: usize,
    pub address: usize
}

///Initializes the VGA buffer for use.
pub fn init() {
    let info = boot::get_vga_info();
    WRITER.lock().init(info);
    clear_screen();
}

///Clears the VGA buffer screen.
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}
