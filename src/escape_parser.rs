//! ANSI escape sequences parser
//!
//! Reference: [https://en.wikipedia.org/wiki/ANSI_escape_code](https://en.wikipedia.org/wiki/ANSI_escape_code)

#![allow(dead_code)]

use super::color::ConsoleColor;
use super::color::Rgb888;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(4))]
pub struct CharacterAttribute {
    /// foreground color
    pub foreground: Rgb888,
    /// background color
    pub background: Rgb888,
    /// show underline
    pub underline: bool,
    /// swap foreground and background colors
    pub reverse: bool,
    /// text marked for deletion
    pub strikethrough: bool,
}

impl Default for CharacterAttribute {
    fn default() -> Self {
        CharacterAttribute {
            foreground: ConsoleColor::White.to_rgb888_cmd(),
            background: ConsoleColor::Black.to_rgb888_cmd(),
            underline: false,
            reverse: false,
            strikethrough: false,
        }
    }
}

impl CharacterAttribute {
    /// Parse and apply SGR (Select Graphic Rendition) parameters.
    pub fn apply_sgr(&mut self, params: &[i64]) {
        let code = *params.get(0).unwrap_or(&0) as u8;
        match code {
            0 => *self = CharacterAttribute::default(),
            4 => self.underline = true,
            7 => self.reverse = true,
            9 => self.strikethrough = true,
            24 => self.underline = false,
            27 => self.reverse = false,
            29 => self.strikethrough = false,
            30..=37 | 90..=97 => {
                self.foreground = ConsoleColor::from_console_code(code)
                    .unwrap()
                    .to_rgb888_cmd()
            }
            38 => self.foreground = Rgb888::new(params[2] as u8, params[3] as u8, params[4] as u8),
            39 => self.foreground = CharacterAttribute::default().foreground,
            40..=47 | 100..=107 => {
                self.background = ConsoleColor::from_console_code(code - 10)
                    .unwrap()
                    .to_rgb888_cmd();
            }
            48 => self.background = Rgb888::new(params[2] as u8, params[3] as u8, params[4] as u8),
            49 => self.background = CharacterAttribute::default().background,
            _ => warn!("unknown SGR: {:?}", params),
        }
    }
}

/// Control Sequence Introducer
///
/// Reference: [https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CSI<'a> {
    CursorMove(i64, i64),
    CursorMoveTo(i64, i64),
    CursorMoveLine(i64),
    CursorMoveLineTo(i64),
    SGR(&'a [i64]),
    EnableAltScreenBuffer,
    DisableAltScreenBuffer,
    EnableAutoWrap,
    DisableAutoWrap,
    SetScrollingRegion(i64, i64),
    WindowManipulation(&'a [i64]),
    EraseDisplay,
    Unknown,
}

impl<'a> CSI<'a> {
    pub fn new(final_byte: u8, params: &'a [i64], _intermediates: &'a [u8]) -> CSI<'a> {
        let n = *params.get(0).unwrap_or(&1);
        match final_byte {
            b'A' => CSI::CursorMove(-n, 0),
            b'B' => CSI::CursorMove(n, 0),
            b'C' => CSI::CursorMove(0, n),
            b'D' => CSI::CursorMove(0, -n),
            b'E' => CSI::CursorMoveLine(n),
            b'F' => CSI::CursorMoveLine(-n),
            b'H' => CSI::CursorMoveTo(
                *params.get(0).unwrap_or(&1) - 1,
                *params.get(1).unwrap_or(&1) - 1,
            ),
            b'J' => CSI::EraseDisplay, // TODO: Erase mode
            b'm' => CSI::SGR(params),
            b'd' => CSI::CursorMoveLineTo(n - 1),
            b'h' => match *params.get(0).unwrap_or(&0) {
                7 => CSI::EnableAutoWrap,
                1049 => CSI::EnableAltScreenBuffer,
                _ => CSI::Unknown,
            },
            b'l' => match *params.get(0).unwrap_or(&0) {
                7 => CSI::DisableAutoWrap,
                1049 => CSI::DisableAltScreenBuffer,
                _ => CSI::Unknown,
            },
            b'r' => CSI::SetScrollingRegion(
                *params.get(0).unwrap_or(&1) - 1,
                *params.get(1).unwrap_or(&1) - 1,
            ),
            b't' => CSI::WindowManipulation(params),
            _ => CSI::Unknown,
        }
    }
}
