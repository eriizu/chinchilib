use std::usize;

pub use pixels;
use pixels::{Pixels, SurfaceTexture};
pub use rgb;
pub use winit;
use winit::window::{Window, WindowId};

/// Mapping for the keys that are recognized. They are centered an AZERTY keyboard's essential keys
/// needed for games.
/// TODO: makes this less centered arround AZERTY
#[derive(Eq, Hash, PartialEq)]
pub enum MyKeys {
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
pub struct WinitHandler {
    winfbx: Option<WinFbx>,
    width: usize,
    height: usize,
    last_frame: std::time::Instant,
    tick: std::time::Duration,
    /// Set to true if your app has something special to do at every tick even if there are no user
    /// events. This can be used if you have physics or an animation to run. Defaults to false to
    /// preserve performance.
    always_tick: bool,
    app: Option<Box<dyn GfxApp>>,
    cursor_pos: (f64, f64),
}

fn hz_to_nanosec_period(hz: u16) -> u64 {
    let nano_period = 1.0 / hz as f64 * 1_000_000_000.0;
    nano_period as u64
}

#[cfg(test)]
mod test {
    #[test]
    fn hz_to_nanosec_period() {
        assert_eq!(super::hz_to_nanosec_period(60), 16_666_666);
        assert_eq!(super::hz_to_nanosec_period(1), 1_000_000_000);
    }
}

impl WinitHandler {
    /// Create a new handler with an app, a window size and a desired tick rate. Run app with
    /// `.run()`
    pub fn new(app: Box<dyn GfxApp>, size: (usize, usize), tick_per_second: u16) -> Self {
        let nsec_period = hz_to_nanosec_period(tick_per_second);
        Self {
            winfbx: None,
            width: size.0,
            height: size.1,
            last_frame: std::time::Instant::now(),
            tick: std::time::Duration::from_nanos(nsec_period),
            app: Some(app),
            cursor_pos: (0.0, 0.0),
            always_tick: false,
        }
    }

    pub fn run(&mut self) -> Result<(), winit::error::EventLoopError> {
        let event_loop = winit::event_loop::EventLoop::new()?;

        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        event_loop.run_app(self)?;
        Ok(())
    }

    /// Set to true if your app has something special to do at every tick even if there are no user
    /// events. This can be used if you have physics or an animation to run. Defaults to false to
    /// preserve performance.
    pub fn set_always_tick(&mut self, val: bool) {
        self.always_tick = val;
    }
}

impl winit::application::ApplicationHandler for WinitHandler {
    /// Resume gets called when window gets loaded for the first time
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!(".resumed() called, creating window");
        if let Some(app) = self.app.take() {
            self.winfbx = Some(WinFbx::new(event_loop, self.width, self.height, app));
        }
    }

    /// Instead of redrawing for every event, or every keyprss, we only try to
    /// render after all evens have been processed.
    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let app = self
            .winfbx
            .as_mut()
            .expect("about_to_wait not to be called if window doesn't exist.");

        if app.done() {
            event_loop.exit();
            return;
        }
        let now = std::time::Instant::now();
        let duration_from_last_tick = now.duration_since(self.last_frame);
        // If time since last tick is greater or equal than tickrate, we want to prompt that app
        // for a redraw.
        // Otherwise if they are key pressed we wait for next tick. If none are pressed we wait
        // until we get an event.
        // TODO: condition this behaviour to a flag
        if duration_from_last_tick >= self.tick {
            self.last_frame = now;
            app.on_tick();
            app.window.request_redraw();
        } else {
            if self.always_tick || !app.pressed_keys.is_empty() {
                let duration_to_next_tick = self.tick - duration_from_last_tick;
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    now + duration_to_next_tick,
                ));
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: WindowId,
        event: winit::event::WindowEvent,
    ) {
        let app = self
            .winfbx
            .as_mut()
            .expect("window_event not to be called if window doesn't exist.");
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
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.cursor_pos = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button: _,
            } if state.is_pressed() => {
                log::info!(
                    "clicked at x: {}, y: {}",
                    self.cursor_pos.0,
                    self.cursor_pos.1
                )
            }
            _ => {}
        }
    }
}

pub fn put_pixel(frame: &mut [u8], width: usize, x: usize, y: usize, color: rgb::RGBA8) {
    use rgb::*;
    let idx = width * y + x;
    frame.as_rgba_mut()[idx] = color;
}

/// Manages the actual winit::Window, the Pixels, handles resizes, records pressed keys into a
/// custom structure and call the given app tick and draw methods.
struct WinFbx {
    window: Window,
    pixels: Pixels,
    pause: bool,
    height: usize,
    width: usize,
    pressed_keys: std::collections::HashSet<MyKeys>,
    released_keys: std::collections::HashSet<MyKeys>,
    needs_render: bool,
    app: Box<dyn GfxApp>,
}

impl WinFbx {
    fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        width: usize,
        height: usize,
        app: Box<dyn GfxApp>,
    ) -> Self {
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
            released_keys: std::collections::HashSet::new(),
            needs_render: true,
            app,
        }
    }

    fn on_redraw(&mut self) {
        if self.needs_render {
            self.app.draw(&mut self.pixels, self.width);

            if let Err(err) = self.pixels.render() {
                log::error!("failed to render with error {}", err);
                return;
            }
        }
        self.needs_render = false;
    }

    fn done(&self) -> bool {
        self.app.done() == DoneStatus::Exit
    }

    fn on_tick(&mut self) {
        if self.app.done() == DoneStatus::NotDone {
            self.needs_render = self.app.on_tick(&self.pressed_keys);
        }
        self.pressed_keys
            .retain(|candidate| !self.released_keys.contains(candidate));
        self.released_keys.clear();
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
                self.released_keys.insert(my_key);
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
        self.needs_render = true;
    }
}

#[derive(Eq, PartialEq)]
pub enum DoneStatus {
    /// The program should quit, the app has nothing left to do.
    Exit,
    /// The program should remain open, but the app is done. Useful when you want the result of the
    /// app to stay on the screen. On `Remain` the `draw` and `on_tick` methodes will not be called
    /// anymore.
    Remain,
    /// The program should continue, the app is not done.
    NotDone,
}

pub trait GfxApp {
    /// Every tick, this method gets called with currently pressed keys. Released keys during the tick are considered still pressed. But will be removed after this call.
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<MyKeys>) -> bool;

    /// You get the pixel array, so you can draw on it before the render.
    fn draw(&self, pixels: &mut Pixels, width: usize);

    /// Indicate if the app logic is done and if the program should remain or exit. For oneshot
    /// drawing, return `DoneStatus::Remain` so that the result stays on screen.
    fn done(&self) -> DoneStatus;
}
