# fish_n_chips
Simple Chip8 emulator written in Rust

## Usage

```
USAGE:
    fish_n_chip [FLAGS] [OPTIONS] <rom_filepath>

FLAGS:
    -g, --gradient-colors    Enable gradient coloring of pixels
    -h, --help               Prints help information
    -V, --version            Prints version information

OPTIONS:
    -c, --clock-rate <clock_rate>    Clock rate of the cpu in Hz [default: 1000]
    -f, --framerate <framerate>      framerate in frame per second [default: 60]
    -v, --frequence <frequency>      Choose frequency for the beep [default: 553.0]

ARGS:
    <rom_filepath>    Filepath to ROM

```

## A Word

This Chip8 is a simple project I started to learn Rust.
There is probably a better way to design it but I quite enjoy doing this project.

I've been able to achieve this using mostly [cowgod's technical documentation](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) 
as well as a few inspiration from projects in C and Rust all over github.
