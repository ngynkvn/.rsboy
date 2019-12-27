mod registers;
use registers::Registers;
pub struct CPU {
    registers: Registers,
    rom: Vec<u8>,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: Registers::new(),
            rom: rom,
        }
    }
    fn curr_u8(&self) -> u8 {
        self.rom[self.registers.pc as usize]
    }
    pub fn read_instruction(&self) {
        match self.curr_u8() {
            0x00 => println!("This is a noop!!"),
            _ => panic!("Unknown Instruction: {:02X}", self.curr_u8()),
        }
    }
}
