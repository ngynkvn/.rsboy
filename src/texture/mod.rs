use std::ops::Range;

const TILE_WIDTH: usize = 8;

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
    pub texture: [u16; 64],
}

impl Tile {
    pub fn construct(palette: u8, tile_data: &[u8]) -> Self {
        let mut texture = [255; 64];
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
                let location = x + y * 8;
                texture[location] = c;
            }
        }
        Self { texture }
    }

    // Size of a tile
    pub fn range(i: usize) -> Range<usize> {
        return i..i + 16;
    }

    pub fn texture(&self) -> &[u16; 64] {
        &self.texture
    }
}

pub struct Map<'a> {
    pub width: usize,
    pub height: usize,
    pub tile_set: Vec<Tile>,
    pub map: &'a [u8],
}

impl<'a> Map<'a> {
    pub fn pitch(&self) -> usize {
        self.width * TILE_WIDTH * 2
    }

    pub fn texture(&self) -> Vec<u8> {
        let mut byte_row = vec![vec![]; TILE_WIDTH * self.height];
        for (i, row) in self.map.chunks_exact(self.width).enumerate() {
            for &index in row {
                // Tile index
                for (j, tile_row) in self.tile_set[index as usize]
                    .texture()
                    .chunks_exact(8)
                    .enumerate()
                {
                    byte_row[i * TILE_WIDTH + j].extend_from_slice(&tile_row);
                }
            }
        }
        byte_row
            .iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes().to_vec())
            .collect()
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pixel_dims(&self) -> (usize, usize) {
        (self.width * TILE_WIDTH, self.height * TILE_WIDTH)
    }
}
