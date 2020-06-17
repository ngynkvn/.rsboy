use crate::gpu::GPU;
use std::fs::File;
use std::io::Read;

const VRAM_START: usize = 0x8000;
const VRAM_END: usize = 0x9FFF;

pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub enum Select {
    Buttons,
    Directions
}

pub struct Bus {
    pub memory: [u8; 0x10000],
    pub bootrom: [u8; 0x100],
    pub in_bios: u8,
    pub int_enabled: u8,
    pub int_flags: u8,
    pub interrupts_enabled: bool,
    pub joypad_io: Select,
    pub gpu: GPU,
}

fn load_bootrom() -> Vec<u8> {
    let mut file = File::open("dmg_boot.bin").expect("Couldn't open bootrom file.");
    let mut bootrom = Vec::new();
    file.read_to_end(&mut bootrom)
        .expect("Couldn't read the file.");
    bootrom
}

impl Bus {
    pub fn new(skip_bios: bool, rom_vec: Vec<u8>) -> Self {
        let mut memory = [0; 0x10000];
        let mut bootrom = [0; 0x100];
        let bootrom_vec = load_bootrom();
        bootrom[..].clone_from_slice(&bootrom_vec[..]);
        memory[..rom_vec.len()].clone_from_slice(&rom_vec[..]);
        Bus {
            memory,
            bootrom,
            interrupts_enabled: false,
            in_bios: skip_bios as u8,
            int_enabled: 0,
            int_flags: 0,
            joypad_io: Select::Buttons,
            gpu: GPU::new(),
        }
    }

    pub fn enable_interrupts(&mut self) {
        self.interrupts_enabled = true;
    }

    pub fn disable_interrupts(&mut self) {
        self.interrupts_enabled = false;
    }

    pub fn handle_vblank(&mut self) {
        self.gpu.irq = false; 
        self.int_flags = self.gpu.irq as u8;
    }

    pub fn cycle(&mut self) -> Result<(), String> {
        self.gpu.cycle()?;
        self.int_flags = self.gpu.irq as u8;
        Ok(())
    }

    // Compare to
    // https://gbdev.gg8.se/wiki/articles/Power_Up_Sequence
    pub fn dump_io(&self) {
        println!("{:04X} = {:02X}; {}", 0xFF05, self.read(0xFF05), "TIMA");
        println!("{:04X} = {:02X}; {}", 0xFF06, self.read(0xFF06), "TMA");
        println!("{:04X} = {:02X}; {}", 0xFF07, self.read(0xFF07), "TAC");
        println!("{:04X} = {:02X}; {}", 0xFF10, self.read(0xFF10), "NR10");
        println!("{:04X} = {:02X}; {}", 0xFF11, self.read(0xFF11), "NR11");
        println!("{:04X} = {:02X}; {}", 0xFF12, self.read(0xFF12), "NR12");
        println!("{:04X} = {:02X}; {}", 0xFF14, self.read(0xFF14), "NR14");
        println!("{:04X} = {:02X}; {}", 0xFF16, self.read(0xFF16), "NR21");
        println!("{:04X} = {:02X}; {}", 0xFF17, self.read(0xFF17), "NR22");
        println!("{:04X} = {:02X}; {}", 0xFF19, self.read(0xFF19), "NR24");
        println!("{:04X} = {:02X}; {}", 0xFF1A, self.read(0xFF1A), "NR30");
        println!("{:04X} = {:02X}; {}", 0xFF1B, self.read(0xFF1B), "NR31");
        println!("{:04X} = {:02X}; {}", 0xFF1C, self.read(0xFF1C), "NR32");
        println!("{:04X} = {:02X}; {}", 0xFF1E, self.read(0xFF1E), "NR33");
        println!("{:04X} = {:02X}; {}", 0xFF20, self.read(0xFF20), "NR41");
        println!("{:04X} = {:02X}; {}", 0xFF21, self.read(0xFF21), "NR42");
        println!("{:04X} = {:02X}; {}", 0xFF22, self.read(0xFF22), "NR43");
        println!("{:04X} = {:02X}; {}", 0xFF23, self.read(0xFF23), "NR44");
        println!("{:04X} = {:02X}; {}", 0xFF24, self.read(0xFF24), "NR50");
        println!("{:04X} = {:02X}; {}", 0xFF25, self.read(0xFF25), "NR51");
        println!("{:04X} = {:02X}; {}", 0xFF26, self.read(0xFF26), "NR52");
        println!("{:04X} = {:02X}; {}", 0xFF40, self.read(0xFF40), "LCDC");
        println!("{:04X} = {:02X}; {}", 0xFF42, self.read(0xFF42), "SCY");
        println!("{:04X} = {:02X}; {}", 0xFF43, self.read(0xFF43), "SCX");
        println!("{:04X} = {:02X}; {}", 0xFF45, self.read(0xFF45), "LYC");
        println!("{:04X} = {:02X}; {}", 0xFF47, self.read(0xFF47), "BGP");
        println!("{:04X} = {:02X}; {}", 0xFF48, self.read(0xFF48), "OBP0");
        println!("{:04X} = {:02X}; {}", 0xFF49, self.read(0xFF49), "OBP1");
        println!("{:04X} = {:02X}; {}", 0xFF4A, self.read(0xFF4A), "WY");
        println!("{:04X} = {:02X}; {}", 0xFF4B, self.read(0xFF4B), "WX");
        println!("{:04X} = {:02X}; {}", 0xFFFF, self.read(0xFFFF), "IE");
    }
}

impl Memory for Bus {
    fn read(&self, address: u16) -> u8 {
        match address as usize {
            0x0000..=0x0100 if self.in_bios == 0 => self.bootrom[address as usize],
            0xFF40 => self.gpu.lcdc,
            0xFF41 => self.gpu.lcdstat,
            0xFF42 => self.gpu.vscroll,
            0xFF43 => self.gpu.hscroll,
            0xFF44 => self.gpu.scanline,
            0xFF47 => panic!("0xFF47 (bg_palette) is WRITE ONLY"),
            0xFF4A => self.gpu.windowy,
            0xFF4B => self.gpu.windowx,
            0xffff => self.int_enabled,
            0xff0f => self.int_flags,
            0xff00 => {
                match self.joypad_io {
                    Select::Buttons => {
                        return 0xff;//Todo
                    },
                    Select::Directions => {
                        return 0xff;
                    }
                }
            }
            // 0xFFFF => &self.gpu.,
            // 0xFF01 => {println!("R: ACC SERIAL TRANSFER DATA"); &self.memory[ias usize]},
            // 0xFF02 => {println!("R: ACC SERIAL TRANSFER DATA FLGS"); &self.memory[i as usize]},
            VRAM_START..=VRAM_END => self.gpu[address - VRAM_START as u16],
            _ => self.memory[address as usize],
        }
    }
    fn write(&mut self, address: u16, value: u8) {
        match address as usize {
            0x0000..=0x0100 if self.in_bios == 0 => panic!(),
            0xff40 => self.gpu.lcdc = value,
            0xff41 => self.gpu.lcdstat = value,
            0xff42 => self.gpu.vscroll = value,
            0xff43 => self.gpu.hscroll = value,
            0xff44 => self.gpu.scanline = value,
            0xff47 => self.gpu.bg_palette = value,
            0xff4a => self.gpu.windowy = value,
            0xff4b => self.gpu.windowx = value,
            0xffff => self.int_enabled = value,
            0xff0f => {
                self.gpu.irq = (value & 0x01) != 0;
                self.int_flags = self.gpu.irq as u8;
            },
            0xff50 => self.in_bios = value,
            0xff80 => {
                if value == 255 {
                    println!("!WARN 255 to ff80")
                }
                self.memory[address as usize] = value;
            },
            0xff00 => {
                let select_buttons = value & 0b0010_0000 != 0;
                let select_directions = value & 0b0001_0000 != 0;
                if select_buttons {
                    self.joypad_io = Select::Buttons;
                } else if select_directions {
                    self.joypad_io = Select::Directions;
                }
            }
            0xff01 => {
                if(self.memory[0xff02] == 0x81) {
                    print!("{}",char::from(value));
                }
                self.memory[address as usize] = value;
            }
            // 0xff02 => {println!("r: acc serial transfer data flgs"); &self.memory[i as usize]},
            VRAM_START..=VRAM_END => self.gpu.vram[address as usize - VRAM_START] = value,
            _ => self.memory[address as usize] = value,
        }
    }
}