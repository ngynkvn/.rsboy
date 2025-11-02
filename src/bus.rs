use tracing::info;

use crate::{
    gpu::{GPU, OAM_END, OAM_START, VRAM_END, VRAM_START},
    timer,
    timer::Timer,
};
use std::{fmt::Display, fs::File, io::Read, path::PathBuf};

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
    pub memory: Box<[u8]>,
    pub bootrom: Box<[u8]>,
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
        let Self {
            clock,
            int_enabled,
            int_flags,
            timer,
            keypresses,
            directions,
            ..
        } = self;
        f.write_fmt(format_args!(
            r"CLK: {clock}, IE: {int_enabled}, IF: {int_flags:08b}
[TIMER]: {timer}
[BTNS]: {keypresses:08b}
[ARWS]: {directions:08b}",
        ))
    }
}

impl Bus {
    /// # Panics
    /// If the bootrom file cannot be read.
    #[must_use]
    pub fn new(rom_vec: &[u8], bootrom_path: Option<PathBuf>) -> Self {
        let memory = vec![0; 0x10000].into_boxed_slice();
        let bootrom = vec![0; 0x100].into_boxed_slice();

        let mut bus = Self {
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

        let file = bootrom_path
            .or_else(|| Some(PathBuf::from("dmg_boot.bin")))
            .map(File::open)
            .transpose()
            .expect("Couldn't open bootrom file.");
        if let Some(mut file) = file {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .expect("Couldn't read the file.");
            bus.bootrom[..].clone_from_slice(&buffer[..]);
        } else {
            bus.in_bios = 1;
            bus.rom_start_signal = true;
            info!("No bootrom provided.");
        }
        bus.memory[..rom_vec.len()].clone_from_slice(rom_vec);

        bus
    }

    pub const fn enable_interrupts(&mut self) {
        self.ime = 1;
    }

    pub const fn disable_interrupts(&mut self) {
        self.ime = 0;
    }

    pub const fn ack_interrupt(&mut self, flag: u8) {
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
        self.read(0xFF00 | u16::from(addr))
    }

    pub fn write_cycle(&mut self, addr: u16, value: u8) {
        self.generic_cycle();
        self.write(addr, value);
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
            // 0xFF01 => {info!("R: ACC SERIAL TRANSFER DATA"); &self.memory[ias usize]},
            // 0xFF02 => {info!("R: ACC SERIAL TRANSFER DATA FLGS"); &self.memory[i as usize]},
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
                let value = u16::from(value);
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
                self.in_bios = value;
            }
            0xff00 => {
                self.select = match value & 0xF0 {
                    0b0001_0000 => Select::Buttons,
                    0b0010_0000 => Select::Directions,
                    // 0b0011_0000 => Select::None,
                    _ => Select::None,
                }
            }
            0xff01 | 0xff80 => {
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
                    self.memory[address as usize] = value;
                }
            }
        }
    }
}
