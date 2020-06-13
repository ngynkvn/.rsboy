use crate::bus::Bus;
use crate::cpu::CPU;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
}

impl Emu {
    pub fn cycle(&mut self) -> Result<usize, String> {
        self.cpu.cycle(&mut self.bus)
    }

    pub fn new(skip_bios: bool, rom: Vec<u8>) -> Emu {
        let cpu = CPU::new(skip_bios);
        let bus = Bus::new(skip_bios, rom);
        Emu { cpu, bus }
    }
}
