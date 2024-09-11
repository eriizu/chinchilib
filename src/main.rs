use std::usize;

use pixels::{Pixels, SurfaceTexture};
use rayon::{iter::ParallelIterator, prelude::*};
use winit::{
    event::ElementState,
    window::{Window, WindowId},
};

mod raycast;

fn main() {
    env_logger::init();

    log::info!("Hello, world!");
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    // event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::default();
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

struct App {
    window: Option<Window>,
    pixels: Option<Pixels>,
    pause: bool,
    height: usize,
    width: usize,
    pressed_keys: std::collections::HashSet<MyKeys>,
    my_pos: (usize, usize),
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            pixels: None,
            pause: false,
            height: 240,
            width: 320,
            pressed_keys: std::collections::HashSet::new(),
            my_pos: (200, 100),
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("resumed called");
        let mut attr = Window::default_attributes();
        let size = winit::dpi::PhysicalSize::new(self.width as u16, self.height as u16);
        attr = attr.with_inner_size(size).with_title("Raycaster");
        let win = event_loop.create_window(attr).unwrap();
        self.pixels = Some({
            let surface_texture = SurfaceTexture::new(self.width as u32, self.height as u32, &win);
            Pixels::new(self.width as u32, self.height as u32, surface_texture).unwrap()
        });
        self.pixels
            .as_mut()
            .unwrap()
            .clear_color(pixels::wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 255.0,
                a: 255.0,
            });
        self.window = Some(win);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.pressed_keys.len() != 0 {
            log::info!("not waiting");
            self.window.as_ref().unwrap().request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;
        match event {
            WindowEvent::CloseRequested => {
                log::info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => self.process_resize(size),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.repeat == false => self.process_kbd_input(event, event_loop),
            WindowEvent::RedrawRequested => self.process_redraw(),
            _ => {}
        }
    }
}

impl App {
    fn process_redraw(&mut self) {
        log::info!("redrawing");
        use MyKeys;
        for key in &self.pressed_keys {
            match key {
                MyKeys::Left => {
                    self.my_pos.0 -= 1;
                }
                MyKeys::KeyQ => {}
                MyKeys::Right => {
                    self.my_pos.0 += 1;
                }
                MyKeys::KeyD => {}
                MyKeys::Up => {
                    self.my_pos.1 -= 1;
                }
                MyKeys::KeyZ => {}
                MyKeys::Down => {
                    self.my_pos.1 += 1;
                }
                MyKeys::KeyS => {}
                MyKeys::KeyA => {}
                MyKeys::KeyE => {}
            }
        }

        let pixels = &mut self.pixels.as_mut().unwrap();
        put_pixel1(
            pixels.frame_mut(),
            self.width,
            self.my_pos.0,
            self.my_pos.1,
            rgb::RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        );

        if let Err(err) = self.pixels.as_mut().unwrap().render() {
            log::error!("failed to render with error {}", err);
            return;
        }
    }

    fn process_kbd_input(
        &mut self,
        event: winit::event::KeyEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        use winit::keyboard::{Key, NamedKey};
        if let Some(my_key) = match &event.logical_key {
            Key::Named(NamedKey::ArrowLeft) => Some(MyKeys::Left),
            Key::Character(name) if name == "q" => Some(MyKeys::KeyQ),
            Key::Named(NamedKey::ArrowRight) => Some(MyKeys::Right),
            Key::Character(name) if name == "d" => Some(MyKeys::KeyD),
            Key::Named(NamedKey::ArrowUp) => Some(MyKeys::Up),
            Key::Character(name) if name == "z" => Some(MyKeys::KeyZ),
            Key::Named(NamedKey::ArrowDown) => Some(MyKeys::Down),
            Key::Character(name) if name == "s" => Some(MyKeys::KeyS),
            Key::Character(a) if a == "a" => Some(MyKeys::KeyA),
            Key::Character(a) if a == "e" => Some(MyKeys::KeyE),
            _ => None,
        } {
            if event.state == winit::event::ElementState::Pressed {
                self.pressed_keys.insert(my_key);
            } else if event.state == winit::event::ElementState::Released {
                self.pressed_keys.remove(&my_key);
            }
        };
        // if self.pressed_keys.len() != 0 {
        //     self.window.as_ref().unwrap().request_redraw();
        // }
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
        if let Some(pixels) = &mut self.pixels {
            pixels.resize_surface(size.width, size.height).unwrap();
            pixels.resize_buffer(size.width, size.height).unwrap();
        }
        self.window.as_ref().unwrap().request_redraw();
    }
}

pub fn put_pixel1(frame: &mut [u8], width: usize, x: usize, y: usize, color: rgb::RGBA8) {
    use rgb::*;
    let idx = width * y + x;
    frame.as_rgba_mut()[idx] = color;
}

pub fn draw_centered_col(
    frame: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    col_height: usize,
    color: rgb::RGBA8,
) {
    let mid = height / 2;
    let up_bound = mid - (col_height / 2);
    let low_bound = mid + (col_height / 2);
    (0..up_bound).for_each(|y| {
        put_pixel1(
            frame,
            width,
            x,
            y,
            rgb::RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
        )
    });
    (up_bound..low_bound).for_each(|y| put_pixel1(frame, width, x, y, color));
    (low_bound..height).for_each(|y| {
        put_pixel1(
            frame,
            width,
            x,
            y,
            rgb::RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
        )
    });
}
