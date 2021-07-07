use std::{error::Error, fs::File, io::Read, path::PathBuf};

use crate::bus::Bus;
use crate::instructions::Instr;
use crate::instructions::INSTR_DATA_LENGTHS;
use crate::instructions::INSTR_TABLE;
use crate::{cpu::CPU, gpu::PixelData};

#[derive(Clone, Debug, Default)]
pub struct InstrListing {
    pub instr: Instr,
    pub data: Option<u16>,
    pub addr: u16,
}
pub struct InstrList {
    pub il: Vec<InstrListing>,
}
pub fn gen_il(mem: &[u8]) -> Vec<InstrListing> {
    let mut view = vec![];
    let mut i = 0;
    while i < mem.len() {
        let op = mem[i];
        let instr = INSTR_TABLE[op as usize];
        let data_length = INSTR_DATA_LENGTHS[op as usize];
        let data = match data_length {
            0 => None,
            1 => Some(mem[i + 1] as u16),
            2 => Some(u16::from_le_bytes([mem[i + 1], mem[i + 2]])),
            _ => unreachable!(),
        };
        view.push(InstrListing {
            instr,
            data,
            addr: i as u16,
        });
        i += 1 + data_length;
    }
    view
}

pub fn str_il(il: &[InstrListing]) -> String {
    il.iter().fold(String::new(), |res, il| {
        res + &format!("{:04x}: {:?} {:?}\n", il.addr, il.instr, il.data)
    })
}

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
    pub framebuffer: Box<PixelData>,
}

impl Emu {
    pub fn emulate_step(&mut self) {
        self.prev = self.cpu.clone();
        // println!("{}", self.cpu);
        self.cpu.step(&mut self.bus);
    }

    pub fn new(rom: Vec<u8>, bootrom: Option<PathBuf>) -> Emu {
        let cpu = CPU::new();
        let bus = Bus::new(rom, bootrom);
        Emu {
            cpu,
            bus,
            framebuffer: Box::new([[0; 256]; 256]),
        }
    }

    pub fn from_path(input: PathBuf, bootrom: Option<PathBuf>) -> Result<Emu, Box<dyn Error>> {
        let mut file = File::open(input)?;
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;
        let mut cpu = CPU::new();
        let bus = Bus::new(rom, bootrom);
        Ok(Emu {
            cpu,
            bus,
            framebuffer: Box::new([[0; 256]; 256]),
        })
    }

    pub fn gen_il(&self, mem: &[u8]) -> Vec<InstrListing> {
        let mut view = vec![];
        let mut i = 0;
        while i < mem.len() {
            let op = mem[i];
            let instr = INSTR_TABLE[op as usize];
            let data_length = INSTR_DATA_LENGTHS[op as usize];
            let data = match data_length {
                0 => None,
                1 => Some(mem[i + 1] as u16),
                2 => Some(u16::from_le_bytes([mem[i + 1], mem[i + 2]])),
                _ => unreachable!(),
            };
            view.push(InstrListing {
                instr,
                data,
                addr: i as u16,
            });
            i += 1 + data_length;
        }
        view
    }

    pub fn view(&self) -> Vec<InstrListing> {
        let pc = self.cpu.op_addr;
        let mem = if self.bus.in_bios == 0 {
            &self.bus.bootrom[..]
        } else {
            &self.bus.memory[..]
        };
        let il = gen_il(&mem);
        il.chunks(10)
            .find(|chunk| chunk.iter().any(|e| e.addr == pc))
            .unwrap_or_else(|| {
                panic!(
                    "PC: {:04x} {:?}",
                    pc, INSTR_TABLE[mem[pc as usize] as usize]
                )
            })
            .to_vec()
    }
}
