use crate::cpu::value::Value;

use crate::instructions::Register;
use crate::instructions::Register::*;
use std::fmt;

// Global emu struct.
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
        Default::default()
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

    pub fn jump(&self, address: u16) -> Self {
        Self {
            pc: address,
            ..(*self)
        }
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
            _ => panic!("inc not impl for {:?}", reg),
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
            _ => panic!("dec not impl for {:?}", reg),
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
            _ => unreachable!(),
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

    pub fn fetch(&self, reg: Register) -> Value {
        match reg {
            A => Value::from(self.a),
            B => Value::from(self.b),
            C => Value::from(self.c),
            D => Value::from(self.d),
            E => Value::from(self.e),
            F => Value::from(self.f),
            H => Value::from(self.h),
            L => Value::from(self.l),
            BC => Value::from(self.bc()),
            DE => Value::from(self.de()),
            HL => Value::from(self.hl()),
            AF => Value::from(self.af()),
            SP => Value::from(self.sp),
            PC => Value::from(self.pc),
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
