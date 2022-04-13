use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

// Println and print macros
#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
	() => ($crate::print!("\n"));
	($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! input {
    () => {
        $crate::vga_buffer::_input()
    };
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().input_mode = false;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn _input() {
    println!("Hello World");
}

#[allow(dead_code)]
#[repr(u8)]
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

#[derive(Clone, Copy)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    color_code: ColorCode,
    pub buffer: &'static mut Buffer,
    pub input_mode: bool,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    fn backspace(&mut self) {
        if self.row_position == 0 && self.column_position == 0 {
            return;
        }
        if self.column_position == 0 {
            self.row_position -= 1;
            self.column_position = self.get_last_col(self.row_position);
            return;
        }
        // Set char at that row and col to blank (space)
        // Push back col position so next char will overwrite that char
        let blank = ScreenChar {
            ascii_character: 0x00,
            color_code: self.color_code,
        };
        self.buffer.chars[self.row_position][self.column_position - 1].write(blank);
        self.column_position -= 1;
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                0x08 => self.backspace(),
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn new_line(&mut self) {
        if self.row_position == (BUFFER_HEIGHT - 1) {
            self.shift_up();
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
    }
    fn shift_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                if !(row <= 0) {
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
        }
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
    fn get_last_col(&mut self, row: usize) -> usize {
        // let mut col: usize = 0;
        // let mut char;
        // for i in 0..BUFFER_WIDTH {
        //     char = self.buffer.chars[row][i].read();
        //     if !char.ascii_character == 0x00 {
        //         col += 1;
        //     }
        // }
        let mut col: usize = 0; // 0
        let mut char = self.buffer.chars[row][0].read();
        while char.ascii_character != 0x00 {
            // 0x00
            col += 1; // col -> 2
            char = self.buffer.chars[row][col].read(); // char -> char at col 2
        }
        return col;
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        input_mode: false
    });
}

// TESTS
#[test_case]
fn test_print_one() {
    print!("Hello World");
}

#[test_case]
fn test_print_many() {
    for _ in 1..100 {
        print!("Hello World");
    }
}

#[test_case]
fn test_print_output() {
    interrupts::without_interrupts(|| {
        println!(); // Make sure no text is on the same line from previous tests
        let printed_str = "Some random text";
        print!("{}", printed_str);
        let writer = WRITER.lock();
        let row_position = writer.row_position;
        for (i, c) in printed_str.chars().enumerate() {
            let screen_char = writer.buffer.chars[row_position][i].read();
            let ascii_char = char::from(screen_char.ascii_character);
            assert_eq!(ascii_char, c);
        }
    })
}
