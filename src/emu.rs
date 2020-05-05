use crate::cpu::Controller;
use crate::cpu::CPU;
use crate::gpu::GPU;
use crate::memory::Memory;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    // pub gpu: GPU,
    pub memory: Memory,
}

impl Emu {
    pub fn cycle(&mut self) {
        let mut i = 0;
        while i < 17556 {
            let cycles = self.cpu.cycle(&mut self.memory);
            i += cycles;
            if cycles == 0 {
                println!("No cycles from cpu..");
                break;
            }
        }
        self.memory.gpu.cycle(i);
    }

    pub fn new(rom: Vec<u8>) -> Emu {
        Emu {
            cpu: CPU::new(),
            // gpu: GPU::new(),
            memory: Memory::new(rom),
        }
    }
}
