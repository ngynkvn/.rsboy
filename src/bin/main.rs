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

use rust_emu::tui::Tui;

use rust_emu::*;

pub const CYCLES_PER_FRAME: usize = GB_CYCLE_SPEED / 60;
const FRAME_TIME: Duration = Duration::from_nanos(16670000);

fn setup_logger() -> Result<(), fern::InitError> {
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
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // just_cpu();
    info!("Setup logging");
    setup_logger()?;
    info!("Running SDL Main");
    sdl_main()
}

fn init_emu() -> Result<Emu, std::io::Error> {
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
fn create_window(context: &sdl2::Sdl) -> Result<Canvas<Window>, Box<dyn Error>> {
    let video = context.video().unwrap();
    video
        .window("Window", WINDOW_WIDTH * 3, WINDOW_HEIGHT * 3)
        .position_centered()
        .build()?
        .into_canvas()
        .build()
        .or_else(|_| Err("Unable to build window".into()))
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

fn sdl_main() -> Result<(), Box<dyn Error>> {
    let mut emu = init_emu().unwrap();

    let context = sdl2::init().unwrap();
    let mut canvas = create_window(&context)?;
    let tc = canvas.texture_creator();
    let mut texture =
        tc.create_texture_streaming(PixelFormatEnum::RGB565, WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let boot_timer = Instant::now();
    let mut timer = Instant::now();
    let mut event_pump = context.event_pump()?;

    let mut tui = Tui::new();
    tui.init()?;

    pump_loop!(event_pump, {
        let f = frame(&mut emu, &mut texture, &mut canvas);
        tui.print_state(&emu)?;
        if f.is_err() {
            break;
        }
        delay_min(FRAME_TIME, &timer);
        timer = Instant::now();
    });
    std::mem::drop(event_pump);

    println!(
        "It took {:?} seconds.",
        Instant::now().duration_since(boot_timer)
    );
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

fn frame(emu: &mut Emu, texture: &mut Texture, canvas: &mut Canvas<Window>) -> Result<(), ()> {
    for _ in 0..(CYCLES_PER_FRAME / 4) {
        emu.emulate_step();
    }
    emu.bus.gpu.render(&mut emu.framebuffer);
    let (h, v) = emu.bus.gpu.scroll();
    texture.copy_window(h, v, &emu.framebuffer);
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();
    Ok(())
}

fn delay_min(min_dur: Duration, timer: &Instant) {
    let time = timer.elapsed();
    if time < min_dur {
        ::std::thread::sleep(min_dur - time);
    }
    // println!("Frame time: {}", timer.elapsed().as_secs_f64());
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
