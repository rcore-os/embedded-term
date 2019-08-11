pub use embedded_graphics::pixelcolor::Rgb888;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConsoleColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl ConsoleColor {
    pub fn to_console_code(&self) -> u8 {
        use self::ConsoleColor::*;
        match self {
            Black => 30,
            Red => 31,
            Green => 32,
            Yellow => 33,
            Blue => 34,
            Magenta => 35,
            Cyan => 36,
            White => 37,
            BrightBlack => 90,
            BrightRed => 91,
            BrightGreen => 92,
            BrightYellow => 93,
            BrightBlue => 94,
            BrightMagenta => 95,
            BrightCyan => 96,
            BrightWhite => 97,
        }
    }
    pub fn from_console_code(code: u8) -> Option<ConsoleColor> {
        use self::ConsoleColor::*;
        match code {
            30 => Some(Black),
            31 => Some(Red),
            32 => Some(Green),
            33 => Some(Yellow),
            34 => Some(Blue),
            35 => Some(Magenta),
            36 => Some(Cyan),
            37 => Some(White),
            90 => Some(BrightBlack),
            91 => Some(BrightRed),
            92 => Some(BrightGreen),
            93 => Some(BrightYellow),
            94 => Some(BrightBlue),
            95 => Some(BrightMagenta),
            96 => Some(BrightCyan),
            97 => Some(BrightWhite),
            _ => None,
        }
    }
}

impl From<ConsoleColor> for Rgb888 {
    /// Convert `ConsoleColor` to `Rgb888`.
    /// use `CMD` color scheme.
    /// (ref: https://en.wikipedia.org/wiki/ANSI_escape_code)
    fn from(color: ConsoleColor) -> Self {
        use self::ConsoleColor::*;
        match color {
            Black => Rgb888::new(0, 0, 0),
            Red => Rgb888::new(128, 0, 0),
            Green => Rgb888::new(0, 128, 8),
            Yellow => Rgb888::new(128, 128, 0),
            Blue => Rgb888::new(0, 0, 128),
            Magenta => Rgb888::new(128, 0, 128),
            Cyan => Rgb888::new(0, 128, 128),
            White => Rgb888::new(192, 192, 192),
            BrightBlack => Rgb888::new(128, 128, 128),
            BrightRed => Rgb888::new(255, 0, 0),
            BrightGreen => Rgb888::new(0, 255, 0),
            BrightYellow => Rgb888::new(255, 255, 0),
            BrightBlue => Rgb888::new(0, 0, 255),
            BrightMagenta => Rgb888::new(255, 0, 255),
            BrightCyan => Rgb888::new(0, 255, 255),
            BrightWhite => Rgb888::new(255, 255, 255),
        }
    }
}
