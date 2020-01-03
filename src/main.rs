#![allow(dead_code)]
#![allow(unused_variables)]

//SDL
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SDLColor;
use std::time::Duration;

//File IO
use env_logger::Env;
use log::info;
use std::env;
use std::fs::File;
use std::io::Read;

mod cpu;
mod gpu;
mod instructions;
mod memory;
mod registers;
mod texture;
use crate::texture::{Map, Tile};
use crate::cpu::CPU;


fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
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
    let mut cpu = CPU::new(rom);
    loop {
        // for i in 0..30 {
        let cpu_cycles = cpu.cycle();
        if cpu.clock > 1000_000 {
            break;
        }
        cpu.memory.gpu.cycle(cpu_cycles);
    }
    vram_viewer(cpu.memory.gpu.vram);
    Ok(())
}

fn vram_viewer(vram: [u8; 0x2000]) -> Result<(), String> {
    // 0x8000-0x87ff
    let mut tiles: Vec<Tile> = vec![];
    for i in (0..0x7ff).step_by(16) {
        let tile_data = &vram[i..(i + 16)];
        tiles.push(Tile::construct(tile_data));
    }
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let scale = 8;
    let mut map = Map::new(16, 10, tiles);
    for i in 0..map.tile_set.len() {
        let (x, y) = (i%16, i / 16);
        println!("{} {} {}", x, y ,i);
        map.set(x, y, i);
    }
    let (w, h) = map.pixel_dims();
    // for i in 0..5 {
    //     for j in 0..5 {
    //         map.set(j, i, i);
    //     }
    // }
    let window = video_subsystem
        .window("VRAM Viewer", (scale * w) as u32, (scale * h) as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;


    let texture_creator = canvas.texture_creator();
    //Texture width = map_w * tile_w
    let (map_w, map_h) = map.dimensions();
    let tile_w = 8;
    let mut texture = texture_creator
        .create_texture_static(
            sdl2::pixels::PixelFormatEnum::RGB24,
            (map_w * tile_w) as u32,
            (map_h * tile_w) as u32,
        )
        .map_err(|e| e.to_string())?;

    println!("{}", map.texture().len());
    // Pitch = n_bytes(3) * map_w * tile_w
    texture
        .update(None, &(map.texture()), map.pitch())
        .map_err(|e| e.to_string())?;
    canvas
        .copy(&texture, None, None)
        .map_err(|e| e.to_string())?;
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

fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.clear();
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

        canvas.clear();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
