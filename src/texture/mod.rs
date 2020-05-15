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
    pub fn bit2color(value: u8) -> Self {
        match value {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => unreachable!("Are you sure you're reading bit data?"),
        }
    }
}

pub struct Tile {
    pub data: [Color; 64], //8 x 8
    // pub data: Vec<Color>,
    pub texture: [u8; 192],
}

impl Tile {
     pub fn construct(palette: u8, tile_data: &[u8]) -> Self {
        let mut data = [Color::White; 64];
        for row in 0..8 {
            for col in 0..8 {
                let hi = tile_data[(row * 2) + 1] >> (7 - col) & 1;
                let lo = tile_data[(row * 2)] >> (7 - col) & 1;
                let index = (hi << 1) | lo;
                let color = (palette >> (index << 1)) & 0b11;
                data[row * 8 + col] = Color::bit2color(color);
            }
        }

        let mut texture  = [255; 192];
        let mut p = 0;
        for i in data.iter() {
            texture[p..(p + 3)].clone_from_slice(i.value());
            p += 3;
        }

        Self { data, texture }
    }

    pub fn coord(i: usize) -> (usize, usize) {
        ((i / 8) as usize, (i % 8) as usize)
    }

    pub fn texture(&self) -> &[u8; 192] {
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
                    .chunks_exact(24)
                    .enumerate()
                {
                    byte_row[i * TILE_WIDTH + j].extend_from_slice(&tile_row);
                }
            }
        }
        let ret: Vec<u8> = byte_row.iter().cloned().flatten().collect();
        ret
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pixel_dims(&self) -> (usize, usize) {
        (self.width * TILE_WIDTH, self.height * TILE_WIDTH)
    }
}
