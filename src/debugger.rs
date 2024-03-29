extern crate imgui_opengl_renderer;
use crate::constants::MaybeErr;
use crate::emu::InstrListing;

use imgui::{Context, Ui};
use imgui_opengl_renderer::Renderer;
use sdl2::video::Window;
use sdl2::{video::GLContext};
use std::collections::VecDeque;

#[derive(Default)]
pub struct Info {
    pub frame_times: Vec<f32>,
    f_i: usize,
    pub il: Vec<InstrListing>,
}

pub struct Imgui<'a> {
    pub imgui: Context,
    pub renderer: Renderer,
    pub window: &'a Window,
    pub _gl_context: GLContext,
    pub info: Info,
}

impl<'a> Imgui<'a> {
    pub fn new(window: &'a Window) -> MaybeErr<Self> {
        let mut imgui = imgui::Context::create();
        imgui.fonts().build_rgba32_texture();
        let _gl_context = window.gl_create_context()?;
        gl::load_with(|s| window.subsystem().gl_get_proc_address(s) as _);

        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            window.subsystem().gl_get_proc_address(s) as _
        });

        let mut info: Info = Default::default();
        info.frame_times.resize(200, 0.0);

        Ok(Self {
            imgui,
            renderer,
            window,
            _gl_context,
            info,
        })
    }
    pub fn capture_io(&mut self, event_pump: &mut sdl2::EventPump) {
        let io = self.imgui.io_mut();
        let state = event_pump.mouse_state();
        let (width, height) = self.window.drawable_size();
        io.display_size = [width as f32, height as f32];
        io.mouse_down = [
            state.left(),
            state.right(),
            state.middle(),
            state.x1(),
            state.x2(),
        ];
        io.mouse_pos = [state.x() as f32, state.y() as f32];
    }
    pub fn frame<F: FnOnce(&mut Info, &Ui)>(&mut self, event_pump: &mut sdl2::EventPump, f: F) {
        self.capture_io(event_pump);
        let ui = self.imgui.frame();
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        f(&mut self.info, &ui);
        self.renderer.render(ui);
        self.window.gl_swap_window();
    }
    pub fn add_frame_time(&mut self, time: f32) {
        self.info.frame_times[self.info.f_i] = time * 1000.0;
        self.info.f_i += 1;
        self.info.f_i %= self.info.frame_times.capacity();
    }
}
