use embedded_graphics_simulator::{DisplayBuilder, RgbDisplay};
use mio::unix::EventedFd;
use mio::*;
use pty::fork::*;
use rcore_console::{Console, Drawing, Pixel, Rgb888};
use std::cell::RefCell;
use std::env::args_os;
use std::io::stdin;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::time::Duration;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        env_logger::init();
        let (width, height) = (800, 600);
        let display = RefCell::new(DisplayBuilder::new().size(width, height).build_rgb());

        let mut console =
            Console::on_frame_buffer(width as u32, height as u32, DisplayWrapper(&display));

        let poll = Poll::new().unwrap();
        poll.register(
            &EventedFd(&master.as_raw_fd()),
            Token(0),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        poll.register(
            &EventedFd(&stdin().as_raw_fd()),
            Token(1),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let mut events = Events::with_capacity(1024);

        display.borrow_mut().run_once();

        loop {
            poll.poll(&mut events, Some(Duration::from_millis(10)))
                .unwrap();

            let mut buffer = [0u8; 10240];
            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        let len = master.read(&mut buffer).unwrap();
                        for c in &buffer[..len] {
                            console.write_byte(*c);
                        }
                        display.borrow_mut().run_once();
                    }
                    Token(1) => {
                        let len = stdin().read(&mut buffer).unwrap();
                        master.write(&buffer[..len]).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
    } else {
        let mut args = args_os();
        args.next(); // skip myself
        let name = args.next().unwrap();
        Command::new(name)
            .args(args)
            .status()
            .expect("could not execute program");
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
