use crate::gpu::PixelData;
use std::ops::Range;

fn pixel(value: u8) -> u16 {
    match value {
        0b00 => 0xE7DA,
        0b01 => 0x8E0E,
        0b10 => 0x360A,
        0b11 => 0x08C4,
        _ => unreachable!("Are you sure you're reading byte data?"),
    }
}

pub struct Tile {
    pub texture: [[u16; 8]; 8],
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

    // PERFORMANCE ISSUE
    pub fn write(palette: u8, pixels: &mut PixelData, location: (usize, usize), tile_data: &[u8]) {
        let (mapx, mapy) = location;
        for (y, d) in tile_data.chunks_exact(2).enumerate() {
            //Each row in tile is pair of 2 bytes.
            let y = mapy * 8 + y;
            let pixels = &mut pixels[y];
            if let [mut lo, mut hi] = d {
                let x = mapx * 8;
                for x in (x..x + 8).rev() {
                    let lo_b = lo & 1;
                    let hi_b = hi & 1;
                    let index = (hi_b << 2) | lo_b << 1;
                    let color = (palette >> index) & 0b11;
                    let c = pixel(color);
                    pixels[x] = c;
                    lo >>= 1;
                    hi >>= 1;
                }
            }
        }
    }

    // Size of a tile
    pub fn range(i: usize) -> Range<usize> {
        i..i + 16
    }

    pub fn texture(&self) -> &[[u16; 8]; 8] {
        &self.texture
    }
}
