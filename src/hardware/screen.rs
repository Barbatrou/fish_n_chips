//!
//! Screen handling using sdl2
//!

use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, TextureCreator, Texture};
use sdl2::pixels::Color;
use sdl2::rect::Point;

use super::memory::Display;

use super::DISPLAY_HEIGHT;
use super::DISPLAY_WIDTH;

use super::BG_COLOR;

use super::PIXEL_COLOR;

use super::GRADIENT_SATURATION;
use super::GRADIENT_VALUE;

fn rgb_from_hsv(hue: u32, saturation: f32, value: f32) -> (u8, u8, u8)
{
    let c = value * saturation;
    let x = c * (1.0 - ((hue as f32 / 60.0) % 2.0 - 1.0).abs()) as f32;
    let m = value - c;

    let rgb = |r, g, b| { (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8) };
    let (r, g, b) = match hue {
        0..=59 => rgb(c, x, 0.0),
        60..=119 => rgb(x, c, 0.0),
        120..=179 => rgb(0.0, c, x),
        180..=239 => rgb(0.0, x, c),
        240..=299 => rgb(x, 0.0, c),
        300..=359 => rgb(c, 0.0, x),
        _ => rgb(c, x, 0.0),
    };
    (r, g, b)
}

pub struct Screen<'r>
{
    texture: Texture<'r>,
    hue: u32,
    use_gradient: bool,
}

impl<'r> Screen<'r>
{
    pub fn new(texture_creator: &'r TextureCreator<WindowContext>, use_gradient: bool) -> Screen<'r>
    {
        Screen {
            texture: texture_creator
                .create_texture_target(texture_creator.default_pixel_format(), DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
                .unwrap(),
            hue: 0,
            use_gradient: use_gradient,
        }
    }

    pub fn draw(&mut self, display_memory: &Display, canvas: &mut Canvas<Window>)
    {
        let (r, g, b) = match self.use_gradient {
            false => PIXEL_COLOR,
            true => {
                self.hue = (self.hue + 1) % 360;
                rgb_from_hsv(self.hue, GRADIENT_SATURATION, GRADIENT_VALUE)
            }
        };
        canvas.with_texture_canvas(&mut self.texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGB(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2));
            texture_canvas.clear();
            texture_canvas.set_draw_color(Color::RGB(r, g, b));
            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    if display_memory[[x, y]] == 1 {
                        texture_canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                    }
                }
            }
        }).unwrap();
        canvas.copy(&self.texture, None, None).unwrap();
    }
}
