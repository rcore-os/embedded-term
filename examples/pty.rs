use embedded_graphics::prelude::Size;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use mio::unix::EventedFd;
use mio::*;
use pty::fork::*;
use rcore_console::{Console, DrawTarget, Pixel, Rgb888};
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

    if let Some(mut master) = fork.is_parent().ok() {
        env_logger::init();
        let (width, height) = (800, 600);
        let display = SimulatorDisplay::<Rgb888>::new(Size::new(width, height));
        let display = RefCell::new(display);

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
                        master.write(&buffer[..len]).unwrap();
                    }
                    _ => unreachable!(),
                }
            }

            master.write(&console.get_result()).unwrap();

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

impl DrawTarget<Rgb888> for DisplayWrapper<'_> {
    type Error = Infallible;

    fn draw_pixel(&mut self, item: Pixel<Rgb888>) -> core::result::Result<(), Self::Error> {
        self.0.borrow_mut().draw_pixel(item)
    }

    fn size(&self) -> Size {
        self.0.borrow().size()
    }
}
