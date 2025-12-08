use crate::operand::{Reg8, Reg16};
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
// Convenience accessors and flag methods
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

impl RegisterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_flags(&mut self, value: [bool; 4]) {
        self.f = flags(value[0], value[1], value[2], value[3]);
    }

    #[inline]
    pub fn set_cf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 4)) | (u8::from(b) << 4);
    }

    #[inline]
    pub fn set_hf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 5)) | (u8::from(b) << 5);
    }

    #[inline]
    pub fn set_nf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 6)) | (u8::from(b) << 6);
    }

    #[inline]
    pub fn set_zf(&mut self, b: bool) {
        self.f = (self.f & !(1 << 7)) | (u8::from(b) << 7);
    }

    #[must_use]
    pub const fn jump(&self, address: u16) -> Self {
        Self { pc: address, ..(*self) }
    }
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

    // ========================================================================
    // Initialization tests
    // ========================================================================

    #[test]
    fn new_register_state_is_zeroed() {
        let reg = RegisterState::new();
        assert_eq!(reg.a, 0);
        assert_eq!(reg.b, 0);
        assert_eq!(reg.c, 0);
        assert_eq!(reg.d, 0);
        assert_eq!(reg.e, 0);
        assert_eq!(reg.f, 0);
        assert_eq!(reg.h, 0);
        assert_eq!(reg.l, 0);
        assert_eq!(reg.sp, 0);
        assert_eq!(reg.pc, 0);
    }

    // ========================================================================
    // Flag function tests
    // ========================================================================

    #[test]
    fn flags_helper_sets_correct_bits() {
        assert_eq!(flags(true, false, false, false), 0b1000_0000); // Z only
        assert_eq!(flags(false, true, false, false), 0b0100_0000); // N only
        assert_eq!(flags(false, false, true, false), 0b0010_0000); // H only
        assert_eq!(flags(false, false, false, true), 0b0001_0000); // C only
        assert_eq!(flags(true, true, true, true), 0b1111_0000);    // All flags
        assert_eq!(flags(false, false, false, false), 0b0000_0000); // No flags
    }

    #[test]
    fn flag_getters_work() {
        let mut reg = RegisterState::new();

        reg.f = flags(true, false, false, false);
        assert!(reg.flg_z());
        assert!(!reg.flg_n());
        assert!(!reg.flg_h());
        assert!(!reg.flg_c());

        reg.f = flags(false, true, true, true);
        assert!(!reg.flg_z());
        assert!(reg.flg_n());
        assert!(reg.flg_h());
        assert!(reg.flg_c());
    }

    #[test]
    fn flag_negative_getters_work() {
        let mut reg = RegisterState::new();

        reg.f = flags(true, true, true, true);
        assert!(!reg.flg_nz()); // NOT zero
        assert!(!reg.flg_nc()); // NOT carry

        reg.f = flags(false, false, false, false);
        assert!(reg.flg_nz());
        assert!(reg.flg_nc());
    }

    #[test]
    fn flag_setters_work() {
        let mut reg = RegisterState::new();

        reg.set_zf(true);
        assert!(reg.flg_z());

        reg.set_nf(true);
        assert!(reg.flg_n());

        reg.set_hf(true);
        assert!(reg.flg_h());

        reg.set_cf(true);
        assert!(reg.flg_c());

        // Clearing flags
        reg.set_zf(false);
        assert!(!reg.flg_z());
    }

    #[test]
    fn set_flags_array_works() {
        let mut reg = RegisterState::new();
        reg.set_flags([true, false, true, false]); // Z, N, H, C
        assert!(reg.flg_z());
        assert!(!reg.flg_n());
        assert!(reg.flg_h());
        assert!(!reg.flg_c());
    }

    // ========================================================================
    // 8-bit register access tests (new typed API)
    // ========================================================================

    #[test]
    fn get_r8_reads_correct_registers() {
        let mut reg = RegisterState::new();
        reg.a = 0x11;
        reg.b = 0x22;
        reg.c = 0x33;
        reg.d = 0x44;
        reg.e = 0x55;
        reg.h = 0x66;
        reg.l = 0x77;
        reg.f = 0x80;

        assert_eq!(reg.get_r8(Reg8::A), 0x11);
        assert_eq!(reg.get_r8(Reg8::B), 0x22);
        assert_eq!(reg.get_r8(Reg8::C), 0x33);
        assert_eq!(reg.get_r8(Reg8::D), 0x44);
        assert_eq!(reg.get_r8(Reg8::E), 0x55);
        assert_eq!(reg.get_r8(Reg8::H), 0x66);
        assert_eq!(reg.get_r8(Reg8::L), 0x77);
        assert_eq!(reg.get_r8(Reg8::F), 0x80);
    }

    #[test]
    fn set_r8_writes_correct_registers() {
        let mut reg = RegisterState::new();

        reg.set_r8(Reg8::A, 0xAA);
        reg.set_r8(Reg8::B, 0xBB);
        reg.set_r8(Reg8::C, 0xCC);
        reg.set_r8(Reg8::D, 0xDD);
        reg.set_r8(Reg8::E, 0xEE);
        reg.set_r8(Reg8::H, 0x11);
        reg.set_r8(Reg8::L, 0x22);

        assert_eq!(reg.a, 0xAA);
        assert_eq!(reg.b, 0xBB);
        assert_eq!(reg.c, 0xCC);
        assert_eq!(reg.d, 0xDD);
        assert_eq!(reg.e, 0xEE);
        assert_eq!(reg.h, 0x11);
        assert_eq!(reg.l, 0x22);
    }

    #[test]
    fn set_r8_f_masks_lower_bits() {
        let mut reg = RegisterState::new();

        // F register lower 4 bits are always 0
        reg.set_r8(Reg8::F, 0xFF);
        assert_eq!(reg.f, 0xF0);

        reg.set_r8(Reg8::F, 0x0F);
        assert_eq!(reg.f, 0x00);
    }

    // ========================================================================
    // 16-bit register pair tests (new typed API)
    // ========================================================================

    #[test]
    fn get_r16_reads_correct_pairs() {
        let mut reg = RegisterState::new();
        reg.b = 0x12;
        reg.c = 0x34;
        reg.d = 0x56;
        reg.e = 0x78;
        reg.h = 0x9A;
        reg.l = 0xBC;
        reg.sp = 0xDEF0;
        reg.a = 0x11;
        reg.f = 0x20;

        assert_eq!(reg.get_r16(Reg16::BC), 0x1234);
        assert_eq!(reg.get_r16(Reg16::DE), 0x5678);
        assert_eq!(reg.get_r16(Reg16::HL), 0x9ABC);
        assert_eq!(reg.get_r16(Reg16::SP), 0xDEF0);
        assert_eq!(reg.get_r16(Reg16::AF), 0x1120);
    }

    #[test]
    fn set_r16_writes_correct_pairs() {
        let mut reg = RegisterState::new();

        reg.set_r16(Reg16::BC, 0x1234);
        assert_eq!(reg.b, 0x12);
        assert_eq!(reg.c, 0x34);

        reg.set_r16(Reg16::DE, 0x5678);
        assert_eq!(reg.d, 0x56);
        assert_eq!(reg.e, 0x78);

        reg.set_r16(Reg16::HL, 0x9ABC);
        assert_eq!(reg.h, 0x9A);
        assert_eq!(reg.l, 0xBC);

        reg.set_r16(Reg16::SP, 0xDEF0);
        assert_eq!(reg.sp, 0xDEF0);
    }

    #[test]
    fn set_r16_af_masks_lower_bits_of_f() {
        let mut reg = RegisterState::new();

        reg.set_r16(Reg16::AF, 0xFFFF);
        assert_eq!(reg.a, 0xFF);
        assert_eq!(reg.f, 0xF0); // Lower 4 bits masked
    }

    #[test]
    fn inc_r16_increments_correctly() {
        let mut reg = RegisterState::new();

        reg.set_r16(Reg16::HL, 0x00FF);
        reg.inc_r16(Reg16::HL);
        assert_eq!(reg.get_r16(Reg16::HL), 0x0100);

        reg.set_r16(Reg16::BC, 0xFFFF);
        reg.inc_r16(Reg16::BC);
        assert_eq!(reg.get_r16(Reg16::BC), 0x0000); // Wraps
    }

    #[test]
    fn dec_r16_decrements_correctly() {
        let mut reg = RegisterState::new();

        reg.set_r16(Reg16::DE, 0x0100);
        reg.dec_r16(Reg16::DE);
        assert_eq!(reg.get_r16(Reg16::DE), 0x00FF);

        reg.set_r16(Reg16::SP, 0x0000);
        reg.dec_r16(Reg16::SP);
        assert_eq!(reg.get_r16(Reg16::SP), 0xFFFF); // Wraps
    }

    // ========================================================================
    // Legacy accessor tests (convenience methods)
    // ========================================================================

    #[test]
    fn hl_accessor_works() {
        let reg = RegisterState {
            h: 0b0000_0001,
            l: 0b1000_0001,
            ..Default::default()
        };
        assert_eq!(reg.hl(), 0b0000_0001_1000_0001);
    }

    #[test]
    fn bc_accessor_works() {
        let reg = RegisterState {
            b: 0xAB,
            c: 0xCD,
            ..Default::default()
        };
        assert_eq!(reg.bc(), 0xABCD);
    }

    #[test]
    fn de_accessor_works() {
        let reg = RegisterState {
            d: 0x12,
            e: 0x34,
            ..Default::default()
        };
        assert_eq!(reg.de(), 0x1234);
    }

    #[test]
    fn af_accessor_works() {
        let reg = RegisterState {
            a: 0x42,
            f: 0x80,
            ..Default::default()
        };
        assert_eq!(reg.af(), 0x4280);
    }

    // ========================================================================
    // Jump helper test
    // ========================================================================

    #[test]
    fn jump_sets_pc() {
        let reg = RegisterState {
            pc: 0x1000,
            ..Default::default()
        };
        let new_reg = reg.jump(0x2000);
        assert_eq!(new_reg.pc, 0x2000);
    }

    // ========================================================================
    // 16-bit INC/DEC with carry tests
    // ========================================================================

    #[test]
    fn inc_hl_carries_correctly() {
        let mut reg = RegisterState {
            h: 0xF0,
            l: 0xFF,
            ..Default::default()
        };
        reg.inc_r16(Reg16::HL);
        assert_eq!(reg.hl(), 0xF100);
    }

    #[test]
    fn dec_hl_borrows_correctly() {
        let mut reg = RegisterState {
            h: 0xFF,
            l: 0x00,
            ..Default::default()
        };
        reg.dec_r16(Reg16::HL);
        assert_eq!(reg.hl(), 0xFEFF);
    }

    #[test]
    fn inc_sp_wraps_correctly() {
        let mut reg = RegisterState {
            sp: 0xFFFF,
            ..Default::default()
        };
        reg.inc_r16(Reg16::SP);
        assert_eq!(reg.sp, 0x0000);
    }

    #[test]
    fn dec_sp_wraps_correctly() {
        let mut reg = RegisterState {
            sp: 0x0000,
            ..Default::default()
        };
        reg.dec_r16(Reg16::SP);
        assert_eq!(reg.sp, 0xFFFF);
    }
}
