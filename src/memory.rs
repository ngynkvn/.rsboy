use std::ops::Index;
use std::ops::IndexMut;  
use std::fs::File;
use std::io::{Read};
use crate::gpu::GPU;


const VRAM_START: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;

pub struct Memory {
    pub rom: [u8; 0xFFFF],
    pub gpu: GPU,
}

fn load_bootrom() -> Vec<u8> {
    let mut file = File::open("dmg_boot.bin").expect("Couldn't open bootrom file.");
    let mut bootrom = Vec::new();
    file.read_to_end(&mut bootrom).expect("Couldn't read the file.");
    bootrom
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = [0; 0xFFFF];
        let bootrom = load_bootrom();
        mem[0..0x100].clone_from_slice(&bootrom[..]);
        mem[0x100..(rom.len()+0x100)].clone_from_slice(&rom[..]);
        Memory { rom: mem, gpu: GPU::new() }
    }
}
impl Index<u16> for Memory {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        match i as usize {
            VRAM_START..=VRAM_END => &self.gpu[i - VRAM_START as u16],
            0xFF44 => &self.gpu.scanline,
            _ => &self.rom[i as usize] 
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        match i as usize {
            VRAM_START..=VRAM_END => &mut self.gpu.vram[i as usize - VRAM_START],
            _ => &mut self.rom[i as usize],
        }
    }
}
