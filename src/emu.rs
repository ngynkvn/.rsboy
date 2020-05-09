use crate::controller::Controller;
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
    pub fn cycle(&mut self) -> Result<usize, String> {
        self.cpu.cycle(&mut self.memory)
    }

    pub fn new(rom: Vec<u8>) -> Emu {
        let cpu = CPU::new();
        let memory = Memory::new(rom);
        Emu {
            cpu,
            memory,
        }
    }
}
