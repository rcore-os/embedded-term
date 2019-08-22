use crate::text_buffer::*;
use embedded_graphics::{fonts::Font8x16, pixelcolor::Rgb888, prelude::*, primitives::Line};

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `Drawing<Rgb888>` trait
/// to draw pixels in RGB format.
pub struct TextOnGraphic<D>
where
    D: Drawing<Rgb888>,
{
    width: u32,
    height: u32,
    graphic: D,
}

impl<D> TextOnGraphic<D>
where
    D: Drawing<Rgb888>,
{
    pub fn new(width: u32, height: u32, graphic: D) -> Self {
        TextOnGraphic {
            width,
            height,
            graphic,
        }
    }
}

impl<D> TextBuffer for TextOnGraphic<D>
where
    D: Drawing<Rgb888>,
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
        let mut style = Style {
            fill_color: Some(ch.attr.background),
            stroke_color: Some(ch.attr.foreground),
            stroke_width: if ch.attr.bold { 5 } else { 1 },
        };
        if ch.attr.reverse {
            core::mem::swap(&mut style.fill_color, &mut style.stroke_color);
        }
        let (x, y) = (col as i32 * 8, row as i32 * 16);
        let item = Font8x16::render_str(s)
            .style(style)
            .translate(Coord::new(x, y));
        self.graphic.draw(item);
        if ch.attr.strikethrough {
            let line = Line::new(Coord::new(x, y + 8), Coord::new(x + 8, y + 8)).style(style);
            self.graphic.draw(line);
        }
        if ch.attr.underline {
            let line = Line::new(Coord::new(x, y + 15), Coord::new(x + 8, y + 15)).style(style);
            self.graphic.draw(line);
        }
    }
}
