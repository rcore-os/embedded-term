use crate::text_buffer::*;
use embedded_graphics::{fonts::Font8x16, pixelcolor::Rgb888, prelude::*, primitives::Line};

/// A `TextBuffer` implementation on the top of `embedded_graphics::Drawing` trait
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
        let chs = [ch.ascii_char];
        let s = core::str::from_utf8(&chs).unwrap();
        let mut style = Style {
            fill_color: Some(Rgb888::from(ch.attr.background)),
            stroke_color: Some(Rgb888::from(ch.attr.foreground)),
            stroke_width: 1,
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
