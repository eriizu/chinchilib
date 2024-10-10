use chinchilib::{put_pixel1, GfxApp, MyKeys, WinitHandler};
use pixels::Pixels;

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
impl GfxApp for MovingPixel {
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<MyKeys>) -> bool {
        let mut ret = false;
        for key in pressed_keys {
            match key {
                MyKeys::Left => {
                    self.pos.0 -= 1;
                    ret = true;
                }
                MyKeys::KeyQ => {}
                MyKeys::Right => {
                    self.pos.0 += 1;
                    ret = true;
                }
                MyKeys::KeyD => {}
                MyKeys::Up => {
                    self.pos.1 -= 1;
                    ret = true;
                }
                MyKeys::KeyZ => {}
                MyKeys::Down => {
                    self.pos.1 += 1;
                    ret = true;
                }
                MyKeys::KeyS => {}
                MyKeys::KeyA => {}
                MyKeys::KeyE => {}
            }
        }
        ret
    }

    fn draw(&self, pixels: &mut Pixels, width: usize) {
        if self.pos.0 * self.pos.1 < pixels.frame().len() {
            put_pixel1(
                pixels.frame_mut(),
                width,
                self.pos.0,
                self.pos.1,
                rgb::RGBA {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            );
        }
    }

    /// Moving pixel is never done.
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

fn main() {
    env_logger::init();

    log::info!("Hello, world!");

    let moving_pixel = Box::new(MovingPixel::new(50, 100));
    let mut app = WinitHandler::new(moving_pixel, (500, 500), 60);
    app.run().unwrap();
}
