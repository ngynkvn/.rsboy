use crate::instructions::*;
use crate::memory::Memory;
use crate::registers::RegisterState;
use std::convert::TryInto;

const HISTORY_SIZE: usize = 10;

pub struct CPU {
    registers: RegisterState,
    pub clock: usize,
}

type CpuResult<T> = Result<T, String>;

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: RegisterState::new(),
            clock: 0,
        }
    }
    fn next_u8(&mut self, memory: &mut Memory) -> u8 {
        self.clock += 1;
        memory.gpu.cycle().unwrap();
        let val = memory[(self.registers.pc)];
        self.registers.pc += 1;
        val
    }
    fn next_u16(&mut self, memory: &mut Memory) -> u16 {
        // Little endianess means LSB comes first.
        self.clock += 1;
        memory.gpu.cycle().unwrap();
        let val = (memory[self.registers.pc + 1] as u16) << 8 | (memory[self.registers.pc] as u16);
        self.registers.pc += 2;
        val
    }
    fn read_byte(&mut self, address: u16, memory: &mut Memory) -> u8 {
        self.clock += 1;
        memory.gpu.cycle().unwrap();
        memory[address]
    }
    fn read_io(&mut self, offset: u8, memory: &mut Memory) -> u8 {
        self.read_byte(0xFF00 + offset as u16, memory)
    }
    fn set_byte(&mut self, address: u16, value: u8, memory: &mut Memory) -> CpuResult<()> {
        self.clock += 1;
        memory.gpu.cycle().unwrap();
        if address == 0xff47 {
            println!("{}", self.registers);
            return Err(String::from(""));
        }
        memory[address] = value;
        Ok(())
    }

    fn read_location(&mut self, location: &Location, memory: &mut Memory) -> u16 {
        match location {
            Location::Immediate(1) => self.next_u8(memory).into(),
            Location::Immediate(2) => self.next_u16(memory),
            Location::Immediate(_) => panic!(),
            Location::Register(r) => self.registers.fetch(r),
            Location::Memory(r) => self.read_byte(self.registers.fetch(r), memory).into(),
            Location::MemOffsetImm => {
                let next = self.next_u8(memory);
                self.read_io(next, memory).into()
            }
            Location::MemOffsetRegister(r) => self
                .read_io(
                    self.registers
                        .fetch(r)
                        .try_into()
                        .expect("Offset was too large."),
                    memory,
                )
                .into(),
            _ => panic!()
        }
    }

    fn load(&mut self, into: &Location, from: &Location, memory: &mut Memory) -> CpuResult<()> {
        let from_value = self.read_location(from, memory);
        match into {
            Location::Immediate(_) => return Err(String::from("Tried to write into ROM?")),
            Location::Register(r) => {
                self.registers = self.registers.put(from_value, r)?;
            }
            Location::Memory(r) => match self.registers.get_dual_reg(r) {
                Some(address) => self.set_byte(address, from_value.try_into().unwrap(), memory)?,
                None => return Err(String::from("I tried to access a u8 as a memory address.")),
            },
            Location::MemOffsetImm => {
                let next = self.next_u8(memory);
                self.set_byte(0xFF00 + next as u16, from_value.try_into().unwrap(), memory)?;
            }
            Location::MemOffsetRegister(r) => {
                let offset = self.registers.fetch_u8(r)?;
                self.set_byte(
                    0xFF00 + offset as u16,
                    from_value.try_into().unwrap(),
                    memory,
                )?;
            }
            Location::MemoryImmediate => {
                let addr = self.next_u16(memory);
                self.set_byte(addr, from_value.try_into().unwrap(), memory)?;
            }
        };
        Ok(())
    }

    fn push_stack(&mut self, value: u16, memory: &mut Memory) -> CpuResult<()> {
        let bytes = value.to_be_bytes();
        self.set_byte(self.registers.sp - 1, bytes[0], memory)?;
        self.set_byte(self.registers.sp, bytes[1], memory)?;
        self.registers.sp -= 2;
        Ok(())
    }

    fn pop_stack(&mut self, memory: &mut Memory) -> CpuResult<u16> {
        let b1 = self.read_byte(self.registers.sp + 1, memory);
        let b2 = self.read_byte(self.registers.sp + 2, memory);
        self.registers.sp += 2;
        Ok(((b1 as u16) << 8) | b2 as u16)
    }

    fn dec(&mut self, r: &Register) {
        self.registers = self.registers.dec(r);
    }

    fn inc(&mut self, r: &Register) {
        self.registers = self.registers.inc(r);
    }

    fn inc_pc(&mut self) {
        self.registers.pc += 1;
    }

    fn read_instruction(&mut self, memory: &mut Memory) -> CpuResult<()> {
        let curr_byte = self.next_u8(memory);
        let instruction = &INSTR_TABLE[curr_byte as usize];
        let Instruction(size, _) = INSTRUCTION_TABLE[curr_byte as usize]; //Todo refactor this ugly thing
        let instr_len = size as u16 + 1;
        // println!(
        //     "0x{:04X}: 0x{:02X} {:?}",
        //     self.registers.pc - 1,
        //     curr_byte,
        //     instruction
        // );
        match instruction {
            Instr::LD(into, from) => self.load(into, from, memory).or_else(|e| {
                Err(format!(
                    "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                    self.registers.pc - instr_len,
                    curr_byte,
                    instruction,
                    e
                ))
            }),
            Instr::LDD(into, from) => {
                self.load(into, from, memory).or_else(|e| {
                    Err(format!(
                        "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                        self.registers.pc - instr_len,
                        curr_byte,
                        instruction,
                        e
                    ))
                })?;
                self.dec(&Register::HL);
                self.clock += 1; // TODO
                memory.gpu.cycle()?;
                Ok(())
            }
            Instr::LDI(into, from) => {
                self.load(into, from, memory).or_else(|e| {
                    Err(format!(
                        "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                        self.registers.pc - instr_len,
                        curr_byte,
                        instruction,
                        e
                    ))
                })?;
                self.inc(&Register::HL);
                self.clock += 1; // TODO
                memory.gpu.cycle()?;
                Ok(())
            }
            Instr::NOOP => Ok(()),
            Instr::RST(size) => {
                self.push_stack(self.registers.pc, memory)?;
                self.registers.pc = *size as u16;
                Ok(())
            }
            Instr::CP(location) => {
                let value = self.read_location(location, memory).try_into().unwrap();
                self.registers = self.registers.cmp(value)?;
                Ok(())
            }
            Instr::ADD(Location::Register(r)) => {
                let value = self.registers.fetch_u8(r)?;
                self.registers.a = self.registers.a.wrapping_add(value as u8);
                Ok(())
            }
            Instr::XOR(Location::Register(r)) => {
                let value = self.registers.fetch_u8(r)?;
                self.registers.a = self.registers.a ^ (value as u8);
                self.registers.f =
                    crate::registers::flags(self.registers.a == 0, false, false, false);
                Ok(())
            }
            Instr::SUB(Location::Register(r)) => {
                let value = self.registers.fetch_u8(r)?;
                self.registers.a = self.registers.a.wrapping_sub(value);
                Ok(())
            }
            Instr::CB => self.handle_cb(memory),
            Instr::JP(jump_type) => {
                let address = self.next_u16(memory);
                self.handle_jump(address, jump_type)
            }
            Instr::JR(jump_type) => {
                let offset = self.next_u8(memory) as i8;
                self.handle_jump(self.registers.pc.wrapping_add(offset as u16), jump_type)
            }
            Instr::CALL(jump_type) => {
                let address = self.next_u16(memory);
                self.push_stack(self.registers.pc, memory)?;
                self.handle_jump(address, jump_type)
            }
            Instr::DEC(Location::Register(r)) => {
                self.dec(r);
                Ok(())
            }
            Instr::INC(Location::Register(r)) => {
                self.inc(r);
                Ok(())
            }
            Instr::PUSH(Location::Register(r)) => {
                let addr = self.registers.fetch_u16(r);
                self.push_stack(addr, memory)?;
                Ok(())
            }
            Instr::POP(Location::Register(r)) => {
                let addr = self.pop_stack(memory)?;
                self.registers = self.registers.put(addr, r)?;
                Ok(())
            }
            Instr::RET(jump_type) => {
                let addr = self.pop_stack(memory)?;
                self.handle_jump(addr, jump_type)
            }
            Instr::RotThruCarry(Direction::LEFT, Location::Register(r)) => {
                self.registers = self.registers.rot_thru_carry(r)?;
                Ok(())
            }
            x => Err(format!(
                "read_instruction: 0x{:04X}: 0x{:02X} {:?}",
                self.registers.pc - instr_len,
                curr_byte,
                x
            )),
        }
    }
    fn check_flag(&mut self, flag: &Flag) -> bool {
        match flag {
            Flag::FlagC => self.registers.flg_c(),
            Flag::FlagNC => self.registers.flg_nc(),
            Flag::FlagZ => self.registers.flg_z(),
            Flag::FlagNZ => self.registers.flg_nz(),
        }
    }

    fn jump(&mut self, address: u16) -> CpuResult<()> {
        self.registers = self.registers.jump(address)?;
        Ok(())
    }
    fn handle_jump(&mut self, address: u16, jt: &JumpType) -> CpuResult<()> {
        match jt {
            JumpType::If(flag) => {
                if self.check_flag(flag) {
                    self.registers = self.registers.jump(address)?;
                }
            }
            JumpType::Always => {
                self.registers = self.registers.jump(address)?;
            }
            JumpType::To(x) => {
                return Err(format!("{:?}", x));
            }
        }
        Ok(())
    }
    fn handle_cb(&mut self, memory: &mut Memory) -> CpuResult<()> {
        let opcode = self.next_u8(memory);
        match opcode {
            0x37 => self.registers = self.registers.swap_nibbles(&Register::A)?,
            0x30 => self.registers = self.registers.swap_nibbles(&Register::B)?,
            0x31 => self.registers = self.registers.swap_nibbles(&Register::C)?,
            0x32 => self.registers = self.registers.swap_nibbles(&Register::D)?,
            0x33 => self.registers = self.registers.swap_nibbles(&Register::E)?,
            0x34 => self.registers = self.registers.swap_nibbles(&Register::H)?,
            0x35 => self.registers = self.registers.swap_nibbles(&Register::L)?,
            0x36 => {
                let address = self.registers.fetch_u16(&Register::HL);
                let byte = self.read_byte(address, memory);
                let [hi, lo] = [byte >> 4, byte & 0xF];
                let new_byte = (lo << 4) | byte;
                self.set_byte(address, new_byte, memory)?;
            }
            0x78 => self.registers = self.registers.test_bit(&Register::B, 7)?,
            0x79 => self.registers = self.registers.test_bit(&Register::C, 7)?,
            0x7A => self.registers = self.registers.test_bit(&Register::D, 7)?,
            0x7B => self.registers = self.registers.test_bit(&Register::E, 7)?,
            0x7C => self.registers = self.registers.test_bit(&Register::H, 7)?,
            0x7D => self.registers = self.registers.test_bit(&Register::L, 7)?,
            0x17 => self.registers = self.registers.rot_thru_carry(&Register::A)?,
            0x10 => self.registers = self.registers.rot_thru_carry(&Register::B)?,
            0x11 => self.registers = self.registers.rot_thru_carry(&Register::C)?,
            0x12 => self.registers = self.registers.rot_thru_carry(&Register::D)?,
            0x13 => self.registers = self.registers.rot_thru_carry(&Register::E)?,
            0x14 => self.registers = self.registers.rot_thru_carry(&Register::H)?,
            0x15 => self.registers = self.registers.rot_thru_carry(&Register::L)?,
            // 0x16 => {
            //     //rot_thru_carry (hl)
            // }
            _ => return Err(format!("CB: {:02X}", opcode)),
        }
        Ok(())
    }
    pub fn cycle<'a>(&mut self, memory: &mut Memory) -> CpuResult<usize> {
        let prev = self.clock;
        self.read_instruction(memory)?;
        Ok(self.clock - prev)
    }
}