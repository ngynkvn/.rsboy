use std::ops::Index;
use std::ops::IndexMut;

pub struct Mem {
    pub mem: [u8; 0xFFFF],
}

impl Mem {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = [0; 0xFFFF];

        mem[..rom.len()].clone_from_slice(&rom[..]);
        Self { mem }
    }
}
impl Index<u16> for Mem {
    type Output = u8;
    fn index(&self, i: u16) -> &Self::Output {
        &self.mem[i as usize]
    }
}

impl IndexMut<u16> for Mem {
    fn index_mut(&mut self, i: u16) -> &mut Self::Output {
        &mut self.mem[i as usize]
    }
}
