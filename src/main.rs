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

#[derive(Copy, Clone, Debug)]
enum Color {
    White,
    LightGrey,
    DarkGrey,
    Black,
}

impl Color {
    fn value(&self) -> [u8; 3] {
        match *self {
            Color::White => [255, 255, 255],
            Color::LightGrey => [192, 192, 192],
            Color::DarkGrey => [96, 96, 96],
            Color::Black => [0, 0, 0],
        }
    }
    fn bit2color(value: u8) -> Self {
        match value {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => unreachable!("Are you sure you're reading bit data?"),
        }
    }
}

struct Tile {
    data: [Color; 64], //8 x 8
    raw_data: [u8; 16],
}

impl Tile {
    fn construct(tile_data: &[u8]) -> Self {
        let mut raw_data = [0; 16];
        let mut data = [Color::White; 64];
        for row in 0..8 {
            for col in 0..8 {
                let hi = tile_data[(row * 2) + 1] >> (7 - col) & 1;
                let lo = tile_data[(row * 2)] >> (7 - col) & 1;
                data[row * 8 + col] = Color::bit2color((hi << 1) | lo);
            }
        }
        raw_data[..].clone_from_slice(tile_data);

        Self { data, raw_data }
    }

    fn coord(i: usize) -> (usize, usize) {
        ((i / 8) as usize, (i % 8) as usize)
    }

    fn texture(&self) -> [u8; 192] {
        //64 * 3
        let mut buffer = [255; 192];
        let mut p = 0;
        for i in self.data.iter() {
            buffer[p..(p + 3)].clone_from_slice(&i.value());
            p += 3;
        }
        buffer
    }
}

struct Map {
    width: usize,
    height: usize,
    tile_set: Vec<Tile>,
    map: Vec<usize>,
}

impl Map {
    fn new(width: usize, height: usize, tile_set: Vec<Tile>) -> Self {
        Self {
            width,
            height,
            tile_set,
            map: vec![0; width * height],
        }
    }
    fn set(&mut self, x: usize, y: usize, i: usize) {
        self.map[y * self.width + x] = i;
    }

    /**
     * Mapping is like this in memory right now:
     *  for a 4x4 tile size
     * [1, 1, 1, 1] [1, 1, 1, 1]
     * [2, 2, 2, 2] [2, 2, 2, 2]
     * [3, 3, 3, 3] [3, 3, 3, 3]
     * [4, 4, 4, 4] [4, 4, 4, 4]
     * 
     * Fine and dandy, but we need the 2d repre to be:
     *        (ROW 1)      (ROW 2)
     * [1, 1, 1, 1, 1, 1, 1, 1,   2, 2, 2, 2, 2, 2, 2, 2, ...]
     */
    fn texture(&self) -> Vec<u8> {
        let mut texture = vec![];
        for &i in self.map.iter() {
            let mut tile = self.tile_set[i].texture();
            texture.extend_from_slice(&tile);
        }
        texture
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
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

    let window = video_subsystem
        .window("VRAM Viewer", 512, 512)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut map = Map::new(5, 5, tiles);
    for i in 0..5 {
        for j in 0..5 {
            map.set(j, i, i);
        }
    }

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
    let pitch = (3 * map_w * 8);
    texture
        .update(None, &(map.texture()), pitch)
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
