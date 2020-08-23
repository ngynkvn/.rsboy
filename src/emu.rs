use crate::bus::Bus;
use crate::cpu::CPU;
use log::info;

extern crate wasm_bindgen;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

// Global emu struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
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
        Emu { cpu, bus, prev }
    }
}
