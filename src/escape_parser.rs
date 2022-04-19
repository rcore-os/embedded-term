//! ANSI escape sequences parser
//!
//! Reference: [https://en.wikipedia.org/wiki/ANSI_escape_code](https://en.wikipedia.org/wiki/ANSI_escape_code)

#![allow(dead_code)]

use vte::Params;

use super::color::ConsoleColor;
use super::color::Rgb888;

/// Display attribute of characters.
///
/// The default attribute is white text on a black background.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(align(4))]
pub struct CharacterAttribute {
    /// Foreground color.
    pub foreground: Rgb888,
    /// Background color.
    pub background: Rgb888,
    /// Show underline.
    pub underline: bool,
    /// Swap foreground and background colors.
    pub reverse: bool,
    /// Text marked for deletion.
    pub strikethrough: bool,
    /// Bold font.
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
    pub(crate) fn apply_sgr(&mut self, params: &[u16]) {
        let code = *params.get(0).unwrap_or(&0) as u8;
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
            38 => {
                if params[1] == 5 {
                    let color = params[2] as u8;
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
                } else {
                    self.foreground = Rgb888::new(params[2] as u8, params[3] as u8, params[4] as u8)
                }
            }
            39 => self.foreground = CharacterAttribute::default().foreground,
            40..=47 | 100..=107 => {
                self.background = ConsoleColor::from_console_code(code - 10)
                    .unwrap()
                    .to_rgb888_cmd();
            }
            48 => {
                if params[1] == 5 {
                    let color = params[2] as u8;
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
                } else {
                    self.background = Rgb888::new(params[2] as u8, params[3] as u8, params[4] as u8)
                }
            }
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
    CursorMove(i16, i16),
    CursorMoveTo(i16, i16),
    CursorMoveRow(i16),
    CursorMoveRowTo(i16),
    CursorMoveColTo(i16),
    Sgr(&'a [u16]),
    EnableAltScreenBuffer,
    DisableAltScreenBuffer,
    EnableAutoWrap,
    DisableAutoWrap,
    SetScrollingRegion(i16, i16),
    WindowManipulation(&'a [u16]),
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
    pub fn new(final_byte: u8, params: &'a Params, _intermediates: &'a [u8]) -> CSI<'a> {
        let params = params.iter().next().unwrap();
        let n = *params.get(0).unwrap_or(&1) as i16;
        match final_byte {
            b'A' => CSI::CursorMove(-n, 0),
            b'B' => CSI::CursorMove(n, 0),
            b'C' => CSI::CursorMove(0, n),
            b'D' => CSI::CursorMove(0, -n),
            b'E' => CSI::CursorMoveRow(n),
            b'F' => CSI::CursorMoveRow(-n),
            b'H' => CSI::CursorMoveTo(
                *params.get(0).unwrap_or(&1) as i16 - 1,
                *params.get(1).unwrap_or(&1) as i16 - 1,
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
            b'G' => CSI::CursorMoveColTo(*params.get(1).unwrap_or(&1) as i16 - 1),
            b'm' => CSI::Sgr(params),
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
                *params.get(0).unwrap_or(&1) as i16 - 1,
                *params.get(1).unwrap_or(&1) as i16 - 1,
            ),
            b't' => CSI::WindowManipulation(params),
            _ => CSI::Unknown,
        }
    }
}
