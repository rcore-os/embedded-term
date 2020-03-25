use crate::color::Rgb888;
use crate::escape_parser::{CharacterAttribute, CSI};
use crate::graphic::TextOnGraphic;
use crate::text_buffer::*;
use crate::text_buffer_cache::TextBufferCache;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;
use embedded_graphics::prelude::DrawTarget;
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
    /// result buffer
    result: Vec<u8>,
}

pub type ConsoleOnGraphic<D> = Console<TextBufferCache<TextOnGraphic<D>>>;

impl<D: DrawTarget<Rgb888>> Console<TextBufferCache<TextOnGraphic<D>>> {
    /// Create a console on top of a frame buffer
    pub fn on_frame_buffer(buffer: D) -> Self {
        Self::on_cached_text_buffer(TextOnGraphic::new(buffer))
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
                result: Vec::new(),
            },
        }
    }

    /// Write a single `byte` to console
    pub fn write_byte(&mut self, byte: u8) {
        trace!("get: {}", byte);
        self.parser.advance(&mut self.inner, byte);
    }

    /// Read result for some commands
    pub fn get_result(&mut self) -> Vec<u8> {
        self.inner.result.clone()
    }

    /// Clear the screen
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.inner.clear();
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

    fn hook(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, action: char) {
        debug!(
            "hook: {:?}, {:?}, {}, {:?}",
            params, intermediates, ignore, action
        );
    }

    fn put(&mut self, byte: u8) {
        debug!("put: {}", byte);
    }

    fn unhook(&mut self) {
        debug!("unhook:");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        warn!(
            "osc: params={:?}, bell_terminated={:?}",
            params, bell_terminated
        );
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
            CSI::CursorMoveRow(dr) => {
                self.row = (self.row as i64 + dr) as usize;
                self.col = 0;
            }
            CSI::CursorMoveRowTo(dr) => {
                self.row = dr as usize;
            }
            CSI::CursorMoveColTo(dc) => {
                self.col = dc as usize;
            }
            CSI::EnableAutoWrap => {
                self.auto_wrap = true;
            }
            CSI::DisableAutoWrap => {
                self.auto_wrap = false;
            }
            CSI::EraseDisplayAll => {
                self.buf.clear();
            }
            CSI::DeviceStatusReport => {
                // CSI
                self.result.push(0x1B);
                self.result.push(b'[');
                self.result.push(b'0');
                self.result.push(b'n');
            }
            CSI::ReportCursorPosition => {
                // CSI
                self.result.push(0x1B);
                self.result
                    .append(&mut (self.row + 1).to_string().into_bytes());
                self.result.push(b';');
                self.result
                    .append(&mut (self.col + 1).to_string().into_bytes());
                self.result.push(b'R');
            }
            CSI::EraseDisplayAbove => {
                for i in 0..self.row {
                    for j in 0..self.buf.width() {
                        self.buf.delete(i, j);
                    }
                }
            }
            CSI::EraseDisplayBelow => {
                for i in self.row..self.buf.height() {
                    for j in 0..self.buf.width() {
                        self.buf.delete(i, j);
                    }
                }
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

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        debug!("esc: {:?}, {:?}, {}", intermediates, ignore, byte);
        match byte {
            b'K' => {
                for i in self.col..self.buf.height() {
                    self.buf.delete(self.row, i);
                }
            }
            _ => {
                warn!("unknown esc: {:?}, {:?}, {}", intermediates, ignore, byte);
            }
        }
    }
}
