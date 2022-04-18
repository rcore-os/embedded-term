use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use mio::unix::EventedFd;
use mio::*;
use pty::fork::*;
use rcore_console::Console;
use std::cell::RefCell;
use std::convert::Infallible;
use std::env::args_os;
use std::fs::File;
use std::io::stdin;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::process::Command;
use std::time::Duration;
use termios::*;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    if let Ok(mut master) = fork.is_parent() {
        env_logger::init();
        let display = SimulatorDisplay::<Rgb888>::new(Size::new(800, 600));
        let display = RefCell::new(display);

        let mut console = Console::on_frame_buffer(DisplayWrapper(&display));

        let poll = Poll::new().unwrap();
        poll.register(
            &EventedFd(&master.as_raw_fd()),
            Token(0),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();

        // set to raw mode
        let fd = stdin().as_raw_fd();
        let mut termios = Termios::from_fd(fd).unwrap();
        cfmakeraw(&mut termios);
        tcsetattr(fd, TCSANOW, &termios).unwrap();

        let mut stdin = unsafe { File::from_raw_fd(fd) };
        poll.register(
            &EventedFd(&fd),
            Token(1),
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let mut events = Events::with_capacity(1024);

        let output_settings = OutputSettingsBuilder::new().build();
        let mut window = Window::new("Example", &output_settings);

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
                    }
                    Token(1) => {
                        let len = stdin.read(&mut buffer).unwrap();
                        master.write_all(&buffer[..len]).unwrap();
                    }
                    _ => unreachable!(),
                }
            }

            master.write_all(&console.get_result()).unwrap();

            window.update(&display.borrow_mut());
            if window.events().any(|e| e == SimulatorEvent::Quit) {
                break;
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

struct DisplayWrapper<'a>(&'a RefCell<SimulatorDisplay<Rgb888>>);

impl Dimensions for DisplayWrapper<'_> {
    fn bounding_box(&self) -> Rectangle {
        self.0.borrow().bounding_box()
    }
}

impl DrawTarget for DisplayWrapper<'_> {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.borrow_mut().draw_iter(pixels)
    }
}
