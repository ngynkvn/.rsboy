use crate::bus::Bus;
use crate::bus::Memory;
use crate::instructions::Register::*;
use crate::instructions::*;
use crate::registers::RegisterState;
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
    trace_ptr: usize
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

macro_rules! source_error {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

impl CPU {
    pub fn new(skip_bios: bool) -> Self {
        // TODO
        Self {
            registers: RegisterState::new(skip_bios),
            debug: false,
            clock: 0,
            running: true,
            encounter: HashMap::new(),
            trace: [0; HISTORY_SIZE],
            trace_ptr: 0,
        }
    }
    fn next_u8(&mut self, bus: &mut Bus) -> u8 {
        self.clock += 1;
        bus.cycle().unwrap();
        let val = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        val
    }
    fn next_u16(&mut self, bus: &mut Bus) -> u16 {
        // Little endianess means LSB comes first.
        self.clock += 1;
        bus.cycle().unwrap();
        let val =
            (bus.read(self.registers.pc + 1) as u16) << 8 | (bus.read(self.registers.pc) as u16);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        val
    }
    fn read_byte(&mut self, address: u16, bus: &mut Bus) -> u8 {
        self.clock += 1;
        bus.cycle().unwrap();
        bus.read(address)
    }
    fn read_io(&mut self, offset: u16, bus: &mut Bus) -> u8 {
        self.read_byte(0xFF00 + offset, bus)
    }
    fn set_byte(&mut self, address: u16, value: u8, bus: &mut Bus) -> CpuResult<()> {
        self.clock += 1;
        bus.cycle().unwrap();
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
                self.clock += 1; // TODO
                bus.cycle()?;
                Ok(())
            }
            Instr::LDI(into, from) => {
                self.load(into, from, bus).or_else(source_error)?;
                self.inc(Register::HL);
                self.clock += 1; // TODO
                bus.cycle()?;
                Ok(())
            }
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
                self.registers.a = result;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                // Maybe: See https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#L55
                self.registers
                    .set_hf(((self.registers.a & 0xf) + (value & 0xf)) & 0x10 == 0x10);
                self.registers.set_cf(carry);
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
                let result = old.wrapping_add(value);
                let [h, l] = old.wrapping_add(value).to_be_bytes();
                self.registers.h = h;
                self.registers.l = l;
                self.registers.set_nf(false);
                self.registers.set_hf(((old & 0x00FF) + (value & 0x00FF)) & 0x0100 == 0x0100);
                self.registers.set_cf(result < old);
                //TODO FLAGS
                Ok(())
            }
            Instr::AND(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a & value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(true);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::XOR(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a ^ value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::OR(location) => {
                let value: u8 = self.read_location(location, bus).try_into().unwrap();
                self.registers.a = self.registers.a | value;
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(false);
                self.registers.set_hf(false);
                self.registers.set_cf(false);
                Ok(())
            }
            Instr::SUB(location) => {
                let value = self.read_location(location, bus).try_into().unwrap();
                let (result, carry) = self.registers.a.overflowing_sub(value);
                self.registers.a = self.registers.a.wrapping_sub(value);
                self.registers.set_zf(self.registers.a == 0);
                self.registers.set_nf(true);
                self.registers.set_hf(
                    // Mooneye
                    (self.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
                );
                self.registers.set_cf(carry);
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
                self.push_stack(self.registers.pc, bus)?;
                self.handle_jump(address, jump_type, bus)
            }
            Instr::DEC(Location::Memory(r)) => {
                let address = self.registers.fetch_u16(r);
                let value = self.read_byte(address, bus);
                let result = value.wrapping_sub(1);
                self.set_byte(address, result, bus)?;
                self.registers.set_zf(result == 0);
                self.registers.set_nf(true);
                self.registers.set_hf(value & 0xf == 0);
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
                self.registers.a = self.registers.a >> 1;
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
            x => Err(format!("{} read_instruction: {:?}", source_error!(), x)),
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
        self.trace[self.trace_ptr] = curr_address;
        self.trace_ptr = (self.trace_ptr + 1) % HISTORY_SIZE;
        if curr_address == 0x0a00 {
            println!("{:04x}", self.registers.sp);
            let n = self.pop_stack(bus)?;
            println!("{:04x}", n);
            for i in 0..HISTORY_SIZE {
                println!("{:04x}", self.trace[self.trace_ptr]);
                self.trace_ptr = (self.trace_ptr + 1) % HISTORY_SIZE;
            }
            panic!();
        }
        let waszero = self.registers.b == 0;
        let ff = self.registers.b == 0xff;
        if bus.in_bios != 0 {
            self.encounter
                .entry(self.registers.pc)
                .and_modify(|x| {
                    *x += 1;
                })
                .or_insert_with(|| {
                    println!(
                        "First encounter: 0x{:04x?} {:?}",
                        // "First encounter: 0x{:04x?}",
                        curr_address,
                        INSTR_TABLE[bus.read(curr_address) as usize],
                    );
                    0
                });
        }
        let curr_byte = self.next_u8(bus);
        let instruction = &INSTR_TABLE[curr_byte as usize];
        let Instruction(size, _) = INSTRUCTION_TABLE[curr_byte as usize]; //Todo refactor this ugly thing
        let instr_len = size as u16 + 1;
        // if curr_address >= 0x02dd && curr_address <= 0x02e1 {
        //     println!(
        //         "0x{:04x?}: {:02x} {:?}, {}",
        //         self.registers.pc - 1,
        //         curr_byte,
        //         instruction,
        //         self.registers
        //     );
        // }
        let result = self.perform_instruction(*instruction, bus);
        result
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
        bus.cycle()?;
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
            0x40..=0x7F => {
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
    pub fn cycle(&mut self, bus: &mut Bus) -> CpuResult<usize> {
        let prev = self.clock;
        self.read_instruction(bus)?;
        Ok(self.clock - prev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::Location::*;
    use crate::instructions::Register::*;

    #[test]
    fn ld() -> Result<(), String> {
        let mut cpu = CPU::new(false);
        cpu.registers.a = 5;
        cpu.registers.b = 8;
        let mut bus = Bus::new(false, vec![]);
        assert_eq!(cpu.registers.a, 0x5);
        cpu.perform_instruction(Instr::LD(Register(A), Register(B)), &mut bus)?;
        assert_eq!(cpu.registers.a, 0x8);
        Ok(())
    }

    #[test]
    fn ldbc() -> Result<(), String> {
        let mut cpu = CPU::new(false);
        cpu.registers.b = 0x21;
        cpu.registers.c = 0x21;
        assert_eq!(cpu.registers.bc(), 0x2121);
        let mut bus = Bus::new(false, vec![]); // LD BC, d16
                                               // TODO, make Bus a trait that I can inherit from so I can mock it.
        bus.bootrom[0] = 0x01;
        bus.bootrom[1] = 0x22;
        bus.bootrom[2] = 0x11;

        cpu.read_instruction(&mut bus)?;
        assert_eq!(cpu.registers.bc(), 0x1122);
        Ok(())
    }
}
