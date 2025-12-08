//! Load/Store instructions

use crate::{
    bus::Bus,
    cpu::CPU,
    operand::{Dst8, Dst16, Reg16, Src8, Src16},
};

/// 8-bit load: LD dst, src
pub fn ld8(dst: Dst8, src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    dst.write(cpu, bus, value);
}

/// 16-bit load: LD dst, src
pub fn ld16(dst: Dst16, src: Src16, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    dst.write(cpu, bus, value);
}

/// LD [HL+], A or LD A, [HL+] - load with HL increment
pub fn ld_inc(dst: Dst8, src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    dst.write(cpu, bus, value);
    cpu.registers.inc_r16(Reg16::HL);
}

/// LD [HL-], A or LD A, [HL-] - load with HL decrement
pub fn ld_dec(dst: Dst8, src: Src8, cpu: &mut CPU, bus: &mut Bus) {
    let value = src.read(cpu, bus);
    dst.write(cpu, bus, value);
    cpu.registers.dec_r16(Reg16::HL);
}

/// LD SP, HL
pub fn ld_sp_hl(cpu: &mut CPU, bus: &mut Bus) {
    cpu.registers.sp = cpu.registers.get_r16(Reg16::HL);
    bus.generic_cycle();
}

/// LD HL, SP+e (signed offset)
pub fn ld_hl_sp_offset(cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.next_u8(bus) as i8;
    let sp = cpu.registers.sp;
    let offset_u16 = i16::from(offset) as u16;

    let result = sp.wrapping_add(offset_u16);

    // Flags are set based on lower byte addition
    let half_carry = (sp & 0x0F) + (offset_u16 & 0x0F) > 0x0F;
    let carry = (sp & 0xFF) + (offset_u16 & 0xFF) > 0xFF;

    cpu.registers.set_r16(Reg16::HL, result);
    bus.generic_cycle();
    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

#[cfg(test)]
mod test {
    use crate::{
        bus::Bus,
        cpu::CPU,
        operand::{Dst8, Reg8, Src8},
        instructions::ld,
    };

    #[test]
    fn ld8_reg_to_reg() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        cpu.registers.a = 5;
        cpu.registers.b = 8;
        assert_eq!(cpu.registers.a, 0x5);
        ld::ld8(Dst8::Reg(Reg8::A), Src8::Reg(Reg8::B), &mut cpu, &mut bus);
        assert_eq!(cpu.registers.a, 0x8);
    }
}
