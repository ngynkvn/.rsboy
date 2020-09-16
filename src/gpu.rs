use crate::{cpu, texture::*};
use std::{
    ops::{Index, Range},
    time,
};

pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const SPRITE_ATTR_RANGE: Range<usize> = 0x1e00..0x1ea0;
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

#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

// Global emu struct.
pub struct GPU {
    mode: GpuMode,
    clock: usize,
    pub scanline: u8,
    pub vram: [u8; 0x2000],
    pub lcdc: u8,
    pub lcdstat: u8,
    pub hscroll: u8,
    pub vscroll: u8,
    pub bgrdpal: u8, //Background Palette
    pub obj0pal: u8, //Object0 Palette
    pub obj1pal: u8, //Object1 Palette
    pub windowx: u8, //
    pub windowy: u8, //
    pub bg_palette: u8,
}

const END_HBLANK: u8 = 143;
const END_VBLANK: u8 = 153;

pub type PixelData = [[u16; 256]; 256];
pub type PixelMap = [u8; 256 * 256 * 2];

struct SpriteAttribute {
    above: bool,
    yflip: bool,
    xflip: bool,
}

impl From<u8> for SpriteAttribute {
    fn from(byte: u8) -> Self {
        Self {
            above: byte & 0x80 != 0,
            yflip: byte & 0x40 != 0,
            xflip: byte & 0x20 != 0,
        }
    }
}

impl GPU {
    pub fn new() -> Self {
        Self {
            mode: GpuMode::HBlank,
            clock: 0,
            scanline: 0,
            lcdc: 0,
            lcdstat: 0,
            vscroll: 0,
            hscroll: 0,
            bgrdpal: 0,
            obj0pal: 0,
            obj1pal: 0,
            windowx: 0,
            windowy: 0,
            bg_palette: 0,
            vram: [0; 0x2000],
        }
    }
    pub fn is_on(&self) -> bool {
        self.lcdc & 0b1000_0000 == 0b1000_0000
    }

    pub fn print_sprite_table(&self) {
        // 0x1e00-0x1ea0
        for i in self.vram[SPRITE_ATTR_RANGE].chunks_exact(4) {
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
        (self.hscroll as u32, self.vscroll as u32)
    }

    pub fn tiles(&self) -> Vec<Tile> {
        self.vram[TILE_DATA_RANGE]
            .chunks_exact(TILE_SIZE) // Tile
            .map(|tile| Tile::construct(self.bg_palette, tile))
            .collect()
    }

    fn blit_tile(&self, pixels: &mut PixelData, vram_index: usize) {
        let tile = self.vram[vram_index] as usize * 16;
        let mapx = (vram_index - 0x1800) % 32;
        let mapy = (vram_index - 0x1800) / 32;
        Tile::write(
            self.bg_palette,
            pixels,
            (mapx, mapy),
            &self.vram[Tile::range(tile)],
        );
    }

    fn blit_texture(&self, pixels: &mut PixelData, mapx: usize, mapy: usize, tile: Tile) {
        for row in 0..8 {
            for col in 0..8 {
                //Find offset from map x and y
                let x = mapx * 8 + col;
                let y = mapy * 8 + row;
                if y < pixels.len() && x < pixels[0].len() {
                    pixels[y][x] = tile.texture[row][col];
                }
            }
        }
    }

    pub fn render(&self, pixels: &mut PixelData) {
        let _start = time::Instant::now();
        for i in MAP_DATA_RANGE {
            self.blit_tile(pixels, i);
        }

        // TODO
        // Need to emulate scanline, and priority rendering
        for sprite_attributes in self.vram[SPRITE_ATTR_RANGE].chunks_exact(4) {
            if let [_flags, pattern, x, y] = sprite_attributes {
                // let _flags = SpriteAttribute::from(*flags);
                let idx = *pattern as usize * 16;
                let tile = Tile::construct(self.bg_palette, &self.vram[Tile::range(idx)]);
                let screen_x = (*x).wrapping_sub(8);
                let screen_y = (*y).wrapping_sub(16);
                self.blit_texture(pixels, screen_x as usize, screen_y as usize, tile);
            }
        }
        // println!("{:?}", time::Instant::now().saturating_duration_since(start));
    }

    // Returns true if interrupt is requested
    pub fn step(&mut self, flag: &mut u8) {
        match self.mode {
            GpuMode::OAM => {
                if self.clock >= 20 {
                    self.clock = 0;
                    self.mode = GpuMode::VRAM
                }
            }
            GpuMode::VRAM => {
                if self.clock >= 43 {
                    self.clock = 0;
                    self.mode = GpuMode::HBlank
                }
            }
            GpuMode::HBlank => {
                if self.clock >= 51 {
                    self.clock = 0;
                    self.scanline += 1;
                    if self.scanline == END_HBLANK {
                        self.mode = GpuMode::VBlank;
                        //Might be wrong position to trigger interrupt
                        *flag |= cpu::VBLANK;
                    }
                }
            }
            GpuMode::VBlank => {
                if self.clock >= 114 {
                    self.clock = 0;
                    self.scanline += 1;
                    if self.scanline == END_VBLANK {
                        self.mode = GpuMode::OAM;
                        self.scanline = 0;
                    }
                }
            }
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
