use crate::memory::Memory;
use std::ops::Index;

enum GpuMode {
    HBlank, // 0
    VBlank, // 1
    OAM,    // 2
    VRAM    // 3
}


pub struct GPU {
    mode: GpuMode,
    clock: usize,
    scanline: u8,
    pub vram: [u8; 0x2000],
    lcdc: u8,
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
            vram: [0; 0x2000],
        }
    }
    pub fn step(&mut self) {
        match self.mode {
            GpuMode::OAM => {
                if self.clock >= 80 {
                    self.clock = 0;
                    self.mode = GpuMode::VRAM
                }
            },
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
                    }
                }
            },
            GpuMode::VBlank => {
                if self.clock >= 456 {
                    self.clock = 0;
                    self.scanline += 1;
                    if self.scanline == END_VBLANK {
                        self.mode = GpuMode::OAM;
                        self.scanline = 0;
                    }
                }
            },
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
