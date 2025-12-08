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
