use embedded_graphics::prelude::Size;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use rcore_console::{Console, DrawTarget, Pixel, Rgb888};

use std::convert::Infallible;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    env_logger::init();
    let (width, height) = (800, 600);
    let display = SimulatorDisplay::<Rgb888>::new(Size::new(width, height));
    let display = Arc::new(Mutex::new(display));

    let mut console =
        Console::on_frame_buffer(width as u32, height as u32, DisplayWrapper(display.clone()));
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
    let mut window = Window::new("Example", &output_settings);
    loop {
        window.update(&display.lock().unwrap());
        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

struct DisplayWrapper(Arc<Mutex<SimulatorDisplay<Rgb888>>>);

impl DrawTarget<Rgb888> for DisplayWrapper {
    type Error = Infallible;

    fn draw_pixel(&mut self, item: Pixel<Rgb888>) -> Result<(), Self::Error> {
        self.0.lock().unwrap().draw_pixel(item)
    }

    fn size(&self) -> Size {
        self.0.lock().unwrap().size()
    }
}
