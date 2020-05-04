use crate::cpu::Controller;
use crate::cpu::CPU;
use crate::gpu::GPU;
use crate::memory::Memory;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub gpu: GPU,
    pub memory: Memory,
}

impl Emu {
    pub fn cycle(&mut self) {
        let cycles = self.cpu.cycle(&mut self.memory);
    }

    pub fn new(rom: Vec<u8>) -> Emu {
        Emu {
            cpu: CPU::new(),
            gpu: GPU::new(),
            memory: Memory::new(rom),
        }
    }
}
