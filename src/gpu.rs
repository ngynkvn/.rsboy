use std::ops::Index;

#[derive(Debug)]
enum GpuMode {
    HBlank, // 0
    VBlank, // 1
    OAM,    // 2
    VRAM    // 3
}


pub struct GPU {
    mode: GpuMode,
    master_clock: usize,
    clock: usize,
    pub scanline: u8,
    pub vram: [u8; 0x2000],
    lcdc: u8,
}

const END_HBLANK: u8 = 143;
const END_VBLANK: u8 = 153;

impl GPU {
    pub fn new() -> Self {
        Self {
            mode: GpuMode::HBlank,
            master_clock: 0,
            clock: 0,
            scanline: 0,
            lcdc: 0,
            vram: [0; 0x2000],
        }
    }
    pub fn cycle(&mut self, clock: usize) {
        self.clock += clock - self.master_clock;
        self.master_clock = clock;
        self.step();
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
