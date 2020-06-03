//!
//! Memory emulator
//!

use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::{Error, ErrorKind};

use super::RAM_SIZE;

use super::DISPLAY_HEIGHT;
use super::DISPLAY_WIDTH;

const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Display
{
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Index<[usize; 2]> for Display
{
    type Output = u8;

    fn index(&self, index: [usize; 2]) -> &Self::Output
    {
        &self.display[index[1] * DISPLAY_WIDTH + index[0]]
    }
}

impl IndexMut<[usize; 2]> for Display
{
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output
    {
        &mut self.display[index[1] * DISPLAY_WIDTH + index[0]]
    }
}

impl Display
{
    pub fn get_sizes(&self) -> (usize, usize)
    {
        (DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }

    pub fn new() -> Display
    {
        Display { display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT] }
    }

    pub fn clear(&mut self)
    {
        self.display = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    }
}

pub struct Memory
{
    pub memory: [u8; RAM_SIZE],
    pub display: Display,
}

impl Deref for Memory
{
    type Target = [u8; RAM_SIZE];

    fn deref(&self) -> &Self::Target
    {
        &self.memory
    }
}

impl DerefMut for Memory
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.memory
    }
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut memory = Memory {
            memory: [0; RAM_SIZE],
            display: Display::new(),
        };
        for (i, &byte) in SPRITES.iter().enumerate() {
            memory[i] = byte;
        }
        memory
    }

    pub fn load(&mut self, filename: &str) -> Result<(), io::Error>
    {
        let mut f = File::open(filename)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let len_memory = self.memory.len();
        let len_buffer = buffer.len();
        if len_buffer > len_memory - 0x200 {
            return Err(Error::new(ErrorKind::Other, format!("ROM size is too big: < {}", len_memory - 0x200)));
        }
        self.memory[0x200..len_buffer + 0x200].copy_from_slice(&buffer);
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn display_insert()
    {
        let mut display = Display::new();

        display[[0,0]] = 1;
        assert_eq!(display.display[0], 1);
        display[[4, 0]] = 1;
        assert_eq!(display.display[4], 1);
        display[[0, 4]] = 1;
        assert_eq!(display.display[4 * DISPLAY_WIDTH], 1);
        display[[4, 4]] = 1;
        assert_eq!(display.display[4 * DISPLAY_WIDTH + 4], 1);
    }

    #[test]
    fn display_index()
    {
        let mut display = Display::new();

        display.display[0] = 1;
        assert_eq!(display[[0,0]], 1);
        display.display[4] = 1;
        assert_eq!(display[[4,0]], 1);
        display.display[4 * DISPLAY_WIDTH] = 1;
        assert_eq!(display[[0,4]], 1);
        display.display[4 * DISPLAY_WIDTH + 4] = 1;
        assert_eq!(display[[4,4]], 1);
    }
}

