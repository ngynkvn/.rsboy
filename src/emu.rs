use crate::cpu::CPU;
use crate::bus::Bus;

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
}

impl Emu {
    pub fn cycle(&mut self) -> Result<usize, String> {
        self.cpu.cycle(&mut self.bus)
    }

    pub fn new(skip_bios: Option<bool>, rom: Vec<u8>) -> Emu {
        let cpu = CPU::new(skip_bios.unwrap_or(false));
        let bus = Bus::new(skip_bios.unwrap_or(false), rom);
        Emu {
            cpu,
            bus,
        }
    }
}
