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

pub fn flags(Z: bool, N: bool, H: bool, C: bool) -> u8 {
    (Z as u8) << 7 + (N as u8) << 6 + (H as u8) << 5 + (C as u8) << 4
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
            sp: 0xFFFE,
            ..Default::default()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_initalizes() {
        let reg = RegisterState::new();
        assert_eq!(reg.sp, 0xFFFE);
    }

    fn flag_function() {
        let z_only = flags(true, false, false, false);
        assert_eq!(z_only, 0b1000_0000);
        let zn = flags(true, false, false, false);
        assert_eq!(zn, 0b1010_0000);
    }
}
