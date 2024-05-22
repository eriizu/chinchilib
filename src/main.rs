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
    // event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

// #[derive(Default)]
struct App {
    window: Option<Window>,
    pixels: Option<Pixels>,
    pause: bool,
    timings: circular_buffer::CircularBuffer<240, std::time::Instant>,
    last_fps_report: std::time::Instant,
    raycast: raycast::World,
    height: usize,
    width: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            pixels: None,
            pause: false,
            timings: circular_buffer::CircularBuffer::default(),
            last_fps_report: std::time::Instant::now(),
            raycast: raycast::World::default(),
            height: 240,
            width: 320,
        }
    }
}

// INFO: source https://chatgpt.com/share/5cebcdd6-fe9d-4c5d-bf68-bc62a0b8c7df
fn timings_avg<'a, T>(iter: T) -> Option<u128>
where
    T: Iterator<Item = &'a std::time::Instant>,
{
    let mut iter = iter.peekable();
    let mut prev = match iter.next() {
        Some(val) => val,
        None => return None, // If the iterator is empty, return None
    };

    let mut count = 0;
    let mut total_diff = 0;

    while let Some(next) = iter.peek() {
        total_diff += next.duration_since(*prev).as_millis();
        count += 1;
        prev = next;
        iter.next(); // Consume the element
    }

    if count == 0 {
        None // If there was only one element, return None
    } else {
        Some(total_diff / count)
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut attr = Window::default_attributes();
        let size = winit::dpi::PhysicalSize::new(self.width as u16, self.height as u16);
        attr = attr.with_inner_size(size).with_title("Raycaster");
        let win = event_loop.create_window(attr).unwrap();
        self.pixels = Some({
            let surface_texture = SurfaceTexture::new(self.width as u32, self.height as u32, &win);
            Pixels::new(self.width as u32, self.height as u32, surface_texture).unwrap()
        });
        self.window = Some(win);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event;
        use winit::event::WindowEvent;
        match event {
            WindowEvent::CloseRequested => {
                log::info!("The close button was pressed; stopping");
                log::info!("{}", timings_avg(self.timings.iter()).unwrap());
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.width = size.width as usize;
                self.height = size.height as usize;
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                    pixels.resize_buffer(size.width, size.height).unwrap();
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.state == event::ElementState::Pressed => {
                use raycast::Heading::*;
                use winit::keyboard::{Key, NamedKey};
                match event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => self.raycast.move_player(Left),
                    Key::Character(name) if name == "q" => self.raycast.move_player(Left),
                    Key::Named(NamedKey::ArrowRight) => self.raycast.move_player(Right),
                    Key::Character(name) if name == "d" => self.raycast.move_player(Right),
                    Key::Named(NamedKey::ArrowUp) => self.raycast.move_player(Forward),
                    Key::Character(name) if name == "z" => self.raycast.move_player(Forward),
                    Key::Named(NamedKey::ArrowDown) => self.raycast.move_player(Backward),
                    Key::Character(name) if name == "s" => self.raycast.move_player(Backward),
                    Key::Character(a) if a == "a" => self.raycast.pan_left(),
                    Key::Character(a) if a == "e" => self.raycast.pan_right(),
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::Space) => {
                        self.pause = !self.pause;
                        self.timings.clear();
                        self.last_fps_report = std::time::Instant::now();
                    }
                    _ => log::debug!("unused kb event {:?}", event),
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();

                self.timings.push_back(now);
                if now.duration_since(self.last_fps_report).as_secs() >= 1 {
                    if let Some(avg) = timings_avg(self.timings.iter()) {
                        // log::info!("{} fps", 1000 / avg);
                    }
                    self.last_fps_report = now;
                }

                if let Some(pixels) = &mut self.pixels {
                    pixels.clear_color(pixels::wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 255.0,
                        a: 255.0,
                    });
                    put_pixel1(
                        pixels.frame_mut(),
                        self.width,
                        10,
                        10,
                        rgb::RGBA {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                    );
                    put_pixel1(
                        pixels.frame_mut(),
                        self.width,
                        self.width - 10,
                        self.height - 10,
                        rgb::RGBA {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                    );
                    self.raycast
                        .distance_to_walls(self.width)
                        .map(|distance| (self.height as f32 / distance) as usize)
                        .enumerate()
                        .for_each(|(idx, mut col_height)| {
                            // log::debug!("{}, {}", idx, col_height);
                            if col_height > self.height {
                                col_height = self.height;
                            }
                            draw_centered_col(
                                pixels.frame_mut(),
                                self.width,
                                self.height,
                                idx,
                                col_height,
                                rgb::RGBA {
                                    r: 255,
                                    g: 0,
                                    b: 0,
                                    a: 255,
                                },
                            );
                        });
                    if let Err(err) = pixels.render() {
                        log::error!("failed to render with error {}", err);
                        return;
                    }
                }
                if !self.pause {
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => {}
        }
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
