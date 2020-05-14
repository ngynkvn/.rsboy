use crate::instructions::*;
use crate::bus::Bus;
use crate::registers::RegisterState;
use std::convert::TryInto;

const HISTORY_SIZE: usize = 10;

pub struct CPU {
    registers: RegisterState,
    pub clock: usize,
}

type CpuResult<T> = Result<T, String>;

impl CPU {
    pub fn new(skip_bios: bool) -> Self { // TODO
        Self {
            registers: RegisterState::new(skip_bios),
            clock: 0,
        }
    }
    fn next_u8(&mut self, bus: &mut Bus) -> u8 {
        self.clock += 1;
        bus.gpu.cycle().unwrap();
        let val = bus[(self.registers.pc)];
        self.registers.pc += 1;
        val
    }
    fn next_u16(&mut self, bus: &mut Bus) -> u16 {
        // Little endianess means LSB comes first.
        self.clock += 1;
        bus.gpu.cycle().unwrap();
        let val = (bus[self.registers.pc + 1] as u16) << 8 | (bus[self.registers.pc] as u16);
        self.registers.pc += 2;
        val
    }
    fn read_byte(&mut self, address: u16, bus: &mut Bus) -> u8 {
        self.clock += 1;
        bus.gpu.cycle().unwrap();
        bus[address]
    }
    fn read_io(&mut self, offset: u8, bus: &mut Bus) -> u8 {
        self.read_byte(0xFF00 + offset as u16, bus)
    }
    fn set_byte(&mut self, address: u16, value: u8, bus: &mut Bus) -> CpuResult<()> {
        self.clock += 1;
        bus.gpu.cycle().unwrap();
        bus[address] = value;
        Ok(())
    }

    fn read_location(&mut self, location: Location, bus: &mut Bus) -> u16 {
        match location {
            Location::Immediate(1) => self.next_u8(bus).into(),
            Location::Immediate(2) => self.next_u16(bus),
            Location::Immediate(_) => panic!(),
            Location::Register(r) => self.registers.fetch(r),
            Location::Memory(r) => self.read_byte(self.registers.fetch(r), bus).into(),
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                self.read_io(next, bus).into()
            }
            Location::MemOffsetRegister(r) => self
                .read_io(
                    self.registers
                        .fetch(r)
                        .try_into()
                        .expect("Offset was too large."),
                    bus,
                )
                .into(),
            _ => panic!()
        }
    }

    fn load(&mut self, into: Location, from: Location, bus: &mut Bus) -> CpuResult<()> {
        let from_value = self.read_location(from, bus);
        match into {
            Location::Immediate(_) => return Err(String::from("Tried to write into ROM?")),
            Location::Register(r) => {
                self.registers = self.registers.put(from_value, r)?;
            }
            Location::Memory(r) => match self.registers.get_dual_reg(r) {
                Some(address) => self.set_byte(address, from_value.try_into().unwrap(), bus)?,
                None => return Err(String::from("I tried to access a u8 as a bus address.")),
            },
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                self.set_byte(0xFF00 + next as u16, from_value.try_into().unwrap(), bus)?;
            }
            Location::MemOffsetRegister(r) => {
                let offset = self.registers.fetch_u8(r)?;
                self.set_byte(
                    0xFF00 + offset as u16,
                    from_value.try_into().unwrap(),
                    bus,
                )?;
            }
            Location::MemoryImmediate => {
                let addr = self.next_u16(bus);
                self.set_byte(addr, from_value.try_into().unwrap(), bus)?;
            }
        };
        Ok(())
    }

    fn push_stack(&mut self, value: u16, bus: &mut Bus) -> CpuResult<()> {
        let bytes = value.to_be_bytes();
        self.set_byte(self.registers.sp - 1, bytes[0], bus)?;
        self.set_byte(self.registers.sp, bytes[1], bus)?;
        self.registers.sp -= 2;
        Ok(())
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> CpuResult<u16> {
        let b1 = self.read_byte(self.registers.sp + 1, bus);
        let b2 = self.read_byte(self.registers.sp + 2, bus);
        self.registers.sp += 2;
        Ok(((b1 as u16) << 8) | b2 as u16)
    }

    fn dec(&mut self, r: Register) {
        self.registers = self.registers.dec(r);
    }

    fn inc(&mut self, r: Register) {
        self.registers = self.registers.inc(r);
    }

    fn inc_pc(&mut self) {
        self.registers.pc += 1;
    }
    
    fn perform_instruction(&mut self, instruction: Instr, instr_len: u16, curr_byte: u8, bus: &mut Bus) -> CpuResult<()> {
        match instruction {
            Instr::LD(into, from) => self.load(into, from, bus).or_else(|e| {
                Err(format!(
                    "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                    self.registers.pc - instr_len,
                    curr_byte,
                    instruction,
                    e
                ))
            }),
            Instr::LDD(into, from) => {
                self.load(into, from, bus).or_else(|e| {
                    Err(format!(
                        "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                        self.registers.pc - instr_len,
                        curr_byte,
                        instruction,
                        e
                    ))
                })?;
                self.dec(Register::HL);
                self.clock += 1; // TODO
                bus.gpu.cycle()?;
                Ok(())
            }
            Instr::LDI(into, from) => {
                self.load(into, from, bus).or_else(|e| {
                    Err(format!(
                        "LoadError: 0x{:04X}: 0x{:02X} {:?}, {:?}",
                        self.registers.pc - instr_len,
                        curr_byte,
                        instruction,
                        e
                    ))
                })?;
                self.inc(Register::HL);
                self.clock += 1; // TODO
                bus.gpu.cycle()?;
                Ok(())
            }
            Instr::NOOP => Ok(()),
            Instr::RST(size) => {
                self.push_stack(self.registers.pc, bus)?;
                self.registers.pc = size as u16;
                Ok(())
            }
            Instr::CP(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                self.registers = self.registers.cmp(value)?;
                Ok(())
            }
            Instr::ADD(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a.wrapping_add(value);
                Ok(())
            }
            Instr::XOR(location) => {
                let value :u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a ^ (value);
                self.registers.f =
                    crate::registers::flags(self.registers.a == 0, false, false, false);
                Ok(())
            }
            Instr::SUB(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a.wrapping_sub(value);
                Ok(())
            }
            Instr::CB => self.handle_cb(bus),
            Instr::JP(jump_type) => {
                let address = self.next_u16(bus);
                self.handle_jump(address, jump_type)
            }
            Instr::JR(jump_type) => {
                let offset = self.next_u8(bus) as i8;
                self.handle_jump(self.registers.pc.wrapping_add(offset as u16), jump_type)
            }
            Instr::CALL(jump_type) => {
                let address = self.next_u16(bus);
                self.push_stack(self.registers.pc, bus)?;
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
                self.push_stack(addr, bus)?;
                Ok(())
            }
            Instr::POP(Location::Register(r)) => {
                let addr = self.pop_stack(bus)?;
                self.registers = self.registers.put(addr, r)?;
                Ok(())
            }
            Instr::RET(jump_type) => {
                let addr = self.pop_stack(bus)?;
                self.handle_jump(addr, jump_type)
            }
            Instr::RotThruCarry(Direction::LEFT, Location::Register(r)) => {
                self.registers = self.registers.rot_thru_carry(r)?;
                Ok(())
            }
            Instr::RETI => {
                Ok(())
            }
            // y => {
            //     println!("Unknown OP: 0x{:04X}: {:?}", self.registers.pc - instr_len, y);
            //     Ok(())
            // }
            x => Err(format!(
                "read_instruction: 0x{:04X}: 0x{:02X} {:?}",
                self.registers.pc - instr_len,
                curr_byte,
                x
            )),
        }
    }

    fn read_instruction(&mut self, bus: &mut Bus) -> CpuResult<()> {
        let curr_byte = self.next_u8(bus);
        let instruction = &INSTR_TABLE[curr_byte as usize];
        let Instruction(size, _) = INSTRUCTION_TABLE[curr_byte as usize]; //Todo refactor this ugly thing
        let instr_len = size as u16 + 1;
        self.perform_instruction(*instruction, instr_len, curr_byte, bus)
    }
    fn check_flag(&mut self, flag: Flag) -> bool {
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
    fn handle_jump(&mut self, address: u16, jt: JumpType) -> CpuResult<()> {
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
    fn handle_cb(&mut self, bus: &mut Bus) -> CpuResult<()> {
        let opcode = self.next_u8(bus);
        match opcode {
            0x37 => self.registers = self.registers.swap_nibbles(Register::A)?,
            0x30 => self.registers = self.registers.swap_nibbles(Register::B)?,
            0x31 => self.registers = self.registers.swap_nibbles(Register::C)?,
            0x32 => self.registers = self.registers.swap_nibbles(Register::D)?,
            0x33 => self.registers = self.registers.swap_nibbles(Register::E)?,
            0x34 => self.registers = self.registers.swap_nibbles(Register::H)?,
            0x35 => self.registers = self.registers.swap_nibbles(Register::L)?,
            0x36 => {
                let address = self.registers.fetch_u16(Register::HL);
                let byte = self.read_byte(address, bus);
                let [hi, lo] = [byte >> 4, byte & 0xF];
                let new_byte = (lo << 4) | byte;
                self.set_byte(address, new_byte, bus)?;
            }
            0x78 => self.registers = self.registers.test_bit(Register::B, 7)?,
            0x79 => self.registers = self.registers.test_bit(Register::C, 7)?,
            0x7A => self.registers = self.registers.test_bit(Register::D, 7)?,
            0x7B => self.registers = self.registers.test_bit(Register::E, 7)?,
            0x7C => self.registers = self.registers.test_bit(Register::H, 7)?,
            0x7D => self.registers = self.registers.test_bit(Register::L, 7)?,
            0x17 => self.registers = self.registers.rot_thru_carry(Register::A)?,
            0x10 => self.registers = self.registers.rot_thru_carry(Register::B)?,
            0x11 => self.registers = self.registers.rot_thru_carry(Register::C)?,
            0x12 => self.registers = self.registers.rot_thru_carry(Register::D)?,
            0x13 => self.registers = self.registers.rot_thru_carry(Register::E)?,
            0x14 => self.registers = self.registers.rot_thru_carry(Register::H)?,
            0x15 => self.registers = self.registers.rot_thru_carry(Register::L)?,
            // 0x16 => {
            //     //rot_thru_carry (hl)
            // }
            _ => return Err(format!("CB: {:02X}", opcode)),
        }
        Ok(())
    }
    pub fn cycle<'a>(&mut self, bus: &mut Bus) -> CpuResult<usize> {
        let prev = self.clock;
        self.read_instruction(bus)?;
        Ok(self.clock - prev)
    }
}