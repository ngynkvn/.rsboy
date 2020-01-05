use crate::instructions::INSTRUCTION_TABLE;
use crate::memory::Memory;
use crate::registers::flags;
use crate::registers::RegisterState;

mod macros;
use self::macros::swap_nibbles;
use crate::{
    ADC, ADD, AND, CALL, CP, DEC, INC, JP, JR, LD, LD16, OR, POP, PUSH, ROT_THRU_CARRY, SUB, SWAP,
    TEST_BIT, XOR,
};

pub struct CPU {
    registers: RegisterState,
    pub clock: usize,
    pub memory: Memory,
    start_debug: bool,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: RegisterState::new(),
            clock: 0,
            memory: Memory::new(rom),
            start_debug: false,
        }
    }
    // TODO I'll clean these functions up later
    fn curr_u8(&mut self) -> u8 {
        self.clock += 1;
        self.memory[self.registers.pc]
    }
    fn next_u8(&mut self) -> u8 {
        self.clock += 1;
        self.memory[(self.registers.pc + 1)]
    }
    fn next_u16(&mut self) -> u16 {
        // Little endianess means LSB comes first.
        self.clock += 1;
        (self.memory[self.registers.pc + 2] as u16) << 8 | self.memory[self.registers.pc + 1] as u16
    }
    fn read_byte(&mut self, address: u16) -> u8 {
        self.clock += 1;
        self.memory[address]
    }
    fn read_io(&mut self, offset: u8) -> u8 {
        self.read_byte(0xFF00 + offset as u16)
    }
    fn set_byte(&mut self, address: u16, value: u8) {
        self.clock += 1;
        self.memory[address] = value;
    }

    fn debug_print_stack(&self) {
        print!("[");
        for i in ((self.registers.sp + 1) as usize)..0xFFFF {
            print!("{:08b},", self.memory[i as u16]);
        }
        println!("]");
    }

    // Returns the delta after processing an instruction.
    pub fn cycle(&mut self) -> usize {
        let prev = self.clock;
        self.read_instruction();
        self.clock - prev
    }

    pub fn read_instruction(&mut self) -> usize {
        // if self.registers.pc > 0x90 {
        //     println!(
        //         "opcode:{:02X}\n{:?}\nregisters:\n{}",
        //         self.curr_u8(),
        //         INSTRUCTION_TABLE[self.curr_u8() as usize],
        //         // 0
        //         self.registers
        //     );
        // }
        //**DEBUGG */
        // if self.curr_u8() == 0xCB {
        //     println!(
        //         "opcode:{:04X}\n{:?}\nregisters:\n{}",
        //         self.curr_u16(),
        //         INSTRUCTION_TABLE[self.curr_u8() as usize],
        //         // 0
        //         self.registers
        //     );
        // } else {
        //     println!(
        //         "opcode:{:02X}\n{:?}\nregisters:\n{}",
        //         self.curr_u8(),
        //         INSTRUCTION_TABLE[self.curr_u8() as usize],
        //         // 0
        //         self.registers
        //     );
        // }
        // match self.registers.pc {
        // 0x00|0x03|0x04|0x07|0x08|0x0a|0x0c|0x0f|0x11|0x13|0x14|0x15|0x16|0x18|0x19|0x1a|0x1c|0x1d|0x1f|0x21|0x24|0x27|0x28|0x2b|0x2e|0x2f|0x30|0x32|0x34|0x37|0x39|0x3a|0x3b|0x3c|0x3d|0x3e|0x40|0x42|0x45|0x48|0x4a|0x4b|0x4d|0x4e|0x4f|0x51|0x53|0x55|0x56|0x58|0x59|0x5b|0x5d|0x5f|0x60|0x62|0x64|0x66|0x68|0x6a|0x6b|0x6d|0x6e|0x70|0x72|0x73|0x74|0x76|0x78|0x7a|0x7c|0x7e|0x80|0x81|0x82|0x83|0x85|0x86|0x88|0x89|0x8b|0x8c|0x8e|0x8f|0x91|0x93|0x95|0x96|0x98|0x99|0x9b|0x9c|0x9d|0x9f|0xa0|0xa1|0xa3|0xa4|0xa5|0xa6|0xa7|0xe0|0xe3|0xe6|0xe7|0xe8|0xe9|0xeb|0xec|0xed|0xef|0xf1|0xf3|0xf4|0xf5|0xf6|0xf7|0xf9|0xfa|0xfc|0xfe => {

        // },
        // _ => panic!("{}", self.registers.pc)
        // }
        if self.memory.in_bios && self.registers.pc == 0x100 {
            self.memory.in_bios = false;
        }
        let prev = self.clock;
        // self.curr_u8() has a side effect so need to check pc first.
        // TODO remove side effect..
        // if self.registers.pc >= 0x100 {
        //     println!("{}", self.registers);
        //     self.memory.dump_io();
        //     println!("Finished!");
        //     return 0;
        // }
        let curr_u8 = self.curr_u8();
        // log::trace!("[REGISTERS]\n{}", self.registers);
        // println!("OP: {:?}\nPC: {:02X}\nHL: {:04X}", INSTRUCTION_TABLE[curr_u8 as usize], self.registers.pc, self.registers.hl());
        match curr_u8 {
            0x00 => {
                self.registers = self.inc_pc(1);
            }
            // 3.3.1. 8-bit Loads
            // 1 LD nn, n
            0x06 => LD!(self, IMMEDIATE, b),
            0x08 => LD16!(self, IMMEDIATE, sp),
            0x0E => LD!(self, IMMEDIATE, c),
            0x16 => LD!(self, IMMEDIATE, d),
            0x17 => ROT_THRU_CARRY!(self, LEFT, a),
            //JR n
            0x18 => {
                let n = self.next_u8() as i8;
                self.registers = RegisterState {
                    pc: ((self.registers.pc as u32 as i32) + (n as i32) + (2 as i32)) as u16,
                    ..self.registers
                }
            }
            0x1E => LD!(self, IMMEDIATE, e),
            0x26 => LD!(self, IMMEDIATE, h),
            0x2E => LD!(self, IMMEDIATE, l),
            0xC1 => POP!(self, b, c),
            0xE1 => POP!(self, h, l),
            0xF1 => POP!(self, a, f),

            0xA7 => AND!(self, self.registers.a, 1),
            0xA0 => AND!(self, self.registers.b, 1),
            0xA1 => AND!(self, self.registers.c, 1),
            0xA2 => AND!(self, self.registers.d, 1),
            0xA3 => AND!(self, self.registers.e, 1),
            0xA4 => AND!(self, self.registers.h, 1),
            0xA5 => AND!(self, self.registers.l, 1),
            0xA6 => AND!(self, self.read_byte(self.registers.hl()), 1),
            0xE6 => AND!(self, self.next_u8(), 2),
            // Playing around with some alternate syntax
            //
            0xB7 => OR!(self, self.registers.a, 1),
            0xB0 => OR!(self, self.registers.b, 1),
            0xB1 => OR!(self, self.registers.c, 1),
            0xB2 => OR!(self, self.registers.d, 1),
            0xB3 => OR!(self, self.registers.e, 1),
            0xB4 => OR!(self, self.registers.h, 1),
            0xB5 => OR!(self, self.registers.l, 1),
            0xB6 => OR!(self, self.read_byte(self.registers.hl()), 1),
            0xF6 => OR!(self, self.next_u8(), 2),

            0xAF => XOR!(self, self.registers.a, 1),
            0xA8 => XOR!(self, self.registers.b, 1),
            0xA9 => XOR!(self, self.registers.c, 1),
            0xAA => XOR!(self, self.registers.d, 1),
            0xAB => XOR!(self, self.registers.e, 1),
            0xAC => XOR!(self, self.registers.h, 1),
            0xAD => XOR!(self, self.registers.l, 1),
            0xAE => XOR!(self, self.read_byte(self.registers.hl()), 1),
            0xEE => XOR!(self, self.next_u8(), 2),
			
            0x8F => ADC!(self, self.registers.a, 1),
            0x88 => ADC!(self, self.registers.b, 1),
            0x89 => ADC!(self, self.registers.c, 1),
            0x8A => ADC!(self, self.registers.d, 1),
            0x8B => ADC!(self, self.registers.e, 1),
            0x8C => ADC!(self, self.registers.h, 1),
            0x8D => ADC!(self, self.registers.l, 1),
            // 0x8E => ADC!(self, (HL)),
            // 0xCE => ADC!(self, #),
            0x07 => ROT_THRU_CARRY!(self, LEFT, a),

            0xF3 => {
                println!("WARNING: NOT IMPLEMENTED: 0xF3 DISABLE INTERRUPTS");
                self.registers = self.inc_pc(1);
            }
            0xFB => {
                println!("WARNING: NOT IMPLEMENTED: 0xFB ENABLE INTERRUPTS");
                self.registers = self.inc_pc(1);
            }

            0xBF => CP!(self, a),
            0xB8 => CP!(self, b),
            0xB9 => CP!(self, c),
            0xBA => CP!(self, d),
            0xBB => CP!(self, e),
            0xBC => CP!(self, h),
            0xBD => CP!(self, l),
            0xBE => CP!(self, hl),
            0xFE => CP!(self, IMMEDIATE),

            0xF0 => {
                let offset = self.next_u8();
                self.registers = RegisterState {
                    a: self.read_io(offset),
                    pc: self.registers.pc + 2,
                    ..self.registers
                }
            }

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
                let value = self.next_u8();
                self.set_byte(self.registers.hl(), value);
                self.registers = self.inc_pc(2);
            }
			0x97 => SUB!(self, a),
			0x90 => SUB!(self, b),
			0x91 => SUB!(self, c),
			0x92 => SUB!(self, d),
			0x93 => SUB!(self, e),
			0x94 => SUB!(self, h),
			0x95 => SUB!(self, l),
			// 0x96 => SUB!(self, (HL)),
			0xD6 => SUB!(self, IMMEDIATE),

            0x87 => ADD!(self, a),
            0x80 => ADD!(self, b),
            0x81 => ADD!(self, c),
            0x82 => ADD!(self, d),
            0x83 => ADD!(self, e),
            0x84 => ADD!(self, h),
            0x85 => ADD!(self, l),
            0x86 => ADD!(self, hl),
            0xC6 => ADD!(self, IMMEDIATE),

            //3. LD A,n
            0x0A => LD!(self, READ_MEM, a, bc),
            0x1A => LD!(self, READ_MEM, a, de),
            0xFA => {
                let addr = self.next_u16();
                let value = self.read_byte(addr);
                self.registers = RegisterState {
                    pc: self.registers.pc + 3,
                    a: value,
                    ..self.registers
                }
            }
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
            0xEA => {
                let addr = self.next_u16();
                self.set_byte(addr, self.registers.a);
                self.registers = RegisterState {
                    pc: self.registers.pc + 3,
                    ..self.registers
                }
            }
            0xE0 => LD!(self, LOAD_MEM_OFFSET, a),

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

            0x03 => INC!(self, NN, b, c),
            0x13 => INC!(self, NN, d, e),
            0x23 => INC!(self, NN, h, l),
            0x33 => INC!(self, NN, sp),

            0x01 => LD16!(self, IMMEDIATE, b, c),
            0x11 => LD16!(self, IMMEDIATE, d, e),
            0x21 => LD16!(self, IMMEDIATE, h, l),
            0x31 => LD16!(self, IMMEDIATE, sp),

            0xC9 => {
                let addr = self.pop_u16();
                self.registers = RegisterState {
                    pc: addr,
                    ..self.registers
                };
                // println!("I RETURNED HERE {}",self.registers);
            }
            0xC3 => JP!(self, IMMEDIATE),

            0x20 => JR!(self, IF, flg_nz),
            0x28 => JR!(self, IF, flg_z),
            0x30 => JR!(self, IF, flg_nc),
            0x38 => JR!(self, IF, flg_c),

            0x3C => INC!(self, a),
            0x04 => INC!(self, b),
            0x0C => INC!(self, c),
            0x14 => INC!(self, d),
            0x1C => INC!(self, e),
            0x24 => INC!(self, h),
            0x2C => INC!(self, l),
            0x34 => INC!(self, hl),

            0x3D => DEC!(self, a),
            0x05 => DEC!(self, b),
            0x0D => DEC!(self, c),
            0x0B => DEC!(self, bc, b, c), //TODO fix hack
            0x15 => DEC!(self, d),
            0x1D => DEC!(self, e),
            0x25 => DEC!(self, h),
            0x2D => DEC!(self, l),
            0x35 => DEC!(self, hl),

            //CALL
            0xCD => CALL!(self),
            0xC4 => CALL!(self, flg_nz),
            0xCC => CALL!(self, flg_nz),
            0xD4 => CALL!(self, flg_nz),
            0xDC => CALL!(self, flg_nz),

            0xF5 => PUSH!(self, af),
            0xC5 => PUSH!(self, bc),
            0xD5 => PUSH!(self, de),
            0xE5 => PUSH!(self, hl),

            //RST
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                self.push_stack(self.registers.pc);
                self.registers = RegisterState {
                    pc: (self.curr_u8() - 0xC7) as u16,
                    ..self.registers
                }
            }

            0xCB => {
                match self.next_u8() {
                    0x37 => SWAP!(self, a),
                    0x30 => SWAP!(self, b),
                    0x31 => SWAP!(self, c),
                    0x32 => SWAP!(self, d),
                    0x33 => SWAP!(self, e),
                    0x34 => SWAP!(self, h),
                    0x35 => SWAP!(self, l),
                    0x36 => SWAP!(self, hl),
					0x3F => ROT_THRU_CARRY!(self, RIGHT, a),
					0x38 => ROT_THRU_CARRY!(self, RIGHT, b),
					0x39 => ROT_THRU_CARRY!(self, RIGHT, c),
					0x3A => ROT_THRU_CARRY!(self, RIGHT, d),
					0x3B => ROT_THRU_CARRY!(self, RIGHT, e),
					0x3C => ROT_THRU_CARRY!(self, RIGHT, h),
					0x3D => ROT_THRU_CARRY!(self, RIGHT, l),
					// 0x3E => ROT_THRU_CARRY!(self, RIGHT, hl),
                    0x17 => ROT_THRU_CARRY!(self, LEFT, a),
                    0x10 => ROT_THRU_CARRY!(self, LEFT, b),
                    0x11 => ROT_THRU_CARRY!(self, LEFT, c),
                    0x12 => ROT_THRU_CARRY!(self, LEFT, d),
                    0x13 => ROT_THRU_CARRY!(self, LEFT, e),
                    0x14 => ROT_THRU_CARRY!(self, LEFT, h),
                    0x15 => ROT_THRU_CARRY!(self, LEFT, l),
                    0x40 => TEST_BIT!(self, b, 0),
                    0x41 => TEST_BIT!(self, c, 0),
                    0x42 => TEST_BIT!(self, d, 0),
                    0x43 => TEST_BIT!(self, e, 0),
                    0x44 => TEST_BIT!(self, h, 0),
                    0x45 => TEST_BIT!(self, l, 0),
                    // 0x46 => TEST_BIT!(self, hl, 0),
                    // 0x47 => TEST_BIT!(self, a, 0),
                    0x48 => TEST_BIT!(self, b, 1),
                    0x49 => TEST_BIT!(self, c, 1),
                    0x4A => TEST_BIT!(self, d, 1),
                    0x4B => TEST_BIT!(self, e, 1),
                    0x4C => TEST_BIT!(self, h, 1),
                    0x4D => TEST_BIT!(self, l, 1),
                    // 0x4E => TEST_BIT!(self),
                    // 0x4F => TEST_BIT!(self),
                    0x50 => TEST_BIT!(self, b, 2),
                    0x51 => TEST_BIT!(self, c, 2),
                    0x52 => TEST_BIT!(self, d, 2),
                    0x53 => TEST_BIT!(self, e, 2),
                    0x54 => TEST_BIT!(self, h, 2),
                    0x55 => TEST_BIT!(self, l, 2),
                    // 0x56 => TEST_BIT!(self),
                    // 0x57 => TEST_BIT!(self),
                    0x58 => TEST_BIT!(self, b, 3),
                    0x59 => TEST_BIT!(self, c, 3),
                    0x5A => TEST_BIT!(self, d, 3),
                    0x5B => TEST_BIT!(self, e, 3),
                    0x5C => TEST_BIT!(self, h, 3),
                    0x5D => TEST_BIT!(self, l, 3),
                    // 0x5E => TEST_BIT!(self),
                    // 0x5F => TEST_BIT!(self),
                    0x60 => TEST_BIT!(self, b, 4),
                    0x61 => TEST_BIT!(self, c, 4),
                    0x62 => TEST_BIT!(self, d, 4),
                    0x63 => TEST_BIT!(self, e, 4),
                    0x64 => TEST_BIT!(self, h, 4),
                    0x65 => TEST_BIT!(self, l, 4),
                    // 0x66 => TEST_BIT!(self),
                    // 0x67 => TEST_BIT!(self),
                    0x68 => TEST_BIT!(self, b, 5),
                    0x69 => TEST_BIT!(self, c, 5),
                    0x6A => TEST_BIT!(self, d, 5),
                    0x6B => TEST_BIT!(self, e, 5),
                    0x6C => TEST_BIT!(self, h, 5),
                    0x6D => TEST_BIT!(self, l, 5),
                    // 0x6E => TEST_BIT!(self),
                    // 0x6F => TEST_BIT!(self),
                    0x70 => TEST_BIT!(self, b, 6),
                    0x71 => TEST_BIT!(self, c, 6),
                    0x72 => TEST_BIT!(self, d, 6),
                    0x73 => TEST_BIT!(self, e, 6),
                    0x74 => TEST_BIT!(self, h, 6),
                    0x75 => TEST_BIT!(self, l, 6),
                    // 0x76 => TEST_BIT!(self),
                    // 0x77 => TEST_BIT!(self),
                    0x78 => TEST_BIT!(self, b, 7),
                    0x79 => TEST_BIT!(self, c, 7),
                    0x7A => TEST_BIT!(self, d, 7),
                    0x7B => TEST_BIT!(self, e, 7),
                    0x7C => TEST_BIT!(self, h, 7),
                    0x7D => TEST_BIT!(self, l, 7),
                    // 0x7E => TEST_BIT!(self),
                    // 0x7F => TEST_BIT!(self),
                    _ => panic!("Unknown CB Instruction: {:02X}", self.next_u8()),
                };
                self.registers = self.inc_pc(1)
            }
            _ => panic!(
                "Unknown Instruction: {:02X}\n{:?}\n{}",
                self.curr_u8(),
                INSTRUCTION_TABLE[self.curr_u8() as usize],
                self.registers
            ),
        };
        if curr_u8 == 0xCB {
            log::trace!(
                "[CLOCK] Cycle for Instr CB {:02X} was {}",
                self.next_u8(),
                self.clock - 1 - prev
            );
            // -1 since next_u8 has a side effect. TODO Fix side effect
            self.clock -= 1;
        } else {
            log::trace!(
                "[CLOCK] Cycle for Instr {:02X} was {}",
                curr_u8,
                self.clock - prev
            );
        }
        self.clock
    }
    // Just guessing for now but I guess just take the value, write the 2 bytes and subtract 2 from SP?
    fn push_stack(&mut self, value: u16) {
        self.set_byte(self.registers.sp, (value >> 8) as u8);
        self.set_byte(self.registers.sp - 1, (value & 0x00FF) as u8);
        self.registers = RegisterState {
            sp: self.registers.sp - 2,
            ..self.registers
        };
        self.clock += 1;
        // log::info!("[STACK_PUSH] Pushed {} at PC: {:02X}", value, self.registers.pc);
    }

    fn pop_u16(&mut self) -> u16 {
        let n = ((self.read_byte(self.registers.sp + 2) as u16) << 8)
            | self.read_byte(self.registers.sp + 1) as u16;
        self.registers = RegisterState {
            sp: self.registers.sp + 2,
            ..self.registers
        };
        // log::info!("[STACK_POP] Popped {} at PC: {:02X}", n, self.registers.pc);
        n
    }

    fn inc_hl(&self) -> RegisterState {
        let next_hl = self.registers.hl() + 1;
        RegisterState {
            h: (next_hl >> 8) as u8,
            l: (next_hl & 0x00FF) as u8,
            ..self.registers
        }
    }
    fn dec_hl(&self) -> RegisterState {
        let next_hl = self.registers.hl() - 1;
        RegisterState {
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
