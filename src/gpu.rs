use crate::{cpu, texture::*};
use std::{
    fmt::Display,
    ops::{Index, Range, RangeInclusive},
    time,
};

pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const OAM_START: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;
pub const TILE_DATA_RANGE: Range<usize> = 0..0x1800;
pub const MAP_DATA_RANGE: Range<usize> = 0x1800..0x1C00;
pub const TILE_SIZE: usize = 16;

#[derive(Debug)]
enum GpuMode {
    HBlank, // 0
    VBlank, // 1
    OAM,    // 2
    VRAM,   // 3
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
    pub lcdc: u8,
    pub lcdstat: u8,
    pub scrollx: u8,
    pub scrolly: u8,
    pub bgrdpal: u8, //Background Palette
    pub obj0pal: u8, //Object0 Palette
    pub obj1pal: u8, //Object1 Palette
    pub windowx: u8, //
    pub windowy: u8, //
    pub _vblank_count: usize,
}

const END_HBLANK: u8 = 144;
const END_VBLANK: u8 = 154;

pub type PixelData = [[u32; 256]; 256];
pub type PixelMap = [u8; 256 * 256 * 4];

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
            mode: GpuMode::OAM,
            clock: 0,
            scanline: 0,
            // FFxx Values
            lcdc: 0,
            lcdstat: 0,
            scrolly: 0,
            scrollx: 0,
            bgrdpal: 0,
            obj0pal: 0,
            obj1pal: 0,
            windowx: 0,
            windowy: 0,
            // FFxx Values end
            _vblank_count: 0,
            vram: [0; 0x2000],
            oam: [0; 0x100],
        }
    }
    //   Bit 7 - LCD Display Enable             (0=Off, 1=On)
    pub fn is_on(&self) -> bool {
        self.lcdc & 0b1000_0000 == 0b1000_0000
    }
    //   Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    fn window_tile_map_display_select(&self) -> RangeInclusive<usize> {
        if self.lcdc & 0b0100_0000 != 0 {
            (0x9C00)..=(0x9FFF)
        } else {
            (0x9800)..=(0x9BFF)
        }
    }

    //   Bit 5 - Window Display Enable          (0=Off, 1=On)
    fn window_display_enabled(&self) -> bool {
        self.lcdc & 0b0010_0000 == 0b0010_0000
    }

    //   Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    fn bg_and_window_tile_data_select(&self) -> RangeInclusive<usize> {
        if self.lcdc & 0b0010_0000 != 0 {
            (0x8000)..=(0x8FFF)
        } else {
            (0x8800)..=(0x97FF)
        }
    }
    fn bg_tile_data(&self, value: u8) -> Range<usize> {
        if self.lcdc & 0b0001_0000 != 0 {
            let start_address = value as usize * 16;
            let end_address = start_address + 16;
            start_address..end_address
        } else {
            let offset = value as i8 as i32;
            let start_address = (0x1000 + (offset * 16) as i32) as usize;
            let end_address = start_address + 16;
            start_address..end_address
        }
    }
    //   Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    fn bg_tile_map_display_select(&self) -> RangeInclusive<usize> {
        if self.lcdc & 0b0001_0000 != 0 {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    //   Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    fn sprite_size(&self) -> SpriteSize {
        if self.lcdc & 0b100 == 0b100 {
            SpriteSize::Square
        } else {
            SpriteSize::Tall
        }
    }
    //   Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    fn sprite_display_enabled(&self) -> bool {
        self.lcdc & 0b10 == 0b10
    }
    //   Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)

    pub fn print_sprite_table(&self) {
        for i in self.oam.chunks_exact(4) {
            println!("{:?}", i);
        }
    }

    // Returns true if IRQ is requested.
    pub fn cycle(&mut self, flag: &mut u8) {
        if !self.is_on() {
            return;
        }
        self.clock += 1;
        self.step(flag)
    }

    pub fn scroll(&self) -> (u32, u32) {
        (self.scrollx as u32, self.scrolly as u32)
    }

    pub fn tiles(&self, palette: u8) -> Vec<Tile> {
        self.vram[TILE_DATA_RANGE]
            .chunks_exact(TILE_SIZE) // Tile
            .map(|tile| Tile::construct(palette, tile))
            .collect()
    }

    fn blit_tile(&self, pixels: &mut PixelData, vram_index: usize) {
        let tile = self.bg_tile_data(self.vram[vram_index]);
        let mapx = (vram_index - 0x1800) % 32;
        let mapy = (vram_index - 0x1800) / 32;
        Tile::write(self.bgrdpal, pixels, (mapx, mapy), &self.vram[tile]);
    }

    fn blit_to_screen(&self, pixels: &mut PixelData, screenx: usize, screeny: usize, tile: Tile) {
        for row in 0..8 {
            for col in 0..8 {
                let (x, y) = self.scroll();
                let x = screenx + col + x as usize;
                let y = screeny + row + y as usize;
                if y < pixels.len() && x < pixels[0].len() {
                    pixels[y][x] = tile.texture[row][col];
                }
            }
        }
    }

    pub fn render_to(&self, pixels: &mut PixelData) {
        for i in MAP_DATA_RANGE {
            self.blit_tile(pixels, i);
        }

        if self.sprite_display_enabled() {
            self.render_sprites(pixels);
        }
    }

    pub fn render(&self, pixels: &mut PixelData) {
        let _start = time::Instant::now();
        for i in MAP_DATA_RANGE {
            self.blit_tile(pixels, i);
        }

        if self.sprite_display_enabled() {
            self.render_sprites(pixels);
        }
    }

    // Renders sprites to the framebuffer using the oam table.
    fn render_sprites(&self, pixels: &mut PixelData) {
        // TODO
        // Need to emulate scanline, and priority rendering
        for sprite_attributes in self.oam.chunks_exact(4) {
            if sprite_attributes.iter().all(|x| *x == 0) {
                continue;
            }
            if let [y, x, pattern, flags] = sprite_attributes {
                let flags = SpriteAttribute::from(flags);
                let idx = *pattern as usize * 16;

                let palette = if flags.obj0 {
                    self.obj0pal
                } else {
                    self.obj1pal
                };
                let tile = Tile::sprite_construct(palette, &self.vram[Tile::range(idx)]);
                let screen_x = (*x).wrapping_sub(8);
                let screen_y = (*y).wrapping_sub(16);
                self.blit_to_screen(pixels, screen_x as usize, screen_y as usize, tile);
            }
        }
    }

    fn check_clock<F: FnOnce(&mut Self)>(&mut self, criteria: usize, f: F) {
        if self.clock >= criteria {
            f(self);
            self.clock = 0;
        }
    }

    // This is a huge can of worms to correct emulate the state of the scanline during emulation.
    // I would revisit this later.
    pub fn step(&mut self, flag: &mut u8) {
        match self.mode {
            GpuMode::OAM => self.check_clock(80, |gpu| gpu.mode = GpuMode::VRAM),
            GpuMode::VRAM => self.check_clock(172, |gpu| gpu.mode = GpuMode::HBlank),
            GpuMode::HBlank => self.check_clock(204, |gpu| {
                gpu.scanline += 1;
                if gpu.scanline == END_HBLANK {
                    gpu._vblank_count += 1;
                    *flag |= cpu::VBLANK;
                    gpu.mode = GpuMode::VBlank;
                } else {
                    gpu.mode = GpuMode::OAM;
                }
            }),
            GpuMode::VBlank => self.check_clock(456, |gpu| {
                gpu.scanline += 1;
                if gpu.scanline == END_VBLANK {
                    gpu.mode = GpuMode::OAM;
                    gpu.scanline = 0;
                }
            }),
        }
    }

    pub fn hex_dump(&self) {
        let mut start = VRAM_START;
        for row in self.vram.chunks_exact(4) {
            println!(
                "{:04x}: {:02x} {:02x} {:02x} {:02x}",
                start, row[0], row[1], row[2], row[3]
            );
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
            r#"LCDC: {:08b}
LCD On: {}, 
Window Tile Map Display Select: {:04X}-{:04X}
Window Display Enable: {} 
BG+Window Tile Data Select: {:04X}-{:04X}
BG Tile Map Display Select: {:04X}-{:04X}
Sprite Size: {:?} 
Sprite Display Enable: {} 
BG Display: UNIMPLEMENTED
STAT: {:08b}"#,
            self.lcdc,
            self.is_on(),
            wtmds.start(),
            wtmds.end(),
            self.window_display_enabled(),
            bgwtds.start(),
            bgwtds.end(),
            bgtmds.start(),
            bgtmds.end(),
            self.sprite_size(),
            self.sprite_display_enabled(),
            // self.bg_display_enabled(),
            self.lcdstat,
        ))
    }
}
