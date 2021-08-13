extern crate gl;
extern crate imgui_opengl_renderer;
//SDL

use crate::constants::CYCLES_PER_FRAME;
use crate::constants::FRAME_TIME;

use crate::constants::MAP_WIDTH;
use crate::constants::WINDOW_HEIGHT;
use crate::constants::WINDOW_WIDTH;

use crate::debugger::Imgui;
use color_eyre::eyre;
use color_eyre::Report;
use glium::backend::Backend;
use glium::backend::Facade;
use glium::draw_parameters::PolygonMode;
use glium::index::PrimitiveType;
use glium::texture::ClientFormat;
use glium::texture::MipmapsOption;
use glium::texture::RawImage2d;
use glium::uniform;
use glium::uniforms;
use glium::DrawParameters;
use glium::IndexBuffer;
use glium::Surface;
use glium::VertexBuffer;
use glium_glue::sdl2::SDL2Facade;
use imgui::im_str;
use imgui::Condition;
use imgui::Slider;

use imgui::Window;
use rust_emu::gpu::PixelMap;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

//File IO
use log::info;

use gpu::PixelData;
use rust_emu::{cpu::JOYPAD, debugger, emu::gen_il, emu::Emu};

use crate::constants::MaybeErr;
use rust_emu::*;

use clap::Clap;

#[derive(Clap)]
#[clap(name = ".rsboy", about = "Rust emulator")]
struct Settings {
    #[clap(parse(from_os_str))]
    input: PathBuf,
    #[clap(parse(from_os_str))]
    logfile: Option<PathBuf>,
    #[clap(short, long, default_value = "dmg_boot.bin")]
    bootrom: PathBuf,
}

use crate::eyre::eyre;
use color_eyre::Result;
use glium_glue::sdl2::DisplayBuild;
fn main() -> Result<()> {
    color_eyre::install()?;
    // When the program starts up, parse command line arguments and setup additional systems.
    let settings = Settings::parse();
    info!("Running SDL Main");
    let mut emu = Emu::from_path(settings.input, settings.bootrom)?;

    let context = sdl2::init().map_err(|s| eyre!("Unable to create SDL2 Context: {}", s))?;

    let video = context.video().unwrap();
    let mut rsboy = video
        .window(".rsboy", WINDOW_WIDTH, WINDOW_HEIGHT)
        .resizable()
        .build_glium()?;

    sdl_main(&mut rsboy, &context, &mut emu).unwrap();
    map_viewer(&context, &emu).unwrap();
    vram_viewer(&context, &emu).unwrap();
    Ok(())
}

const VERT: &str = "#version 330 core
in vec2 pos;
in vec2 tex;

out vec2 tc;

uniform mat4 projection;
uniform mat4 model;

void main()
{
    gl_Position = vec4(pos, 0.0, 1.0);
    tc = tex;
}";

const FRAG: &str = "#version 330 core
in vec2 tc;
out vec4 color;

uniform sampler2D image;
void main() {
    color = texelFetch(image, ivec2(tc.xy * vec2(256.0)), 0);
}";

use glium::implement_vertex;
#[derive(Clone, Copy)]
pub struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}
implement_vertex!(Vertex, pos, tex);

extern crate nalgebra_glm as glm;

fn sdl_main(video: &mut SDL2Facade, context: &sdl2::Sdl, emu: &mut Emu) -> Result<()> {
    let mut event_pump = context.event_pump().unwrap();
    let program = glium::Program::from_source(video, VERT, FRAG, None)?;
    let mut framebuffer: PixelData = [[0; 256]; 256];
    let texture = glium::Texture2d::empty_with_mipmaps(video, MipmapsOption::NoMipmap, 256, 256)?;

    let mut quad = [
        Vertex {
            pos: [-1.0, 1.0],
            tex: [0.0, 1.0], // top left
        },
        Vertex {
            pos: [1.0, -1.0],
            tex: [1.0, 0.0], // bottom right
        },
        Vertex {
            pos: [-1.0, -1.0],
            tex: [0.0, 0.0], // bottom left
        },
        Vertex {
            pos: [1.0, 1.0],
            tex: [1.0, 1.0], // top right
        },
    ];
    // TEMP
    // Insetting the vertices for the display.
    for v in &mut quad {
        if v.pos[0] < 0.0 {
            v.pos[0] += 0.05;
        } else {
            v.pos[0] += -0.05;
        }
        if v.pos[1] < 0.0 {
            v.pos[1] += 0.05;
        } else {
            v.pos[1] += -0.05;
        }
    }
    let vertex_buffer = VertexBuffer::new(video, &quad)?;
    // building the index buffer
    let index_buffer =
        IndexBuffer::new(video, PrimitiveType::TrianglesList, &[0u16, 1, 2, 0, 3, 1])?;

    let projection = glm::ortho::<f32>(-256 as _, 256 as _, 256 as _, -256 as _, -1 as _, 1 as _);
    let mut imgui = imgui::Context::create();
    let mut imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| unsafe {
        video.backend.get_proc_address(s)
    });
    imgui.fonts().build_rgba32_texture();
    imgui.io_mut().display_size = [WINDOW_WIDTH as _, WINDOW_HEIGHT as _];

    loop {
        let io = imgui.io_mut();
        let now = Instant::now();
        for event in event_pump.poll_iter() {
            emu.bus.directions |= 0x0F;
            emu.bus.keypresses |= 0x0F;
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => {
                    io.display_size = [w as _, h as _];
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Down => {
                        emu.bus.directions &= !0b1000;
                        emu.bus.int_flags |= JOYPAD;
                    }
                    Keycode::Up => {
                        emu.bus.directions &= !0b0100;
                        emu.bus.int_flags |= JOYPAD;
                    }
                    Keycode::Left => {
                        emu.bus.directions &= !0b0010;
                        emu.bus.int_flags |= JOYPAD;
                    }
                    Keycode::Right => {
                        emu.bus.directions &= !0b0001;
                        emu.bus.int_flags |= JOYPAD;
                    }
                    Keycode::Return => {
                        emu.bus.keypresses &= !0b1000;
                        emu.bus.int_flags |= JOYPAD;
                    }
                    Keycode::Z => {
                        //A?
                    }
                    Keycode::B => {
                        //B?
                    }
                    key => {
                        println!("{:?}", key);
                    }
                },
                Event::MouseMotion { x, y, .. } => {
                    io.mouse_pos = [x as _, y as _];
                }
                Event::MouseWheel { y, .. } => {
                    io.mouse_wheel = y as _;
                }
                _ => {}
            }
        }
        let mouse_state = event_pump.mouse_state();
        io.mouse_down = [mouse_state.left(), mouse_state.right(), false, false, false];

        let mut delta_clock = 0;
        // if !pause {
        let before = emu.bus.clock;
        while emu.bus.clock < before + CYCLES_PER_FRAME {
            emu.emulate_step();
        }
        delta_clock = emu.bus.clock - before;
        // }
        // Render to framebuffer and copy.
        emu.bus.gpu.render_to(&mut framebuffer);
        let (h, v) = emu.bus.gpu.scroll();
        let mut target = video.draw();

        unsafe {
            let flat = std::mem::transmute::<PixelData, PixelMap>(framebuffer);
            let raw = RawImage2d::from_raw_rgba_reversed(&flat, (256, 256));
            texture.write(
                glium::Rect {
                    left: 0,
                    bottom: 0,
                    width: 256,
                    height: 256,
                },
                raw,
            );
        }

        let sampler = texture
            .sampled()
            .magnify_filter(uniforms::MagnifySamplerFilter::Nearest)
            .minify_filter(uniforms::MinifySamplerFilter::Nearest);

        use num_traits::identities::One;
        let uniforms = uniform! {
            image: sampler,
            offset: (h, v)
        };

        let imgui = imgui.frame();
        Window::new(im_str!("Test"))
            .size([100.0, 100.0], Condition::Once)
            .scroll_bar(false)
            .build(&imgui, || {
                imgui.text(im_str!("Test"));
            });
        imgui.show_about_window(&mut true);
        imgui.show_demo_window(&mut true);
        imgui.show_metrics_window(&mut true);

        // imgui

        target.clear_color(0.2, 0.2, 0.2, 1.0);

        target.draw(
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniforms,
            &Default::default(),
        )?;
        target.draw(
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniforms,
            &DrawParameters {
                polygon_mode: PolygonMode::Line,
                ..Default::default()
            },
        )?;
        imgui_renderer.render(imgui);
        target.finish()?;

        // TODO HERE
        // texture.copy_window(h, v, &emu.framebuffer);
        // video.copy(&texture, None, None).unwrap();
        // video.present();

        // Delay a minimum of 16.67 milliseconds (60 fps).
        if let Some(time) = FRAME_TIME.checked_sub(now.elapsed()) {
            spin_sleep::sleep(time);
        }

        // Log frame time
        // let after_delay = now.elapsed();
        // debugger.add_frame_time(after_delay.as_secs_f32());

        // //ImGui display frame.
        // debugger.frame(&mut event_pump, |info, ui| {
        //     ui.text(format!("Frame time: {:?}", after_delay));
        //     let i = info.frame_times.as_slice();
        //     ui.plot_lines(im_str!("Frame times"), i)
        //         .graph_size([300.0, 100.0])
        //         .build();
        //     let cpu_hz = delta_clock as f64 / after_delay.as_secs_f64();
        //     ui.text(format!("CPU HZ: {}", cpu_hz));
        //     ui.text(format!("Register State:\n{}", emu.cpu.registers));
        //     if ui.button(im_str!("Pause"), [200.0, 50.0]) {
        //         println!("Pause");
        //         pause = !pause;
        //     }
        //     ui.input_int(im_str!("Run for n cycles"), &mut cycle_jump)
        //         .build();
        //     Slider::new(im_str!(""))
        //         .range(0..=(69905))
        //         .build(ui, &mut cycle_jump);
        //     if ui.button(im_str!("Go"), [200.0, 50.0]) {
        //         let before = emu.bus.clock as i32;
        //         while emu.bus.clock < (before + cycle_jump) as usize {
        //             emu.emulate_step();
        //         }
        //     }
        //     ui.text(format!("Bus Info:\n{}", emu.bus));
        //     ui.text(format!("GPU Info:\n{}", emu.bus.gpu));
        //     if ui.button(im_str!("Hex Dump"), [200.0, 50.0]) {
        //         emu.bus.gpu.hex_dump()
        //     }
        //     if ui.button(im_str!("Frame"), [200.0, 50.0]) {
        //         println!("Frame");
        //         let before = emu.bus.clock;
        //         while emu.bus.clock < before + CYCLES_PER_FRAME {
        //             emu.emulate_step();
        //         }
        //     }
        // });
    }
}

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
                    let bytes = framebuffer[y][x].to_be_bytes();
                    buffer[i..(i + 4)].copy_from_slice(&bytes);
                    i += 4;
                }
            }
        })
        .unwrap();
    }
    fn copy_map(&mut self, buffer: &PixelData) {
        let mut i = 0;
        self.with_lock(None, |tbuffer, _| {
            for y in buffer.iter() {
                for x in y.iter() {
                    let bytes = x.to_be_bytes();
                    tbuffer[i..(i + 4)].copy_from_slice(&bytes);
                    i += 4;
                }
            }
        })
        .unwrap();
    }
}

fn map_viewer(sdl_context: &sdl2::Sdl, emu: &emu::Emu) -> Result<(), String> {
    let gpu = &emu.bus.gpu;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Map Viewer", 256, 256)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA32, 256, 256)
        .map_err(|e| e.to_string())?;

    // Pitch = n_bytes(3) * map_w * tile_w
    texture.copy_map(&emu.framebuffer);
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
    }

    Ok(())
}

fn vram_viewer(sdl_context: &sdl2::Sdl, emu: &emu::Emu) -> MaybeErr<()> {
    let gpu = &emu.bus.gpu;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("VRAM Viewer", 1024, 512)
        .position_centered()
        .build()?;
    let mut canvas = window.into_canvas().build()?;

    let texture_creator = canvas.texture_creator();

    let mut update = |palette: u8| -> MaybeErr<()> {
        let tiles = gpu.tiles(palette);
        for (i, t) in tiles.iter().enumerate() {
            let i = i as i32;
            let mut tex =
                texture_creator.create_texture_streaming(PixelFormatEnum::RGBA32, 8, 8)?;
            tex.with_lock(None, |data, _| {
                let mut c = 0;
                for i in t.texture.iter() {
                    for j in i.iter() {
                        let d = j.to_be_bytes();
                        data[c..(c + 4)].copy_from_slice(&d);
                        c += 4;
                    }
                }
            })?;
            let rect = ((i % 32) * 32, (i / 32) * 32, 32, 32);
            let rect = Rect::from(rect);
            canvas.copy(&tex, None, rect)?
        }
        canvas.present();
        Ok(())
    };
    let ps = [gpu.bgrdpal, gpu.obj0pal, gpu.obj1pal];
    let mut i = 0;
    update(ps[i])?;
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Return => {
                        i += 1;
                        i %= ps.len();
                        println!("{}", i);
                        update(ps[i])?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
