use color_eyre::Result;
use std::{fs::File, io::Read, path::PathBuf};

use crate::{
    bus::Bus,
    cpu::CPU,
    gpu::PixelData,
    instructions::{INSTR_DATA_LENGTHS, INSTR_TABLE, Instr},
};

#[derive(Clone, Debug, Default)]
pub struct InstrListing {
    pub instr: Instr,
    pub data: Option<u16>,
    pub addr: u16,
}
pub struct InstrList {
    pub il: Vec<InstrListing>,
}

#[must_use]
pub fn gen_il(mem: &[u8]) -> Vec<InstrListing> {
    let mut view = vec![];
    let mut i = 0;
    while i < mem.len() {
        let op = mem[i];
        let instr = INSTR_TABLE[op as usize];
        let data_length = INSTR_DATA_LENGTHS[op as usize];
        let data = match data_length {
            0 => None,
            1 => Some(u16::from(mem[i + 1])),
            2 => Some(u16::from_le_bytes([mem[i + 1], mem[i + 2]])),
            _ => unreachable!(),
        };
        view.push(InstrListing { instr, data, addr: i as u16 });
        i += 1 + data_length;
    }
    view
}

#[must_use]
pub fn str_il(il: &[InstrListing]) -> String {
    il.iter()
        .fold(String::new(), |res, il| res + &format!("{:04x}: {:?} {:?}\n", il.addr, il.instr, il.data))
}

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
    pub framebuffer: PixelData,
}

impl Emu {
    pub fn emulate_step(&mut self) {
        // self.prev = self.cpu.clone();
        // info!("{}", self.cpu);
        self.cpu.step(&mut self.bus);
    }

    pub fn run_until(&mut self, target_clock: usize) -> usize {
        while self.bus.mclock() < target_clock {
            self.emulate_step();
        }
        self.bus.mclock()
    }

    #[must_use]
    pub fn new(rom: &[u8], bootrom: Option<PathBuf>) -> Self {
        let cpu = CPU::new();
        let bus = Bus::new(rom, bootrom);
        let buf = ndarray::Array2::<u32>::zeros((256, 256));
        Self { cpu, bus, framebuffer: buf }
    }

    /// # Errors
    pub fn from_path(input: PathBuf, bootrom: Option<PathBuf>) -> Result<Self> {
        let mut file = File::open(input)?;
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;
        let cpu = CPU::new();
        let bus = Bus::new(&rom, bootrom);
        Ok(Self {
            cpu,
            bus,
            framebuffer: ndarray::Array2::<u32>::zeros((256, 256)),
        })
    }

    #[must_use]
    pub fn gen_il(&self, mem: &[u8]) -> Vec<InstrListing> {
        let mut view = vec![];
        let mut i = 0;
        while i < mem.len() {
            let op = mem[i];
            let instr = INSTR_TABLE[op as usize];
            let data_length = INSTR_DATA_LENGTHS[op as usize];
            let data = match data_length {
                0 => None,
                1 => Some(u16::from(mem[i + 1])),
                2 => Some(u16::from_le_bytes([mem[i + 1], mem[i + 2]])),
                _ => unreachable!(),
            };
            view.push(InstrListing { instr, data, addr: i as u16 });
            i += 1 + data_length;
        }
        view
    }

    pub fn view(&self) -> Vec<InstrListing> {
        let pc = self.cpu.op_addr;
        let mem = if self.bus.in_bios == 0 { &self.bus.bootrom[..] } else { &self.bus.memory[..] };
        let il = gen_il(mem);
        il.chunks(10)
            .find(|chunk| chunk.iter().any(|e| e.addr == pc))
            .unwrap_or_else(|| panic!("PC: {:04x} {:?}", pc, INSTR_TABLE[mem[pc as usize] as usize]))
            .to_vec()
    }
}
