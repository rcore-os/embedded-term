use crate::color::Rgb888;
use crate::escape_parser::EscapeParser;
use crate::graphic::TextOnGraphic;
use crate::text_buffer::*;
use crate::text_buffer_cache::TextBufferCache;
use core::fmt;
use embedded_graphics::prelude::Drawing;

/// Console
///
/// Input string with control sequence,
/// output to a [`TextBuffer`].
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

pub type ConsoleOnGraphic<D> = Console<TextBufferCache<TextOnGraphic<D>>>;

impl<D: Drawing<Rgb888>> Console<TextBufferCache<TextOnGraphic<D>>> {
    /// Create a console on top of a frame buffer
    pub fn on_frame_buffer(width: u32, height: u32, buffer: D) -> Self {
        Self::on_cached_text_buffer(TextOnGraphic::new(width, height, buffer))
    }
}

impl<T: TextBuffer> Console<TextBufferCache<T>> {
    /// Create a console on top of a [`TextBuffer`] with a cache layer
    pub fn on_cached_text_buffer(buffer: T) -> Self {
        Self::on_text_buffer(TextBufferCache::new(buffer))
    }
}

impl<T: TextBuffer> Console<T> {
    /// Create a console on top of a [`TextBuffer`]
    pub fn on_text_buffer(buffer: T) -> Self {
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

    /// Write a single `byte` to console
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

    /// Clear the screen
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
