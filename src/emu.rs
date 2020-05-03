use crate::cpu::CPU2;
use crate::gpu::GPU;
use crate::memory::Memory;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU2,
    pub gpu: GPU,
    pub memory: Memory,
}

impl Emu {
    fn cycle() {

    }
}