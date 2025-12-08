use bitflags::bitflags;

use crate::{
    constants::{WINDOW_HEIGHT, WINDOW_WIDTH},
    cpu,
    prelude::*,
};
use std::{
    fmt::Display,
    ops::{Index, Range, RangeInclusive},
};

pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const OAM_START: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;
pub const TILE_DATA_RANGE: Range<usize> = 0..0x1800;
pub const MAP_DATA_RANGE: Range<usize> = 0x1800..0x1C00;
pub const TILE_SIZE: usize = 16;
pub const DRAM_ADDR: usize = 0xFF46;

#[derive(Debug)]
enum GpuMode {
    HBlank, // 0
    VBlank, // 1
    Oam,    // 2
    Vram,   // 3
}
#[derive(Debug)]
enum SpriteSize {
    Square,
    Tall,
}

// Global GPU struct.
// Holds I/O Registers relevant to GPU. Make sure these are available from bus struct.
pub struct GPU {
    mode: GpuMode,
    clock: usize,
    pub scanline: u8,
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0x100],
    pub lcdc: LCDC,
    pub lcdstat: u8,
    pub scrollx: u8,
    pub scrolly: u8,
    pub bgrdpal: u8, //Background Palette
    pub obj0pal: u8, //Object0 Palette
    pub obj1pal: u8, //Object1 Palette
    pub windowx: u8, //
    pub windowy: u8, //
    pub vblank_count: usize,
    pub framebuffer: PixelData,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LCDC: u8 {
        const BG_DISPLAY_ENABLED = 1 << 0;
        const OBJ_ENABLED = 1 << 1;
        const OBJ_SIZE = 1 << 2;
        const BG_TILE_MAP_DISPLAY = 1 << 3;
        const BG_AND_WINDOW_TILES = 1 << 4;
        const WINDOW_DISPLAY_ENABLED = 1 << 5;
        const WINDOW_TILE_MAP = 1 << 6;
        const LCD_PPU_ENABLE = 1 << 7;
    }
}

pub struct BackgroundPalette(pub u8);
impl Display for BackgroundPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#08b}\n", self.0))?;
        for i in 0..4 {
            let color = (self.0 >> (6 - i * 2)) & 0b11;
            match color {
                0b00 => f.write_str("(White)"),
                0b01 => f.write_str("(Light Gray)"),
                0b10 => f.write_str("(Dark Gray)"),
                0b11 => f.write_str("(Black)"),
                _ => unreachable!(),
            }?;
        }
        Ok(())
    }
}

const END_HBLANK: u8 = 144;
const END_VBLANK: u8 = 154;

pub type PixelData = ndarray::Array2<u32>;
pub type PixelMap = [u8; 256 * 256 * 4];

#[allow(dead_code, clippy::struct_excessive_bools)]
struct SpriteAttribute {
    above: bool,
    yflip: bool,
    xflip: bool,
    obj0: bool, //True for OBJ0, OBJ1 otherwise.
}
impl From<&u8> for SpriteAttribute {
    fn from(byte: &u8) -> Self {
        Self {
            above: byte & 0x80 != 0,
            yflip: byte & 0x40 != 0,
            xflip: byte & 0x20 != 0,
            obj0: byte & 0x10 == 0,
        }
    }
}

impl Default for GPU {
    fn default() -> Self {
        Self::new()
    }
}

impl GPU {
    pub fn new() -> Self {
        Self {
            mode: GpuMode::Oam,
            clock: 0,
            scanline: 0,
            lcdc: LCDC::empty(),
            lcdstat: 0,
            scrolly: 0,
            scrollx: 0,
            bgrdpal: 0,
            obj0pal: 0,
            obj1pal: 0,
            windowx: 0,
            windowy: 0,
            // FFxx Values end
            vblank_count: 0,
            vram: [0; 0x2000],
            oam: [0; 0x100],
            framebuffer: PixelData::zeros((WINDOW_HEIGHT as usize, WINDOW_WIDTH as usize)),
        }
    }
    //   Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    const fn window_tile_map_display_select(&self) -> RangeInclusive<usize> {
        if self.lcdc.contains(LCDC::WINDOW_TILE_MAP) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    //   Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    const fn bg_and_window_tile_data_select(&self) -> RangeInclusive<usize> {
        if self.lcdc.contains(LCDC::BG_AND_WINDOW_TILES) {
            0x0000..=0x0FFF
        } else {
            0x0800..=0x17FF
        }
    }

    /// Returns the VRAM range for tile data based on tile index.
    /// When LCDC bit 4 is set: unsigned addressing from 0x8000 (tiles 0-255)
    /// When LCDC bit 4 is clear: signed addressing from 0x9000 (tiles -128 to 127)
    const fn tile_data_address(&self, tile_index: u8) -> Range<usize> {
        if self.lcdc.contains(LCDC::BG_AND_WINDOW_TILES) {
            // Unsigned: tile_index 0-255 maps to 0x8000-0x8FFF (VRAM 0x0000-0x0FFF)
            let start = tile_index as usize * 16;
            start..start + 16
        } else {
            // Signed: tile_index as i8, base at 0x9000 (VRAM 0x1000)
            let offset = tile_index as i8 as isize;
            #[allow(clippy::cast_sign_loss)]
            let start = (0x1000_isize + offset * 16) as usize;
            start..start + 16
        }
    }
    //   Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    const fn bg_tile_map_display_select(&self) -> RangeInclusive<usize> {
        if self.lcdc.contains(LCDC::BG_TILE_MAP_DISPLAY) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    //   Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    const fn sprite_size(&self) -> SpriteSize {
        if self.lcdc.contains(LCDC::OBJ_SIZE) {
            SpriteSize::Square
        } else {
            SpriteSize::Tall
        }
    }
    //   Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)

    pub fn print_sprite_table(&self) {
        for i in self.oam.chunks_exact(4) {
            info!("{i:?}");
        }
    }

    pub fn cycle(&mut self, flag: &mut u8) {
        if !self.lcdc.contains(LCDC::LCD_PPU_ENABLE) {
            return;
        }
        self.clock += 1;
        self.step(flag);
    }

    /// Returns the VRAM offset for the background tile map (0x1800 or 0x1C00)
    const fn bg_tile_map_base(&self) -> usize {
        if self.lcdc.contains(LCDC::BG_TILE_MAP_DISPLAY) { 0x1C00 } else { 0x1800 }
    }

    /// Decode a single pixel from tile data.
    /// `tile_data` should be 16 bytes (8 rows × 2 bytes per row).
    /// `row` is 0-7, `col` is 0-7.
    #[inline]
    fn decode_pixel(palette: u8, tile_data: &[u8], row: usize, col: usize) -> u32 {
        let lo_byte = tile_data[row * 2];
        let hi_byte = tile_data[row * 2 + 1];
        let bit = 7 - col;
        let lo_bit = (lo_byte >> bit) & 1;
        let hi_bit = (hi_byte >> bit) & 1;
        let color_index = (hi_bit << 1) | lo_bit;
        let color = (palette >> (color_index << 1)) & 0b11;
        match color {
            0b00 => 0xE0F8_D0FF, // White
            0b01 => 0x88C0_70FF, // Light Gray
            0b10 => 0x3468_56FF, // Dark Gray
            0b11 => 0x0818_20FF, // Black
            _ => 0,
        }
    }

    fn draw_line(&mut self) {
        let ly = self.scanline;

        // Draw background
        if self.lcdc.contains(LCDC::BG_DISPLAY_ENABLED) {
            let tile_map_base = self.bg_tile_map_base();
            let ypos = self.scrolly.wrapping_add(ly) as usize;
            let tile_row = ypos / 8;
            let pixel_row = ypos % 8;

            for screen_x in 0..WINDOW_WIDTH as usize {
                let xpos = (self.scrollx as usize + screen_x) & 0xFF;
                let tile_col = xpos / 8;
                let pixel_col = xpos % 8;

                // Get tile index from tile map
                let tile_map_addr = tile_map_base + (tile_row & 31) * 32 + (tile_col & 31);
                let tile_index = self.vram[tile_map_addr];

                // Get tile data address
                let tile_data_range = self.tile_data_address(tile_index);
                let tile_data = &self.vram[tile_data_range];

                let pixel = Self::decode_pixel(self.bgrdpal, tile_data, pixel_row, pixel_col);
                self.framebuffer[(ly as _, screen_x)] = pixel;
            }
        }

        // Draw sprites (OBJ)
        if self.lcdc.contains(LCDC::OBJ_ENABLED) {
            self.draw_sprites_on_line();
        }
    }

    fn draw_sprites_on_line(&mut self) {
        let ly = self.scanline;
        let sprite_height: u8 = if self.lcdc.contains(LCDC::OBJ_SIZE) { 16 } else { 8 };

        // Scan OAM for sprites on this scanline (max 10 per line on real hardware)
        for sprite in self.oam.chunks_exact(4) {
            let [sprite_y, sprite_x, tile_index, flags] = sprite else { continue };

            // Sprite Y is offset by 16, X by 8
            let screen_y = sprite_y.wrapping_sub(16);
            let screen_x = sprite_x.wrapping_sub(8);

            // Check if sprite is on this scanline
            if ly < screen_y || ly >= screen_y.wrapping_add(sprite_height) {
                continue;
            }

            let flags = SpriteAttribute::from(flags);
            let palette = if flags.obj0 { self.obj0pal } else { self.obj1pal };

            // Calculate which row of the sprite we're drawing
            let mut sprite_row = (ly - screen_y) as usize;
            if flags.yflip {
                sprite_row = (sprite_height as usize - 1) - sprite_row;
            }

            // Get tile data (8x16 sprites use tile_index & 0xFE for top, | 0x01 for bottom)
            let actual_tile = if sprite_height == 16 {
                if sprite_row < 8 {
                    tile_index & 0xFE
                } else {
                    sprite_row -= 8;
                    tile_index | 0x01
                }
            } else {
                *tile_index
            };

            let tile_start = actual_tile as usize * 16;
            let tile_data = &self.vram[tile_start..tile_start + 16];

            // Draw 8 pixels of the sprite
            for pixel_col in 0..8usize {
                let dest_x = screen_x.wrapping_add(pixel_col as u8) as usize;
                if dest_x >= WINDOW_WIDTH as usize {
                    continue;
                }

                let col = if flags.xflip { 7 - pixel_col } else { pixel_col };
                let lo_byte = tile_data[sprite_row * 2];
                let hi_byte = tile_data[sprite_row * 2 + 1];
                let bit = 7 - col;
                let lo_bit = (lo_byte >> bit) & 1;
                let hi_bit = (hi_byte >> bit) & 1;
                let color_index = (hi_bit << 1) | lo_bit;

                // Color index 0 is transparent for sprites
                if color_index == 0 {
                    continue;
                }

                // BG priority: if set, sprite only shows over BG color 0
                if flags.above {
                    // TODO: check if BG pixel is color 0
                }

                let color = (palette >> (color_index << 1)) & 0b11;
                let pixel = match color {
                    0b00 => 0xE0F8_D0FF,
                    0b01 => 0x88C0_70FF,
                    0b10 => 0x3468_56FF,
                    0b11 => 0x0818_20FF,
                    _ => 0,
                };
                self.framebuffer[(self.scanline as usize, dest_x)] = pixel;
            }
        }
    }

    // This is a huge can of worms to correct emulate the state of the scanline during emulation.
    // I would revisit this later.
    pub fn step(&mut self, flag: &mut u8) {
        let triggered = self.clock
            >= match self.mode {
                GpuMode::Oam => 80,
                GpuMode::Vram => 172,
                GpuMode::HBlank => 204,
                GpuMode::VBlank => 456,
            };
        if triggered {
            self.clock = 0;
            match self.mode {
                GpuMode::Oam => self.mode = GpuMode::Vram,
                GpuMode::Vram => {
                    self.draw_line();
                    self.mode = GpuMode::HBlank;
                }
                GpuMode::HBlank | GpuMode::VBlank => {
                    self.scanline += 1;
                    match self.mode {
                        GpuMode::HBlank => {
                            if self.scanline == END_HBLANK {
                                self.vblank_count += 1;
                                self.mode = GpuMode::VBlank;
                                *flag |= cpu::interrupts::VBLANK;
                            } else {
                                self.mode = GpuMode::Oam;
                            }
                        }
                        GpuMode::VBlank => {
                            if self.scanline == END_VBLANK {
                                self.scanline = 0;
                                self.mode = GpuMode::Oam;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn hex_dump(&self) {
        let mut start = VRAM_START;
        for row in self.vram.chunks_exact(4) {
            info!("{:04x}: {:02x} {:02x} {:02x} {:02x}", start, row[0], row[1], row[2], row[3]);
            start += 4;
        }
    }
}

impl Index<u16> for GPU {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        match i {
            0x44 => &self.scanline,
            _ => &self.vram[i as usize - 0x8000],
        }
    }
}

impl Display for GPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // No I'm not a monster I'll change these later.
        // TODO
        let wtmds = self.window_tile_map_display_select();
        let bgwtds = self.bg_and_window_tile_data_select();
        let bgtmds = self.bg_tile_map_display_select();
        f.write_fmt(format_args!(
            r"LCDC: {:08b}
LCD On: {}, 
Window Tile Map Display Select: {:04X}-{:04X}
Window Display Enable: {} 
BG+Window Tile Data Select: {:04X}-{:04X}
BG Tile Map Display Select: {:04X}-{:04X}
Sprite Size: {:?} 
Sprite Display Enable: {} 
BG Display: {}
STAT: {:08b}",
            self.lcdc,
            self.lcdc.contains(LCDC::LCD_PPU_ENABLE),
            wtmds.start(),
            wtmds.end(),
            self.lcdc.contains(LCDC::WINDOW_DISPLAY_ENABLED),
            bgwtds.start(),
            bgwtds.end(),
            bgtmds.start(),
            bgtmds.end(),
            self.sprite_size(),
            self.lcdc.contains(LCDC::OBJ_ENABLED),
            self.lcdc.contains(LCDC::BG_DISPLAY_ENABLED),
            self.lcdstat,
        ))
    }
}
