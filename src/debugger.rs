use std::collections::VecDeque;

use crate::emu::InstrListing;
use color_eyre::{Result, eyre};

use glow::HasContext;
use imgui::{Context, Ui};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::video::{GLContext, Window};

#[derive(Default)]
pub struct Info {
    pub frame_times: VecDeque<f32>,
    pub before_sleep: VecDeque<f32>,
    pub memory_usage_peak: VecDeque<f32>,
    pub memory_usage_curr: VecDeque<f32>,
    pub cpu_hz: VecDeque<f32>,
    pub il: Vec<InstrListing>,
}

pub struct Imgui {
    pub imgui: Context,
    pub renderer: AutoRenderer,
    pub platform: SdlPlatform,
    pub window: Window,
    pub info: Info,
    pub gl_context: GLContext,
}

impl Imgui {
    /// # Errors
    ///
    /// This function will return an error if the window's GL context cannot be created.
    pub fn new(window: Window) -> Result<Self> {
        let gl_context = window.gl_create_context().map_err(|e| eyre::eyre!(e))?;
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                window.subsystem().gl_get_proc_address(s).cast()
            })
        };
        let mut imgui = imgui::Context::create();
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        /* create platform and renderer */
        let platform = SdlPlatform::new(&mut imgui);
        let renderer = AutoRenderer::new(gl, &mut imgui)?;

        let mut info: Info = Info::default();
        info.frame_times.resize(200, 0.0);

        Ok(Self {
            imgui,
            renderer,
            platform,
            window,
            info,
            gl_context,
        })
    }

    /// # Panics
    ///
    /// This function will panic if the renderer fails to render.
    pub fn frame<F: FnOnce(&mut Info, &Ui)>(&mut self, event_pump: &mut sdl2::EventPump, f: F) {
        /* call prepare_frame before calling imgui.new_frame() */
        self.platform
            .prepare_frame(&mut self.imgui, &self.window, event_pump);
        let ui = self.imgui.new_frame();

        f(&mut self.info, ui);
        let draw_data = self.imgui.render();
        if draw_data.draw_lists_count() > 0 {
            unsafe {
                self.renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
            }
            self.renderer.render(draw_data).unwrap();
            self.window.gl_swap_window();
        }
    }
}

impl Info {
    pub fn add_frame_time(&mut self, time: f32) {
        self.frame_times.push_back(time * 1000.0);
        self.frame_times.truncate_front(100);
    }

    pub fn add_before_sleep_time(&mut self, time: f32) {
        self.before_sleep.push_back(time * 1000.0);
        self.before_sleep.truncate_front(100);
    }
    pub fn add_memory_usage(&mut self, usage: (f32, f32)) {
        self.memory_usage_curr.push_back(usage.0);
        self.memory_usage_peak.push_back(usage.1);
        self.memory_usage_curr.truncate_front(100);
        self.memory_usage_peak.truncate_front(100);
    }
}
