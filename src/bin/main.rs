//SDL
use std::error::Error;

use cpu::GB_CYCLE_SPEED;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::time::Duration;
use std::time::Instant;

//File IO
use log::info;
use std::env;
use std::fs::File;
use std::io::Read;

use gpu::{PixelData, PixelMap};
use rust_emu::emu::Emu;

use rust_emu::*;

pub const CYCLES_PER_FRAME: usize = GB_CYCLE_SPEED / 60;
const FRAME_TIME: Duration = Duration::from_nanos(16670000);

type R<T> = Result<T, Box<dyn Error>>;

fn setup_logger() -> R<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}:{}] {}",
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                message
            ))
        })
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        // Apply globally
        .apply()
        .map_err(|x| x.into())
}

fn main() -> R<()> {
    // just_cpu();
    info!("Setup logging");
    setup_logger()?;
    info!("Running SDL Main");
    sdl_main()
}

fn init_emu() -> R<Emu> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: ./gboy [rom]");
        panic!();
    }
    info!("{:?}", args);
    info!("Attempting to load {:?}", args[1]);
    let mut file = File::open(args[1].to_string())?;
    let mut rom = Vec::new();
    file.read_to_end(&mut rom)?;
    let emu = Emu::new(rom);
    Ok(emu)
}

macro_rules! pump_loop {
    ($e: expr, $body:block) => {
        'running: loop {
            for event in $e.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            $body;
        }
    };
}

pub struct Panel<'a> {
    canvas: Canvas<Window>,
    texture: Texture<'a>,
}

impl Panel<'_> {
    fn frame<F: FnOnce(&mut Panel)>(&mut self, f: F) {
        f(self);
    }
}

fn sdl_main() -> R<()> {
    let mut emu = init_emu()?;

    let context = sdl2::init()?;
    let mut rsboy = context.video()?
        .window(".rsboy", WINDOW_WIDTH * 3, WINDOW_HEIGHT * 3)
        .position_centered()
        .build()?
        .into_canvas()
        .build()?;
    let tc = rsboy.texture_creator();
    let mut texture =
        tc.create_texture_streaming(PixelFormatEnum::RGB565, WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let mut machine = Panel { canvas: rsboy, texture };

    let mut debugger = context.video()?
        .window("debugger", 100, 100)
        .opengl()
        .build()?

    debugger.


    let mut timer = Instant::now();
    let mut event_pump = context.event_pump()?;

    // let mut tui = Tui::new();
    // tui.init()?;

    use crossterm::cursor::MoveTo;
    use crossterm::{
        cursor::*,
        event, execute,
        style::{Color::*, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        terminal::ClearType::All,
        terminal::*,
        ExecutableCommand,
    };
    use std::io::stdout;

    type EmuState = (u8, usize, usize, Duration);
    let (tx, rx) = std::sync::mpsc::channel::<EmuState>();

    let calc_relative_error = |x_hat, x| (x_hat - x) * 100.0 / x;

    std::thread::spawn(move || -> Result<(), crossterm::ErrorKind> {
        let mut std = stdout();
        loop {
            if let Ok((tac, ticks, c, duration)) = rx.recv() {
                let duration_relative_error =
                    calc_relative_error(duration.as_secs_f64(), FRAME_TIME.as_secs_f64());
                let cpu_hz = c as f64 / duration.as_secs_f64();
                let cpu_relative_error = calc_relative_error(cpu_hz, cpu::GB_CYCLE_SPEED as f64);
                let timer_hz = ticks as f64 / duration.as_secs_f64();
                std.execute(MoveTo(0, 0))?
                    .execute(Clear(All))?
                    .execute(Print("Frame time:"))?
                    .execute(Print(format!("{:?}: ", duration)))?
                    //Relative error
                    .execute(Print(format!("{}\n", duration_relative_error)))?
                    .execute(Print("CPU HZ: "))?
                    .execute(Print(format!("{} hz: ", cpu_hz)))?
                    .execute(Print(format!("{}\n", cpu_relative_error)))?
                    .execute(Print(format!("Timer: {}\n", tac & 0b11)))?
                    .execute(Print(format!("{} hz: ", timer_hz)))?;
            }
        }
        // some work here
    });

    pump_loop!(event_pump, {
        let b = emu.bus.timer.tick;
        let c = emu.bus.clock;
        machine.frame(|panel| {
            let before = emu.bus.clock;
            while emu.bus.clock < before + CYCLES_PER_FRAME {
                emu.emulate_step();
            }
            emu.bus.gpu.render(&mut emu.framebuffer);
            let (h, v) = emu.bus.gpu.scroll();
            panel.texture.copy_window(h, v, &emu.framebuffer);
            panel.canvas.copy(&panel.texture, None, None).unwrap();
            panel.canvas.present();
        });
        // tui.print_state(&emu)?;
        delay_min(FRAME_TIME, &timer);
        let now = Instant::now();
        tx.send((
            emu.bus.timer.tac,
            emu.bus.timer.tick - b,
            emu.bus.clock - c,
            now.duration_since(timer),
        ))?;
        println!(
            "{:?} [{}] {}",
            now.duration_since(timer),
            emu.bus.timer.tac & 0b11,
            (emu.bus.timer.tick - b) as f64 / now.duration_since(timer).as_secs_f64()
        );
        timer = now;
    });
    std::mem::drop(event_pump);

    // vram_viewer(&context, emu.bus.gpu.vram).unwrap();
    map_viewer(&context, emu)?;
    Ok(())
}

const WINDOW_HEIGHT: u32 = 144;
const WINDOW_WIDTH: u32 = 160;
const MAP_WIDTH: u32 = 256;

trait GBWindow {
    fn copy_window(&mut self, h: u32, v: u32, buffer: &PixelData);
    fn copy_map(&mut self, buffer: &PixelData);
}
impl GBWindow for Texture<'_> {
    fn copy_window(&mut self, horz: u32, vert: u32, framebuffer: &PixelData) {
        self.with_lock(None, |buffer, _| {
            let mut i = 0;
            for y in vert..vert + WINDOW_HEIGHT {
                let y = (y % MAP_WIDTH) as usize;
                for x in horz..horz + WINDOW_WIDTH {
                    let x = (x % MAP_WIDTH) as usize;
                    let [lo, hi] = framebuffer[y][x].to_le_bytes();
                    buffer[i] = lo;
                    buffer[i + 1] = hi;
                    i += 2;
                }
            }
        })
        .unwrap();
    }
    fn copy_map(&mut self, buffer: &PixelData) {
        let (_, buffer, _) = unsafe { buffer.align_to::<u8>() };
        self.update(None, buffer, 256 * 2).unwrap()
    }
}

fn delay_min(min_dur: Duration, timer: &Instant) {
    let time = timer.elapsed();
    if time < min_dur {
        spin_sleep::sleep(min_dur - time);
    }
}

fn map_viewer(sdl_context: &sdl2::Sdl, emu: emu::Emu) -> Result<(), String> {
    let gpu = emu.bus.gpu;
    let video_subsystem = sdl_context.video()?;
    let (w, h) = (32 * 8, 32 * 8);
    let window = video_subsystem
        .window("Map Viewer", w as u32, h as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let (map_w, map_h) = (32, 32);
    let tile_w = 8;
    let mut texture = texture_creator
        .create_texture_static(
            PixelFormatEnum::RGB565,
            (map_w * tile_w) as u32,
            (map_h * tile_w) as u32,
        )
        .map_err(|e| e.to_string())?;

    // Pitch = n_bytes(3) * map_w * tile_w
    let buffer = unsafe { std::mem::transmute::<PixelData, PixelMap>(*emu.framebuffer) };
    texture
        .update(None, &buffer, 256 * 2)
        .map_err(|e| e.to_string())?;
    canvas.copy(&texture, None, None)?;
    let (h, v) = gpu.scroll();
    println!("{} {}", h, v);
    canvas
        .draw_rect(Rect::from((
            h as i32,
            v as i32,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )))
        .unwrap();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
