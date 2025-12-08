use crate::{
    gpu::{self, BackgroundPalette, GPU, LCDC, OAM_START, VRAM_START},
    prelude::*,
    timer::{Timer, addr as timer_addr},
};
use std::{fmt::Display, fs::File, io::Read, path::PathBuf};

/// Memory map constants
pub mod addr {
    /// Joypad input register
    pub const JOYPAD: u16 = 0xFF00;
    /// Serial transfer data
    pub const SERIAL_DATA: u16 = 0xFF01;
    /// Serial transfer control
    pub const SERIAL_CTRL: u16 = 0xFF02;
    /// Bootrom disable register
    pub const BOOTROM_DISABLE: u16 = 0xFF50;
    /// High RAM start
    pub const HRAM_START: u16 = 0xFF80;
    /// Interrupt flags
    pub const IF: u16 = 0xFF0F;
    /// Interrupt enable
    pub const IE: u16 = 0xFFFF;
}

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
    mclock: usize, // CPU clock M-cycles
    pub ime: u8,
    pub select: Select,
    pub directions: u8,
    pub keypresses: u8,
    pub gpu: GPU,
    pub rom_start_signal: bool,
    pub timer: Timer,
    pub io: String,
}

impl Bus {
    pub const fn mclock(&self) -> usize {
        self.mclock
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            mclock: clock,
            int_enabled,
            int_flags,
            timer,
            keypresses,
            directions,
            ..
        } = self;
        f.write_fmt(format_args!(
            r"CLK: {clock}, IE: {int_enabled:08b}, IF: {int_flags:08b}
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
            mclock: 0,
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
            file.read_to_end(&mut buffer).expect("Couldn't read the file.");
            bus.bootrom[..].clone_from_slice(&buffer[..]);
        } else {
            bus.in_bios = 1;
            bus.rom_start_signal = true;
            info!("No bootrom provided.");
        }
        bus.memory[..rom_vec.len()].clone_from_slice(rom_vec);

        bus
    }

    pub fn enable_interrupts(&mut self) {
        trace!("Enabling interrupts at clock {}", self.mclock);
        self.ime = 1;
    }

    pub fn disable_interrupts(&mut self) {
        trace!("Disabling interrupts at clock {}", self.mclock);
        self.ime = 0;
    }

    pub const fn ack_interrupt(&mut self, flag: u8) {
        self.int_flags &= !flag;
    }

    /// Advance emulation by 1 M-cycle (4 T-cycles)
    #[instrument(ret, skip(self), fields(clock = self.mclock, to = self.mclock + 1))]
    pub fn generic_cycle(&mut self) {
        self.mclock += 1;
        self.gpu.cycle(&mut self.int_flags);
        self.timer.tick(&mut self.int_flags);
    }

    #[instrument(ret, skip(self), fields(clock = self.mclock))]
    pub fn read_cycle(&mut self, addr: u16) -> u8 {
        self.generic_cycle();
        self.read(addr)
    }

    #[instrument(ret, skip(self), fields(clock = self.mclock))]
    pub fn read_cycle_high(&mut self, addr: u8) -> u8 {
        self.generic_cycle();
        self.read(0xFF00 | u16::from(addr))
    }

    #[instrument(ret, skip(self), fields(clock = self.mclock))]
    pub fn write_cycle(&mut self, addr: u16, value: u8) {
        self.generic_cycle();
        self.write(addr, value);
        info!("Wrote {:#02x} to {:#04x} at clock {}", value, addr, self.mclock);
    }
}

impl Memory for Bus {
    #[allow(clippy::match_same_arms)]
    fn read(&self, address: u16) -> u8 {
        match address {
            // Bootrom overlay
            0x0000..=0x00FF if self.in_bios == 0 => self.bootrom[address as usize],

            // Timer registers
            timer_addr::DIV => self.timer.read_div(),
            timer_addr::TAC => self.timer.read_tac(),
            timer_addr::TMA => self.timer.tma,
            timer_addr::TIMA => self.timer.tima,

            // GPU registers
            gpu::LCDC_ADDR => self.gpu.lcdc.bits(),
            gpu::STAT_ADDR => self.gpu.lcdstat,
            gpu::SCY_ADDR => self.gpu.scrolly,
            gpu::SCX_ADDR => self.gpu.scrollx,
            gpu::LY_ADDR => self.gpu.scanline,
            gpu::BGP_ADDR => self.gpu.bgrdpal,
            gpu::WY_ADDR => self.gpu.windowy,
            gpu::WX_ADDR => self.gpu.windowx,

            // Interrupt registers
            addr::IE => self.int_enabled,
            addr::IF => self.int_flags,

            // Joypad
            addr::JOYPAD => match self.select {
                Select::Buttons => self.keypresses,
                Select::Directions => self.directions,
                Select::None => 0xFF,
            },

            // VRAM and OAM
            gpu::VRAM_START_U16..=gpu::VRAM_END_U16 => self.gpu[address],
            gpu::OAM_START_U16..=gpu::OAM_END_U16 => self.gpu.oam[address as usize - OAM_START],

            // Everything else from main memory
            _ => self.memory[address as usize],
        }
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, address: u16, value: u8) {
        match address {
            // Bootrom area is read-only when bootrom is active
            0x0000..=0x00FF if self.in_bios == 0 => {
                warn!("Attempted write to bootrom area: {address:#06x}");
            }

            // Timer registers
            timer_addr::DIV => self.timer.write_div(value),
            timer_addr::TAC => self.timer.write_tac(value),
            timer_addr::TIMA => self.timer.tima = value,
            timer_addr::TMA => self.timer.tma = value,

            // GPU registers
            gpu::LCDC_ADDR => self.gpu.lcdc = LCDC::from_bits_retain(value),
            gpu::STAT_ADDR => self.gpu.lcdstat = value,
            gpu::SCY_ADDR => self.gpu.scrolly = value,
            gpu::SCX_ADDR => self.gpu.scrollx = value,
            gpu::LY_ADDR => self.gpu.scanline = value,
            gpu::DMA_ADDR => {
                // OAM DMA Transfer
                let src_start = u16::from(value) << 8;
                for i in 0..0xA0 {
                    self.gpu.oam[i] = self.memory[(src_start + i as u16) as usize];
                    self.generic_cycle();
                }
            }
            gpu::BGP_ADDR => {
                trace!("BGP Palette: {}", BackgroundPalette(value));
                self.gpu.bgrdpal = value;
            }
            gpu::OBP0_ADDR => {
                trace!("OBP0 Palette: {}", BackgroundPalette(value));
                self.gpu.obj0pal = value;
            }
            gpu::OBP1_ADDR => {
                trace!("OBP1 Palette: {}", BackgroundPalette(value));
                self.gpu.obj1pal = value;
            }
            gpu::WY_ADDR => self.gpu.windowy = value,
            gpu::WX_ADDR => self.gpu.windowx = value,

            // Interrupt registers
            addr::IE => self.int_enabled = value,
            addr::IF => self.int_flags = value,

            // Bootrom disable
            addr::BOOTROM_DISABLE => {
                if value != 0 && !self.rom_start_signal {
                    self.rom_start_signal = true;
                }
                self.in_bios = value;
            }

            // Joypad
            addr::JOYPAD => {
                self.select = match value & 0x30 {
                    0x10 => Select::Buttons,
                    0x20 => Select::Directions,
                    _ => Select::None,
                };
            }

            // Serial I/O
            addr::SERIAL_DATA | addr::HRAM_START => {
                self.memory[address as usize] = value;
            }
            addr::SERIAL_CTRL => {
                if value == 0x81 {
                    let ch = char::from(self.memory[addr::SERIAL_DATA as usize]);
                    self.io.push(ch);
                    print!("{ch}");
                }
                self.memory[address as usize] = value;
            }

            // VRAM
            gpu::VRAM_START_U16..=gpu::VRAM_END_U16 => {
                self.gpu.vram[address as usize - VRAM_START] = value;
            }
            // OAM
            gpu::OAM_START_U16..=gpu::OAM_END_U16 => {
                self.gpu.oam[address as usize - OAM_START] = value;
            }

            // RAM (WRAM, Echo RAM, HRAM) - everything else above 0x9FFF except OAM
            0xA000..=0xFDFF | 0xFEA0..=0xFF7F | 0xFF81..=0xFFFE => {
                self.memory[address as usize] = value;
            }

            // ROM area writes (mapper control - ignored for now) and unused areas
            _ => {}
        }
    }
}
