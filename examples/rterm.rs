use embedded_graphics::{fonts::Font8x16, pixelcolor::Rgb888, prelude::*, text_8x16};
use embedded_graphics_simulator::{DisplayBuilder, RgbDisplay};
use rcore_console::console::Console;
use rcore_console::graphic::TextOnGraphic;
use std::cell::RefCell;

fn main() {
    let (width, height) = (320, 200);
    let display = RefCell::new(DisplayBuilder::new().size(width, height).build_rgb());

    let mut console =
        Console::on_frame_buffer(width as u32, height as u32, DisplayWrapper(&display));

    ncurses::initscr();
    ncurses::raw();
    ncurses::noecho();
    while !display.borrow_mut().run_once() {
        let c = ncurses::getch() as u8;
        if c == 0xff {
            break;
        }
        console.write_byte(c);
    }
}

struct DisplayWrapper<'a>(&'a RefCell<RgbDisplay>);

impl Drawing<Rgb888> for DisplayWrapper<'_> {
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<Rgb888>>,
    {
        self.0.borrow_mut().draw(item)
    }
}
