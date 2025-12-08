//! ALU (Arithmetic Logic Unit) instructions

use crate::{
    bus::Bus,
    cpu::CPU,
    operand::{Reg16, RmwOperand8, Src8},
};

// ============================================================================
// 8-bit ALU operations (operate on A register)
// ============================================================================

pub fn add(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    let a = cpu.registers.a;
    let (result, carry) = a.overflowing_add(value);
    let half_carry = (a & 0x0F) + (value & 0x0F) > 0x0F;

    cpu.registers.a = result;
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

pub fn adc(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    let a = cpu.registers.a;
    let carry_in = u8::from(cpu.registers.flg_c());

    let result = a.wrapping_add(value).wrapping_add(carry_in);
    let half_carry = (a & 0x0F) + (value & 0x0F) + carry_in > 0x0F;
    let carry = u16::from(a) + u16::from(value) + u16::from(carry_in) > 0xFF;

    cpu.registers.a = result;
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

pub fn sub(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    let a = cpu.registers.a;
    let result = a.wrapping_sub(value);

    cpu.registers.a = result;
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf((a & 0x0F).wrapping_sub(value & 0x0F) & 0x10 != 0);
    cpu.registers.set_cf(a < value);
}

pub fn sbc(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    let a = cpu.registers.a;
    let carry_in = u8::from(cpu.registers.flg_c());

    let result = a.wrapping_sub(value).wrapping_sub(carry_in);
    let half_carry = (a & 0x0F).wrapping_sub(value & 0x0F).wrapping_sub(carry_in) & 0x10 != 0;
    let carry = u16::from(a) < u16::from(value) + u16::from(carry_in);

    cpu.registers.a = result;
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

pub fn and(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    cpu.registers.a &= value;

    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(true);
    cpu.registers.set_cf(false);
}

pub fn xor(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    cpu.registers.a ^= value;

    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}

pub fn or(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    cpu.registers.a |= value;

    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}

pub fn cp(src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    let a = cpu.registers.a;

    // CP is SUB without storing result
    cpu.registers.set_zf(a == value);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf((a & 0x0F).wrapping_sub(value & 0x0F) & 0x10 != 0);
    cpu.registers.set_cf(a < value);
}

// ============================================================================
// 8-bit INC/DEC
// ============================================================================

pub fn inc8(op: RmwOperand8, cpu: &mut CPU, bus: &mut Bus) {
    let value = op.read(cpu, bus);
    let result = value.wrapping_add(1);

    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf((value & 0x0F) == 0x0F);
    // Carry not affected

    op.write(cpu, bus, result);
}

pub fn dec8(op: RmwOperand8, cpu: &mut CPU, bus: &mut Bus) {
    let value = op.read(cpu, bus);
    let result = value.wrapping_sub(1);

    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(value.trailing_zeros() >= 4);
    // Carry not affected

    op.write(cpu, bus, result);
}

// ============================================================================
// 16-bit INC/DEC
// ============================================================================

pub fn inc16(r: Reg16, cpu: &mut CPU, bus: &mut Bus) {
    cpu.registers.inc_r16(r);
    bus.generic_cycle(); // 16-bit inc takes extra cycle
}

pub fn dec16(r: Reg16, cpu: &mut CPU, bus: &mut Bus) {
    cpu.registers.dec_r16(r);
    bus.generic_cycle(); // 16-bit dec takes extra cycle
}

// ============================================================================
// 16-bit ADD
// ============================================================================

pub fn add_hl(r: Reg16, cpu: &mut CPU, bus: &mut Bus) {
    let hl = cpu.registers.get_r16(Reg16::HL);
    let value = cpu.registers.get_r16(r);

    let (result, carry) = hl.overflowing_add(value);
    let half_carry = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;

    cpu.registers.set_r16(Reg16::HL, result);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
    // Zero flag not affected

    bus.generic_cycle();
}

pub fn add_sp(cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.next_u8(bus) as i8;
    let sp = cpu.registers.sp;
    let offset_u16 = i16::from(offset) as u16;

    let result = sp.wrapping_add(offset_u16);

    // Flags are set based on lower byte addition
    let half_carry = (sp & 0x0F) + (offset_u16 & 0x0F) > 0x0F;
    let carry = (sp & 0xFF) + (offset_u16 & 0xFF) > 0xFF;

    cpu.registers.sp = result;
    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);

    bus.generic_cycle();
    bus.generic_cycle();
}

// ============================================================================
// Rotate operations on A (fast versions that don't set zero flag)
// ============================================================================

pub fn rlca(cpu: &mut CPU) {
    let a = cpu.registers.a;
    let carry = (a & 0x80) != 0;
    cpu.registers.a = (a << 1) | u8::from(carry);

    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(carry);
}

pub fn rrca(cpu: &mut CPU) {
    let a = cpu.registers.a;
    let carry = (a & 0x01) != 0;
    cpu.registers.a = (a >> 1) | (u8::from(carry) << 7);

    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(carry);
}

pub fn rla(cpu: &mut CPU) {
    let a = cpu.registers.a;
    let old_carry = cpu.registers.flg_c();
    let new_carry = (a & 0x80) != 0;
    cpu.registers.a = (a << 1) | u8::from(old_carry);

    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(new_carry);
}

pub fn rra(cpu: &mut CPU) {
    let a = cpu.registers.a;
    let old_carry = cpu.registers.flg_c();
    let new_carry = (a & 0x01) != 0;
    cpu.registers.a = (a >> 1) | (u8::from(old_carry) << 7);

    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(new_carry);
}

// ============================================================================
// Misc ALU operations
// ============================================================================

pub fn daa(cpu: &mut CPU) {
    let mut a = cpu.registers.a;

    if cpu.registers.flg_n() {
        // After subtraction
        if cpu.registers.flg_c() {
            a = a.wrapping_sub(0x60);
        }
        if cpu.registers.flg_h() {
            a = a.wrapping_sub(0x06);
        }
    } else {
        // After addition
        if cpu.registers.flg_c() || a > 0x99 {
            a = a.wrapping_add(0x60);
            cpu.registers.set_cf(true);
        }
        if cpu.registers.flg_h() || (a & 0x0F) > 0x09 {
            a = a.wrapping_add(0x06);
        }
    }

    cpu.registers.a = a;
    cpu.registers.set_zf(a == 0);
    cpu.registers.set_hf(false);
}

pub fn cpl(cpu: &mut CPU) {
    cpu.registers.a = !cpu.registers.a;
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(true);
}

pub fn scf(cpu: &mut CPU) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(true);
}

pub fn ccf(cpu: &mut CPU) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(!cpu.registers.flg_c());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operand::Reg8;

    /// Helper to create CPU with bus for testing
    fn setup() -> (CPU, Bus) {
        let cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        bus.in_bios = 1;
        (cpu, bus)
    }

    // ========================================================================
    // ADD tests
    // ========================================================================

    #[test]
    fn add_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x05;

        add(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x15);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn add_sets_zero_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x00;
        cpu.registers.b = 0x00;

        add(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
    }

    #[test]
    fn add_sets_half_carry_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x0F;
        cpu.registers.b = 0x01;

        add(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x10);
        assert!(cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn add_sets_carry_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xFF;
        cpu.registers.b = 0x01;

        add(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
        assert!(cpu.registers.flg_h());
        assert!(cpu.registers.flg_c());
    }

    // ========================================================================
    // ADC tests
    // ========================================================================

    #[test]
    fn adc_without_carry_in() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x05;
        cpu.registers.set_cf(false);

        adc(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x15);
    }

    #[test]
    fn adc_with_carry_in() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x05;
        cpu.registers.set_cf(true);

        adc(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x16); // 0x10 + 0x05 + 1
    }

    #[test]
    fn adc_carry_in_causes_overflow() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xFF;
        cpu.registers.b = 0x00;
        cpu.registers.set_cf(true);

        adc(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
        assert!(cpu.registers.flg_c());
    }

    // ========================================================================
    // SUB tests
    // ========================================================================

    #[test]
    fn sub_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x15;
        cpu.registers.b = 0x05;

        sub(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x10);
        assert!(!cpu.registers.flg_z());
        assert!(cpu.registers.flg_n()); // N always set for SUB
        assert!(!cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn sub_sets_zero_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x42;
        cpu.registers.b = 0x42;

        sub(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
    }

    #[test]
    fn sub_sets_half_borrow_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x01;

        sub(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(cpu.registers.flg_h());
    }

    #[test]
    fn sub_sets_borrow_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x00;
        cpu.registers.b = 0x01;

        sub(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(cpu.registers.flg_c());
    }

    // ========================================================================
    // SBC tests
    // ========================================================================

    #[test]
    fn sbc_without_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x15;
        cpu.registers.b = 0x05;
        cpu.registers.set_cf(false);

        sbc(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn sbc_with_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x15;
        cpu.registers.b = 0x05;
        cpu.registers.set_cf(true);

        sbc(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x0F); // 0x15 - 0x05 - 1
    }

    // ========================================================================
    // AND tests
    // ========================================================================

    #[test]
    fn and_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xFF;
        cpu.registers.b = 0x0F;

        and(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x0F);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(cpu.registers.flg_h()); // H always set for AND
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn and_sets_zero_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xF0;
        cpu.registers.b = 0x0F;

        and(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
    }

    // ========================================================================
    // XOR tests
    // ========================================================================

    #[test]
    fn xor_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xFF;
        cpu.registers.b = 0x0F;

        xor(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0xF0);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn xor_with_self_zeroes() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x42;

        xor(Src8::Reg(Reg8::A), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
    }

    // ========================================================================
    // OR tests
    // ========================================================================

    #[test]
    fn or_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0xF0;
        cpu.registers.b = 0x0F;

        or(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn or_zero_with_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x00;
        cpu.registers.b = 0x00;

        or(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.flg_z());
    }

    // ========================================================================
    // CP tests
    // ========================================================================

    #[test]
    fn cp_equal_values() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x42;
        cpu.registers.b = 0x42;

        cp(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        // A should NOT change
        assert_eq!(cpu.registers.a, 0x42);
        assert!(cpu.registers.flg_z());
        assert!(cpu.registers.flg_n());
    }

    #[test]
    fn cp_different_values() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x42;
        cpu.registers.b = 0x10;

        cp(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x42); // Unchanged
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn cp_sets_borrow() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x42;

        cp(Src8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.a, 0x10); // Unchanged
        assert!(cpu.registers.flg_c());
    }

    // ========================================================================
    // INC/DEC 8-bit tests
    // ========================================================================

    #[test]
    fn inc8_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0x10;

        inc8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0x11);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
    }

    #[test]
    fn inc8_sets_half_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0x0F;

        inc8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0x10);
        assert!(cpu.registers.flg_h());
    }

    #[test]
    fn inc8_wraps_to_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0xFF;

        inc8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0x00);
        assert!(cpu.registers.flg_z());
        assert!(cpu.registers.flg_h());
    }

    #[test]
    fn inc8_does_not_affect_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0xFF;
        cpu.registers.set_cf(true);

        inc8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert!(cpu.registers.flg_c()); // Should remain set
    }

    #[test]
    fn dec8_basic() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0x10;

        dec8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0x0F);
        assert!(!cpu.registers.flg_z());
        assert!(cpu.registers.flg_n());
    }

    #[test]
    fn dec8_sets_zero_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0x01;

        dec8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0x00);
        assert!(cpu.registers.flg_z());
    }

    #[test]
    fn dec8_wraps_to_ff() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.b = 0x00;

        dec8(RmwOperand8::Reg(Reg8::B), &mut cpu, &mut bus);

        assert_eq!(cpu.registers.b, 0xFF);
    }

    // ========================================================================
    // Rotate tests
    // ========================================================================

    #[test]
    fn rlca_rotates_left() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0b1000_0001;

        rlca(&mut cpu);

        assert_eq!(cpu.registers.a, 0b0000_0011);
        assert!(cpu.registers.flg_c());
        assert!(!cpu.registers.flg_z()); // Never sets Z
    }

    #[test]
    fn rrca_rotates_right() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0b1000_0001;

        rrca(&mut cpu);

        assert_eq!(cpu.registers.a, 0b1100_0000);
        assert!(cpu.registers.flg_c());
    }

    #[test]
    fn rla_rotates_through_carry() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0b0000_0001;
        cpu.registers.set_cf(true);

        rla(&mut cpu);

        assert_eq!(cpu.registers.a, 0b0000_0011); // Old carry shifted in
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn rra_rotates_through_carry() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0b1000_0000;
        cpu.registers.set_cf(true);

        rra(&mut cpu);

        assert_eq!(cpu.registers.a, 0b1100_0000); // Old carry shifted in
        assert!(!cpu.registers.flg_c());
    }

    // ========================================================================
    // Misc ALU tests
    // ========================================================================

    #[test]
    fn cpl_inverts_a() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0b1010_0101;

        cpl(&mut cpu);

        assert_eq!(cpu.registers.a, 0b0101_1010);
        assert!(cpu.registers.flg_n());
        assert!(cpu.registers.flg_h());
    }

    #[test]
    fn scf_sets_carry() {
        let (mut cpu, _) = setup();
        cpu.registers.set_cf(false);
        cpu.registers.set_nf(true);
        cpu.registers.set_hf(true);

        scf(&mut cpu);

        assert!(cpu.registers.flg_c());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
    }

    #[test]
    fn ccf_complements_carry() {
        let (mut cpu, _) = setup();
        cpu.registers.set_cf(true);

        ccf(&mut cpu);
        assert!(!cpu.registers.flg_c());

        ccf(&mut cpu);
        assert!(cpu.registers.flg_c());
    }

    // ========================================================================
    // DAA tests
    // ========================================================================

    #[test]
    fn daa_after_addition() {
        let (mut cpu, _) = setup();
        // 0x15 + 0x27 = 0x3C (binary), should become 0x42 (BCD for 42)
        cpu.registers.a = 0x3C;
        cpu.registers.set_nf(false);
        cpu.registers.set_hf(false);
        cpu.registers.set_cf(false);

        daa(&mut cpu);

        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn daa_with_half_carry() {
        let (mut cpu, _) = setup();
        // When H flag is set, lower nibble > 9
        cpu.registers.a = 0x0A;
        cpu.registers.set_nf(false);
        cpu.registers.set_hf(true);

        daa(&mut cpu);

        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn daa_sets_zero_flag() {
        let (mut cpu, _) = setup();
        cpu.registers.a = 0x00;
        cpu.registers.set_nf(false);

        daa(&mut cpu);

        assert!(cpu.registers.flg_z());
    }
}
