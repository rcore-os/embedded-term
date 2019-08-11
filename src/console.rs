use crate::escape_parser::EscapeParser;
use crate::text_buffer::*;
use core::fmt;

/// Console structure
///
/// Input string with control sequence
/// Output to a `TextBuffer`
pub struct Console<T: TextBuffer> {
    /// cursor row
    row: usize,
    /// cursor column
    col: usize,
    /// escape sequence parser
    parser: EscapeParser,
    /// character buffer
    buf: T,
}

impl<T: TextBuffer> Console<T> {
    pub fn new(buffer: T) -> Console<T> {
        Console {
            row: 0,
            col: 0,
            parser: EscapeParser::new(),
            buf: buffer,
        }
    }

    fn new_line(&mut self) {
        let attr_blank = ConsoleChar {
            ascii_char: b' ',
            attr: self.parser.char_attribute(),
        };
        for j in self.col..self.buf.width() {
            self.buf.write(self.row, j, attr_blank);
        }
        self.col = 0;
        if self.row < self.buf.height() - 1 {
            self.row += 1;
        } else {
            self.buf.new_line();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if self.parser.is_parsing() {
            self.parser.parse(byte);
            return;
        }
        match byte {
            b'\x7f' | b'\x08' => {
                if self.col > 0 {
                    self.col -= 1;
                    self.buf.delete(self.row, self.col);
                } else if self.row > 0 {
                    self.row -= 1;
                    self.col = self.buf.width() - 1;
                    self.buf.delete(self.row, self.col);
                }
            }
            b'\t' => {
                self.write_byte(b' ');
                while self.col % 8 != 0 {
                    self.write_byte(b' ');
                }
            }
            b'\n' => self.new_line(),
            b'\r' => self.col = 0,
            b'\x1b' => self.parser.start_parse(),
            byte => {
                if self.col >= self.buf.width() {
                    self.new_line();
                }

                let ch = ConsoleChar {
                    ascii_char: byte,
                    attr: self.parser.char_attribute(),
                };
                self.buf.write(self.row, self.col, ch);
                self.col += 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.row = 0;
        self.col = 0;
        self.parser = EscapeParser::new();
        self.buf.clear();
    }
}

impl<T: TextBuffer> fmt::Write for Console<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
