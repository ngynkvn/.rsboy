use crate::registers::RegisterState;
use crate::memory::Mem;

pub struct CPU {
    registers: RegisterState,
    memory: Mem,
}

macro_rules! LD {
    // LD n, u8
    ($self:ident, IMMEDIATE, $r:ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 2,
            $r: $self.next_u8(),
            ..$self.registers
        };
    }};

    // LD r1, r2
    ($self:ident, REGISTER, $r1:ident, $r2:ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: $self.registers.$r2(),
            ..$self.registers
        }
    }};

    // LD r1, (r2), from MEM
    ($self:ident, READ_MEM, $r1:ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: $self.read_byte($self.registers.hl()),
            ..$self.registers
        }
    }};

    // LD (r1), r2, to MEM
    ($self:ident, LOAD_MEM, $r1:ident) => {{
        $self.set_byte($self.registers.hl(), $self.registers.$r1());
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: RegisterState::new(),
            memory: Mem::new(rom),
        }
    }
    fn curr_u8(&self) -> u8 {
        self.memory[self.registers.pc]
    }
    fn next_u8(&self) -> u8 {
        self.memory[(self.registers.pc + 1)]
    }
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address]
    }
    fn set_byte(&mut self, address: u16, value: u8) {
        self.memory[address] = value;
    }
    pub fn read_instruction(&mut self) {
        println!("opcode:{:02X}\nregisters:{:?}",self.curr_u8(),self.registers);
        match self.curr_u8() {
            0x00 => {
                self.registers = RegisterState {
                    pc: self.registers.pc + 1,
                    ..self.registers
                }
            }
            // 3.3.1. 8-bit Loads
            // 1 LD nn, n
            0x06 => LD!(self, IMMEDIATE, b),
            0x0E => LD!(self, IMMEDIATE, c),
            0x16 => LD!(self, IMMEDIATE, d),
            0x1E => LD!(self, IMMEDIATE, e),
            0x26 => LD!(self, IMMEDIATE, h),
            0x2E => LD!(self, IMMEDIATE, l),

            //2 LD r1, r2
            0x7F => LD!(self, REGISTER, a, a),
            0x78 => LD!(self, REGISTER, a, b),
            0x79 => LD!(self, REGISTER, a, c),
            0x7A => LD!(self, REGISTER, a, d),
            0x7B => LD!(self, REGISTER, a, e),
            0x7C => LD!(self, REGISTER, a, h),
            0x7D => LD!(self, REGISTER, a, l),
            0x7E => LD!(self, READ_MEM, a),
            0x40 => LD!(self, REGISTER, b, b),
            0x41 => LD!(self, REGISTER, b, c),
            0x42 => LD!(self, REGISTER, b, d),
            0x43 => LD!(self, REGISTER, b, e),
            0x44 => LD!(self, REGISTER, b, h),
            0x45 => LD!(self, REGISTER, b, l),
            0x46 => LD!(self, READ_MEM, b),
            0x48 => LD!(self, REGISTER, c, b),
            0x49 => LD!(self, REGISTER, c, c),
            0x4A => LD!(self, REGISTER, c, d),
            0x4B => LD!(self, REGISTER, c, e),
            0x4C => LD!(self, REGISTER, c, h),
            0x4D => LD!(self, REGISTER, c, l),
            0x4E => LD!(self, READ_MEM, c),
            0x50 => LD!(self, REGISTER, d, b),
            0x51 => LD!(self, REGISTER, d, c),
            0x52 => LD!(self, REGISTER, d, d),
            0x53 => LD!(self, REGISTER, d, e),
            0x54 => LD!(self, REGISTER, d, h),
            0x55 => LD!(self, REGISTER, d, l),
            0x56 => LD!(self, READ_MEM, d),
            0x58 => LD!(self, REGISTER, e, b),
            0x59 => LD!(self, REGISTER, e, c),
            0x5A => LD!(self, REGISTER, e, d),
            0x5B => LD!(self, REGISTER, e, e),
            0x5C => LD!(self, REGISTER, e, h),
            0x5D => LD!(self, REGISTER, e, l),
            0x5E => LD!(self, READ_MEM, e),
            0x60 => LD!(self, REGISTER, h, b),
            0x61 => LD!(self, REGISTER, h, c),
            0x62 => LD!(self, REGISTER, h, d),
            0x63 => LD!(self, REGISTER, h, e),
            0x64 => LD!(self, REGISTER, h, h),
            0x65 => LD!(self, REGISTER, h, l),
            0x66 => LD!(self, READ_MEM, h),
            0x68 => LD!(self, REGISTER, l, b),
            0x69 => LD!(self, REGISTER, l, c),
            0x6A => LD!(self, REGISTER, l, d),
            0x6B => LD!(self, REGISTER, l, e),
            0x6C => LD!(self, REGISTER, l, h),
            0x6D => LD!(self, REGISTER, l, l),
            0x6E => LD!(self, READ_MEM, l),
            0x70 => LD!(self, LOAD_MEM, b),
            0x71 => LD!(self, LOAD_MEM, c),
            0x72 => LD!(self, LOAD_MEM, d),
            0x73 => LD!(self, LOAD_MEM, e),
            0x74 => LD!(self, LOAD_MEM, h),
            0x75 => LD!(self, LOAD_MEM, l),
            0x36 => {
                self.set_byte(self.registers.hl(), self.next_u8());
                self.registers = RegisterState {
                    pc: self.registers.pc + 2,
                    ..self.registers
                }
            }
            _ => panic!("Unknown Instruction: {:02X}", self.curr_u8()),
        }
    }
}
