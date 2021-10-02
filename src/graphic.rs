use crate::text_buffer::{ConsoleChar, TextBuffer};
use embedded_graphics::{
    mono_font::{
        iso_8859_1::{FONT_9X18, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Drawable, Point, Size},
    text::{Baseline, Text, TextStyle},
};

const CHAR_SIZE: Size = FONT_9X18.character_size;

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `DrawTarget<Rgb888>` trait
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
    fn width(&self) -> usize {
        (self.width / CHAR_SIZE.width) as usize
    }
    fn height(&self) -> usize {
        (self.height / CHAR_SIZE.height) as usize
    }
    fn read(&self, _row: usize, _col: usize) -> ConsoleChar {
        unimplemented!("reading char from graphic is unsupported")
    }
    fn write(&mut self, row: usize, col: usize, ch: ConsoleChar) {
        let mut utf8_buf = [0u8; 8];
        let s = ch.char.encode_utf8(&mut utf8_buf);
        let (foreground, background) = if ch.attr.reverse {
            (ch.attr.background, ch.attr.foreground)
        } else {
            (ch.attr.foreground, ch.attr.background)
        };
        let mut style = MonoTextStyleBuilder::new()
            .text_color(foreground)
            .background_color(background);
        if ch.attr.bold {
            style = style.font(&FONT_9X18_BOLD);
        } else {
            style = style.font(&FONT_9X18);
        }
        if ch.attr.strikethrough {
            style = style.strikethrough();
        }
        if ch.attr.underline {
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
