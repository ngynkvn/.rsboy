use crate::gpu::GPU;
use std::fs::File;
use std::io::Read;
use std::ops::Index;
use std::ops::IndexMut;

const VRAM_START: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;

pub struct Memory {
    pub memory: [u8; 0x10000],
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
        let mut memory = [0; 0x10000];
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

    // Compare to
    // https://gbdev.gg8.se/wiki/articles/Power_Up_Sequence
    pub fn dump_io(&self) {
        println!("{:04X} = {:02X}; {}", 0xFF05, self[0xFF05], "TIMA");
        println!("{:04X} = {:02X}; {}", 0xFF06, self[0xFF06], "TMA");
        println!("{:04X} = {:02X}; {}", 0xFF07, self[0xFF07], "TAC");
        println!("{:04X} = {:02X}; {}", 0xFF10, self[0xFF10], "NR10");
        println!("{:04X} = {:02X}; {}", 0xFF11, self[0xFF11], "NR11");
        println!("{:04X} = {:02X}; {}", 0xFF12, self[0xFF12], "NR12");
        println!("{:04X} = {:02X}; {}", 0xFF14, self[0xFF14], "NR14");
        println!("{:04X} = {:02X}; {}", 0xFF16, self[0xFF16], "NR21");
        println!("{:04X} = {:02X}; {}", 0xFF17, self[0xFF17], "NR22");
        println!("{:04X} = {:02X}; {}", 0xFF19, self[0xFF19], "NR24");
        println!("{:04X} = {:02X}; {}", 0xFF1A, self[0xFF1A], "NR30");
        println!("{:04X} = {:02X}; {}", 0xFF1B, self[0xFF1B], "NR31");
        println!("{:04X} = {:02X}; {}", 0xFF1C, self[0xFF1C], "NR32");
        println!("{:04X} = {:02X}; {}", 0xFF1E, self[0xFF1E], "NR33");
        println!("{:04X} = {:02X}; {}", 0xFF20, self[0xFF20], "NR41");
        println!("{:04X} = {:02X}; {}", 0xFF21, self[0xFF21], "NR42");
        println!("{:04X} = {:02X}; {}", 0xFF22, self[0xFF22], "NR43");
        println!("{:04X} = {:02X}; {}", 0xFF23, self[0xFF23], "NR44");
        println!("{:04X} = {:02X}; {}", 0xFF24, self[0xFF24], "NR50");
        println!("{:04X} = {:02X}; {}", 0xFF25, self[0xFF25], "NR51");
        println!("{:04X} = {:02X}; {}", 0xFF26, self[0xFF26], "NR52");
        println!("{:04X} = {:02X}; {}", 0xFF40, self[0xFF40], "LCDC");
        println!("{:04X} = {:02X}; {}", 0xFF42, self[0xFF42], "SCY");
        println!("{:04X} = {:02X}; {}", 0xFF43, self[0xFF43], "SCX");
        println!("{:04X} = {:02X}; {}", 0xFF45, self[0xFF45], "LYC");
        println!("{:04X} = {:02X}; {}", 0xFF47, self[0xFF47], "BGP");
        println!("{:04X} = {:02X}; {}", 0xFF48, self[0xFF48], "OBP0");
        println!("{:04X} = {:02X}; {}", 0xFF49, self[0xFF49], "OBP1");
        println!("{:04X} = {:02X}; {}", 0xFF4A, self[0xFF4A], "WY");
        println!("{:04X} = {:02X}; {}", 0xFF4B, self[0xFF4B], "WX");
        println!("{:04X} = {:02X}; {}", 0xFFFF, self[0xFFFF], "IE");
    }
}
impl Index<u16> for Memory {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        match i as usize {
            0x0000..=0x0100 if self.in_bios => &self.bootrom[i as usize],
            0xFF40 => &self.gpu.lcdc,
            0xFF41 => &self.gpu.lcdstat,
            0xFF42 => &self.gpu.vscroll,
            0xFF43 => &self.gpu.hscroll,
            0xFF44 => &self.gpu.scanline,
            VRAM_START..=VRAM_END => &self.gpu[i - VRAM_START as u16],
            _ => &self.memory[i as usize],
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        match i as usize {
            0x0000..=0x0100 if self.in_bios => {
                panic!("We tried to mutate bootrom while in bios mode.")
            }
            0xFF40 => &mut self.gpu.lcdc,
            0xFF41 => &mut self.gpu.lcdstat,
            0xFF42 => &mut self.gpu.vscroll,
            0xFF43 => &mut self.gpu.hscroll,
            0xFF44 => &mut self.gpu.scanline,
            0xFF4A | 0xFF4B => panic!("NOT IMPLEMENTED IN MEM MAP"),
            VRAM_START..=VRAM_END => &mut self.gpu.vram[i as usize - VRAM_START],
            _ => &mut self.memory[i as usize],
        }
    }
}
