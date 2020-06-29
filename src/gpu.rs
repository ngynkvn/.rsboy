use crate::texture::*;
use std::{ops::Index, time};

pub const VRAM_START: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;

#[derive(Debug)]
enum GpuMode {
    HBlank, // 0
    VBlank, // 1
    OAM,    // 2
    VRAM,   // 3
}

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

type PixelData = [u16; 256 * 256];

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

type SpriteEntry = (u8, u8, u8, SpriteAttribute);

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
        for i in self.vram[0x1e00..0x1ea0].chunks_exact(4) {
            println!("{:?}", i);
        }
    }

    // Returns true if IRQ is requested.
    pub fn cycle(&mut self) -> bool {
        if !self.is_on() {
            return false;
        }
        self.clock += 1;
        self.step()
    }

    pub fn scroll(&self) -> (u32, u32) {
        (self.hscroll as u32, self.vscroll as u32)
    }

    pub fn background(&self) -> Map {
        Map {
            width: 32,
            height: 32,
            tile_set: self.tiles(),
            map: &self.vram[0x1800..0x1C00],
        }
    }

    pub fn tiles(&self) -> Vec<Tile> {
        self.vram[..0x1800]
            .chunks_exact(16) // Tile
            .map(|tile| Tile::construct(self.bg_palette, tile))
            .collect()
    }

    fn render_tile(&self, pixels: &mut PixelData, vram_index: usize) {
        let tile = self.vram[vram_index] as usize * 16;
        let tile = Tile::construct(self.bg_palette, &self.vram[Tile::range(tile)]);
        let mapx = (vram_index - 0x1800) % 32;
        let mapy = (vram_index - 0x1800) / 32;
        for row in 0..8 {
            for col in 0..8 {
                let t = col + row * 8;

                //Find offset from map x and y
                let location = mapx * 8 + col + mapy * 8 * 256 + row * 256;
                pixels[location] = tile.texture[t];
            }
        }
    }

    pub fn render_map(&self, texture: &mut sdl2::render::Texture) {
        let mut pixels: PixelData = [0; 256 * 256];
        let start = time::Instant::now();
        for i in 0x1800..0x1C00 {
            self.render_tile(&mut pixels, i);
        }

        // TODO
        // Need to emulate scanline, and priority rendering
        // for sprite_attributes in self.vram[..0x1000].chunks_exact(4) {
        //     if let [x, y, pattern, flags] = sprite_attributes {
        //         let idx = *pattern as usize * 16;
        //         let tile = Tile::construct(self.bg_palette, &self.vram[Tile::range(idx)]);
        //         let screen_x = x.wrapping_sub(8);
        //         let screen_y = y.wrapping_sub(16);
        //         self.render_tile(
        //             &mut pixels,
        //             screen_x as usize,
        //             screen_y as usize,
        //             tile.texture(),
        //         );
        //     }
        // }

        let pixels = unsafe { std::mem::transmute::<PixelData, [u8; 256 * 256 * 2]>(pixels) };
        // println!(
        //     "{:?}",
        //     time::Instant::now().saturating_duration_since(start)
        // );
        texture.update(None, &pixels, 256 * 2).unwrap();
    }

    // Returns true if interrupt is requested
    pub fn step(&mut self) -> bool {
        match self.mode {
            GpuMode::OAM => {
                if self.clock >= 80 {
                    self.clock = 0;
                    self.mode = GpuMode::VRAM
                }
            }
            GpuMode::VRAM => {
                if self.clock >= 172 {
                    self.clock = 0;
                    self.mode = GpuMode::HBlank
                }
            }
            GpuMode::HBlank => {
                if self.clock >= 204 {
                    self.clock = 0;
                    self.scanline += 1;
                    if self.scanline == END_HBLANK {
                        self.mode = GpuMode::VBlank;
                        //Might be wrong position to trigger interrupt
                        return true;
                    }
                }
            }
            GpuMode::VBlank => {
                if self.clock >= 456 {
                    self.clock = 0;
                    self.scanline += 1;
                    if self.scanline == END_VBLANK {
                        self.mode = GpuMode::OAM;
                        self.scanline = 0;
                    }
                }
            }
        }
        false
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
