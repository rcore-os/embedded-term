use crate::text_buffer::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyleBuilder},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyleBuilder},
    text::Text,
};

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement [`DrawTarget`] trait
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
    pub fn new(graphic: D) -> Self {
        TextOnGraphic {
            width: graphic.bounding_box().size.width,
            height: graphic.bounding_box().size.height,
            graphic,
        }
    }
}

impl<D> TextBuffer for TextOnGraphic<D>
where
    D: DrawTarget<Color = Rgb888>,
{
    fn width(&self) -> usize {
        self.width as usize / 8
    }
    fn height(&self) -> usize {
        self.height as usize / 16
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
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_8X13)
            .text_color(foreground)
            .background_color(background)
            .build();
        let (x, y) = (col as i32 * 8, row as i32 * 16);
        let _ = Text::new(s, Point::new(x, y), style).draw(&mut self.graphic);

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground)
            .stroke_width(if ch.attr.bold { 5 } else { 1 })
            .fill_color(background)
            .build();
        if ch.attr.strikethrough {
            let _ = Line::new(Point::new(x, y + 8), Point::new(x + 8, y + 8))
                .into_styled(style)
                .draw(&mut self.graphic);
        }
        if ch.attr.underline {
            let _ = Line::new(Point::new(x, y + 15), Point::new(x + 8, y + 15))
                .into_styled(style)
                .draw(&mut self.graphic);
        }
    }
}
