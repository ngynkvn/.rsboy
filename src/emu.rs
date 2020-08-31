use crate::bus::Bus;
use crate::{cpu::CPU, gpu::PixelData};

extern crate wasm_bindgen;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
    pub framebuffer: Box<PixelData>,
    prev: CPU,
}

impl Emu {
    pub fn cycle(&mut self) -> Result<usize, String> {
        self.prev = self.cpu.clone();
        let result = self.cpu.cycle(&mut self.bus);
        result
    }

    pub fn new(rom: Vec<u8>) -> Emu {
        let cpu = CPU::new();
        let bus = Bus::new(rom);
        let prev = cpu.clone();
        Emu {
            cpu,
            bus,
            framebuffer: Box::new([[0; 256]; 256]),
            prev,
        }
    }
}
