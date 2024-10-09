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
}

fn main() {
    env_logger::init();

    log::info!("Hello, world!");
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let moving_pixel = Box::new(MovingPixel::new(100, 100));
    let mut app = WinitHandler::new(moving_pixel);
    event_loop.run_app(&mut app).unwrap();
}
