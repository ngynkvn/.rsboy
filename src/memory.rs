use crate::gpu::GPU;
use std::fs::File;
use std::io::Read;
use std::ops::Index;
use std::ops::IndexMut;

const VRAM_START: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;

pub struct Memory {
    pub memory: [u8; 0xFFFF],
    pub bootrom: [u8; 0x100],
    pub in_bios: bool,
    pub gpu: GPU,
}

fn load_bootrom() -> Vec<u8> {
    let mut file = File::open("dmg_boot.bin").expect("Couldn't open bootrom file.");
    let mut bootrom = Vec::new();
    file.read_to_end(&mut bootrom)
        .expect("Couldn't read the file.");
    bootrom
}

impl Memory {
    pub fn new(rom_vec: Vec<u8>) -> Self {
        let mut memory = [0; 0xFFFF];
        let mut bootrom = [0; 0x100];
        let bootrom_vec = load_bootrom();
        bootrom[..].clone_from_slice(&bootrom_vec[..]);
        memory[..rom_vec.len()].clone_from_slice(&rom_vec[..]);
        Memory {
            memory,
            bootrom,
            in_bios: true,
            gpu: GPU::new(),
        }
    }
}
impl Index<u16> for Memory {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        match i as usize {
            0x0000..=0x0100 if self.in_bios => &self.bootrom[i as usize],
            VRAM_START..=VRAM_END => &self.gpu[i - VRAM_START as u16],
            0xFF44 => &self.gpu.scanline,
            _ => &self.memory[i as usize],
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        match i as usize {
            0x0000..=0x0100 if self.in_bios => {
                panic!("We tried to access bootrom while in bios mode.")
            }
            VRAM_START..=VRAM_END => &mut self.gpu.vram[i as usize - VRAM_START],
            _ => &mut self.memory[i as usize],
        }
    }
}
