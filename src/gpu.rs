use crate::texture::*;
use crate::emu::Emu;
use std::ops::Index;

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
    pub irq: bool,
}

const END_HBLANK: u8 = 143;
const END_VBLANK: u8 = 153;

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
            irq: false,
            vram: [0; 0x2000],
        }
    }
    pub fn is_on(&self) -> bool {
        self.lcdc & 0b1000_0000 == 0b1000_0000
    }
    pub fn cycle(&mut self, emu: &mut Emu, clock: usize) {
        if !self.is_on() {
            return;
        }
        self.clock += clock;
        self.step();
    }

    pub fn scroll(&self) -> (u32, u32) {
        (self.hscroll as u32, self.vscroll as u32)
    }

    pub fn background(&self) -> Map {
        Map {
            width: 32,
            height: 32,
            tile_set: self.tiles(),
            map: self.vram[0x1800..0x1C00]
                .iter()
                .map(|x| *x as usize)
                .collect(),
        }
    }

    pub fn tiles(&self) -> Vec<Tile> {
        // 0x8000-0x87ff
        let mut tiles: Vec<Tile> = vec![];
        for i in (0..0x7ff).step_by(16) {
            let tile_data = &self.vram[i..(i + 16)];
            tiles.push(Tile::construct(self.bg_palette, tile_data));
        }
        tiles
    }

    pub fn step(&mut self) {
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
                        self.irq = true;
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
    }
}
impl Index<u16> for GPU {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        match i {
            0x44 => &self.scanline,
            _ => &self.vram[i as usize],
        }
    }
}
