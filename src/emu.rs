use crate::instructions::Instr;
use std::ops::Range;
use std::iter::Zip;
use crate::bus::Bus;
use crate::{cpu::CPU, gpu::PixelData};


pub struct IL {
    pub ty: Instr,
    pub data: Option<u16>,
    pub addr: u16,
}

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
    pub framebuffer: Box<PixelData>,
    prev: CPU,
}

impl Emu {
    pub fn emulate_step(&mut self) -> usize {
        self.prev = self.cpu.clone();
        let cycles = self.bus.clock;
        self.cpu.step(&mut self.bus);
        self.bus.clock - cycles
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
