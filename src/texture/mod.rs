use crate::gpu::PixelData;
use std::ops::Range;

fn pixel(value: u8) -> u32 {
    match value {
        0b00 => 0xE0F8D0FFu32, // White
        0b01 => 0x88C070FF,    // Light Gray
        0b10 => 0x346856FF,    // Dark Gray
        0b11 => 0x081820FF,    // Black
        _ => 0,
    }
    .swap_bytes()
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
            for x in 0..8 {
                let lo = d[0] >> (7 - x) & 1;
                let hi = d[1] >> (7 - x) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                let c = pixel(color);
                texture[y][x] = c;
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
            for x in 0..8 {
                let lo = d[0] >> (7 - x) & 1;
                let hi = d[1] >> (7 - x) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                let mut c = pixel(color);
                if color == 0 {
                    c &= 0xFFFFFF00;
                }
                texture[y][x] = c;
            }
        }
        Self { texture }
    }

    // PERFORMANCE ISSUE -- sike
    pub fn write(palette: u8, pixels: &mut PixelData, location: (usize, usize), tile_data: &[u8]) {
        let (mapx, mapy) = location;
        for i in 0..8 {
            let y = (mapy * 8) + i;

            let pixels = &mut pixels[y];

            let mut lo = tile_data[i * 2];
            let mut hi = tile_data[i * 2 + 1];
            let x = mapx * 8;
            for offset in 0..8 {
                let lo_b = lo & 1;
                let hi_b = hi & 1;
                let index = (hi_b << 2) | lo_b << 1;
                let color = (palette >> index) & 0b11;
                let c = pixel(color);
                pixels[x + 7 - offset] = c;
                lo >>= 1;
                hi >>= 1;
            }
        }
    }

    // Size of a tile
    pub fn range(i: usize) -> Range<usize> {
        i..i + 16
    }

    pub fn texture(&self) -> &[[u32; 8]; 8] {
        &self.texture
    }
}
