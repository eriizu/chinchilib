use std::usize;

use pixels::{Pixels, SurfaceTexture};
use winit::window::{Window, WindowId};

mod raycast;

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
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = WinitHandler::default();
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Eq, Hash, PartialEq)]
enum MyKeys {
    KeyA,
    KeyZ,
    KeyE,
    KeyQ,
    KeyS,
    KeyD,
    Up,
    Down,
    Left,
    Right,
}

impl std::convert::TryFrom<&winit::keyboard::Key> for MyKeys {
    type Error = ();
    fn try_from(value: &winit::keyboard::Key) -> Result<Self, ()> {
        use winit::keyboard::{Key, NamedKey};
        match value {
            Key::Named(NamedKey::ArrowLeft) => Some(MyKeys::Left),
            Key::Named(NamedKey::ArrowRight) => Some(MyKeys::Right),
            Key::Named(NamedKey::ArrowUp) => Some(MyKeys::Up),
            Key::Named(NamedKey::ArrowDown) => Some(MyKeys::Down),
            Key::Character(name) if name == "q" => Some(MyKeys::KeyQ),
            Key::Character(name) if name == "d" => Some(MyKeys::KeyD),
            Key::Character(name) if name == "z" => Some(MyKeys::KeyZ),
            Key::Character(name) if name == "s" => Some(MyKeys::KeyS),
            Key::Character(name) if name == "a" => Some(MyKeys::KeyA),
            Key::Character(name) if name == "e" => Some(MyKeys::KeyE),
            _ => None,
        }
        .ok_or(())
    }
}

/// Everyting about the window. Pixels and Window are options because they
/// are constructed on "resume" and cannot be construted earlier
struct WinitHandler {
    app: Option<App>,
    height: usize,
    width: usize,
    last_frame: std::time::Instant,
    tick: std::time::Duration,
}

impl Default for WinitHandler {
    fn default() -> Self {
        Self {
            app: None,
            height: 240,
            width: 320,
            last_frame: std::time::Instant::now(),
            tick: std::time::Duration::from_millis(100),
        }
    }
}

impl winit::application::ApplicationHandler for WinitHandler {
    /// Resume gets called when window gets loaded for the first time
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("resumed called");
        self.app = Some(App::new(event_loop, self.width, self.height));
    }

    /// Instead of redrawing for every event, or every keyprss, we only try to
    /// render after all evens have been processed.
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let app = self.app.as_mut().unwrap();
        if app.pressed_keys.len() != 0 {
            log::debug!("not waiting there are keys currently pressed");
            app.on_tick();
            app.window.request_redraw();
        } else {
            // log::debug!("waiting nothing to do");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: WindowId,
        event: winit::event::WindowEvent,
    ) {
        log::debug!("got event {:?}", event);
        let app = self.app.as_mut().unwrap();
        use winit::event::WindowEvent;
        match event {
            WindowEvent::CloseRequested => {
                log::info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => app.process_resize(size),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.repeat == false => app.process_kbd_input(event, event_loop),
            WindowEvent::RedrawRequested => app.on_redraw(),
            _ => {}
        }
    }
}

pub fn put_pixel1(frame: &mut [u8], width: usize, x: usize, y: usize, color: rgb::RGBA8) {
    use rgb::*;
    let idx = width * y + x;
    frame.as_rgba_mut()[idx] = color;
}

struct App {
    window: Window,
    pixels: Pixels,
    pause: bool,
    height: usize,
    width: usize,
    pressed_keys: std::collections::HashSet<MyKeys>,
    moving_pixel: MovingPixel,
}

impl App {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop, width: usize, height: usize) -> Self {
        let mut attr = Window::default_attributes();
        let size = winit::dpi::PhysicalSize::new(width as u16, height as u16);
        attr = attr.with_inner_size(size).with_title("Box");
        let win = event_loop.create_window(attr).unwrap();

        let mut pixels = {
            let surface_texture = SurfaceTexture::new(width as u32, height as u32, &win);
            Pixels::new(width as u32, height as u32, surface_texture).unwrap()
        };
        pixels.clear_color(pixels::wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 255.0,
            a: 255.0,
        });
        Self {
            window: win,
            pixels,
            height,
            width,
            pause: false,
            pressed_keys: std::collections::HashSet::new(),
            moving_pixel: MovingPixel::new(width / 2, height / 2),
        }
    }

    fn on_redraw(&mut self) {
        log::debug!("redrawing");

        self.moving_pixel.draw(&mut self.pixels, self.width);

        if let Err(err) = self.pixels.render() {
            log::error!("failed to render with error {}", err);
            return;
        }
    }

    fn on_tick(&mut self) {
        self.moving_pixel.on_tick(&self.pressed_keys);
    }

    fn process_kbd_input(
        &mut self,
        event: winit::event::KeyEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        use winit::keyboard::{Key, NamedKey};
        if let Ok(my_key) = (&event.logical_key).try_into() {
            if event.state == winit::event::ElementState::Pressed {
                self.pressed_keys.insert(my_key);
            } else if event.state == winit::event::ElementState::Released {
                self.pressed_keys.remove(&my_key);
            }
        };
        if event.state == winit::event::ElementState::Pressed {
            match event.logical_key {
                Key::Named(NamedKey::Escape) => event_loop.exit(),
                Key::Named(NamedKey::Space) => {
                    self.pause = !self.pause;
                }
                _ => {}
            }
        }
    }

    fn process_resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.width = size.width as usize;
        self.height = size.height as usize;
        self.pixels.resize_surface(size.width, size.height).unwrap();
        self.pixels.resize_buffer(size.width, size.height).unwrap();
        self.window.request_redraw();
    }
}

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
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<MyKeys>) {
        for key in pressed_keys {
            match key {
                MyKeys::Left => {
                    self.pos.0 -= 1;
                }
                MyKeys::KeyQ => {}
                MyKeys::Right => {
                    self.pos.0 += 1;
                }
                MyKeys::KeyD => {}
                MyKeys::Up => {
                    self.pos.1 -= 1;
                }
                MyKeys::KeyZ => {}
                MyKeys::Down => {
                    self.pos.1 += 1;
                }
                MyKeys::KeyS => {}
                MyKeys::KeyA => {}
                MyKeys::KeyE => {}
            }
        }
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
