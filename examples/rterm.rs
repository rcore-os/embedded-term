use embedded_graphics::{fonts::Font8x16, pixelcolor::Rgb888, prelude::*, text_8x16};
use embedded_graphics_simulator::{DisplayBuilder, RgbDisplay};
use rcore_console::console::Console;
use rcore_console::graphic::TextOnGraphic;
use std::cell::RefCell;


fn main() {
    let (width, height) = (320, 200);
    let display = RefCell::new(DisplayBuilder::new().size(width, height).build_rgb());

    let graphic = TextOnGraphic::new(width as u32, height as u32, DisplayWrapper(&display));
    let mut console = Console::new(graphic);

    ncurses::initscr();
    ncurses::raw();
    while !display.borrow_mut().run_once() {
        let c = ncurses::getch() as u8;
        console.write_byte(c);
    }
}

struct DisplayWrapper<'a>(&'a RefCell<RgbDisplay>);

impl Drawing<Rgb888> for DisplayWrapper<'_>
{
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<Rgb888>>,
    {
        self.0.borrow_mut().draw(item)
    }
}
