const VERSION: &str = "0.1.0";

const WINDOW_TITLE: &str = "fish n chips";

mod hardware;

use std::{thread, time};
use clap::{Arg, App};
use sdl2::{Sdl, EventPump, AudioSubsystem};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas};

use hardware::{
    Cpu,
    Memory,
    Display,
    Screen,
    Keyboard,
    Beeper,
};

fn init_sdl_window() -> (Sdl, WindowCanvas, AudioSubsystem)
{
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window(WINDOW_TITLE, 64 * 20, 32 * 20)
        .position_centered().resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let audio_subsystem = sdl_context.audio().unwrap();

    (sdl_context, canvas, audio_subsystem)
}

fn draw_window(canvas: &mut WindowCanvas, screen: &mut Screen, memory_display: &Display)
{
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    screen.draw(memory_display, canvas);
    canvas.present();
}

fn check_terminate_events(event_pump: &mut EventPump) -> Result<(), ()>
{
    let mut result = Ok(());
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), ..  } => {
                result = Err(());
                break;
            },
            _ => {}
        };
    }
    result
}

fn run() -> Result<(), i32>
{
    let arg = App::new(WINDOW_TITLE)
        .version(VERSION)
        .author("Arthur Cros <arthur.cros@etna.io>")
        .about("Simple Chip8 emulator")
        .arg(Arg::with_name("clock_rate")
            .short("c")
            .long("clock-rate")
            .default_value("1000")
            .help("Clock rate of the cpu in Hz"))
        .arg(Arg::with_name("framerate")
            .short("f")
            .long("framerate")
            .default_value("60")
            .help("framerate in frame per second"))
        .arg(Arg::with_name("frequency")
            .short("v")
            .long("frequence")
            .default_value("553.0")
            .help("Choose frequency for the beep"))
        .arg(Arg::with_name("gradient")
            .short("g")
            .long("gradient-colors")
            .help("Enable gradient coloring of pixels"))
        .arg(Arg::with_name("rom_filepath")
            .required(true)
            .help("Filepath to ROM"))
        .get_matches();

    let clock_rate = match arg.value_of("clock_rate").unwrap().parse::<f32>() {
        Ok(clock_rate) => (1.0 / clock_rate * 1000.0) as u32,
        Err(e) => {
            eprintln!("Clock rate must be a number: {}", e);
            return Err(1);
        },
    };

    let framerate = match arg.value_of("framerate").unwrap().parse::<f32>() {
        Ok(framerate) => (1000.0 / framerate) as u32,
        Err(e) => {
            eprintln!("Clock rate must be a number: {}", e);
            return Err(1);
        },
    };

    let frequency = match arg.value_of("frequency").unwrap().parse::<f32>() {
        Ok(freq) => freq,
        Err(e) => {
            eprintln!("Frequency must be a number: {}", e);
            return Err(1);
        },
    };

    let (sdl_context, mut canvas, audio_subsystem) = init_sdl_window();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut memory = Memory::new();
    let mut screen = Screen::new(&texture_creator, arg.is_present("gradient"));
    let mut keyboard = Keyboard::new();
    let beeper = Beeper::new(&audio_subsystem, frequency);
    let mut cpu = Cpu::new();
    if let Err(io_err) = memory.load(arg.value_of("rom_filepath").unwrap()) {
        eprintln!("Cannot load ROM file {}: {}", arg.value_of("rom_filepath").unwrap(), io_err);
        return Err(1);
    }

    let mut last_tick = time::Instant::now();
    #[allow(unused_assignments)]
    let mut delta = 0;
    let mut delta_render = 0;
    let mut delta_timer = 0;
    let mut delta_cycle = 0;

    'running: loop {
        let tick = time::Instant::now();
        delta = tick.duration_since(last_tick).as_millis();
        last_tick = tick;
        delta_render += delta;
        delta_timer += delta;
        delta_cycle += delta;
        if let Err(()) = check_terminate_events(&mut event_pump) {
            break 'running;
        }
        keyboard.read(&event_pump);
        if delta_cycle > clock_rate as u128 {
            cpu.do_cycle(&mut memory, &keyboard);
            delta_cycle = 0;
        }
        if delta_timer > (1.0 / 60.0 * 1000.0) as u128 {
            if let Ok(_) = cpu.update_timers() {
                delta_timer = 0;
            }
        }
        if cpu.beeping {
            beeper.beep();
        } else {
            beeper.pause_beep();
        }
        if delta_render > framerate as u128 {
            draw_window(&mut canvas, &mut screen, &memory.display);
            delta_render = 0;
        }
        thread::sleep(time::Duration::from_millis(1));
    }
    Ok(())
}

fn main()
{
    std::process::exit(match run() {
        Ok(_) => 1,
        Err(errcode) => errcode,
    });
}

