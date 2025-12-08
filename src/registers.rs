use crate::{
    instructions::{Register, Register::*},
    location::Read,
    operand::{Reg8, Reg16},
};
use std::fmt;

#[derive(Default, Debug, Clone)]
pub struct RegisterState {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8, //flags
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

// ============================================================================
// New typed register access (using Reg8/Reg16)
// ============================================================================

impl RegisterState {
    /// Get an 8-bit register value using the typed Reg8 enum
    #[inline]
    pub fn get_r8(&self, r: Reg8) -> u8 {
        match r {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::F => self.f,
        }
    }

    /// Set an 8-bit register value using the typed Reg8 enum
    #[inline]
    pub fn set_r8(&mut self, r: Reg8, value: u8) {
        match r {
            Reg8::A => self.a = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
            Reg8::F => self.f = value & 0xF0, // Lower 4 bits of F are always 0
        }
    }

    /// Get a 16-bit register pair value using the typed Reg16 enum
    #[inline]
    pub fn get_r16(&self, r: Reg16) -> u16 {
        match r {
            Reg16::BC => u16::from_be_bytes([self.b, self.c]),
            Reg16::DE => u16::from_be_bytes([self.d, self.e]),
            Reg16::HL => u16::from_be_bytes([self.h, self.l]),
            Reg16::SP => self.sp,
            Reg16::AF => u16::from_be_bytes([self.a, self.f]),
        }
    }

    /// Set a 16-bit register pair value using the typed Reg16 enum
    #[inline]
    pub fn set_r16(&mut self, r: Reg16, value: u16) {
        match r {
            Reg16::BC => {
                let [hi, lo] = value.to_be_bytes();
                self.b = hi;
                self.c = lo;
            }
            Reg16::DE => {
                let [hi, lo] = value.to_be_bytes();
                self.d = hi;
                self.e = lo;
            }
            Reg16::HL => {
                let [hi, lo] = value.to_be_bytes();
                self.h = hi;
                self.l = lo;
            }
            Reg16::SP => self.sp = value,
            Reg16::AF => {
                let [hi, lo] = value.to_be_bytes();
                self.a = hi;
                self.f = lo & 0xF0; // Lower 4 bits of F are always 0
            }
        }
    }

    /// Increment a 16-bit register pair
    #[inline]
    pub fn inc_r16(&mut self, r: Reg16) {
        self.set_r16(r, self.get_r16(r).wrapping_add(1));
    }

    /// Decrement a 16-bit register pair
    #[inline]
    pub fn dec_r16(&mut self, r: Reg16) {
        self.set_r16(r, self.get_r16(r).wrapping_sub(1));
    }
}

// ============================================================================
// Legacy register access (using old Register enum) - kept for compatibility
// ============================================================================

/// `u16_reg(n, a, b)` will create a u16 "register" named `n` defined as a | b
macro_rules! u16_reg {
    ($fn_name:ident, $r1:ident, $r2:ident) => {
        pub const fn $fn_name(&self) -> u16 {
            ((self.$r1 as u16) << 8) | (self.$r2 as u16)
        }
    };

    ($fn_name:ident) => {
        pub const fn $fn_name(&self) -> u16 {
            self.$fn_name
        }
    };
}

macro_rules! u8_reg {
    ($fn_name: ident) => {
        pub const fn $fn_name(&self) -> u8 {
            self.$fn_name
        }
    };
}
macro_rules! INC {
    ($self: ident, $r1: ident) => {{
        let prev = $self.$r1;
        let result = prev.wrapping_add(1);
        $self.f = flags(result == 0, false, (prev & 0x0f) == 0x0f, $self.flg_c());
        $self.$r1 = result;
    }};
}

macro_rules! DEC {
    ($self: ident, $r1: ident) => {{
        let prev = $self.$r1;
        let result = prev.wrapping_sub(1);
        $self.f = flags(result == 0, true, result & 0x0f == 0x0f, $self.flg_c());
        $self.$r1 = result;
    }};
}

impl RegisterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn set_flags(&mut self, value: [bool; 4]) {
        self.f = flags(value[0], value[1], value[2], value[3]);
    }

    pub const fn set_cf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 4)) | ((b as u8) << 4);
    }
    pub const fn set_hf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 5)) | ((b as u8) << 5);
    }
    pub const fn set_nf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 6)) | ((b as u8) << 6);
    }
    pub const fn set_zf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 7)) | ((b as u8) << 7);
    }

    #[must_use]
    pub const fn jump(&self, address: u16) -> Self {
        Self { pc: address, ..(*self) }
    }

    pub fn inc(&mut self, reg: Register) {
        match reg {
            HL => {
                let n = self.hl().wrapping_add(1);
                let [h, l] = n.to_be_bytes();
                self.h = h;
                self.l = l;
            }
            BC => {
                let n = self.bc().wrapping_add(1);
                let [b, c] = n.to_be_bytes();
                self.b = b;
                self.c = c;
            }
            DE => {
                let n = self.de().wrapping_add(1);
                let [d, e] = n.to_be_bytes();
                self.d = d;
                self.e = e;
            }
            SP => {
                self.sp = self.sp().wrapping_add(1);
            }
            A => INC!(self, a),
            B => INC!(self, b),
            C => INC!(self, c),
            D => INC!(self, d),
            E => INC!(self, e),
            H => INC!(self, h),
            L => INC!(self, l),
            _ => panic!("inc not impl for {reg:?}"),
        }
    }

    pub fn dec(&mut self, reg: Register) {
        match reg {
            HL => {
                let n = self.hl().wrapping_sub(1);
                let [h, l] = n.to_be_bytes();
                self.h = h;
                self.l = l;
            }
            BC => {
                let n = self.bc().wrapping_sub(1);
                let [b, c] = n.to_be_bytes();
                self.b = b;
                self.c = c;
            }
            DE => {
                let n = self.de().wrapping_sub(1);
                let [d, e] = n.to_be_bytes();
                self.d = d;
                self.e = e;
            }
            SP => {
                self.sp = self.sp().wrapping_sub(1);
            }
            A => DEC!(self, a),
            B => DEC!(self, b),
            C => DEC!(self, c),
            D => DEC!(self, d),
            E => DEC!(self, e),
            H => DEC!(self, h),
            L => DEC!(self, l),
            _ => panic!("dec not impl for {reg:?}"),
        }
    }

    pub fn fetch_u8(&self, reg: Register) -> u8 {
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            F => self.f,
            H => self.h,
            L => self.l,
            _ => panic!("fetch_u8 not impl for {reg:?}"),
        }
    }

    pub fn fetch_u16(&self, reg: Register) -> u16 {
        match reg {
            SP => self.sp(),
            PC => self.pc(),
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            AF => self.af(),
            _ => panic!("fetch_u16 not impl for {reg:?}"),
        }
    }

    pub const fn get_dual_reg(&self, reg: Register) -> Option<u16> {
        match reg {
            SP => Some(self.sp()),
            PC => Some(self.pc()),
            BC => Some(self.bc()),
            DE => Some(self.de()),
            HL => Some(self.hl()),
            AF => Some(self.af()),
            _ => None,
        }
    }

    pub fn fetch(&self, reg: Register) -> Read {
        match reg {
            A | B | C | D | E | F | H | L => Read::Byte(self.fetch_u8(reg)),
            BC | DE | HL | AF | SP | PC => Read::Word(self.fetch_u16(reg)),
        }
    }
    // todo see if swapping these makes a difference..
    // probably not
    pub const fn flg_z(&self) -> bool {
        (self.f & 0b1000_0000) != 0
    }
    pub const fn flg_nz(&self) -> bool {
        !self.flg_z()
    }
    pub const fn flg_n(&self) -> bool {
        (self.f & 0b0100_0000) != 0
    }
    pub const fn flg_nn(&self) -> bool {
        !self.flg_n()
    }
    pub const fn flg_h(&self) -> bool {
        (self.f & 0b0010_0000) != 0
    }
    pub const fn flg_nh(&self) -> bool {
        !self.flg_h()
    }
    pub const fn flg_c(&self) -> bool {
        (self.f & 0b0001_0000) != 0
    }
    pub const fn flg_nc(&self) -> bool {
        !self.flg_c()
    }

    u8_reg!(a);
    u8_reg!(b);
    u8_reg!(c);
    u8_reg!(d);
    u8_reg!(e);
    u8_reg!(f);
    u8_reg!(h);
    u8_reg!(l);
    u16_reg!(sp);
    u16_reg!(pc);

    //Special registers
    u16_reg!(af, a, f);
    u16_reg!(bc, b, c);
    u16_reg!(de, d, e);
    u16_reg!(hl, h, l);
}

#[allow(clippy::fn_params_excessive_bools)]
pub const fn flags(z: bool, n: bool, h: bool, c: bool) -> u8 {
    ((z as u8) << 7) | ((n as u8) << 6) | ((h as u8) << 5) | ((c as u8) << 4)
}
impl fmt::Display for RegisterState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PC:{:04x} SP:{:04x} \
       A:{:02x} F:{:04b} B:{:02x} C:{:02x} \
       D:{:02x} E:{:02x} H:{:02x} L:{:02x}",
            self.pc, self.sp, self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_initalizes() {
        let _reg = RegisterState::new();
    }

    #[test]
    fn flag_function() {
        let z_only = flags(true, false, false, false);
        assert_eq!(z_only, 0b1000_0000);
        let zn = flags(true, false, true, false);
        assert_eq!(zn, 0b1010_0000);
    }

    #[test]
    fn hl() {
        let reg = RegisterState {
            h: 0b0000_0001,
            l: 0b1000_0001,
            ..Default::default()
        };
        assert_eq!(reg.hl(), 0b0000_0001_1000_0001);
    }

    #[test]
    fn inc() {
        let mut reg = RegisterState {
            h: 0xF0,
            l: 0xFF,
            ..Default::default()
        };
        reg.inc(Register::HL);
        assert_eq!(reg.hl(), 0xF100);
    }
    #[test]
    fn dec() {
        let mut reg = RegisterState {
            h: 0xFF,
            l: 0x00,
            ..Default::default()
        };
        reg.dec(Register::HL);
        assert_eq!(reg.hl(), 0xFEFF);
    }
}
