use crate::color::Rgb888;
use crate::escape_parser::{CharacterAttribute, CSI};
use crate::graphic::TextOnGraphic;
use crate::text_buffer::*;
use crate::text_buffer_cache::TextBufferCache;
use core::fmt;
use embedded_graphics::prelude::Drawing;
use vte::{Parser, Perform};

/// Console
///
/// Input string with control sequence,
/// output to a [`TextBuffer`].
pub struct Console<T: TextBuffer> {
    /// ANSI escape sequence parser
    parser: Parser,
    /// Inner state
    inner: ConsoleInner<T>,
}

struct ConsoleInner<T: TextBuffer> {
    /// cursor row
    row: usize,
    /// cursor column
    col: usize,
    /// char attribute
    attribute: CharacterAttribute,
    /// character buffer
    buf: T,
    /// auto wrap
    auto_wrap: bool,
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
            parser: Parser::new(),
            inner: ConsoleInner {
                row: 0,
                col: 0,
                attribute: CharacterAttribute::default(),
                buf: buffer,
                auto_wrap: true,
            },
        }
    }
    /// Write a single `byte` to console
    pub fn write_byte(&mut self, byte: u8) {
        trace!("get: {}", byte);
        self.parser.advance(&mut self.inner, byte);
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

impl<T: TextBuffer> ConsoleInner<T> {
    fn new_line(&mut self) {
        self.col = 0;
        if self.row < self.buf.height() - 1 {
            self.row += 1;
        } else {
            self.buf.new_line();
        }
    }

    /// Clear the screen
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.row = 0;
        self.col = 0;
        self.buf.clear();
    }
}

/// Perform actions
impl<T: TextBuffer> Perform for ConsoleInner<T> {
    fn print(&mut self, c: char) {
        debug!("print: {:?}", c);
        if self.col >= self.buf.width() {
            if !self.auto_wrap {
                // skip this one
                return;
            }
            self.new_line();
        }
        let ch = ConsoleChar {
            char: c,
            attr: self.attribute,
        };
        self.buf.write(self.row, self.col, ch);
        self.col += 1;
    }

    fn execute(&mut self, byte: u8) {
        debug!("execute: {}", byte);
        match byte {
            0x7f | 0x8 => {
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
                self.print(' ');
                while self.col % 8 != 0 {
                    self.print(' ');
                }
            }
            b'\n' => self.new_line(),
            b'\r' => self.col = 0,
            _ => warn!("unknown control code: {}", byte),
        }
    }

    fn hook(&mut self, params: &[i64], intermediates: &[u8], ignore: bool) {
        debug!("hook: {:?}, {:?}, {}", params, intermediates, ignore);
    }

    fn put(&mut self, byte: u8) {
        debug!("put: {}", byte);
    }

    fn unhook(&mut self) {
        debug!("unhook:");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]]) {
        warn!("osc: {:?}", params);
    }

    fn csi_dispatch(
        &mut self,
        params: &[i64],
        intermediates: &[u8],
        ignore: bool,
        final_byte: char,
    ) {
        let parsed = CSI::new(final_byte as u8, params, intermediates);
        debug!(
            "csi: {:?}, {:?}, {:?}, {} as {:?}",
            params, intermediates, ignore, final_byte, parsed
        );
        match parsed {
            CSI::SGR(code) => self.attribute.apply_sgr(code),
            CSI::CursorMove(dr, dc) => {
                self.row = (self.row as i64 + dr) as usize;
                self.col = (self.col as i64 + dc) as usize;
            }
            CSI::CursorMoveTo(dr, dc) => {
                self.row = dr as usize;
                self.col = dc as usize;
            }
            CSI::CursorMoveLine(dr) => {
                self.row = (self.row as i64 + dr) as usize;
                self.col = 0;
            }
            CSI::CursorMoveLineTo(dr) => {
                self.row = dr as usize;
            }
            CSI::EnableAutoWrap => {
                self.auto_wrap = true;
            }
            CSI::DisableAutoWrap => {
                self.auto_wrap = false;
            }
            CSI::EraseDisplay => {
                self.buf.clear();
            }
            CSI::Unknown => warn!(
                "unknown CSI: {:?}, {:?}, {:?}, {}",
                params, intermediates, ignore, final_byte
            ),
            _ => {
                // do nothing
            }
        }
    }

    fn esc_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, byte: u8) {
        warn!(
            "esc: {:?}, {:?}, {:?}, {}",
            params, intermediates, ignore, byte
        );
    }
}
