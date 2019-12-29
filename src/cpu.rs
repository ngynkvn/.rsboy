use crate::memory::Mem;
use crate::registers::flags;
use crate::registers::RegisterState;

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
    ($self:ident, READ_MEM, $r1:ident, $r2:ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: $self.read_byte($self.registers.$r2()),
            ..$self.registers
        }
    }};

    // LD (r1), r2, to MEM
    ($self:ident, LOAD_MEM, $r1:ident, $r2:ident) => {{
        $self.set_byte($self.registers.$r1(), $self.registers.$r2());
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};
}

macro_rules! LD16 {
    ($self:ident, IMMEDIATE, $r1:ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 3,
            $r1: $self.next_u16(),
            ..$self.registers
        }
    }};
    ($self:ident, IMMEDIATE, $r1:ident, $r2:ident) => {{
        let value = $self.next_u16();
        $self.registers = RegisterState {
            pc: $self.registers.pc + 3,
            $r1: (value >> 8) as u8,
            $r2: (value & 0x00FF) as u8,
            ..$self.registers
        }
    }};
}

macro_rules! XOR {
    ($self:ident, $r1:ident, $r2:ident) => {{
        let xor = $self.registers.$r1() ^ $self.registers.$r2();

        $self.registers = RegisterState {
            a: xor,
            f: flags(xor == 0, false, false, false),
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};
}

macro_rules! JP {
    ($self:ident, IMMEDIATE) => {
        $self.registers = RegisterState {
            pc: $self.next_u16(),
            ..$self.registers
        }
    };
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
    fn next_u16(&self) -> u16 {
        // Little endianess means LSB comes first.
        (self.memory[self.registers.pc + 2] as u16) << 8 | self.memory[self.registers.pc + 1] as u16
    }
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address]
    }
    fn set_byte(&mut self, address: u16, value: u8) {
        self.memory[address] = value;
    }
    pub fn read_instruction(&mut self) {
        println!(
            "opcode:{:02X}\nregisters:\n{}",
            self.curr_u8(),
            self.registers
        );
        match self.curr_u8() {
            0x00 => {
                self.registers = self.inc_pc(1);
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
            0x7E => LD!(self, READ_MEM, a, hl),
            0x40 => LD!(self, REGISTER, b, b),
            0x41 => LD!(self, REGISTER, b, c),
            0x42 => LD!(self, REGISTER, b, d),
            0x43 => LD!(self, REGISTER, b, e),
            0x44 => LD!(self, REGISTER, b, h),
            0x45 => LD!(self, REGISTER, b, l),
            0x46 => LD!(self, READ_MEM, b, hl),
            0x48 => LD!(self, REGISTER, c, b),
            0x49 => LD!(self, REGISTER, c, c),
            0x4A => LD!(self, REGISTER, c, d),
            0x4B => LD!(self, REGISTER, c, e),
            0x4C => LD!(self, REGISTER, c, h),
            0x4D => LD!(self, REGISTER, c, l),
            0x4E => LD!(self, READ_MEM, c, hl),
            0x50 => LD!(self, REGISTER, d, b),
            0x51 => LD!(self, REGISTER, d, c),
            0x52 => LD!(self, REGISTER, d, d),
            0x53 => LD!(self, REGISTER, d, e),
            0x54 => LD!(self, REGISTER, d, h),
            0x55 => LD!(self, REGISTER, d, l),
            0x56 => LD!(self, READ_MEM, d, hl),
            0x58 => LD!(self, REGISTER, e, b),
            0x59 => LD!(self, REGISTER, e, c),
            0x5A => LD!(self, REGISTER, e, d),
            0x5B => LD!(self, REGISTER, e, e),
            0x5C => LD!(self, REGISTER, e, h),
            0x5D => LD!(self, REGISTER, e, l),
            0x5E => LD!(self, READ_MEM, e, hl),
            0x60 => LD!(self, REGISTER, h, b),
            0x61 => LD!(self, REGISTER, h, c),
            0x62 => LD!(self, REGISTER, h, d),
            0x63 => LD!(self, REGISTER, h, e),
            0x64 => LD!(self, REGISTER, h, h),
            0x65 => LD!(self, REGISTER, h, l),
            0x66 => LD!(self, READ_MEM, h, hl),
            0x68 => LD!(self, REGISTER, l, b),
            0x69 => LD!(self, REGISTER, l, c),
            0x6A => LD!(self, REGISTER, l, d),
            0x6B => LD!(self, REGISTER, l, e),
            0x6C => LD!(self, REGISTER, l, h),
            0x6D => LD!(self, REGISTER, l, l),
            0x6E => LD!(self, READ_MEM, l, hl),
            0x70 => LD!(self, LOAD_MEM, hl, b),
            0x71 => LD!(self, LOAD_MEM, hl, c),
            0x72 => LD!(self, LOAD_MEM, hl, d),
            0x73 => LD!(self, LOAD_MEM, hl, e),
            0x74 => LD!(self, LOAD_MEM, hl, h),
            0x75 => LD!(self, LOAD_MEM, hl, l),
            0x36 => {
                self.set_byte(self.registers.hl(), self.next_u8());
                self.registers = self.inc_pc(2);
            }
            //3. LD A,n
            0x0A => LD!(self, READ_MEM, a, bc),
            0x1A => LD!(self, READ_MEM, a, de),
            // 0xFA =>  LD A, (nn) <-- THIS IS A SHORT
            0x3E => LD!(self, IMMEDIATE, a),

            0x47 => LD!(self, REGISTER, b, a),
            0x4F => LD!(self, REGISTER, c, a),
            0x57 => LD!(self, REGISTER, d, a),
            0x5F => LD!(self, REGISTER, e, a),
            0x67 => LD!(self, REGISTER, h, a),
            0x6F => LD!(self, REGISTER, l, a),
            0x02 => LD!(self, LOAD_MEM, bc, a),
            0x12 => LD!(self, LOAD_MEM, de, a),
            0x77 => LD!(self, LOAD_MEM, hl, a),
            // 0xEA => LD!(self, LOAD_MEM_IMMEDIATE, --, a),

            // 5
            0xF2 => {
                self.registers = RegisterState {
                    pc: self.registers.pc + 1,
                    a: self.read_byte(0xFF00 + self.registers.c() as u16),
                    ..self.registers
                }
            }
            // 6
            0xE2 => {
                self.set_byte(0xFF00 + self.registers.c() as u16, self.registers.a());
                self.registers = self.inc_pc(1);
            }

            // 9.
            0x3A => {
                LD!(self, READ_MEM, a, hl);
                self.registers = self.dec_hl();
            }

            // 12. LDD (HL), A
            0x32 => {
                LD!(self, LOAD_MEM, hl, a);
                self.registers = self.dec_hl();
            }

            // 14.
            0x2A => {
                LD!(self, READ_MEM, a, hl);
                self.registers = self.inc_hl();
            }

            // 18.
            0x22 => {
                LD!(self, LOAD_MEM, hl, a);
                self.registers = self.inc_hl();
            }

            0xAF => XOR!(self, a, a),
            0xA8 => XOR!(self, a, b),
            0xA9 => XOR!(self, a, c),
            0xAA => XOR!(self, a, d),
            0xAB => XOR!(self, a, e),
            0xAC => XOR!(self, a, h),
            0xAD => XOR!(self, a, l),

            0x01 => LD16!(self, IMMEDIATE, b, c),
            0x11 => LD16!(self, IMMEDIATE, d, e),
            0x21 => LD16!(self, IMMEDIATE, h, l),
            0x31 => LD16!(self, IMMEDIATE, sp),

            0xC3 => JP!(self, IMMEDIATE),


            _ => panic!("Unknown Instruction: {:02X}", self.curr_u8()),
        }
    }

    fn inc_hl(&self) -> RegisterState {
        let next_hl = self.registers.hl() + 1;
        RegisterState {
            pc: self.registers.pc + 1,
            h: (next_hl >> 8) as u8,
            l: (next_hl & 0x00FF) as u8,
            ..self.registers
        }
    }
    fn dec_hl(&self) -> RegisterState {
        let next_hl = self.registers.hl() - 1;
        RegisterState {
            pc: self.registers.pc + 1,
            h: (next_hl >> 8) as u8,
            l: (next_hl & 0x00FF) as u8,
            ..self.registers
        }
    }

    fn inc_pc(&self, n: u16) -> RegisterState {
        RegisterState {
            pc: self.registers.pc + n,
            ..self.registers
        }
    }
}
