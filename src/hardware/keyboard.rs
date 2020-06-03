//!
//! Keyboard emulator
//!

use std::ops::{Deref, DerefMut};

use sdl2::EventPump;
use sdl2::keyboard::Keycode;

pub struct Keyboard
{
    keyboard: [u8; 16],
}

impl Deref for Keyboard
{
    type Target = [u8; 16];

    fn deref(&self) -> &Self::Target
    {
        &self.keyboard
    }
}

impl DerefMut for Keyboard
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.keyboard
    }
}

impl Keyboard
{
    pub fn new() -> Keyboard
    {
        Keyboard {
            keyboard: [0; 16],
        }
    }

    pub fn read(&mut self, event_pump: &EventPump)
    {
        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        self.keyboard = [0; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xC),
                Keycode::A => Some(0x4),
                Keycode::Z => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xD),
                Keycode::Q => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xE),
                Keycode::W => Some(0xA),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xB),
                Keycode::V => Some(0xF),
                _ => None,
            };
            if let Some(i) = index {
                self.keyboard[i] = 1;
            }

        }
    }
}
