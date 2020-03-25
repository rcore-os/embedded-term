use crate::text_buffer::*;
use embedded_graphics::{
    egline, egtext, fonts::Font8x16, pixelcolor::Rgb888, prelude::*, primitive_style, text_style,
};

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `DrawTarget<Rgb888>` trait
/// to draw pixels in RGB format.
pub struct TextOnGraphic<D>
where
    D: DrawTarget<Rgb888>,
{
    width: u32,
    height: u32,
    graphic: D,
}

impl<D> TextOnGraphic<D>
where
    D: DrawTarget<Rgb888>,
{
    /// Create a new text buffer on graphic.
    pub fn new(graphic: D) -> Self {
        TextOnGraphic {
            width: graphic.size().width,
            height: graphic.size().height,
            graphic,
        }
    }
}

impl<D> TextBuffer for TextOnGraphic<D>
where
    D: DrawTarget<Rgb888>,
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
        let style = text_style!(
            font = Font8x16,
            text_color = foreground,
            background_color = background,
        );
        let (x, y) = (col as i32 * 8, row as i32 * 16);
        let text = egtext!(text = s, top_left = (x, y), style = style);
        let _ = text.draw(&mut self.graphic);

        let style = primitive_style!(
            stroke_color = foreground,
            fill_color = background,
            stroke_width = if ch.attr.bold { 5 } else { 1 },
        );
        if ch.attr.strikethrough {
            let line = egline!(start = (x, y + 8), end = (x + 8, y + 8), style = style);
            let _ = line.draw(&mut self.graphic);
        }
        if ch.attr.underline {
            let line = egline!(start = (x, y + 15), end = (x + 8, y + 15), style = style);
            let _ = line.draw(&mut self.graphic);
        }
    }
}
