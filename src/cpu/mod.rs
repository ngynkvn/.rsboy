use crate::bus::Bus;
use crate::bus::Memory;
use crate::instructions::Register::*;
use crate::instructions::*;
use crate::registers::RegisterState;
use log::info;
use std::collections::HashMap;
use std::convert::TryInto;

const HISTORY_SIZE: usize = 20;

pub struct CPU {
    pub registers: RegisterState,
    debug: bool,
    pub running: bool,
    pub clock: usize,
    encounter: HashMap<u16, usize>,
    trace: [u16; HISTORY_SIZE],
    trace_ptr: usize,
}

type CpuResult<T> = Result<T, String>;

fn propagate_error(f: String) -> CpuResult<()> {
    Err(f)
}

fn prefix(prefix: String) -> impl FnOnce(String) -> Result<(), String> {
    move |e| Err(format!("{:?}: {:?}", prefix, e))
}

fn source_error<T>(e: String) -> Result<T, String> {
    Err(format!("{}:{}:{}: {:?}", file!(), line!(), column!(), e))
}

#[inline]
fn swapped_nibbles(byte: u8) -> u8 {
    let [hi, lo] = [byte >> 4, byte & 0xF];
    (lo << 4) | hi
}

macro_rules! source_error {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

impl CPU {
    pub fn new() -> Self {
        // TODO
        Self {
            registers: RegisterState::new(),
            debug: false,
            clock: 0,
            running: true,
            encounter: HashMap::new(),
            trace: [0; HISTORY_SIZE],
            trace_ptr: 0,
        }
    }
    fn next_u8(&mut self, bus: &mut Bus) -> u8 {
        self.tick(bus);
        let val = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        val
    }
    fn next_u16(&mut self, bus: &mut Bus) -> u16 {
        // Little endianess means LSB comes first.
        self.tick(bus);
        let lo = self.next_u8(bus);
        let hi = self.next_u8(bus);
        u16::from_le_bytes([lo, hi])
    }
    fn read_byte(&mut self, address: u16, bus: &mut Bus) -> u8 {
        self.tick(bus);
        bus.read(address)
    }
    fn read_io(&mut self, offset: u16, bus: &mut Bus) -> u8 {
        self.read_byte(0xFF00 + offset, bus)
    }
    fn set_byte(&mut self, address: u16, value: u8, bus: &mut Bus) -> CpuResult<()> {
        self.tick(bus);
        bus.write(address, value);
        Ok(())
    }

    fn read_from(&mut self, location: Location, bus: &mut Bus) -> u16 {
        match location {
            Location::Immediate(1) => self.next_u8(bus).into(),
            Location::Immediate(2) => self.next_u16(bus),
            Location::Immediate(_) => panic!(),
            Location::MemoryImmediate => {
                let address = self.next_u16(bus);
                self.read_byte(address, bus).into()
            }
            Location::Register(r) => self.registers.fetch(r),
            Location::Memory(r) => self.read_byte(self.registers.fetch(r), bus).into(),
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                self.read_io(next as u16, bus).into()
            }
            Location::MemOffsetRegister(r) => self.read_io(self.registers.fetch(r), bus).into(),
        }
    }

    fn read_location(&mut self, location: Location, bus: &mut Bus) -> u16 {
        self.read_from(location, bus)
    }

    fn write_into(&mut self, into: Location, from_value: u16, bus: &mut Bus) -> CpuResult<()> {
        let get_u8 = || -> Result<u8, String> {
            from_value
                .try_into()
                .or_else(|_| Err(format!("into: {:?}, value: {}", into, from_value)))
        };
        match into {
            Location::Immediate(2) => {
                let address = self.next_u16(bus);
                let value = get_u8()?;
                self.set_byte(address, value, bus)?;
            }
            Location::Register(r) => {
                self.registers = self.registers.put(from_value, r)?;
            }
            Location::Memory(r) => match self.registers.get_dual_reg(r) {
                Some(address) => {
                    let value = get_u8()?;
                    self.set_byte(address, value, bus)?
                }
                None => return Err(String::from("I tried to access a u8 as a bus address.")),
            },
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                let value = get_u8()?;
                self.set_byte(0xFF00 + next as u16, value, bus)?;
            }
            Location::MemOffsetRegister(r) => {
                let offset = self.registers.fetch_u8(r)?;
                let value = get_u8()?;
                self.set_byte(0xFF00 + offset as u16, value, bus)?;
            }
            Location::MemoryImmediate => {
                let address = self.next_u16(bus);
                if let Ok(value) = get_u8() {
                    self.set_byte(address, value, bus)?;
                } else {
                    let [lo, hi] = from_value.to_be_bytes();
                    self.set_byte(address, lo, bus)?;
                    self.set_byte(address + 1, hi, bus)?;
                }
            }
            _ => unimplemented!("{:?}", into),
        };
        Ok(())
    }

    fn tick(&mut self, bus: &mut Bus) {
        self.clock += 1;
        bus.cycle();
    }

    fn load(&mut self, into: Location, from: Location, bus: &mut Bus) -> CpuResult<()> {
        let from_value = self.read_location(from, bus);
        self.write_into(into, from_value, bus)
    }

    fn push_stack(&mut self, value: u16, bus: &mut Bus) -> CpuResult<()> {
        let [lo, hi] = value.to_le_bytes();
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.set_byte(self.registers.sp, hi, bus)?;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.set_byte(self.registers.sp, lo, bus)?;
        Ok(())
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> CpuResult<u16> {
        let lo = self.read_byte(self.registers.sp, bus);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let hi = self.read_byte(self.registers.sp, bus);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        Ok(u16::from_le_bytes([lo, hi]))
    }
    fn peek_stack(&mut self, bus: &mut Bus) -> u16 {
        let lo = bus.memory[self.registers.sp as usize];
        let hi = bus.memory[(self.registers.sp + 1) as usize];
        u16::from_le_bytes([lo, hi])
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
    // if (!n_flag) {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
    //   if (c_flag || a > 0x99) { a += 0x60; c_flag = 1; }
    //   if (h_flag || (a & 0x0f) > 0x09) { a += 0x6; }
    // } else {  // after a subtraction, only adjust if (half-)carry occurred
    //   if (c_flag) { a -= 0x60; }
    //   if (h_flag) { a -= 0x6; }
    // }
    // // these flags are always updated
    // z_flag = (a == 0); // the usual z flag
    // h_flag = 0; // h flag is always cleared
    fn bcd_adjust(&mut self, value: u8) -> u8 {
        let mut value = value;
        if self.registers.flg_nn() {
            if self.registers.flg_c() || value > 0x99 {
                value = value.wrapping_add(0x60);
                self.registers.set_cf(true);
            }
            if self.registers.flg_h() || (value & 0x0F) > 0x09 {
                value = value.wrapping_add(0x6);
            }
        } else {
            if self.registers.flg_c() || value > 0x99 {
                value = value.wrapping_sub(0x60);
            }
            if self.registers.flg_h() || (value & 0x0F) > 0x09 {
                value = value.wrapping_sub(0x6);
            }
        }
        self.registers.set_zf(value == 0);
        self.registers.set_hf(false);
        value
    }

    fn perform_instruction(&mut self, instruction: Instr, bus: &mut Bus) -> CpuResult<()> {
        match instruction {
            Instr::LD(into, from) => self.load(into, from, bus).or_else(source_error),
            Instr::LDD(into, from) => {
                self.load(into, from, bus).or_else(source_error)?;
                self.dec(Register::HL);
                self.tick(bus);
                Ok(())
            }
            Instr::LDI(into, from) => {
                self.load(into, from, bus).or_else(source_error)?;
                self.inc(Register::HL);
                self.tick(bus);
                Ok(())
            }
            Instr::LDSP => {
                let offset = self.next_u8(bus) as i8 as u16;
                let result = self.registers.sp.wrapping_add(offset);
                let half_carry = (self.registers.sp & 0x0F).wrapping_add(offset & 0x0F) > 0x0F;
                let carry = (self.registers.sp & 0xFF).wrapping_add(offset & 0xFF) > 0xFF;
                self.registers.set_zf(false);
                self.registers.set_nf(false);
                self.registers.set_hf(half_carry);
                self.registers.set_cf(carry);
                Ok(())
            }
            Instr::STOP => panic!("STOP: {:04x}", self.registers.pc - 1), // TODO ?
            Instr::NOOP => Ok(()),
            Instr::RST(size) => {
                if size == 0x38 {
                    panic!("0xFF hit")
                }
                self.push_stack(self.registers.pc, bus)?;
                self.registers.pc = size as u16;
                Ok(())
            }
            Instr::CP(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                self.registers.set_zf(self.registers.a == value);
                self.registers.set_nf(true);
                //https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu.rs#L156
                self.registers
                    .set_hf((self.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0);
                self.registers.set_cf(self.registers.a < value);
                Ok(())
            }
            Instr::ADD(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                let (result, carry) = self.registers.a.overflowing_add(value);
                //https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#L55
                let half_carry = (self.registers.a & 0x0f)
                    .checked_add(value | 0xf0)
                    .is_none();
                self.registers.a = result;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(half_carry);
                self.registers.set_cf(carry);
                Ok(())
            }
            Instr::SUB(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a.wrapping_sub(value);
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(true);
                self.registers.set_hf(
                    // Mooneye
                    (self.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
                );
                self.registers
                    .set_cf((self.registers.a as u16) < (value as u16));
                Ok(())
            }
            Instr::ADC(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                let carry = self.registers.flg_c() as u8;
                let result = self.registers.a.wrapping_add(value).wrapping_add(carry);
                self.registers.a = result;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                // Maybe: See https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#L55
                self.registers
                    .set_hf((self.registers.a & 0xf) + (value & 0xf) + carry > 0xf);
                self.registers
                    .set_cf(self.registers.a as u16 + value as u16 + carry as u16 > 0xff);
                Ok(())
            }
            Instr::ADDHL(location) => {
                let old = self.registers.hl();
                let value = self.read_location(location, bus);
                let (value, overflow) = old.overflowing_add(value);
                let [h, l] = value.to_be_bytes();
                self.registers.h = h;
                self.registers.l = l;
                self.registers.set_nf(false);
                self.registers
                    .set_hf((old & 0x0fff + value & 0x0fff) & 0x0800 == 0x0800);
                self.registers.set_cf(overflow);
                Ok(())
            }
            Instr::AND(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a &= value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(true);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::XOR(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a ^= value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::OR(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a |= value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::NOT(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = !value;
                self.registers.set_nf(true);
                self.registers.set_hf(true);
                Ok(())
            }
            Instr::CCF => {
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(!self.registers.flg_c());
                Ok(())
            }
            Instr::SCF => {
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(true);
                Ok(())
            }
            Instr::HALT => {
                //TODO
                Ok(())
            }
            Instr::CB => self.handle_cb(bus),
            Instr::JP(jump_type) => {
                let address = self.next_u16(bus);
                self.handle_jump(address, jump_type, bus)
            }
            Instr::JR(jump_type) => {
                let offset = self.next_u8(bus) as i8;
                self.handle_jump(
                    self.registers.pc.wrapping_add(offset as u16),
                    jump_type,
                    bus,
                )
            }
            Instr::CALL(jump_type) => {
                let address = self.next_u16(bus);
                match jump_type {
                    JumpType::If(flag) => {
                        if self.check_flag(flag) {
                            self.push_stack(self.registers.pc, bus)?;
                            self.registers = self.registers.jump(address)?;
                        }
                    }
                    JumpType::Always => {
                        self.push_stack(self.registers.pc, bus)?;
                        self.registers = self.registers.jump(address)?;
                    }
                    _ => unreachable!(),
                }
                Ok(())
            }
            Instr::DEC(Location::Memory(r)) => {
                let address = self.registers.fetch_u16(r);
                let value = self.read_byte(address, bus);
                let result = value.wrapping_sub(1);
                self.set_byte(address, result, bus)?;
                self.registers.set_zf(result == 0);
                self.registers.set_nf(true);
                self.registers.set_hf(value == 0);
                Ok(())
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
                match jump_type {
                    JumpType::If(flag) => {
                        if self.check_flag(flag) {
                            let address = self.pop_stack(bus)?;
                            self.registers = self.registers.jump(address)?;
                        }
                    }
                    JumpType::Always => {
                        let address = self.pop_stack(bus)?;
                        self.registers = self.registers.jump(address)?;
                    }
                    _ => unreachable!(),
                }
                Ok(())
            }
            Instr::RRA => {
                let carry = self.registers.a & 1 != 0;
                self.registers.a >>= 1;
                if self.registers.flg_c() {
                    self.registers.a |= 0b1000_0000;
                }
                self.registers.set_zf(false);
                self.registers.set_hf(false);
                self.registers.set_nf(false);
                self.registers.set_cf(carry);
                Ok(())
            }
            Instr::RLA => {
                let overflow = self.registers.a & 0x80 != 0;
                let result = self.registers.a << 1;
                self.registers.a = result | (self.registers.flg_c() as u8);
                self.registers.set_zf(false);
                self.registers.set_hf(false);
                self.registers.set_nf(false);
                self.registers.set_cf(overflow);
                Ok(())
            }
            Instr::RLCA => {
                let (result, overflow) = self.registers.a.overflowing_shl(1);
                self.registers.a = result;
                self.registers.set_zf(false);
                self.registers.set_hf(false);
                self.registers.set_nf(false);
                self.registers.set_cf(overflow);
                Ok(())
            }
            Instr::ADDSP => {
                let offset = self.next_u8(bus) as i8;
                let old = self.registers.sp;
                let (result, overflow) = self.registers.sp.overflowing_add(offset as u16);
                let half_carry = (((old & 0x00FF) + (offset as u16 & 0x00FF)) & 0x0100) == 0x0100;
                self.registers.sp = result;
                self.registers.set_zf(false);
                self.registers.set_nf(false);
                self.registers.set_hf(half_carry);
                self.registers.set_cf(overflow);
                Ok(())
            }
            Instr::RETI => {
                bus.enable_interrupts();
                let addr = self.pop_stack(bus)?;
                self.handle_jump(addr, JumpType::Always, bus)
            }
            Instr::DAA => {
                self.registers.a = self.bcd_adjust(self.registers.a);
                Ok(())
            }
            Instr::EnableInterrupts => {
                bus.enable_interrupts();
                Ok(())
            }
            Instr::DisableInterrupts => {
                bus.disable_interrupts();
                Ok(())
            }
            Instr::INC(l) => {
                let n: u8 = self.read_location(l, bus).try_into().unwrap();
                let half_carry = (n & 0x0f) == 0x0f;
                let n = n.wrapping_add(1);
                self.registers.set_zf(n == 0);
                self.registers.set_nf(true);
                self.registers.set_hf(half_carry);
                Ok(())
            }
            Instr::UNIMPLEMENTED => unimplemented!(),
            Instr::SBC(_) => unimplemented!(),
            Instr::RRCA => unimplemented!(),
            _ => unreachable!(),
        }
    }

    fn read_instruction(&mut self, bus: &mut Bus) -> CpuResult<()> {
        if bus.interrupts_enabled {
            //&& bus.int_enabled != 0 && bus.int_flags != 0{
            let fired = bus.int_enabled & bus.int_flags;
            if fired & 0x01 != 0 {
                bus.handle_vblank();
                return self.perform_instruction(Instr::RST(0x40), bus);
            }
        }
        let curr_address = self.registers.pc;
        self.debug_area(bus, curr_address);
        let curr_byte = self.next_u8(bus);
        let instruction = &INSTR_TABLE[curr_byte as usize];
        let Instruction(size, _) = INSTRUCTION_TABLE[curr_byte as usize]; //Todo refactor this ugly thing
        let instr_len = size as u16 + 1;
        self.perform_instruction(*instruction, bus)
    }

    fn debug_area(&mut self, bus: &mut Bus, curr_address: u16) {
        if curr_address == 0x0825 {
            // if curr_address == 0x0d601 {
            self.debug = true;
        }
        if self.debug {
            println!("{:04x}: \n{}", curr_address, self.registers);
        }
        // match curr_address {
        //     0x0430..=0x0473 => {
        //         println!("{:04x}: \n{}", curr_address, self.registers);
        //     }
        //     _ => {}
        // }
        if self.debug && curr_address == 0x074a {
            panic!()
        }
        self.trace[self.trace_ptr] = curr_address;
        self.trace_ptr = (self.trace_ptr + 1) % HISTORY_SIZE;
        let r = &self.registers;
        let waszero = self.registers.b == 0;
        let ff = self.registers.b == 0xff;
        if bus.in_bios != 0 {
            self.encounter
                .entry(self.registers.pc)
                .and_modify(|x| {
                    *x += 1;
                })
                .or_insert_with(|| {
                    info!(
                        "First encounter: 0x{:04x?} {:?}",
                        curr_address,
                        INSTR_TABLE[bus.read(curr_address) as usize],
                    );
                    println!("HERE: \n{}", r);
                    0
                });
        }
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
    fn handle_jump(&mut self, address: u16, jt: JumpType, bus: &mut Bus) -> CpuResult<()> {
        match jt {
            JumpType::If(flag) => {
                if self.check_flag(flag) {
                    self.registers = self.registers.jump(address)?;
                }
            }
            JumpType::Always => {
                self.registers = self.registers.jump(address)?;
            }
            JumpType::To(location) => {
                let address = self.read_location(location, bus);
                self.registers = self.registers.jump(address)?;
            }
        }
        Ok(())
    }
    fn cb_location(opcode: u8) -> Location {
        match opcode & 0x0F {
            0x00 => Location::Register(B),
            0x08 => Location::Register(B),
            0x01 => Location::Register(C),
            0x09 => Location::Register(C),
            0x02 => Location::Register(D),
            0x0a => Location::Register(D),
            0x03 => Location::Register(E),
            0x0b => Location::Register(E),
            0x04 => Location::Register(H),
            0x0c => Location::Register(H),
            0x05 => Location::Register(L),
            0x0d => Location::Register(L),
            0x06 => Location::Memory(HL),
            0x0e => Location::Memory(HL),
            0x07 => Location::Register(A),
            0x0f => Location::Register(A),
            _ => panic!(),
        }
    }
    fn handle_cb(&mut self, bus: &mut Bus) -> CpuResult<()> {
        let opcode = self.next_u8(bus);
        bus.cycle();
        match opcode {
            0x30..=0x37 => {
                // SWAP
                let target = CPU::cb_location(opcode);
                let value = self.read_from(target, bus).try_into().unwrap();
                let value = swapped_nibbles(value);
                self.write_into(target, value as u16, bus)?;
                self.registers.set_zf(value == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(false);
            }
            0x40..=0x7F => {
                // BIT
                let target = CPU::cb_location(opcode);
                let mut bit_index = (((opcode & 0xF0) >> 4) - 4) * 2;
                if opcode & 0x08 != 0 {
                    bit_index += 1;
                }
                let value = self.read_from(target, bus);
                let check_zero = value & (1 << bit_index) == 0;
                self.registers.set_zf(check_zero);
                self.registers.set_nf(false);
                self.registers.set_hf(true);
            }
            0x10..=0x18 => {
                //RL
                let target = CPU::cb_location(opcode);
                let value = self.read_from(target, bus);
                let result = value << 1 | self.registers.flg_c() as u16;
                self.registers.set_zf(result == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(result & 0x100 != 0);
                self.write_into(target, result & 0xFF, bus)?;
            }
            0x19 => self.registers = self.registers.rr(Register::C)?,
            0x1A => self.registers = self.registers.rr(Register::D)?,
            0x1B => self.registers = self.registers.rr(Register::E)?,
            0x1C => self.registers = self.registers.rr(Register::H)?,
            0x1D => self.registers = self.registers.rr(Register::L)?,
            // 0x1E => self.registers = self.registers.rr(Register::B),
            0x38 => self.registers = self.registers.srl(Register::B)?,
            0x39 => self.registers = self.registers.srl(Register::C)?,
            0x3A => self.registers = self.registers.srl(Register::D)?,
            0x3B => self.registers = self.registers.srl(Register::E)?,
            0x3C => self.registers = self.registers.srl(Register::H)?,
            0x3D => self.registers = self.registers.srl(Register::L)?,
            0x3E => {
                let address = self.registers.fetch_u16(Register::HL);
                let byte = self.read_byte(address, bus);
                let shifted = byte >> 1;
                self.set_byte(address, shifted, bus)?;
                self.registers.set_zf(shifted == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(byte & 1 != 0);
            }
            0x3F => self.registers = self.registers.srl(Register::A)?,
            0x20..=0x27 => {
                // SLA
                let target = CPU::cb_location(opcode);
                let value = self.read_from(target, bus);
                let result = self.read_from(target, bus) << 1;
                self.registers.set_zf(result == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(result & 0x100 != 0);
                self.write_into(target, result & 0xFF, bus)?;
            }
            0x28..=0x2F => {
                // SRA
                let target = CPU::cb_location(opcode);
                let value = self.read_from(target, bus);
                let result = value >> 1 | (value & 0x80);
                self.registers.set_zf(result == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(value & 0x1 != 0);
                self.write_into(target, result & 0xFF, bus)?;
            }
            0x80..=0xBF => {
                let mut bit_index = (((opcode & 0xF0) >> 4) - 8) * 2;
                if opcode & 0x08 != 0 {
                    bit_index += 1;
                }
                let target = CPU::cb_location(opcode);
                let mut n = self.read_from(target, bus);
                n &= !(1 << bit_index);
                self.write_into(target, n, bus)?;
            }
            _ => {
                let s = source_error!();
                return Err(format!("{} CB: {:02X}", s, opcode));
            } // _ => return Err(format!("CB: {:02X}", opcode)),
        }
        Ok(())
    }

    // TODO hide this
    fn load_start_values(&mut self, bus: &mut Bus) {
        self.registers.a = 0x01;
        self.registers.f = 0xb0;
        self.registers.b = 0x00;
        self.registers.c = 0x13;
        self.registers.d = 0x00;
        self.registers.e = 0xd8;
        self.registers.h = 0x01;
        self.registers.l = 0x4d;
        self.registers.sp = 0xfffe;
        bus.memory[0xFF06] = 0x00; // TMA
        bus.memory[0xFF07] = 0x00; // TAC
        bus.memory[0xFF10] = 0x80; // NR10
        bus.memory[0xFF11] = 0xBF; // NR11
        bus.memory[0xFF12] = 0xF3; // NR12
        bus.memory[0xFF14] = 0xBF; // NR14
        bus.memory[0xFF16] = 0x3F; // NR21
        bus.memory[0xFF17] = 0x00; // NR22
        bus.memory[0xFF19] = 0xBF; // NR24
        bus.memory[0xFF1A] = 0x7F; // NR30
        bus.memory[0xFF1B] = 0xFF; // NR31
        bus.memory[0xFF1C] = 0x9F; // NR32
        bus.memory[0xFF1E] = 0xBF; // NR33
        bus.memory[0xFF20] = 0xFF; // NR41
        bus.memory[0xFF21] = 0x00; // NR42
        bus.memory[0xFF22] = 0x00; // NR43
        bus.memory[0xFF23] = 0xBF; // NR30
        bus.memory[0xFF24] = 0x77; // NR50
        bus.memory[0xFF25] = 0xF3; // NR51
        bus.memory[0xFF26] = 0xF1; // NR52
        bus.memory[0xFF40] = 0x91; // LCDC
        bus.memory[0xFF42] = 0x00; // SCY
        bus.memory[0xFF43] = 0x00; // SCX
        bus.memory[0xFF45] = 0x00; // LYC
        bus.memory[0xFF47] = 0xFC; // BGP
        bus.memory[0xFF48] = 0xFF; // OBP0
        bus.memory[0xFF49] = 0xFF; // OBP1
        bus.memory[0xFF4A] = 0x00; // WY
        bus.memory[0xFF4B] = 0x00; // WX
        bus.memory[0xFFFF] = 0x00; // IE
    }

    pub fn cycle(&mut self, bus: &mut Bus) -> CpuResult<usize> {
        let prev = self.clock;
        if bus.rom_start_signal {
            bus.rom_start_signal = false;
            self.load_start_values(bus);
        }
        self.read_instruction(bus)?;
        Ok(self.clock - prev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::Location::*;

    #[test]
    fn ld() -> Result<(), String> {
        let mut cpu = CPU::new();
        cpu.registers.a = 5;
        cpu.registers.b = 8;
        let mut bus = Bus::new(vec![]);
        assert_eq!(cpu.registers.a, 0x5);
        cpu.perform_instruction(Instr::LD(Register(A), Register(B)), &mut bus)?;
        assert_eq!(cpu.registers.a, 0x8);
        Ok(())
    }

    #[test]
    fn ldbc() -> Result<(), String> {
        let mut cpu = CPU::new();
        cpu.registers.b = 0x21;
        cpu.registers.c = 0x21;
        assert_eq!(cpu.registers.bc(), 0x2121);
        let mut bus = Bus::new(vec![]); // LD BC, d16
                                        // TODO, make Bus a trait that I can inherit from so I can mock it.
        bus.bootrom[0] = 0x01;
        bus.bootrom[1] = 0x22;
        bus.bootrom[2] = 0x11;

        cpu.read_instruction(&mut bus)?;
        assert_eq!(cpu.registers.bc(), 0x1122);
        Ok(())
    }

    // #[test]
    // fn test_ld16() {
    //     let mut cpu = CPU::new(false);
    //     cpu.registers.sp = 0xFFFF;
    //     cpu.registers.b = 0x21;
    //     cpu.registers.c = 0x21;
    //     assert_eq!(cpu.registers.bc(), 0x2121);
    // }
}
