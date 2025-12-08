use crate::{
    gpu::{self, BackgroundPalette, GPU, LCDC},
    prelude::*,
    timer::{self, Timer},
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
        trace!("Wrote {:#02x} to {:#04x} at clock {}", value, addr, self.mclock);
    }
}

impl Memory for Bus {
    #[allow(clippy::match_same_arms)]
    fn read(&self, address: u16) -> u8 {
        match address {
            // Bootrom overlay
            0x0000..=0x00FF if self.in_bios == 0 => self.bootrom[address as usize],

            // Timer registers
            timer::addr::DIV => self.timer.read_div(),
            timer::addr::TAC => self.timer.read_tac(),
            timer::addr::TMA => self.timer.tma,
            timer::addr::TIMA => self.timer.tima,

            // GPU registers
            gpu::addr::LCDC_ADDR => self.gpu.lcdc.bits(),
            gpu::addr::STAT_ADDR => self.gpu.lcdstat,
            gpu::addr::SCY_ADDR => self.gpu.scrolly,
            gpu::addr::SCX_ADDR => self.gpu.scrollx,
            gpu::addr::LY_ADDR => self.gpu.scanline,
            gpu::addr::BGP_ADDR => self.gpu.bgrdpal,
            gpu::addr::WY_ADDR => self.gpu.windowy,
            gpu::addr::WX_ADDR => self.gpu.windowx,

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
            gpu::addr::VRAM_START_U16..=gpu::addr::VRAM_END_U16 => self.gpu[address],
            gpu::addr::OAM_START_U16..=gpu::addr::OAM_END_U16 => self.gpu.oam[address as usize - gpu::addr::OAM_START],

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
            timer::addr::DIV => self.timer.write_div(value),
            timer::addr::TAC => self.timer.write_tac(value),
            timer::addr::TIMA => self.timer.tima = value,
            timer::addr::TMA => self.timer.tma = value,

            // GPU registers
            gpu::addr::LCDC_ADDR => self.gpu.lcdc = LCDC::from_bits_retain(value),
            gpu::addr::STAT_ADDR => self.gpu.lcdstat = value,
            gpu::addr::SCY_ADDR => self.gpu.scrolly = value,
            gpu::addr::SCX_ADDR => self.gpu.scrollx = value,
            gpu::addr::LY_ADDR => self.gpu.scanline = value,
            gpu::addr::DMA_ADDR => {
                // OAM DMA Transfer
                let src_start = u16::from(value) << 8;
                for i in 0..0xA0 {
                    self.gpu.oam[i] = self.memory[(src_start + i as u16) as usize];
                    self.generic_cycle();
                }
            }
            gpu::addr::BGP_ADDR => {
                trace!("BGP Palette: {}", BackgroundPalette(value));
                self.gpu.bgrdpal = value;
            }
            gpu::addr::OBP0_ADDR => {
                trace!("OBP0 Palette: {}", BackgroundPalette(value));
                self.gpu.obj0pal = value;
            }
            gpu::addr::OBP1_ADDR => {
                trace!("OBP1 Palette: {}", BackgroundPalette(value));
                self.gpu.obj1pal = value;
            }
            gpu::addr::WY_ADDR => self.gpu.windowy = value,
            gpu::addr::WX_ADDR => self.gpu.windowx = value,

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
            gpu::addr::VRAM_START_U16..=gpu::addr::VRAM_END_U16 => {
                self.gpu.vram[address as usize - gpu::addr::VRAM_START] = value;
            }
            // OAM
            gpu::addr::OAM_START_U16..=gpu::addr::OAM_END_U16 => {
                self.gpu.oam[address as usize - gpu::addr::OAM_START] = value;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Bus {
        let mut bus = Bus::new(&[], None);
        bus.in_bios = 1; // Skip bootrom
        bus
    }

    fn setup_with_rom(rom: &[u8]) -> Bus {
        let mut bus = Bus::new(rom, None);
        bus.in_bios = 1;
        bus
    }

    // Memory region tests
    #[test]
    fn read_rom_returns_loaded_data() {
        let rom = vec![0x00, 0x31, 0xFE, 0xFF]; // NOP, LD SP,$FFFE
        let bus = setup_with_rom(&rom);
        assert_eq!(bus.read(0x0000), 0x00);
        assert_eq!(bus.read(0x0001), 0x31);
        assert_eq!(bus.read(0x0002), 0xFE);
        assert_eq!(bus.read(0x0003), 0xFF);
    }

    #[test]
    fn write_to_wram_persists() {
        let mut bus = setup();
        bus.write(0xC000, 0x42);
        assert_eq!(bus.read(0xC000), 0x42);

        bus.write(0xCFFF, 0xAB);
        assert_eq!(bus.read(0xCFFF), 0xAB);
    }

    #[test]
    fn write_to_hram_persists() {
        let mut bus = setup();
        bus.write(0xFF80, 0x12);
        assert_eq!(bus.read(0xFF80), 0x12);

        bus.write(0xFFFE, 0x34);
        assert_eq!(bus.read(0xFFFE), 0x34);
    }

    #[test]
    fn vram_access_routes_to_gpu() {
        let mut bus = setup();
        bus.write(0x8000, 0xAA);
        assert_eq!(bus.gpu.vram[0], 0xAA);
        assert_eq!(bus.read(0x8000), 0xAA);

        bus.write(0x9FFF, 0xBB);
        assert_eq!(bus.gpu.vram[0x1FFF], 0xBB);
        assert_eq!(bus.read(0x9FFF), 0xBB);
    }

    #[test]
    fn oam_access_routes_to_gpu() {
        let mut bus = setup();
        bus.write(0xFE00, 0xCC);
        assert_eq!(bus.gpu.oam[0], 0xCC);
        assert_eq!(bus.read(0xFE00), 0xCC);

        bus.write(0xFE9F, 0xDD);
        assert_eq!(bus.gpu.oam[0x9F], 0xDD);
        assert_eq!(bus.read(0xFE9F), 0xDD);
    }

    // Timer register tests
    #[test]
    fn timer_div_read_returns_upper_bits() {
        let mut bus = setup();
        bus.timer.internal = 0x1234;
        assert_eq!(bus.read(timer_addr::DIV), 0x12);
    }

    #[test]
    fn timer_div_write_resets_internal() {
        let mut bus = setup();
        bus.timer.internal = 0xFFFF;
        bus.write(timer_addr::DIV, 0x42); // Any value resets
        assert_eq!(bus.timer.internal, 0);
    }

    #[test]
    fn timer_tima_read_write() {
        let mut bus = setup();
        bus.write(timer_addr::TIMA, 0xAB);
        assert_eq!(bus.timer.tima, 0xAB);
        assert_eq!(bus.read(timer_addr::TIMA), 0xAB);
    }

    #[test]
    fn timer_tma_read_write() {
        let mut bus = setup();
        bus.write(timer_addr::TMA, 0xCD);
        assert_eq!(bus.timer.tma, 0xCD);
        assert_eq!(bus.read(timer_addr::TMA), 0xCD);
    }

    #[test]
    fn timer_tac_read_has_upper_bits_set() {
        let mut bus = setup();
        bus.write(timer_addr::TAC, 0b101);
        // Upper 5 bits read as 1
        assert_eq!(bus.read(timer_addr::TAC), 0b1111_1101);
    }

    // Interrupt register tests
    #[test]
    fn interrupt_enable_read_write() {
        let mut bus = setup();
        bus.write(addr::IE, 0x1F);
        assert_eq!(bus.int_enabled, 0x1F);
        assert_eq!(bus.read(addr::IE), 0x1F);
    }

    #[test]
    fn interrupt_flags_read_write() {
        let mut bus = setup();
        bus.write(addr::IF, 0x0F);
        assert_eq!(bus.int_flags, 0x0F);
        assert_eq!(bus.read(addr::IF), 0x0F);
    }

    #[test]
    fn enable_interrupts_sets_ime() {
        let mut bus = setup();
        assert_eq!(bus.ime, 0);
        bus.enable_interrupts();
        assert_eq!(bus.ime, 1);
    }

    #[test]
    fn disable_interrupts_clears_ime() {
        let mut bus = setup();
        bus.ime = 1;
        bus.disable_interrupts();
        assert_eq!(bus.ime, 0);
    }

    #[test]
    fn ack_interrupt_clears_flag() {
        let mut bus = setup();
        bus.int_flags = 0b0001_1111;
        bus.ack_interrupt(0b0000_0100); // Clear timer flag
        assert_eq!(bus.int_flags, 0b0001_1011);
    }

    // Joypad tests
    #[test]
    fn joypad_button_select() {
        let mut bus = setup();
        bus.keypresses = 0xAB;
        bus.directions = 0xCD;

        bus.write(addr::JOYPAD, 0x10); // Select buttons
        assert!(matches!(bus.select, Select::Buttons));
        assert_eq!(bus.read(addr::JOYPAD), 0xAB);

        bus.write(addr::JOYPAD, 0x20); // Select directions
        assert!(matches!(bus.select, Select::Directions));
        assert_eq!(bus.read(addr::JOYPAD), 0xCD);
    }

    #[test]
    fn joypad_no_select_returns_ff() {
        let mut bus = setup();
        bus.keypresses = 0x00;
        bus.directions = 0x00;
        bus.write(addr::JOYPAD, 0x00); // Neither selected
        assert!(matches!(bus.select, Select::None));
        assert_eq!(bus.read(addr::JOYPAD), 0xFF);
    }

    // Bootrom tests
    #[test]
    fn bootrom_disable_sets_flag() {
        let mut bus = setup();
        bus.in_bios = 0;
        bus.rom_start_signal = false;

        bus.write(addr::BOOTROM_DISABLE, 0x01);
        assert_eq!(bus.in_bios, 0x01);
        assert!(bus.rom_start_signal);
    }

    #[test]
    fn bootrom_overlay_when_active() {
        let rom = vec![0xAA; 0x100];
        let mut bus = Bus::new(&rom, None);
        // Fill bootrom with different data
        for i in 0..0x100 {
            bus.bootrom[i] = i as u8;
        }
        bus.in_bios = 0; // Bootrom active

        // Should read from bootrom, not ROM
        assert_eq!(bus.read(0x00), 0x00);
        assert_eq!(bus.read(0x50), 0x50);
        assert_eq!(bus.read(0xFF), 0xFF);
    }

    #[test]
    fn rom_visible_after_bootrom_disabled() {
        let mut rom = vec![0x00; 0x8000];
        rom[0x00] = 0x31;
        rom[0x50] = 0xAB;
        let mut bus = Bus::new(&rom, None);
        bus.in_bios = 1; // Bootrom disabled

        assert_eq!(bus.read(0x00), 0x31);
        assert_eq!(bus.read(0x50), 0xAB);
    }

    // GPU register tests
    #[test]
    fn gpu_lcdc_read_write() {
        let mut bus = setup();
        bus.write(gpu::LCDC_ADDR, 0x91);
        assert_eq!(bus.gpu.lcdc.bits(), 0x91);
        assert_eq!(bus.read(gpu::LCDC_ADDR), 0x91);
    }

    #[test]
    fn gpu_scroll_registers() {
        let mut bus = setup();
        bus.write(gpu::SCY_ADDR, 0x12);
        bus.write(gpu::SCX_ADDR, 0x34);
        assert_eq!(bus.gpu.scrolly, 0x12);
        assert_eq!(bus.gpu.scrollx, 0x34);
        assert_eq!(bus.read(gpu::SCY_ADDR), 0x12);
        assert_eq!(bus.read(gpu::SCX_ADDR), 0x34);
    }

    #[test]
    fn gpu_scanline_register() {
        let mut bus = setup();
        bus.write(gpu::LY_ADDR, 0x90);
        assert_eq!(bus.gpu.scanline, 0x90);
        assert_eq!(bus.read(gpu::LY_ADDR), 0x90);
    }

    #[test]
    fn gpu_palette_register() {
        let mut bus = setup();
        bus.write(gpu::BGP_ADDR, 0xE4);
        assert_eq!(bus.gpu.bgrdpal, 0xE4);
        assert_eq!(bus.read(gpu::BGP_ADDR), 0xE4);
    }

    #[test]
    fn gpu_window_registers() {
        let mut bus = setup();
        bus.write(gpu::WY_ADDR, 0x10);
        bus.write(gpu::WX_ADDR, 0x07);
        assert_eq!(bus.gpu.windowy, 0x10);
        assert_eq!(bus.gpu.windowx, 0x07);
        assert_eq!(bus.read(gpu::WY_ADDR), 0x10);
        assert_eq!(bus.read(gpu::WX_ADDR), 0x07);
    }

    // Cycle timing tests
    #[test]
    fn generic_cycle_advances_clock() {
        let mut bus = setup();
        assert_eq!(bus.mclock(), 0);
        bus.generic_cycle();
        assert_eq!(bus.mclock(), 1);
        bus.generic_cycle();
        assert_eq!(bus.mclock(), 2);
    }

    #[test]
    fn read_cycle_advances_clock_and_returns_value() {
        let mut bus = setup();
        bus.memory[0xC000] = 0x42;
        let value = bus.read_cycle(0xC000);
        assert_eq!(value, 0x42);
        assert_eq!(bus.mclock(), 1);
    }

    #[test]
    fn write_cycle_advances_clock_and_writes_value() {
        let mut bus = setup();
        bus.write_cycle(0xC000, 0xAB);
        assert_eq!(bus.memory[0xC000], 0xAB);
        assert_eq!(bus.mclock(), 1);
    }

    #[test]
    fn read_cycle_high_reads_from_ff_page() {
        let mut bus = setup();
        bus.memory[0xFF80] = 0x99;
        let value = bus.read_cycle_high(0x80);
        assert_eq!(value, 0x99);
        assert_eq!(bus.mclock(), 1);
    }

    // Serial I/O tests
    #[test]
    fn serial_data_read_write() {
        let mut bus = setup();
        bus.write(addr::SERIAL_DATA, 0x41); // 'A'
        assert_eq!(bus.memory[addr::SERIAL_DATA as usize], 0x41);
    }

    #[test]
    fn serial_ctrl_triggers_output() {
        let mut bus = setup();
        bus.memory[addr::SERIAL_DATA as usize] = 0x48; // 'H'
        bus.write(addr::SERIAL_CTRL, 0x81);
        assert_eq!(bus.io, "H");
    }

    // Memory trait conformance
    #[test]
    fn memory_trait_read_write_roundtrip() {
        let mut bus = setup();
        let addr = 0xC100;
        bus.write(addr, 0xDE);
        assert_eq!(bus.read(addr), 0xDE);
    }

    // Edge cases
    #[test]
    fn write_to_bootrom_area_when_active_is_ignored() {
        let mut bus = setup();
        bus.in_bios = 0; // Bootrom active
        bus.bootrom[0x50] = 0xAA;
        bus.write(0x0050, 0xBB);
        assert_eq!(bus.bootrom[0x50], 0xAA); // Unchanged
    }

    #[test]
    fn echo_ram_works() {
        let mut bus = setup();
        bus.write(0xE000, 0x12);
        assert_eq!(bus.read(0xE000), 0x12);
    }

    #[test]
    fn display_format_includes_key_info() {
        let bus = setup();
        let display = format!("{bus}");
        assert!(display.contains("CLK:"));
        assert!(display.contains("IE:"));
        assert!(display.contains("IF:"));
        assert!(display.contains("[TIMER]"));
    }
}
