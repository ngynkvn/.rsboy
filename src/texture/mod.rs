const TILE_WIDTH: usize = 8;
#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    LightGrey,
    DarkGrey,
    Black,
}

impl Color {
    pub fn value(self) -> [u8; 3] {
        match self {
            Color::White => [224, 248, 208],
            Color::LightGrey => [136, 192, 112],
            Color::DarkGrey => [52, 104, 86],
            Color::Black => [8, 24, 32],
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
    pub raw_data: [u8; 16],
}

impl Tile {
    pub fn construct(tile_data: &[u8]) -> Self {
        let mut raw_data = [0; 16];
        let mut data = [Color::White; 64];
        for row in 0..8 {
            for col in 0..8 {
                let hi = tile_data[(row * 2) + 1] >> (7 - col) & 1;
                let lo = tile_data[(row * 2)] >> (7 - col) & 1;
                data[row * 8 + col] = Color::bit2color((hi << 1) | lo);
            }
        }
        raw_data[..].clone_from_slice(tile_data);

        Self { data, raw_data }
    }

    pub fn coord(i: usize) -> (usize, usize) {
        ((i / 8) as usize, (i % 8) as usize)
    }

    pub fn texture(&self) -> [u8; 192] {
        //64 * 3
        let mut buffer = [255; 192];
        let mut p = 0;
        for i in self.data.iter() {
            buffer[p..(p + 3)].clone_from_slice(&i.value());
            p += 3;
        }
        buffer
    }
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tile_set: Vec<Tile>,
    pub map: Vec<usize>,
}

impl Map {
    pub fn new(width: usize, height: usize, tile_set: Vec<Tile>) -> Self {
        Self {
            width,
            height,
            tile_set,
            map: vec![0; width * height],
        }
    }
    pub fn set(&mut self, x: usize, y: usize, i: usize) {
        self.map[x + y * self.width] = i;
    }

    pub fn pitch(&self) -> usize {
        self.width * TILE_WIDTH * 3
    }

    /**
     * Mapping is like this in memory right now:
     *  for a 4x4 tile size
     * [1, 1, 1, 1] [1, 1, 1, 1]
     * [2, 2, 2, 2] [2, 2, 2, 2]
     * [3, 3, 3, 3] [3, 3, 3, 3]
     * [4, 4, 4, 4] [4, 4, 4, 4]
     *
     * Fine and dandy, but we need the 2d repre to be:
     *        (ROW 1)      (ROW 2)
     * [1, 1, 1, 1, 1, 1, 1, 1,   2, 2, 2, 2, 2, 2, 2, 2, ...]
     *
     * This should definitely be revisited for optimization down the line.
     */
    pub fn texture(&self) -> Vec<u8> {
        let mut byte_row = vec![vec![]; TILE_WIDTH * self.height];
        for (i, row) in self.map.chunks_exact(self.width).enumerate() {
            for &index in row {
                // Tile index
                for (j, tile_row) in self.tile_set[index].texture().chunks_exact(24).enumerate() {
                    byte_row[i * TILE_WIDTH + j].extend_from_slice(&tile_row);
                }
            }
        }
        byte_row.iter().cloned().flatten().collect()
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn pixel_dims(&self) -> (usize, usize) {
        (self.width * TILE_WIDTH, self.height * TILE_WIDTH)
    }
}
