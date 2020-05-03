use crate::instructions::Register;
use crate::instructions::Register::*;
use std::convert::TryInto;
use std::fmt;

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

impl RegisterState {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn put(&self, value: u16, reg: &Register) -> Result<Self, String> {
        match reg {
            A => Ok(Self {
                a: value.try_into().unwrap(),
                ..(*self)
            }),
            SP => Ok(Self {
                sp: value,
                ..(*self)
            }),
            _ => Err(value.to_string()),
        }
    }

    pub fn fetch_u8(&self, reg: &Register) -> u8 {
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            F => self.f,
            _ => panic!(),
        }
    }

    pub fn fetch_u16(&self, reg: &Register) -> u16 {
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

    pub fn get_dual_reg(&self, reg: &Register) -> Option<u16> {
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

    pub fn fetch(&self, reg: &Register) -> u16 {
        match reg {
            A => self.a.into(),
            B => self.b.into(),
            C => self.c.into(),
            D => self.d.into(),
            E => self.e.into(),
            F => self.f.into(),
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            AF => self.af(),
            SP => self.sp,
            PC => self.pc,
            _ => panic!(),
        }
    }

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
        let zn = flags(true, false, false, false);
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
}
