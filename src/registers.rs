use crate::instructions::Register;
use crate::instructions::Register::*;
use std::convert::TryInto;
use std::fmt;

#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

// Global emu struct.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Default, Debug)]
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

/**
 * u16_reg(n, a, b) will create a u16 "register" named `n` defined as a | b
 */
macro_rules! u16_reg {
    ($fn_name:ident, $r1:ident, $r2:ident) => {
        pub fn $fn_name(&self) -> u16 {
            ((self.$r1 as u16) << 8) | (self.$r2 as u16)
        }
    };

    ($fn_name:ident) => {
        pub fn $fn_name(&self) -> u16 {
            self.$fn_name
        }
    };
}

macro_rules! u8_reg {
    ($fn_name: ident) => {
        pub fn $fn_name(&self) -> u8 {
            self.$fn_name
        }
    };
}

macro_rules! TEST_BIT {
    ($self: ident, $reg: ident, $bit: expr) => {{
        let r = $self.$reg & (1 << ($bit)) == 0;
        Ok(RegisterState {
            f: flags(r, false, true, $self.flg_c()),
            ..(*$self)
        })
    }};
}

macro_rules! INC {
    ($self: ident, $r1: ident) => {{
        let n = $self.$r1;
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_add(1);
        RegisterState {
            $r1: n,
            f: flags(n == 0, false, half_carry, $self.flg_c()),
            ..(*$self)
        }
    }};
}

macro_rules! DEC {
    ($self: ident, $r1: ident) => {{
        let old = $self.$r1;
        let n = old.wrapping_sub(1);
        RegisterState {
            $r1: n,
            f: flags(n == 0, true, old == 0x00, $self.flg_c()),
            ..(*$self)
        }
    }};
}

macro_rules! RR {
    ($self: ident, $r1: ident) => {{
        let mut n = ($self.$r1 >> 1);
        if $self.flg_c() {
            n |= 0b1000_0000
        }
        Ok(RegisterState {
            $r1: n,
            f: flags(n == 0, false, false, $self.$r1 & 1 != 0),
            ..(*$self)
        })
    }};
}

macro_rules! SRL {
    ($self: ident, $r1: ident) => {{
        let n = ($self.$r1 >> 1);
        Ok(RegisterState {
            $r1: n,
            f: flags(n == 0, false, false, $self.$r1 & 1 != 0),
            ..(*$self)
        })
    }};
}

impl RegisterState {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_cf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 4)) | ((b as u8) << 4);
    }
    pub fn set_hf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 5)) | ((b as u8) << 5);
    }
    pub fn set_nf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 6)) | ((b as u8) << 6);
    }
    pub fn set_zf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 7)) | ((b as u8) << 7);
    }

    pub fn jump(&self, address: u16) -> Result<Self, String> {
        Ok(Self {
            pc: address,
            ..(*self)
        })
    }

    pub fn test_bit(&self, reg: Register, bit: usize) -> Result<Self, String> {
        match reg {
            A => TEST_BIT!(self, a, bit),
            B => TEST_BIT!(self, b, bit),
            C => TEST_BIT!(self, c, bit),
            D => TEST_BIT!(self, d, bit),
            E => TEST_BIT!(self, e, bit),
            H => TEST_BIT!(self, h, bit),
            L => TEST_BIT!(self, l, bit),
            _ => Err(format!("swap_nibble: {:?}", reg)),
        }
    }

    pub fn srl(&self, reg: Register) -> Result<Self, String> {
        match reg {
            A => SRL!(self, a),
            B => SRL!(self, b),
            C => SRL!(self, c),
            D => SRL!(self, d),
            E => SRL!(self, e),
            H => SRL!(self, h),
            L => SRL!(self, l),
            _ => Err(format!("srl: {:?}", reg)),
        }
    }
    pub fn rr(&self, reg: Register) -> Result<Self, String> {
        match reg {
            A => RR!(self, a),
            B => RR!(self, b),
            C => RR!(self, c),
            D => RR!(self, d),
            E => RR!(self, e),
            H => RR!(self, h),
            L => RR!(self, l),
            _ => Err(format!("rr: {:?}", reg)),
        }
    }

    pub fn put(&self, value: u16, reg: Register) -> Result<Self, String> {
        match reg {
            A => Ok(Self {
                a: value.try_into().unwrap(),
                ..(*self)
            }),
            B => Ok(Self {
                b: value.try_into().unwrap(),
                ..(*self)
            }),
            C => Ok(Self {
                c: value.try_into().unwrap(),
                ..(*self)
            }),
            D => Ok(Self {
                d: value.try_into().unwrap(),
                ..(*self)
            }),
            E => Ok(Self {
                e: value.try_into().unwrap(),
                ..(*self)
            }),
            H => Ok(Self {
                h: value.try_into().unwrap(),
                ..(*self)
            }),
            L => Ok(Self {
                l: value.try_into().unwrap(),
                ..(*self)
            }),
            SP => Ok(Self {
                sp: value,
                ..(*self)
            }),
            HL => {
                let [h, l] = value.to_be_bytes();
                Ok(Self { h, l, ..(*self) })
            }
            DE => {
                let [d, e] = value.to_be_bytes();
                Ok(Self { d, e, ..(*self) })
            }
            BC => {
                let [b, c] = value.to_be_bytes();
                Ok(Self { b, c, ..(*self) })
            }
            AF => {
                let [a, f] = (value & 0b1111_1111_1111_0000).to_be_bytes();
                Ok(Self { a, f, ..(*self) })
            }
            _ => Err(format!("Put: {} into {:?}", value.to_string(), reg)),
        }
    }

    pub fn inc(&self, reg: Register) -> Self {
        match reg {
            HL => {
                let n = self.hl().wrapping_add(1);
                let [h, l] = n.to_be_bytes();
                Self { h, l, ..(*self) }
            }
            BC => {
                let n = self.bc().wrapping_add(1);
                let [b, c] = n.to_be_bytes();
                Self { b, c, ..(*self) }
            }
            DE => {
                let n = self.de().wrapping_add(1);
                let [d, e] = n.to_be_bytes();
                Self { d, e, ..(*self) }
            }
            A => INC!(self, a),
            B => INC!(self, b),
            C => INC!(self, c),
            D => INC!(self, d),
            E => INC!(self, e),
            H => INC!(self, h),
            L => INC!(self, l),
            _ => panic!("inc not impl for {:?}", reg),
        }
    }

    pub fn dec(&self, reg: Register) -> Self {
        match reg {
            HL => {
                let n = self.hl().wrapping_sub(1);
                let [h, l] = n.to_be_bytes();
                Self { h, l, ..(*self) }
            }
            BC => {
                let n = self.bc().wrapping_sub(1);
                let [b, c] = n.to_be_bytes();
                Self { b, c, ..(*self) }
            }
            A => DEC!(self, a),
            B => DEC!(self, b),
            C => DEC!(self, c),
            D => DEC!(self, d),
            E => DEC!(self, e),
            H => DEC!(self, h),
            L => DEC!(self, l),
            _ => panic!("dec not impl for {:?}", reg),
        }
    }

    pub fn fetch_u8(&self, reg: Register) -> Result<u8, String> {
        match reg {
            A => Ok(self.a),
            B => Ok(self.b),
            C => Ok(self.c),
            D => Ok(self.d),
            E => Ok(self.e),
            F => Ok(self.f),
            _ => Err(format!("fetchu8 {:?}", reg)),
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
            _ => panic!(),
        }
    }

    pub fn get_dual_reg(&self, reg: Register) -> Option<u16> {
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

    pub fn fetch(&self, reg: Register) -> u16 {
        match reg {
            A => self.a.into(),
            B => self.b.into(),
            C => self.c.into(),
            D => self.d.into(),
            E => self.e.into(),
            F => self.f.into(),
            H => self.h.into(),
            L => self.l.into(),
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            AF => self.af(),
            SP => self.sp,
            PC => self.pc,
        }
    }
    // TODO See if swapping these makes a difference..
    // Probably not
    pub fn flg_z(&self) -> bool {
        (self.f & 0b1000_0000) != 0
    }
    pub fn flg_nz(&self) -> bool {
        !self.flg_z()
    }
    pub fn flg_n(&self) -> bool {
        (self.f & 0b0100_0000) != 0
    }
    pub fn flg_nn(&self) -> bool {
        !self.flg_n()
    }
    pub fn flg_h(&self) -> bool {
        (self.f & 0b0010_0000) != 0
    }
    pub fn flg_nh(&self) -> bool {
        !self.flg_h()
    }
    pub fn flg_c(&self) -> bool {
        (self.f & 0b0001_0000) != 0
    }
    pub fn flg_nc(&self) -> bool {
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

pub fn flags(z: bool, n: bool, h: bool, c: bool) -> u8 {
    ((z as u8) << 7) | ((n as u8) << 6) | ((h as u8) << 5) | ((c as u8) << 4)
}

impl fmt::Display for RegisterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            concat!(
                "AF: 0x{:04X}\n",
                "BC: 0x{:04X}\n",
                "DE: 0x{:04X}\n",
                "HL: 0x{:04X}\n",
                "SP: 0x{:04X}\n",
                "PC: 0x{:04X}\n"
            ),
            self.af(),
            self.bc(),
            self.de(),
            self.hl(),
            self.sp(),
            self.pc()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_initalizes() {
        let reg = RegisterState::new();
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
        let reg = RegisterState {
            h: 0xF0,
            l: 0xFF,
            ..Default::default()
        };
        let reg = reg.inc(Register::HL);
        assert_eq!(reg.hl(), 0xF100);
    }
    #[test]
    fn dec() {
        let reg = RegisterState {
            h: 0xFF,
            l: 0x00,
            ..Default::default()
        };
        let reg = reg.dec(Register::HL);
        assert_eq!(reg.hl(), 0xFEFF);
    }
}
