use crate::gpu::GPU;
use crate::gpu::OAM_END;
use crate::gpu::OAM_START;
use crate::gpu::VRAM_END;
use crate::gpu::VRAM_START;
use crate::timer;
use crate::timer::Timer;
use std::io::Cursor;
use std::path::PathBuf;
use std::{fmt::Display, fs::File};

pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub enum Select {
    Buttons,
    Directions,
    None,
}

// Global emu struct.
pub struct Bus {
    pub memory: [u8; 0x10000],
    pub bootrom: [u8; 0x100],
    pub in_bios: u8,
    pub int_enabled: u8,
    pub int_flags: u8,
    pub clock: usize,
    pub ime: u8,
    pub select: Select,
    pub directions: u8,
    pub keypresses: u8,
    pub gpu: GPU,
    pub rom_start_signal: bool,
    pub timer: Timer,
    pub io: String,
}

impl Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            r#"CLK: {}, IE: {}, IF: {:08b}
[TIMER]: {}
[BTNS]: {:08b}
[ARWS]: {:08b}"#,
            self.clock,
            self.int_enabled,
            self.int_flags,
            self.timer,
            self.keypresses,
            self.directions,
        ))
    }
}

impl Bus {
    pub fn new(rom_vec: Vec<u8>, bootrom_path: PathBuf) -> Self {
        let memory = [0; 0x10000];
        let bootrom = [0; 0x100];

        let mut bus = Bus {
            memory,
            bootrom,
            in_bios: 0,
            int_enabled: 0,
            int_flags: 0,
            clock: 0,
            ime: 0,
            select: Select::Buttons,
            directions: 0,
            keypresses: 0,
            gpu: GPU::new(),
            rom_start_signal: false,
            timer: Timer::new(),
            io: String::new(),
        };

        if let Ok(mut file) = File::open(bootrom_path) {
            std::io::copy(&mut file, &mut Cursor::new(&mut bus.bootrom[..])).unwrap();
        } else {
            bus.in_bios = 1;
            bus.rom_start_signal = true;
            eprintln!("No bootrom provided.");
        }
        std::io::copy(
            &mut Cursor::new(&rom_vec[..]),
            &mut Cursor::new(&mut bus.memory[..]),
        )
        .unwrap();

        bus
    }

    pub fn from_bytes(mut rom: &[u8], mut bootrom_slice: &[u8]) -> Self {
        let mut memory = [0; 0x10000];
        let mut bootrom = [0; 0x100];
        std::io::copy(&mut rom, &mut Cursor::new(&mut memory[..])).unwrap();
        std::io::copy(&mut bootrom_slice, &mut Cursor::new(&mut bootrom[..])).unwrap();

        Bus {
            memory,
            bootrom,
            in_bios: 0,
            int_enabled: 0,
            int_flags: 0,
            clock: 0,
            ime: 0,
            select: Select::Buttons,
            directions: 0,
            keypresses: 0,
            gpu: GPU::new(),
            rom_start_signal: false,
            timer: Timer::new(),
            io: String::new(),
        }
    }

    pub fn enable_interrupts(&mut self) {
        self.ime = 1;
    }

    pub fn disable_interrupts(&mut self) {
        self.ime = 0;
    }

    pub fn ack_interrupt(&mut self, flag: u8) {
        self.ime = 0;
        self.int_flags &= !flag;
    }

    // Cycle refers to 1 T-cycle
    pub fn generic_cycle(&mut self) {
        self.clock += 1;
        self.gpu.cycle(&mut self.int_flags);
        self.timer.tick_timer_counter(&mut self.int_flags);
    }

    pub fn read_cycle(&mut self, addr: u16) -> u8 {
        self.generic_cycle();
        self.read(addr)
    }

    pub fn read_cycle_high(&mut self, addr: u8) -> u8 {
        self.generic_cycle();
        self.read(0xFF00 | (addr as u16))
    }

    pub fn write_cycle(&mut self, addr: u16, value: u8) {
        self.generic_cycle();
        self.write(addr, value)
    }
}

impl Memory for Bus {
    fn read(&self, address: u16) -> u8 {
        match address as usize {
            0x0000..=0x0100 if self.in_bios == 0 => self.bootrom[address as usize],
            timer::DIV => self.timer.div(),
            timer::TAC => self.timer.tac,
            timer::TMA => self.timer.tma,
            timer::TIMA => self.timer.tima,
            0xFF40 => self.gpu.lcdc,
            0xFF41 => self.gpu.lcdstat,
            0xFF42 => self.gpu.scrolly,
            0xFF43 => self.gpu.scrollx,
            0xFF44 => self.gpu.scanline,
            0xFF47 => panic!("0xFF47 (bg_palette) is WRITE ONLY"),
            0xFF4A => self.gpu.windowy,
            0xFF4B => self.gpu.windowx,
            0xffff => self.int_enabled,
            0xff0f => self.int_flags,
            0xff00 => match self.select {
                Select::Buttons => self.keypresses,
                Select::Directions => self.directions,
                Select::None => 0xFF,
            },
            // 0xFFFF => &self.gpu.,
            // 0xFF01 => {println!("R: ACC SERIAL TRANSFER DATA"); &self.memory[ias usize]},
            // 0xFF02 => {println!("R: ACC SERIAL TRANSFER DATA FLGS"); &self.memory[i as usize]},
            VRAM_START..=VRAM_END => self.gpu[address],
            OAM_START..=OAM_END => self.gpu.oam[address as usize - OAM_START],
            _ => self.memory[address as usize],
        }
    }
    fn write(&mut self, address: u16, value: u8) {
        match address as usize {
            0x0000..=0x0100 if self.in_bios == 0 => panic!(),
            timer::DIV => self.timer.update_internal(&mut self.int_flags, 0),
            timer::TAC => self.timer.tac = 0b1111_1000 | value,
            timer::TIMA => self.timer.tima = value,
            timer::TMA => self.timer.tma = value,
            0xff40 => self.gpu.lcdc = value,
            0xff41 => self.gpu.lcdstat = value,
            0xff42 => self.gpu.scrolly = value,
            0xff43 => self.gpu.scrollx = value,
            0xff44 => self.gpu.scanline = value,
            0xff46 => {
                //OAM Transfer request
                let value = value as u16;
                if value <= 0xF1 {
                    let range = ((value << 8) as usize)..=((value << 8) as usize | 0xFF);
                    self.gpu.oam.copy_from_slice(&self.memory[range]);
                    self.memory[address as usize] = value as u8;
                }
            }
            0xff47 => self.gpu.bgrdpal = value,
            0xff48 => self.gpu.obj0pal = value,
            0xff49 => self.gpu.obj1pal = value,
            0xff4a => self.gpu.windowy = value,
            0xff4b => self.gpu.windowx = value,
            0xffff => self.int_enabled = value,
            0xff0f => {
                self.int_flags |= value;
            }
            0xff50 => {
                if value != 0 && !self.rom_start_signal {
                    self.rom_start_signal = true;
                }
                self.in_bios = value
            }
            0xff80 => {
                self.memory[address as usize] = value;
            }
            0xff00 => {
                self.select = match value & 0xF0 {
                    0b0001_0000 => Select::Buttons,
                    0b0010_0000 => Select::Directions,
                    0b0011_0000 => Select::None,
                    _ => Select::None,
                }
            }
            0xff01 => {
                self.memory[address as usize] = value;
            }
            0xff02 => {
                if value == 0x81 {
                    self.io.push(char::from(self.memory[0xff01]));
                }
                self.memory[address as usize] = value;
            }
            VRAM_START..=VRAM_END => self.gpu.vram[address as usize - VRAM_START] = value,
            OAM_START..=OAM_END => self.gpu.oam[address as usize - OAM_START] = value,
            _ => {
                if address >= 0x8000 {
                    self.memory[address as usize] = value
                }
            }
        }
    }
}
