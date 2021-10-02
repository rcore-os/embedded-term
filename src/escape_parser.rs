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
    /// bold font
    pub bold: bool,
}

impl Default for CharacterAttribute {
    fn default() -> Self {
        CharacterAttribute {
            foreground: ConsoleColor::White.to_rgb888_cmd(),
            background: ConsoleColor::Black.to_rgb888_cmd(),
            underline: false,
            reverse: false,
            strikethrough: false,
            bold: false,
        }
    }
}

impl CharacterAttribute {
    /// Parse and apply SGR (Select Graphic Rendition) parameters.
    pub fn apply_sgr(&mut self, params: &[i64]) {
        let mut i = 0;
        while i < params.len() {
            let code = params[i] as u8;
            match code {
                0 => *self = CharacterAttribute::default(),
                1 => self.bold = true,
                4 => self.underline = true,
                7 => self.reverse = true,
                9 => self.strikethrough = true,
                22 => self.bold = false,
                24 => self.underline = false,
                27 => self.reverse = false,
                29 => self.strikethrough = false,
                30..=37 | 90..=97 => {
                    self.foreground = ConsoleColor::from_console_code(code)
                        .unwrap()
                        .to_rgb888_cmd()
                }
                38 => match params[i + 1] {
                    5 => {
                        let color = params[i + 2] as u8;
                        match color {
                            0..=7 => {
                                self.foreground = ConsoleColor::from_console_code(color + 30)
                                    .unwrap()
                                    .to_rgb888_cmd()
                            }
                            8..=15 => {
                                self.foreground = ConsoleColor::from_console_code(color + 82)
                                    .unwrap()
                                    .to_rgb888_cmd()
                            }
                            _ => warn!("unknown 8-bit color: {:?}", params),
                        }
                        i += 2;
                    }
                    2 => {
                        self.foreground = Rgb888::new(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                    _ => warn!("invalid params when set foreground color: {:?}", params),
                },
                39 => self.foreground = CharacterAttribute::default().foreground,
                40..=47 | 100..=107 => {
                    self.background = ConsoleColor::from_console_code(code - 10)
                        .unwrap()
                        .to_rgb888_cmd();
                }
                48 => match params[i + 1] {
                    5 => {
                        let color = params[i + 2] as u8;
                        match color {
                            0..=7 => {
                                self.background = ConsoleColor::from_console_code(color + 30)
                                    .unwrap()
                                    .to_rgb888_cmd()
                            }
                            8..=15 => {
                                self.background = ConsoleColor::from_console_code(color + 82)
                                    .unwrap()
                                    .to_rgb888_cmd()
                            }
                            _ => warn!("unknown 8-bit color: {:?}", params),
                        }
                        i += 2;
                    }
                    2 => {
                        self.background = Rgb888::new(
                            params[i + 2] as u8,
                            params[i + 3] as u8,
                            params[i + 4] as u8,
                        );
                        i += 4;
                    }
                    _ => warn!("invalid params when set background color: {:?}", params),
                },
                49 => self.background = CharacterAttribute::default().background,
                _ => warn!("unknown SGR: {:?}", params),
            }
            i += 1;
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
    CursorMoveRow(i64),
    CursorMoveRowTo(i64),
    CursorMoveColTo(i64),
    SGR(&'a [i64]),
    EnableAltScreenBuffer,
    DisableAltScreenBuffer,
    EnableAutoWrap,
    DisableAutoWrap,
    SetScrollingRegion(i64, i64),
    WindowManipulation(&'a [i64]),
    HideCursor,
    ShowCursor,
    EraseDisplayBelow,
    EraseDisplayAbove,
    EraseDisplayAll,
    EraseLineRight,
    EraseLineLeft,
    EraseLineAll,
    EnableBracketedPasteMode,
    DisableBracketedPasteMode,
    StartBlinkingCursor,
    StopBlinkingCursor,
    DeviceStatusReport,
    ReportCursorPosition,
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
            b'E' => CSI::CursorMoveRow(n),
            b'F' => CSI::CursorMoveRow(-n),
            b'H' => CSI::CursorMoveTo(
                *params.get(0).unwrap_or(&1) - 1,
                *params.get(1).unwrap_or(&1) - 1,
            ),
            b'J' => match *params.get(1).unwrap_or(&0) {
                0 => CSI::EraseDisplayBelow,
                1 => CSI::EraseDisplayAbove,
                2 => CSI::EraseDisplayAll,
                _ => CSI::Unknown,
            },
            b'K' => match *params.get(1).unwrap_or(&0) {
                0 => CSI::EraseLineRight,
                1 => CSI::EraseLineLeft,
                2 => CSI::EraseLineAll,
                _ => CSI::Unknown,
            },
            b'G' => CSI::CursorMoveColTo(*params.get(1).unwrap_or(&1) - 1),
            b'm' => CSI::SGR(params),
            b'n' => match *params.get(0).unwrap_or(&0) {
                5 => CSI::DeviceStatusReport,
                6 => CSI::ReportCursorPosition,
                _ => CSI::Unknown,
            },
            b'd' => CSI::CursorMoveRowTo(n - 1),
            b'h' => match *params.get(0).unwrap_or(&0) {
                7 => CSI::EnableAutoWrap,
                12 => CSI::StartBlinkingCursor,
                25 => CSI::ShowCursor,
                1049 => CSI::EnableAltScreenBuffer,
                2004 => CSI::EnableBracketedPasteMode,
                _ => CSI::Unknown,
            },
            b'l' => match *params.get(0).unwrap_or(&0) {
                7 => CSI::DisableAutoWrap,
                12 => CSI::StopBlinkingCursor,
                25 => CSI::HideCursor,
                1049 => CSI::DisableAltScreenBuffer,
                2004 => CSI::DisableBracketedPasteMode,
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
