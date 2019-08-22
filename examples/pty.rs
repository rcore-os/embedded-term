extern crate pty;

use embedded_graphics_simulator::{DisplayBuilder, RgbDisplay};
use pty::fork::*;
use rcore_console::{Console, Drawing, Pixel, Rgb888};
use std::cell::RefCell;
use std::env::args_os;
use std::io::Read;
use std::process::Command;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    if let Some(master) = fork.is_parent().ok() {
        env_logger::init();
        let (width, height) = (800, 600);
        let display = RefCell::new(DisplayBuilder::new().size(width, height).build_rgb());

        let mut console =
            Console::on_frame_buffer(width as u32, height as u32, DisplayWrapper(&display));

        display.borrow_mut().run_once();
        for c in master.bytes() {
            let c = c.unwrap();
            if c == 0xff {
                break;
            }
            console.write_byte(c);
            display.borrow_mut().run_once();
        }
    } else {
        let mut args = args_os().peekable();
        args.next();
        Command::new(args.peek().unwrap())
            .args(args)
            .status()
            .expect("could not execute tty");
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
