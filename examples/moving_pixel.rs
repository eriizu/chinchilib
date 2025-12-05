use chinchilib::pixels::Pixels;
use chinchilib::rgb;
use chinchilib::{put_pixel, GfxApp, Key, WinitHandler};

fn main() {
    env_logger::init();

    log::info!("Hello, world!");

    let moving_pixel = Box::new(MovingPixel::new(50, 100));
    let mut app = WinitHandler::new(moving_pixel, (500, 500), 60);
    // We don't have any physics or animations, false helps to preserve performance.
    app.set_always_tick(false);
    app.run().unwrap();
}

/// Example app that only feature a pixel that moves.
struct MovingPixel {
    pos: (usize, usize),
}

impl Default for MovingPixel {
    fn default() -> Self {
        Self { pos: (0, 0) }
    }
}

impl MovingPixel {
    fn new(x: usize, y: usize) -> Self {
        Self { pos: (x, y) }
    }
}

const RED: rgb::RGBA8 = rgb::RGBA8 {
    r: u8::MAX,
    g: 0,
    b: 0,
    a: u8::MAX,
};

impl GfxApp for MovingPixel {
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<Key>) -> bool {
        let mut needs_redraw = true;
        for key in pressed_keys {
            match key {
                Key::Left => {
                    self.pos.0 -= 1;
                }
                Key::Right => {
                    self.pos.0 += 1;
                }
                Key::Up => {
                    self.pos.1 -= 1;
                }
                Key::Down => {
                    self.pos.1 += 1;
                }
                _ => {
                    needs_redraw = false;
                }
            }
        }
        needs_redraw
    }

    fn draw(&mut self, pixels: &mut Pixels, width: usize) {
        if self.pos.0 * self.pos.1 < pixels.frame().len() {
            put_pixel(pixels.frame_mut(), width, self.pos.0, self.pos.1, RED);
        }
    }

    /// For the sake of the example, when x goes under 50, we inidcate that we are done and that
    /// the windows should remain open, when y goes under 50 we indicate that the window should
    /// close, otherwise we are not done.
    fn done(&self) -> chinchilib::DoneStatus {
        if self.pos.0 < 50 {
            chinchilib::DoneStatus::Remain
        } else if self.pos.1 < 50 {
            chinchilib::DoneStatus::Exit
        } else {
            chinchilib::DoneStatus::NotDone
        }
    }
}
