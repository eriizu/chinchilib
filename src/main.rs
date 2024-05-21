use std::usize;

use pixels::{Error, Pixels, SurfaceTexture};
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
    app.world = World::new();
    event_loop.run_app(&mut app).unwrap();
}

// #[derive(Default)]
struct App {
    window: Option<Window>,
    world: World,
    pixels: Option<Pixels>,
    pause: bool,
    timings: circular_buffer::CircularBuffer<240, std::time::Instant>,
    last_fps_report: std::time::Instant,
    raycast: raycast::World,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            world: World::default(),
            pixels: None,
            pause: false,
            timings: circular_buffer::CircularBuffer::default(),
            last_fps_report: std::time::Instant::now(),
            raycast: raycast::World::default(),
        }
    }
}

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

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
        let size = winit::dpi::LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        attr = attr.with_inner_size(size).with_title("Patate");
        let win = event_loop.create_window(attr).unwrap();
        self.pixels = Some({
            let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &win);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
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
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.state == event::ElementState::Pressed => {
                use winit::keyboard::{Key, NamedKey};
                match event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => self.world.box_x -= 10,
                    Key::Named(NamedKey::ArrowRight) => self.world.box_x += 10,
                    Key::Named(NamedKey::ArrowUp) => self.world.box_y -= 10,
                    Key::Named(NamedKey::ArrowDown) => self.world.box_y += 10,
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::Space) => {
                        self.pause = !self.pause;
                        self.timings.clear();
                        self.last_fps_report = std::time::Instant::now();
                    }
                    _ => log::debug!("kb event {:?}", event),
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();

                self.timings.push_back(now);
                if now.duration_since(self.last_fps_report).as_secs() >= 1 {
                    if let Some(avg) = timings_avg(self.timings.iter()) {
                        log::info!("{} fps", 1000 / avg);
                    }
                    self.last_fps_report = now;
                }

                // println!("redraw");
                if let Some(pixels) = &mut self.pixels {
                    self.world.update();
                    self.world.draw(pixels.frame_mut());
                    put_pixel1(
                        pixels.frame_mut(),
                        WIDTH as usize,
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
                        WIDTH as usize,
                        WIDTH as usize - 10,
                        HEIGHT as usize - 10,
                        rgb::RGBA {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                    );
                    self.raycast
                        .distance_to_walls(WIDTH.try_into().unwrap())
                        .map(|distance| {
                            dbg!(distance);
                            (HEIGHT as f32 / distance) as usize
                        })
                        .enumerate()
                        .for_each(|(idx, mut col_height)| {
                            log::debug!("{}, {}", idx, col_height);
                            if col_height > HEIGHT as usize {
                                col_height = HEIGHT as usize;
                            }
                            draw_centered_col(
                                pixels.frame_mut(),
                                WIDTH as usize,
                                HEIGHT as usize,
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
                        // log_error("pixels.render", err);
                        // *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                if !self.pause {
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
    // box_pos: (i16, i16),
    // velocity: (i16, i16),
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn put_pixel(frame: &mut [u8], width: u32, x: u32, y: u32, color: rgb::RGBA8) {
    use rgb::*;
    let idx = (width * y + x) * 4;
    let idx: usize = idx.try_into().unwrap();
    if idx <= frame.len() - 4 {
        let pixel = &mut frame[idx..idx + 4];
        pixel.copy_from_slice(color.as_slice());
    } else {
        log::warn!("impossible value {}", idx);
    }
}

pub fn put_pixel1(frame: &mut [u8], width: usize, x: usize, y: usize, color: rgb::RGBA8) {
    use rgb::*;
    let idx = (width * y + x);
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
    dbg!(mid);
    dbg!(col_height);
    let up_bound = mid - (col_height / 2);
    let low_bound = mid + (col_height / 2);
    (up_bound..low_bound).for_each(|y| put_pixel1(frame, width, x, y, color));
}
