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

macro_rules! TEST_BIT {
    ($self: ident, $reg: ident, $bit: expr) => {{
        let r = $self.$reg & (1 << ($bit)) == 0;
        Ok(RegisterState {
            f: flags(r, false, true, $self.flg_c()),
            ..(*$self)
        })
    }};
}

macro_rules! SWAP_NIBBLE {
    ($self: ident, $reg: ident) => {
        Ok(RegisterState {
            $reg: swapped_nibbles($self.$reg),
            ..(*$self)
        })
    };
}

fn swapped_nibbles(byte: u8) -> u8 {
    let [hi, lo] = [byte >> 4, byte & 0xF];
    (lo << 4) | byte
}

impl RegisterState {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn jump(&self, address: u16) -> Result<Self, String> {
        Ok(Self {
            pc: address,
            ..(*self)
        })
    }

    pub fn test_bit(&self, reg: &Register, bit: usize) -> Result<Self, String> {
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

    pub fn swap_nibbles(&self, reg: &Register) -> Result<Self, String> {
        match reg {
            A => SWAP_NIBBLE!(self, a),
            B => SWAP_NIBBLE!(self, b),
            C => SWAP_NIBBLE!(self, c),
            D => SWAP_NIBBLE!(self, d),
            E => SWAP_NIBBLE!(self, e),
            H => SWAP_NIBBLE!(self, h),
            L => SWAP_NIBBLE!(self, l),
            _ => Err(format!("swap_nibble: {:?}", reg)),
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
            HL => {
                let [h, l] = value.to_be_bytes();
                Ok(Self { h, l, ..(*self) })
            }
            _ => Err(format!("Put: {}", value.to_string())),
        }
    }

    pub fn inc(&self, reg: &Register) -> Self {
        match reg {
            HL => {
                let n = self.hl();
                let [h, l] = (n.wrapping_add(1)).to_be_bytes();
                Self { h, l, ..(*self) }
            }
            BC => {
                let n = self.bc();
                let [b, c] = (n.wrapping_add(1)).to_be_bytes();
                Self { b, c, ..(*self) }
            }
            C => {
                let n = self.c;
                let c = self.c.wrapping_add(1);
                let half_carry = (n & 0x0f) == 0x0f;
                Self {
                    f: flags(n == 0, true, half_carry, self.flg_c()),
                    c,
                    ..(*self)
                }
            }
            _ => panic!("inc not impl for {:?}", reg),
        }
    }

    pub fn dec(&self, reg: &Register) -> Self {
        match reg {
            HL => {
                let n = self.hl();
                let [h, l] = (n.wrapping_sub(1)).to_be_bytes();
                Self { h, l, ..(*self) }
            }
            BC => {
                let n = self.bc();
                let [b, c] = (n.wrapping_sub(1)).to_be_bytes();
                Self { b, c, ..(*self) }
            }
            C => {
                let n = self.c;
                let c = self.c.wrapping_sub(1);
                let half_carry = (n & 0x0f) == 0x0f;
                Self {
                    f: flags(n == 0, true, half_carry, self.flg_c()),
                    c,
                    ..(*self)
                }
            }
            _ => panic!("dec not impl for {:?}", reg),
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
