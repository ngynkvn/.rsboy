pub mod value;

use crate::bus::Bus;
use crate::bus::Memory;
use crate::instructions::Register::*;
use crate::instructions::*;
use crate::registers::RegisterState;
use value::Value;
use value::Value::*;
use value::Writable;

#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

pub const GB_CYCLE_SPEED: usize = 4194304;
#[derive(Debug, Clone)]
pub enum CPUState {
    Running,
    Interrupted,
    Halted,
}
// Global emu struct.
#[derive(Debug, Clone)]
pub struct CPU {
    pub registers: RegisterState,
    pub state: CPUState,
    pub opcode: &'static Instr,
}

pub const VBLANK: u8 = 0b1;
pub const LCDSTAT: u8 = 0b10;
pub const TIMER: u8 = 0b100;
pub const SERIAL: u8 = 0b1000;
pub const JOYPAD: u8 = 0b10000;

#[inline]
pub fn swapped_nibbles(byte: u8) -> u8 {
    let [hi, lo] = [byte >> 4, byte & 0xF];
    (lo << 4) | hi
}

impl CPU {
    pub fn new() -> Self {
        // TODO
        Self {
            registers: RegisterState::new(),
            opcode: &Instr::NOOP,
            state: CPUState::Running,
        }
    }

    pub fn prefetch_op(&mut self, bus: &mut Bus, addr: u16) -> CPUState {
        let opcode = bus.read_cycle(addr);
        self.opcode = &INSTR_TABLE[opcode as usize];
        if self.interrupt_detected(bus) {
            return CPUState::Interrupted;
        }
        self.registers.pc = self.registers.pc.wrapping_add(1);
        CPUState::Running
    }
    pub fn next_u8(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.registers.pc;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        bus.read_cycle(addr)
    }

    pub fn dump_state(&mut self) {
        println!("{:#}", self.registers);
    }
    pub fn next_u16(&mut self, bus: &mut Bus) -> u16 {
        // Little endianess means LSB comes first.
        let lo = self.next_u8(bus);
        let hi = self.next_u8(bus);
        u16::from_le_bytes([lo, hi])
    }

    pub fn read_from(&mut self, location: Location, bus: &mut Bus) -> Value {
        match location {
            Location::Immediate(1) => U8(self.next_u8(bus)),
            Location::Immediate(2) => U16(self.next_u16(bus)),
            Location::Immediate(_) => panic!(),
            Location::MemoryImmediate => {
                let address = self.next_u16(bus);
                U8(bus.read_cycle(address))
            }
            Location::Register(r) => self.registers.fetch(r),
            Location::Memory(r) => U8(bus.read_cycle(self.registers.get_dual_reg(r).unwrap())),
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                U8(bus.read_cycle_high(next))
            }
            Location::MemOffsetRegister(r) => U8(bus.read_cycle_high(self.registers.fetch_u8(r))),
            Location::Literal(x) => x,
        }
    }

    pub fn write_into<T>(&mut self, into: Location, write_value: T, bus: &mut Bus)
    where
        T: Writable,
    {
        match into {
            Location::Immediate(2) => {
                let address = self.next_u16(bus);
                write_value.to_memory_address(address, bus);
            }
            Location::MemoryImmediate => {
                let address = self.next_u16(bus);
                write_value.to_memory_address(address, bus);
            }
            Location::Register(r) => write_value.to_register(&mut self.registers, r),
            Location::Memory(r) => match self.registers.get_dual_reg(r) {
                Some(address) => {
                    write_value.to_memory_address(address, bus);
                }
                None => panic!("I tried to access a u8 as a bus address."),
            },
            Location::MemOffsetImm => {
                let next = self.next_u8(bus);
                write_value.to_memory_address(0xFF00 + next as u16, bus);
            }
            Location::MemOffsetRegister(r) => {
                let offset = self.registers.fetch_u8(r);
                write_value.to_memory_address(0xFF00 + offset as u16, bus);
            }
            _ => unimplemented!("{:?}", into),
        };
    }

    pub fn load(&mut self, into: Location, from: Location, bus: &mut Bus) {
        let from_value = self.read_from(from, bus);
        self.write_into(into, from_value, bus)
    }

    pub fn push_stack(&mut self, value: u16, bus: &mut Bus) {
        let [lo, hi] = value.to_le_bytes();
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_cycle(self.registers.sp, hi);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_cycle(self.registers.sp, lo);
    }

    pub fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        let lo = bus.read_cycle(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let hi = bus.read_cycle(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        u16::from_le_bytes([lo, hi])
    }

    pub fn bcd_adjust(&mut self, value: u8) -> u8 {
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
            if self.registers.flg_c() {
                value = value.wrapping_sub(0x60);
            }
            if self.registers.flg_h() {
                value = value.wrapping_sub(0x6);
            }
        }
        self.registers.set_zf(value == 0);
        self.registers.set_hf(false);
        value
    }

    pub fn interrupt_detected(&mut self, bus: &mut Bus) -> bool {
        bus.ime != 0 && (bus.int_enabled & bus.int_flags) != 0
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        let fired = bus.int_enabled & bus.int_flags;
        bus.generic_cycle();
        self.push_stack(self.registers.pc, bus);
        if fired & VBLANK != 0 {
            bus.ack_interrupt(VBLANK);
            self.registers.pc = 0x40;
            let opcode = self.next_u8(bus);
            self.opcode = &INSTR_TABLE[opcode as usize];
        } else if fired & LCDSTAT != 0 {
            bus.ack_interrupt(LCDSTAT);
            self.registers.pc = 0x48;
            let opcode = self.next_u8(bus);
            self.opcode = &INSTR_TABLE[opcode as usize];
        } else if fired & TIMER != 0 {
            bus.ack_interrupt(TIMER);
            self.registers.pc = 0x50;
            self.dump_state();
            bus.dump_timer_info();
            panic!();
            let opcode = self.next_u8(bus);
            self.opcode = &INSTR_TABLE[opcode as usize];
        } else if fired & SERIAL != 0 {
            bus.ack_interrupt(SERIAL);
            self.registers.pc = 0x58;
            let opcode = self.next_u8(bus);
            self.opcode = &INSTR_TABLE[opcode as usize];
        } else if fired & JOYPAD != 0 {
            bus.ack_interrupt(JOYPAD);
            self.registers.pc = 0x60;
            let opcode = self.next_u8(bus);
            self.opcode = &INSTR_TABLE[opcode as usize];
        }
    }

    pub fn check_flag(&mut self, flag: Flag) -> bool {
        match flag {
            Flag::FlagC => self.registers.flg_c(),
            Flag::FlagNC => self.registers.flg_nc(),
            Flag::FlagZ => self.registers.flg_z(),
            Flag::FlagNZ => self.registers.flg_nz(),
        }
    }

    pub fn jumping<F: FnOnce(&mut Self, &mut Bus)>(
        &mut self,
        jt: Option<Flag>,
        bus: &mut Bus,
        f: F,
    ) {
        match jt {
            Some(flag) => {
                if self.check_flag(flag) {
                    f(self, bus);
                    bus.generic_cycle();
                }
            }
            None => {
                f(self, bus);
                bus.generic_cycle();
            }
        }
    }
    pub fn cb_location(opcode: u8) -> Location {
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
    pub fn handle_cb(&mut self, bus: &mut Bus) {
        let opcode = self.next_u8(bus);
        let target = CPU::cb_location(opcode);
        if let U8(value) = self.read_from(target, bus) {
            match opcode {
                0x00..=0x07 => {
                    //RLC
                    let carry = value & 0x80 != 0;
                    let result = value << 1 | carry as u8;
                    self.registers.set_zf(result == 0);
                    self.registers.set_hf(false);
                    self.registers.set_nf(false);
                    self.registers.set_cf(carry);
                    self.write_into(target, result, bus);
                }
                0x08..=0x0F => {
                    //RRC
                    let carry = value & 0x01 != 0;
                    let result = ((carry as u8) << 7) | (value >> 1);
                    self.registers.set_zf(result == 0);
                    self.registers.set_hf(false);
                    self.registers.set_nf(false);
                    self.registers.set_cf(carry);
                    self.write_into(target, result, bus);
                }
                0x10..=0x17 => {
                    //RL
                    let result = value << 1 | self.registers.flg_c() as u8;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x80 != 0);
                    self.write_into(target, result, bus);
                }
                0x18..=0x1F => {
                    //RR
                    let result = (value >> 1) | ((self.registers.flg_c() as u8) << 7);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x01 != 0);
                    self.write_into(target, result, bus);
                }
                0x30..=0x37 => {
                    // SWAP
                    let result = swapped_nibbles(value);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(false);
                    self.write_into(target, result, bus);
                }
                0x40..=0x7F => {
                    // BIT
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 4) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let check_zero = value & (1 << bit_index) == 0;
                    self.registers.set_zf(check_zero);
                    self.registers.set_nf(false);
                    self.registers.set_hf(true);
                    if let Location::Memory(_) = target {
                        bus.generic_cycle();
                    }
                }
                0xC0..=0xFF => {
                    // SET
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 0xC) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let result = value | (1 << bit_index);
                    self.write_into(target, result, bus);
                }
                0x38..=0x3F => {
                    let result = value >> 1;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 1 != 0);
                    self.write_into(target, result, bus);
                }
                0x20..=0x27 => {
                    // SLA
                    let result = value << 1;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x80 != 0);
                    self.write_into(target, result, bus);
                }
                0x28..=0x2F => {
                    // SRA
                    let result = value >> 1 | (value & 0x80);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x1 != 0);
                    self.write_into(target, result, bus);
                }
                0x80..=0xBF => {
                    // RES
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 8) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let result = value & !(1 << bit_index);
                    self.write_into(target, result, bus);
                }
            };
        } else {
            unreachable!();
        }
    }

    // TODO hide this
    pub fn load_start_values(&mut self, bus: &mut Bus) {
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
                                   // assert_eq!(bus.memory[0xFF04], 0xAB);
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if bus.rom_start_signal {
            bus.rom_start_signal = false;
            self.load_start_values(bus);
        }
        match &self.state {
            CPUState::Running => {
                self.opcode.execute(self, bus);
                self.state = self.prefetch_op(bus, self.registers.pc);
            }
            CPUState::Interrupted => {
                self.handle_interrupts(bus);
                self.state = CPUState::Running;
            }
            CPUState::Halted => {
                panic!();
            }
        }
    }
}

#[cfg(test)]
mod test;
