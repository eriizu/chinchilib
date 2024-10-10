# Chinchilib

A rust *hommage* to the [Bunny Library](https://github.com/Damdoshi/LibLapin) designed for teaching the fundamentals of graphical programming, such as how to place a pixel and draw lines when given an array of memory that will rendered to screen.

It's mostly a wrapper arround pixels and winit that takes care of refresh rate, and keeps track of a set of pressed keys.

# Usage

`WinitHandler` creates a window for you and manages events and timing. You can package your code into a `chinchilib::GfxApp` implementing struct such as `MovingPixel` in the example bellow.

`WinitHandler` will make calls to:
- `on_tick` when its time for updating your state with the keys that are pressed right now (or have been pressed in between ticks), return `true` when you want `draw` to be called afterwards;
- `draw` when its time to modify the framebuffer so that the image on screen changes;
- `done` when it wants to know if your app has anything left to do:
  - `NotDone` is self-explanatory,
  - `Remain` means that your app is done, but you want the result to stay on screen until the user exits,
  - `Exit` means that your app is done and the window should close.

`WinitHandler` closes the window when the user presses the close button or the escape key.

```rust
use chinchilib::pixels::Pixels;
use chinchilib::rgb;
use chinchilib::{put_pixel1, GfxApp, MyKeys, WinitHandler};

fn main() {
    let moving_pixel = Box::new(MovingPixel::new(50, 100));
    let mut app = WinitHandler::new(moving_pixel, (500, 500), 60);
    app.run().unwrap();
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
}

const RED: rgb::RGBA8 = rgb::RGBA8 {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};

impl GfxApp for MovingPixel {
    fn on_tick(&mut self, pressed_keys: &std::collections::HashSet<MyKeys>) -> bool {
        let mut needs_redraw = true;
        for key in pressed_keys {
            match key {
                MyKeys::Left => {
                    self.pos.0 -= 1;
                }
                MyKeys::Right => {
                    self.pos.0 += 1;
                }
                MyKeys::Up => {
                    self.pos.1 -= 1;
                }
                MyKeys::Down => {
                    self.pos.1 += 1;
                }
                _ => {
                    needs_redraw = false;
                }
            }
        }
        needs_redraw
    }

    fn draw(&self, pixels: &mut Pixels, width: usize) {
        if self.pos.0 * self.pos.1 < pixels.frame().len() {
            put_pixel1(pixels.frame_mut(), width, self.pos.0, self.pos.1, RED);
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
```
