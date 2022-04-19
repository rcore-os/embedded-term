use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_term::Console;

use std::convert::Infallible;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    env_logger::init();
    let display = SimulatorDisplay::<Rgb888>::new(Size::new(800, 600));
    let display = Arc::new(Mutex::new(display));

    let mut console = Console::on_frame_buffer(DisplayWrapper(display.clone()));
    std::thread::spawn(move || {
        for c in std::io::stdin().lock().bytes() {
            let c = c.unwrap();
            if c == 0xff {
                break;
            }
            console.write_byte(c);
        }
    });

    let output_settings = OutputSettingsBuilder::new().build();
    let mut window = Window::new("rterm", &output_settings);
    loop {
        window.update(&display.lock().unwrap());
        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

struct DisplayWrapper(Arc<Mutex<SimulatorDisplay<Rgb888>>>);

impl Dimensions for DisplayWrapper {
    fn bounding_box(&self) -> Rectangle {
        self.0.lock().unwrap().bounding_box()
    }
}

impl DrawTarget for DisplayWrapper {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.lock().unwrap().draw_iter(pixels)
    }
}
