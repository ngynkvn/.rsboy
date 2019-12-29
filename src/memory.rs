use std::ops::Index;

pub struct Mem {
    pub mem: [u8; 0xFFFF],
}

impl Mem {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = [0; 0xFFFF];

        for i in 0..rom.len() {
            mem[i] = rom[i];
        }

        Self { mem: mem }
    }
}

impl Index<usize> for Mem {
    type Output = u8;
    fn index(&self, i: usize) -> &Self::Output {
        &self.mem[i]
    }
}
