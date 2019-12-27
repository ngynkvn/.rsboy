#[derive(Default)]
pub struct Registers {
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
}

impl Registers {
    pub fn new () -> Self {
        Self {
            pc: 0x100,
            ..Default::default()
        }
    }

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
        let reg = Registers::new();
        assert_eq!(reg.pc, 0x100);
    } 
}