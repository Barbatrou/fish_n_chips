const RAM_SIZE: usize = 4096;

const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;
const BG_COLOR: (u8, u8, u8) = (74, 74, 74);

// if GRADIENT_DISPLAY is off
const PIXEL_COLOR: (u8, u8, u8) = (255, 205, 230);

// if GRADIENT_DISPLAY is on
const GRADIENT_SATURATION: f32 = 0.2;
const GRADIENT_VALUE: f32 = 1.0;

mod cpu;
mod memory;
mod screen;
mod keyboard;
mod audio;

pub use cpu::Cpu;
pub use memory::{Memory, Display};
pub use screen::Screen;
pub use keyboard::Keyboard;
pub use audio::Beeper;

