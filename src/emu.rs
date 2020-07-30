use crate::bus::Bus;
use crate::cpu::CPU;

extern crate wasm_bindgen;

// Global emu struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
}

impl Emu {
    pub fn cycle(&mut self) -> Result<usize, String> {
        self.cpu.cycle(&mut self.bus)
    }

    pub fn new(rom: Vec<u8>) -> Emu {
        let cpu = CPU::new();
        let bus = Bus::new(rom);
        Emu { cpu, bus }
    }
}
