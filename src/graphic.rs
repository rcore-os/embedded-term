use crate::cell::{Cell, Flags};
use crate::text_buffer::TextBuffer;
use embedded_graphics::{
    mono_font::{
        iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Drawable, Point, Size},
    text::{Baseline, Text, TextStyle},
};

const CHAR_SIZE: Size = FONT.character_size;

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `DrawTarget<Color = Rgb888>` trait
/// to draw pixels in RGB format.
pub struct TextOnGraphic<D>
where
    D: DrawTarget,
{
    width: u32,
    height: u32,
    graphic: D,
}

impl<D> TextOnGraphic<D>
where
    D: DrawTarget,
{
    /// Create a new text buffer on graphic.
    pub fn new(graphic: D, width: u32, height: u32) -> Self {
        TextOnGraphic {
            width,
            height,
            graphic,
        }
    }
}

impl<D> TextBuffer for TextOnGraphic<D>
where
    D: DrawTarget<Color = Rgb888>,
{
    #[inline]
    fn width(&self) -> usize {
        (self.width / CHAR_SIZE.width) as usize
    }

    #[inline]
    fn height(&self) -> usize {
        (self.height / CHAR_SIZE.height) as usize
    }

    fn read(&self, _row: usize, _col: usize) -> Cell {
        unimplemented!("reading char from graphic is unsupported")
    }

    #[inline]
    fn write(&mut self, row: usize, col: usize, cell: Cell) {
        if row >= self.height() || col >= self.width() {
            return;
        }
        let mut utf8_buf = [0u8; 8];
        let s = cell.c.encode_utf8(&mut utf8_buf);
        let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };
        let mut style = MonoTextStyleBuilder::new()
            .text_color(fg.to_rgb())
            .background_color(bg.to_rgb());
        if cell.flags.contains(Flags::BOLD) {
            style = style.font(&FONT_BOLD);
        } else {
            style = style.font(&FONT);
        }
        if cell.flags.contains(Flags::STRIKEOUT) {
            style = style.strikethrough();
        }
        if cell.flags.contains(Flags::UNDERLINE) {
            style = style.underline();
        }
        let text = Text::with_text_style(
            s,
            Point::new(
                col as i32 * CHAR_SIZE.width as i32,
                row as i32 * CHAR_SIZE.height as i32,
            ),
            style.build(),
            TextStyle::with_baseline(Baseline::Top),
        );
        text.draw(&mut self.graphic).ok();
    }
}
