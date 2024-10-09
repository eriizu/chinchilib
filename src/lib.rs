use std::usize;

use pixels::{Pixels, SurfaceTexture};
use winit::window::{Window, WindowId};

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
    height: usize,
    width: usize,
    last_frame: std::time::Instant,
    tick: std::time::Duration,
    app: Option<Box<dyn GfxApp>>,
}

impl WinitHandler {
    pub fn new(app: Box<dyn GfxApp>) -> Self {
        Self {
            winfbx: None,
            height: 240,
            width: 320,
            last_frame: std::time::Instant::now(),
            tick: std::time::Duration::from_nanos(16666666),
            app: Some(app),
        }
    }
}

impl winit::application::ApplicationHandler for WinitHandler {
    /// Resume gets called when window gets loaded for the first time
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("resumed called");
        if let Some(app) = self.app.take() {
            self.winfbx = Some(WinFbx::new(event_loop, self.width, self.height, app));
        }
    }

    /// Instead of redrawing for every event, or every keyprss, we only try to
    /// render after all evens have been processed.
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let app = self
            .winfbx
            .as_mut()
            .expect("about_to_wait not to be called if window doesn't exist.");

        if app.pressed_keys.len() != 0 {
            // INFO: there is actually noting to do on key still pressed
            // due to new key management logic
        }
        let now = std::time::Instant::now();
        let duration_from = now.duration_since(self.last_frame);
        if duration_from >= self.tick {
            self.last_frame = now;
            app.on_tick();
            log::debug!("requesting redraw from about to wait");
            app.window.request_redraw();
        } else {
            log::debug!("waiting nothing to do");
            let duration_to = self.tick - duration_from;
            std::thread::sleep(duration_to);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: WindowId,
        event: winit::event::WindowEvent,
    ) {
        log::debug!("got event {:?}", event);
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
            _ => {}
        }
    }
}

pub fn put_pixel1(frame: &mut [u8], width: usize, x: usize, y: usize, color: rgb::RGBA8) {
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
        log::debug!("redrawing");

        if self.needs_render {
            self.app.draw(&mut self.pixels, self.width);

            if let Err(err) = self.pixels.render() {
                log::error!("failed to render with error {}", err);
                return;
            }
        }
        self.needs_render = false;
    }

    fn on_tick(&mut self) {
        self.needs_render = self.app.on_tick(&self.pressed_keys);
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
                // INFO: released keys are not immediatly removed from pressed keys
                // as there are yet to be processed by the tick.
                // They will be removed after the tick.
                self.released_keys.insert(my_key);
                // self.pressed_keys.remove(&my_key);
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

pub trait GfxApp {
    /// Every tick, this method gets called with currently pressed keys. Released keys during the tick are considered still pressed. But will be removed after this call.
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<MyKeys>) -> bool;

    /// You get the pixel array, so you can draw on it before the render.
    fn draw(&self, pixels: &mut Pixels, width: usize);
}
