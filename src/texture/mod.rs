const TILE_WIDTH: usize = 8;
#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    LightGrey,
    DarkGrey,
    Black,
}

impl Color {
    pub fn value(self) -> &'static [u8; 3] {
        match self {
            Color::White => &[224, 248, 208],
            Color::LightGrey => &[136, 192, 112],
            Color::DarkGrey => &[52, 104, 86],
            Color::Black => &[8, 24, 32],
        }
    }
    pub fn pixel(value: u8) -> u16 {
        match value {
            0b00 => 0xE7DA,
            0b01 => 0x8E0E,
            0b10 => 0x360A,
            0b11 => 0x08C4,
            _ => unreachable!("Are you sure you're reading byte data?"),
        }    
    }
    pub fn byte2color(value: u8) -> Self {
        match value {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => unreachable!("Are you sure you're reading byte data?"),
        }
    }
}

pub struct Tile {
    pub texture: [u16; 64],
}

impl Tile {
     pub fn construct(palette: u8, tile_data: &[u8]) -> Self {
        let mut texture  = [255; 64];
        for row in 0..8 {
            for col in 0..8 {
                let hi = tile_data[(row * 2) + 1] >> (7 - col) & 1;
                let lo = tile_data[(row * 2)] >> (7 - col) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                texture[row * 8 + col] = Color::pixel(color);
            }
        }

        Self { texture }
    }

    pub fn coord(i: usize) -> (usize, usize) {
        ((i / 8) as usize, (i % 8) as usize)
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
        self.width * TILE_WIDTH * 3
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
        let ret = byte_row.iter().flatten().flat_map(|x| x.to_le_bytes().to_vec()).collect();
        ret
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pixel_dims(&self) -> (usize, usize) {
        (self.width * TILE_WIDTH, self.height * TILE_WIDTH)
    }
}
