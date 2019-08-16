use embedded_graphics_simulator::{DisplayBuilder, RgbDisplay};
use rcore_console::{Console, Rgb888, Drawing, Pixel};
use std::cell::RefCell;
use std::io;
use std::io::Read;

fn main() {
    env_logger::init();
    let (width, height) = (320, 200);
    let display = RefCell::new(DisplayBuilder::new().size(width, height).build_rgb());

    let mut console =
        Console::on_frame_buffer(width as u32, height as u32, DisplayWrapper(&display));

    for c in io::stdin().lock().bytes() {
        let c = c.unwrap();
        if c == 0xff {
            break;
        }
        console.write_byte(c);
        display.borrow_mut().run_once();
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
