pub mod ops;
pub mod value;

use std::fmt::Display;

use crate::bus::{Bus, Memory};

use crate::instructions::Register::*;
use crate::instructions::*;
use crate::registers::RegisterState;
use value::Value;
use value::Value::*;
use value::Writable;

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
    pub opcode: u8,
    pub op_addr: u16,
    pub halt: bool,
}

pub const VBLANK: u8 = 0b1;
pub const LCDSTAT: u8 = 0b10;
pub const TIMER: u8 = 0b100;
pub const SERIAL: u8 = 0b1000;
pub const JOYPAD: u8 = 0b10000;

impl CPU {
    pub fn new() -> Self {
        // TODO
        Self {
            registers: RegisterState::new(),
            opcode: 0,
            op_addr: 0,
            state: CPUState::Running,
            halt: false,
        }
    }

    pub fn execute_op(&mut self, bus: &mut Bus) {
        match self.opcode {
            0x0 => self.noop(bus),
            0x1 => self.ld(Location::Register(BC), Location::Immediate(2), bus),
            0x2 => self.ld(Location::Memory(BC), Location::Register(A), bus),
            0x3 => self.inc_reg(Register::BC, bus),
            0x4 => self.inc_reg(Register::B, bus),
            0x5 => self.dec_reg(B, bus),
            0x6 => self.ld(Location::Register(B), Location::Immediate(1), bus),
            0x7 => self.rlca(bus),
            0x8 => self.ld(Location::Immediate(2), Location::Register(SP), bus),
            0x9 => self.addhl(Location::Register(BC), bus),
            0xA => self.ld(Location::Register(A), Location::Memory(BC), bus),
            0xB => self.dec_reg(BC, bus),
            0xC => self.inc_reg(Register::C, bus),
            0xD => self.dec_reg(C, bus),
            0xE => self.ld(Location::Register(C), Location::Immediate(1), bus),
            0xF => self.rrca(bus),
            0x10 => self.stop(bus),
            0x11 => self.ld(Location::Register(DE), Location::Immediate(2), bus),
            0x12 => self.ld(Location::Memory(DE), Location::Register(A), bus),
            0x13 => self.inc_reg(DE, bus),
            0x14 => self.inc_reg(D, bus),
            0x15 => self.dec_reg(D, bus),
            0x16 => self.ld(Location::Register(D), Location::Immediate(1), bus),
            0x17 => self.rla(bus),
            0x18 => self.jr(None, bus),
            0x19 => self.addhl(Location::Register(DE), bus),
            0x1A => self.ld(Location::Register(A), Location::Memory(DE), bus),
            0x1B => self.dec_reg(DE, bus),
            0x1C => self.inc_reg(E, bus),
            0x1D => self.dec_reg(E, bus),
            0x1E => self.ld(Location::Register(E), Location::Immediate(1), bus),
            0x1F => self.rra(bus),
            0x20 => self.jr(Some(Flag::FlagNZ), bus),
            0x21 => self.ld(Location::Register(HL), Location::Immediate(2), bus),
            0x22 => self.ldi(Location::Memory(HL), Location::Register(A), bus),
            0x23 => self.inc_reg(Register::HL, bus),
            0x24 => self.inc_reg(Register::H, bus),
            0x25 => self.dec_reg(H, bus),
            0x26 => self.ld(Location::Register(H), Location::Immediate(1), bus),
            0x27 => self.daa(bus),
            0x28 => self.jr(Some(Flag::FlagZ), bus),
            0x29 => self.addhl(Location::Register(HL), bus),
            0x2A => self.ldi(Location::Register(A), Location::Memory(HL), bus),
            0x2B => self.dec_reg(HL, bus),
            0x2C => self.inc_reg(Register::L, bus),
            0x2D => self.dec_reg(L, bus),
            0x2E => self.ld(Location::Register(L), Location::Immediate(1), bus),
            0x2F => self.not(Location::Register(A), bus),
            0x30 => self.jr(Some(Flag::FlagNC), bus),
            0x31 => self.ld(Location::Register(SP), Location::Immediate(2), bus),
            0x32 => self.ldd(Location::Memory(HL), Location::Register(A), bus),
            0x33 => self.inc_reg(Register::SP, bus),
            0x34 => self.inc_mem(HL, bus),
            0x35 => self.dec_mem(HL, bus),
            0x36 => self.ld(Location::Memory(HL), Location::Immediate(1), bus),
            0x37 => self.scf(bus),
            0x38 => self.jr(Some(Flag::FlagC), bus),
            0x39 => self.addhl(Location::Register(SP), bus),
            0x3A => self.ldd(Location::Register(A), Location::Memory(HL), bus),
            0x3B => self.dec_reg(SP, bus),
            0x3C => self.inc_reg(Register::A, bus),
            0x3D => self.dec_reg(A, bus),
            0x3E => self.ld(Location::Register(A), Location::Immediate(1), bus),
            0x3F => self.ccf(bus),
            0x40 => self.ld(Location::Register(B), Location::Register(B), bus),
            0x41 => self.ld(Location::Register(B), Location::Register(C), bus),
            0x42 => self.ld(Location::Register(B), Location::Register(D), bus),
            0x43 => self.ld(Location::Register(B), Location::Register(E), bus),
            0x44 => self.ld(Location::Register(B), Location::Register(H), bus),
            0x45 => self.ld(Location::Register(B), Location::Register(L), bus),
            0x46 => self.ld(Location::Register(B), Location::Memory(HL), bus),
            0x47 => self.ld(Location::Register(B), Location::Register(A), bus),
            0x48 => self.ld(Location::Register(C), Location::Register(B), bus),
            0x49 => self.ld(Location::Register(C), Location::Register(C), bus),
            0x4A => self.ld(Location::Register(C), Location::Register(D), bus),
            0x4B => self.ld(Location::Register(C), Location::Register(E), bus),
            0x4C => self.ld(Location::Register(C), Location::Register(H), bus),
            0x4D => self.ld(Location::Register(C), Location::Register(L), bus),
            0x4E => self.ld(Location::Register(C), Location::Memory(HL), bus),
            0x4F => self.ld(Location::Register(C), Location::Register(A), bus),
            0x50 => self.ld(Location::Register(D), Location::Register(B), bus),
            0x51 => self.ld(Location::Register(D), Location::Register(C), bus),
            0x52 => self.ld(Location::Register(D), Location::Register(D), bus),
            0x53 => self.ld(Location::Register(D), Location::Register(E), bus),
            0x54 => self.ld(Location::Register(D), Location::Register(H), bus),
            0x55 => self.ld(Location::Register(D), Location::Register(L), bus),
            0x56 => self.ld(Location::Register(D), Location::Memory(HL), bus),
            0x57 => self.ld(Location::Register(D), Location::Register(A), bus),
            0x58 => self.ld(Location::Register(E), Location::Register(B), bus),
            0x59 => self.ld(Location::Register(E), Location::Register(C), bus),
            0x5A => self.ld(Location::Register(E), Location::Register(D), bus),
            0x5B => self.ld(Location::Register(E), Location::Register(E), bus),
            0x5C => self.ld(Location::Register(E), Location::Register(H), bus),
            0x5D => self.ld(Location::Register(E), Location::Register(L), bus),
            0x5E => self.ld(Location::Register(E), Location::Memory(HL), bus),
            0x5F => self.ld(Location::Register(E), Location::Register(A), bus),
            0x60 => self.ld(Location::Register(H), Location::Register(B), bus),
            0x61 => self.ld(Location::Register(H), Location::Register(C), bus),
            0x62 => self.ld(Location::Register(H), Location::Register(D), bus),
            0x63 => self.ld(Location::Register(H), Location::Register(E), bus),
            0x64 => self.ld(Location::Register(H), Location::Register(H), bus),
            0x65 => self.ld(Location::Register(H), Location::Register(L), bus),
            0x66 => self.ld(Location::Register(H), Location::Memory(HL), bus),
            0x67 => self.ld(Location::Register(H), Location::Register(A), bus),
            0x68 => self.ld(Location::Register(L), Location::Register(B), bus),
            0x69 => self.ld(Location::Register(L), Location::Register(C), bus),
            0x6A => self.ld(Location::Register(L), Location::Register(D), bus),
            0x6B => self.ld(Location::Register(L), Location::Register(E), bus),
            0x6C => self.ld(Location::Register(L), Location::Register(H), bus),
            0x6D => self.ld(Location::Register(L), Location::Register(L), bus),
            0x6E => self.ld(Location::Register(L), Location::Memory(HL), bus),
            0x6F => self.ld(Location::Register(L), Location::Register(A), bus),
            0x70 => self.ld(Location::Memory(HL), Location::Register(B), bus),
            0x71 => self.ld(Location::Memory(HL), Location::Register(C), bus),
            0x72 => self.ld(Location::Memory(HL), Location::Register(D), bus),
            0x73 => self.ld(Location::Memory(HL), Location::Register(E), bus),
            0x74 => self.ld(Location::Memory(HL), Location::Register(H), bus),
            0x75 => self.ld(Location::Memory(HL), Location::Register(L), bus),
            0x76 => self.halt(bus),
            0x77 => self.ld(Location::Memory(HL), Location::Register(A), bus),
            0x78 => self.ld(Location::Register(A), Location::Register(B), bus),
            0x79 => self.ld(Location::Register(A), Location::Register(C), bus),
            0x7A => self.ld(Location::Register(A), Location::Register(D), bus),
            0x7B => self.ld(Location::Register(A), Location::Register(E), bus),
            0x7C => self.ld(Location::Register(A), Location::Register(H), bus),
            0x7D => self.ld(Location::Register(A), Location::Register(L), bus),
            0x7E => self.ld(Location::Register(A), Location::Memory(HL), bus),
            0x7F => self.ld(Location::Register(A), Location::Register(A), bus),
            0x80 => self.add(Location::Register(B), bus),
            0x81 => self.add(Location::Register(C), bus),
            0x82 => self.add(Location::Register(D), bus),
            0x83 => self.add(Location::Register(E), bus),
            0x84 => self.add(Location::Register(H), bus),
            0x85 => self.add(Location::Register(L), bus),
            0x86 => self.add(Location::Memory(HL), bus),
            0x87 => self.add(Location::Register(A), bus),
            0x88 => self.adc(Location::Register(B), bus),
            0x89 => self.adc(Location::Register(C), bus),
            0x8A => self.adc(Location::Register(D), bus),
            0x8B => self.adc(Location::Register(E), bus),
            0x8C => self.adc(Location::Register(H), bus),
            0x8D => self.adc(Location::Register(L), bus),
            0x8E => self.adc(Location::Memory(HL), bus),
            0x8F => self.adc(Location::Register(A), bus),
            0x90 => self.sub(Location::Register(B), bus),
            0x91 => self.sub(Location::Register(C), bus),
            0x92 => self.sub(Location::Register(D), bus),
            0x93 => self.sub(Location::Register(E), bus),
            0x94 => self.sub(Location::Register(H), bus),
            0x95 => self.sub(Location::Register(L), bus),
            0x96 => self.sub(Location::Memory(HL), bus),
            0x97 => self.sub(Location::Register(A), bus),
            0x98 => self.sbc(Location::Register(B), bus),
            0x99 => self.sbc(Location::Register(C), bus),
            0x9A => self.sbc(Location::Register(D), bus),
            0x9B => self.sbc(Location::Register(E), bus),
            0x9C => self.sbc(Location::Register(H), bus),
            0x9D => self.sbc(Location::Register(L), bus),
            0x9E => self.sbc(Location::Memory(HL), bus),
            0x9F => self.sbc(Location::Register(A), bus),
            0xA0 => self.and(Location::Register(B), bus),
            0xA1 => self.and(Location::Register(C), bus),
            0xA2 => self.and(Location::Register(D), bus),
            0xA3 => self.and(Location::Register(E), bus),
            0xA4 => self.and(Location::Register(H), bus),
            0xA5 => self.and(Location::Register(L), bus),
            0xA6 => self.and(Location::Memory(HL), bus),
            0xA7 => self.and(Location::Register(A), bus),
            0xA8 => self.xor(Location::Register(B), bus),
            0xA9 => self.xor(Location::Register(C), bus),
            0xAA => self.xor(Location::Register(D), bus),
            0xAB => self.xor(Location::Register(E), bus),
            0xAC => self.xor(Location::Register(H), bus),
            0xAD => self.xor(Location::Register(L), bus),
            0xAE => self.xor(Location::Memory(HL), bus),
            0xAF => self.xor(Location::Register(A), bus),
            0xB0 => self.orr(Location::Register(B), bus),
            0xB1 => self.orr(Location::Register(C), bus),
            0xB2 => self.orr(Location::Register(D), bus),
            0xB3 => self.orr(Location::Register(E), bus),
            0xB4 => self.orr(Location::Register(H), bus),
            0xB5 => self.orr(Location::Register(L), bus),
            0xB6 => self.orr(Location::Memory(HL), bus),
            0xB7 => self.orr(Location::Register(A), bus),
            0xB8 => self.cp(Location::Register(B), bus),
            0xB9 => self.cp(Location::Register(C), bus),
            0xBA => self.cp(Location::Register(D), bus),
            0xBB => self.cp(Location::Register(E), bus),
            0xBC => self.cp(Location::Register(H), bus),
            0xBD => self.cp(Location::Register(L), bus),
            0xBE => self.cp(Location::Memory(HL), bus),
            0xBF => self.cp(Location::Register(A), bus),
            0xC0 => self.ret(Some(Flag::FlagNZ), bus),
            0xC1 => self.pop(Register::BC, bus),
            0xC2 => self.jp(Some(Flag::FlagNZ), bus),
            0xC3 => self.jp(None, bus),
            0xC4 => self.call(Some(Flag::FlagNZ), bus),
            0xC5 => self.push(Register::BC, bus),
            0xC6 => self.add(Location::Immediate(1), bus),
            0xC7 => self.rst(0x0, bus),
            0xC8 => self.ret(Some(Flag::FlagZ), bus),
            0xC9 => self.ret(None, bus),
            0xCA => self.jp(Some(Flag::FlagZ), bus),
            0xCB => self.handle_cb(bus),
            0xCC => self.call(Some(Flag::FlagZ), bus),
            0xCD => self.call(None, bus),
            0xCE => self.adc(Location::Immediate(1), bus),
            0xCF => self.rst(0x8, bus),
            0xD0 => self.ret(Some(Flag::FlagNC), bus),
            0xD1 => self.pop(Register::DE, bus),
            0xD2 => self.jp(Some(Flag::FlagNC), bus),
            0xD3 => unimplemented!(),
            0xD4 => self.call(Some(Flag::FlagNC), bus),
            0xD5 => self.push(Register::DE, bus),
            0xD6 => self.sub(Location::Immediate(1), bus),
            0xD7 => self.rst(0x10, bus),
            0xD8 => self.ret(Some(Flag::FlagC), bus),
            0xD9 => self.reti(bus),
            0xDA => self.jp(Some(Flag::FlagC), bus),
            0xDB => unimplemented!(),
            0xDC => self.call(Some(Flag::FlagC), bus),
            0xDD => unimplemented!(),
            0xDE => self.sbc(Location::Immediate(1), bus),
            0xDF => self.rst(0x18, bus),
            0xE0 => self.ld(Location::MemOffsetImm, Location::Register(A), bus),
            0xE1 => self.pop(Register::HL, bus),
            0xE2 => self.ld(Location::MemOffsetC, Location::Register(A), bus),
            0xE3 => unimplemented!(),
            0xE4 => unimplemented!(),
            0xE5 => self.push(Register::HL, bus),
            0xE6 => self.and(Location::Immediate(1), bus),
            0xE7 => self.rst(0x20, bus),
            0xE8 => self.addsp(bus),
            0xE9 => self.jp_hl(bus),
            0xEA => self.ld(Location::MemoryImmediate, Location::Register(A), bus),
            0xEB => unimplemented!(),
            0xEC => unimplemented!(),
            0xED => unimplemented!(),
            0xEE => self.xor(Location::Immediate(1), bus),
            0xEF => self.rst(0x28, bus),
            0xF0 => self.ld(Location::Register(A), Location::MemOffsetImm, bus),
            0xF1 => self.pop(Register::AF, bus),
            0xF2 => self.ld(Location::Register(A), Location::MemOffsetC, bus),
            0xF3 => self.disableinterrupts(bus),
            0xF4 => unimplemented!(),
            0xF5 => self.push(Register::AF, bus),
            0xF6 => self.orr(Location::Immediate(1), bus),
            0xF7 => self.rst(0x30, bus),
            0xF8 => self.ldsp(bus),
            0xF9 => {
                self.ld(Location::Register(SP), Location::Register(HL), bus);
                bus.generic_cycle();
            }
            0xFA => self.ld(Location::Register(A), Location::MemoryImmediate, bus),
            0xFB => self.enableinterrupts(bus),
            0xFC => unimplemented!(),
            0xFD => unimplemented!(),
            0xFE => self.cp(Location::Immediate(1), bus),
            0xFF => self.rst(0x38, bus),
            _ => unimplemented!(),
        }
    }

    pub fn prefetch_op(&mut self, bus: &mut Bus, addr: u16) -> CPUState {
        let opcode = bus.read_cycle(addr);
        self.op_addr = addr;
        self.opcode = opcode;
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
            Location::MemOffsetC => U8(bus.read_cycle_high(self.registers.c)),
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
            Location::MemOffsetC => {
                write_value.to_memory_address(0xFF00 + self.registers.c as u16, bus);
            }
            _ => unimplemented!("{:?}", into),
        };
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
            self.opcode = opcode;
        } else if fired & LCDSTAT != 0 {
            bus.ack_interrupt(LCDSTAT);
            self.registers.pc = 0x48;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & TIMER != 0 {
            bus.ack_interrupt(TIMER);
            self.registers.pc = 0x50;
            // println!("{}", self);
            // panic!();
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & SERIAL != 0 {
            bus.ack_interrupt(SERIAL);
            self.registers.pc = 0x58;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & JOYPAD != 0 {
            bus.ack_interrupt(JOYPAD);
            self.registers.pc = 0x60;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
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
        bus.write(0xFF06, 0x00); // TMA
        bus.write(0xFF07, 0x00); // TAC
        bus.write(0xFF10, 0x80); // NR10
        bus.write(0xFF11, 0xBF); // NR11
        bus.write(0xFF12, 0xF3); // NR12
        bus.write(0xFF14, 0xBF); // NR14
        bus.write(0xFF16, 0x3F); // NR21
        bus.write(0xFF17, 0x00); // NR22
        bus.write(0xFF19, 0xBF); // NR24
        bus.write(0xFF1A, 0x7F); // NR30
        bus.write(0xFF1B, 0xFF); // NR31
        bus.write(0xFF1C, 0x9F); // NR32
        bus.write(0xFF1E, 0xBF); // NR33
        bus.write(0xFF20, 0xFF); // NR41
        bus.write(0xFF21, 0x00); // NR42
        bus.write(0xFF22, 0x00); // NR43
        bus.write(0xFF23, 0xBF); // NR30
        bus.write(0xFF24, 0x77); // NR50
        bus.write(0xFF25, 0xF3); // NR51
        bus.write(0xFF26, 0xF1); // NR52
        bus.write(0xFF40, 0x91); // LCDC
        bus.write(0xFF42, 0x00); // SCY
        bus.write(0xFF43, 0x00); // SCX
        bus.write(0xFF45, 0x00); // LYC
        bus.write(0xFF47, 0xFC); // BGP
        bus.write(0xFF48, 0xFF); // OBP0
        bus.write(0xFF49, 0xFF); // OBP1
        bus.write(0xFF4A, 0x00); // WY
        bus.write(0xFF4B, 0x00); // WX
        bus.write(0xFFFF, 0x00); // IE
                                 // assert_eq!(bus.memory[0xFF04], 0xAB);
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if bus.rom_start_signal {
            bus.rom_start_signal = false;
            self.load_start_values(bus);
        }
        match &self.state {
            CPUState::Running => {
                // self.opcode.execute(self, bus);
                self.execute_op(bus);
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

impl Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#}", self.registers))
    }
}

#[cfg(test)]
mod test;
