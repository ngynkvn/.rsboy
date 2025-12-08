use crate::{
    constants::{CYCLES_PER_FRAME, FRAME_TIME, WINDOW_HEIGHT, WINDOW_WIDTH},
    debugger::Imgui,
};
use clap::Parser;
use color_eyre::{Result, eyre::eyre};
use gpu::PixelData;
use rust_emu::{
    constants::{self, setup_logger},
    cpu::interrupts,
    debugger,
    emu::{Emu, gen_il},
    gpu,
    prelude::*,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::Texture, video::Window};
use std::{path::PathBuf, time::Instant};
use tap::Tap;

#[derive(Parser)]
#[command(name = ".rsboy", about = "Rust emulator")]
struct Settings {
    #[arg()]
    input: PathBuf,
    #[arg(short, long)]
    _logfile: Option<PathBuf>,
    #[arg(short, long)]
    bootrom: Option<PathBuf>,
    #[arg(short, long)]
    _repl: bool,
    #[allow(clippy::option_option)]
    #[arg(long)]
    headless: Option<Option<usize>>,
}

fn main() -> Result<()> {
    println!("Starting program");
    // When the program starts up, parse command line arguments and setup additional systems.
    let settings = Settings::parse();
    info!("Setup logging");
    setup_logger()?;
    info!("Running SDL Main");

    let mut emu = Emu::from_path(settings.input, settings.bootrom).map_err(|e| eyre!(e))?;
    if let Some(cycles) = settings.headless {
        let cycles = cycles.unwrap_or(100_000_000);
        emu.run_until(cycles);
        error!("Emulated {cycles} cycles");
        error!("CPU: {}", emu.cpu);
        error!("Bus: {}", emu.bus);
        error!("{}", emu.bus.timer);
        return Ok(());
    }

    let context = sdl2::init().map_err(|e| eyre!(e))?;
    let video = context.video().map_err(|e| eyre!(e))?;
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);
    gl_attr.set_context_flags().forward_compatible().set();

    let mut rsboy = video
        .window(".rsboy", WINDOW_WIDTH * 3, WINDOW_HEIGHT * 3)
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()?
        .into_canvas()
        .build()?;

    let debugger = video
        .window("debugger", 1000, 800)
        .position(0, 20)
        .opengl()
        .allow_highdpi()
        .resizable()
        .build()?;

    // Wrapper struct for imgui to handle frame-by-frame rendering.
    let mut debugger = Imgui::new(debugger).map_err(|e| eyre!(e))?;
    sdl_main(&mut rsboy, &mut debugger, &context, &mut emu).map_err(|e| eyre!(e))?;
    // TODO: Re-implement debug viewers for scanline renderer
    // map_viewer(&context, &emu).map_err(|e| eyre!(e))?;
    // vram_viewer(&context, &emu).map_err(|e| eyre!(e))?;
    Ok(())
}

fn sdl_main(video: &mut sdl2::render::Canvas<Window>, debugger: &mut Imgui, context: &sdl2::Sdl, emu: &mut Emu) -> Result<()> {
    // Setup gl attributes, then create the texture that we will copy our framebuffer to.
    let tc = video.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGBA32, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    // Some UI state
    let mut cycle_jump = 0;
    let mut running = true;

    let mut event_pump = context.event_pump().map_err(|e| eyre!(e))?;

    let il = gen_il(&emu.bus.memory);
    debugger.info.il = il;

    loop {
        let now = Instant::now();
        for event in event_pump.poll_iter() {
            emu.bus.directions |= 0x0F;
            emu.bus.keypresses |= 0x0F;
            if let Some(value) = parse_event(debugger, emu, &event) {
                return value.tap(|v| info!("{v:?}"));
            }
            debugger.platform.handle_event(&mut debugger.imgui, &event);
        }

        let dt = if running {
            emu.run_until(emu.bus.mclock() + CYCLES_PER_FRAME) - emu.bus.mclock()
        } else {
            0
        };

        // Copy from GPU framebuffer (scanline renderer populates this during emulation)
        texture.copy_framebuffer(&emu.bus.gpu.framebuffer);
        video.copy(&texture, None, None).unwrap();
        video.present();
        let before_sleep = now.elapsed();
        // Delay a minimum of 16.67 milliseconds (60 fps).
        if let Some(time) = FRAME_TIME.checked_sub(now.elapsed()) {
            spin_sleep::sleep(time);
        }
        let after_delay = now.elapsed();

        // ImGui display frame.
        debugger.frame(&mut event_pump, |info, ui| {
            // Log frame time
            info.add_frame_time(after_delay.as_secs_f32());
            info.add_before_sleep_time(before_sleep.as_secs_f32());
            // info.add_memory_usage((
            //     PEAK_ALLOC.current_usage_as_kb(),
            //     PEAK_ALLOC.peak_usage_as_kb(),
            // ));
            draw_debugger(info, ui, dt, &mut running, &mut cycle_jump, emu);
        });
    }
}

fn parse_event(debugger: &mut Imgui, emu: &mut Emu, event: &Event) -> Option<Result<()>> {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => return Some(Ok(())),
        Event::KeyDown { keycode: Some(keycode), .. } => match *keycode {
            Keycode::Down => {
                emu.bus.directions &= !0b1000;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Up => {
                emu.bus.directions &= !0b0100;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Left => {
                emu.bus.directions &= !0b0010;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Right => {
                emu.bus.directions &= !0b0001;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Return => {
                // Start
                emu.bus.keypresses &= !0b1000;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Delete => {
                // Select
                emu.bus.keypresses &= !0b0100;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::Z => {
                // A
                emu.bus.keypresses &= !0b0001;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            Keycode::X => {
                // B
                emu.bus.keypresses &= !0b0010;
                emu.bus.int_flags |= interrupts::JOYPAD;
            }
            key => {
                info!("{key:?}");
            }
        },
        #[allow(clippy::cast_precision_loss)]
        Event::MouseWheel { y, .. } => {
            debugger.imgui.io_mut().mouse_wheel = *y as f32;
        }
        _ => {}
    }
    None
}

trait GBWindow {
    fn copy_framebuffer(&mut self, buffer: &PixelData);
}
impl GBWindow for Texture<'_> {
    fn copy_framebuffer(&mut self, framebuffer: &PixelData) {
        self.with_lock(None, |buffer, _| {
            let mut i = 0;
            for pixel in framebuffer {
                let bytes = pixel.to_be_bytes();
                buffer[i..(i + 4)].copy_from_slice(&bytes);
                i += 4;
            }
        })
        .unwrap();
    }
}

fn draw_debugger(info: &mut debugger::Info, ui: &imgui::Ui, dt: usize, running: &mut bool, cycle_jump: &mut i32, emu: &mut Emu) {
    if let Some(&after_delay) = info.frame_times.back() {
        ui.text(format!("Frame time: {after_delay:?}"));
        let i = info.frame_times.make_contiguous();
        ui.plot_lines("Frame times", i).graph_size([300.0, 100.0]).build();

        #[allow(clippy::cast_precision_loss)]
        let cpu_hz = dt as f32 / after_delay;
        ui.text(format!("CPU HZ: {cpu_hz}"));
    }
    if let Some(&current) = info.memory_usage_curr.back() {
        ui.text(format!("Memory usage: {current:.2} KB"));
        let i = info.memory_usage_curr.make_contiguous();
        ui.plot_lines("Memory usage", i).graph_size([400.0, 100.0]).build();
    }
    if let Some(&current) = info.memory_usage_peak.back() {
        ui.text(format!("Memory usage peak: {current:.2} KB"));
        let i = info.memory_usage_peak.make_contiguous();
        ui.plot_lines("Memory usage", i).graph_size([400.0, 100.0]).build();
    }

    ui.text(format!("Register State:\n{}", emu.cpu.registers));
    if ui.button("Pause") {
        info!("Pause");
        *running = !*running;
    }

    ui.input_int("Run for n cycles", cycle_jump).build();
    _ = ui.slider("##", 0, 69905, cycle_jump);
    if ui.button("Go") {
        let target_clock = emu.bus.mclock().checked_add_signed(*cycle_jump as isize).unwrap();
        emu.run_until(target_clock);
    }

    ui.text(format!("Bus Info:\n{}", emu.bus));
    ui.text(format!("GPU Info:\n{}", emu.bus.gpu));

    if ui.button("Hex Dump") {
        emu.bus.gpu.hex_dump();
    }
    if ui.button("Frame") {
        info!("Frame");
        emu.run_until(emu.bus.mclock() + CYCLES_PER_FRAME);
    }
}
