//SDL
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

//File IO
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn read_instruction(buffer: &Vec<u8>, ptr: usize) {
    match buffer[ptr] {
        0xC3 => println!("Hey this is the first byte in my tetris rom!"),
        _ => panic!("Unknown Instruction: {:02X}", buffer[ptr])
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    println!("Attempting to load {:?}", args[1]);
    let mut file = File::open(args[1].to_string())?;
    let mut buffer = Vec::new();
    let ptr = 0;
    file.read_to_end(&mut buffer)?;
    read_instruction(&buffer, ptr);
    Ok(())
}

fn run() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 255, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
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