use crate::bus::Bus;
use crate::instructions::Instr;
use crate::instructions::INSTR_DATA_LENGTHS;
use crate::instructions::INSTR_TABLE;
use crate::{cpu::CPU, gpu::PixelData};

#[derive(Clone, Debug)]
pub struct IL {
    pub instr: Instr,
    pub data: Option<u16>,
    pub addr: u16,
}

// Global emu struct.
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
    pub framebuffer: Box<PixelData>,
    prev: CPU,
    il: Vec<IL>,
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
            il: vec![],
        }
    }

    pub fn gen_il(&self, mem: &[u8]) -> Vec<IL> {
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
            view.push(IL {
                instr,
                data,
                addr: i as u16,
            });
            i += 1 + data_length;
        }
        view
    }

    pub fn view(&self) -> Vec<IL> {
        let pc = self.cpu.op_addr;
        let mem = if self.bus.in_bios == 0 {
            &self.bus.bootrom[..]
        } else {
            &self.bus.memory[..]
        };
        let il = self.gen_il(&mem);
        il.chunks(10)
            .find(|chunk| chunk.iter().any(|e| e.addr == pc))
            .expect(&format!("PC: {:04x} {:?}", pc, INSTR_TABLE[mem[pc as usize] as usize]))
            .to_vec()
    }
}
