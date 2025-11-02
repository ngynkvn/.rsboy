use crate::gpu::PixelData;
use std::ops::Range;

const fn pixel(value: u8) -> u32 {
    match value {
        0b00 => 0xE0F8_D0FF, // White
        0b01 => 0x88C0_70FF, // Light Gray
        0b10 => 0x3468_56FF, // Dark Gray
        0b11 => 0x0818_20FF, // Black
        _ => 0,
    }
}

pub struct Tile {
    pub texture: [[u32; 8]; 8],
}

impl Tile {
    pub fn construct(palette: u8, tile_data: &[u8]) -> Self {
        let mut texture = [[0; 8]; 8];
        // We receive in order of
        // low byte, then high byte
        for (y, d) in tile_data.chunks_exact(2).enumerate() {
            //Each row in tile is pair of 2 bytes.
            for (x, t) in texture[y].iter_mut().enumerate() {
                let lo = d[0] >> (7 - x) & 1;
                let hi = d[1] >> (7 - x) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                let c = pixel(color);
                *t = c;
            }
        }
        Self { texture }
    }

    pub fn sprite_construct(palette: u8, tile_data: &[u8]) -> Self {
        let mut texture = [[0; 8]; 8];
        // We receive in order of
        // low byte, then high byte
        for (y, d) in tile_data.chunks_exact(2).enumerate() {
            //Each row in tile is pair of 2 bytes.
            for (x, t) in texture[y].iter_mut().enumerate() {
                let lo = d[0] >> (7 - x) & 1;
                let hi = d[1] >> (7 - x) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                let mut c = pixel(color);
                if color == 0 {
                    c &= 0xFFFF_FF00;
                }
                *t = c;
            }
        }
        Self { texture }
    }

    // PERFORMANCE ISSUE -- sike
    pub fn write(palette: u8, pixels: &mut PixelData, location: (usize, usize), tile_data: &[u8]) {
        let (mx, my) = location;
        for i in 0..8 {
            let y = (my * 8) + i;

            let mut lo = tile_data[i * 2];
            let mut hi = tile_data[i * 2 + 1];
            let x = mx * 8;
            for offset in 0..8 {
                let lo_b = lo & 1;
                let hi_b = hi & 1;
                let index = (hi_b << 2) | lo_b << 1;
                let color = (palette >> index) & 0b11;
                let c = pixel(color);
                let ind = x + 7 - offset;
                pixels[(y, ind)] = c;
                lo >>= 1;
                hi >>= 1;
            }
        }
    }

    // Size of a tile
    pub const fn range(i: usize) -> Range<usize> {
        i..i + 16
    }

    pub const fn texture(&self) -> &[[u32; 8]; 8] {
        &self.texture
    }
}
