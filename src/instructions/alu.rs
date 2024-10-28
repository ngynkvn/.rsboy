use crate::instructions::Bus;
use crate::instructions::Location;
use crate::instructions::Memory;
use crate::instructions::Register;
use crate::instructions::CPU;

pub fn inc(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    match location {
        Memory(reg) => inc_mem(reg, cpu, bus),
        Register(reg) => inc_reg(reg, cpu, bus),
        _ => unimplemented!(),
    }
}
pub fn inc_mem(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.registers.fetch_u16(register);
    let value = bus.read_cycle(address);
    let result = value.wrapping_add(1);
    bus.write_cycle(address, result);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(value & 0x0f == 0x0f);
}

pub fn inc_reg(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    cpu.registers.inc(register);
    if register.is_dual_register() {
        bus.generic_cycle();
    }
}

pub fn dec(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    match location {
        Memory(reg) => dec_mem(reg, cpu, bus),
        Register(reg) => dec_reg(reg, cpu, bus),
        _ => unimplemented!(),
    }
}

pub fn dec_mem(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.registers.fetch_u16(register);
    let value = bus.read_cycle(address);
    let result = value.wrapping_sub(1);
    bus.write_cycle(address, result);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(result & 0x0f == 0x0f);
}

pub fn dec_reg(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    cpu.registers.dec(register);
    if register.is_dual_register() {
        bus.generic_cycle();
    }
}

pub fn cp(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    cpu.registers.set_zf(cpu.registers.a == value);
    cpu.registers.set_nf(true);
    //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu.rs#l156
    cpu.registers
        .set_hf((cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0);
    cpu.registers.set_cf(cpu.registers.a < value);
}

pub fn add(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    let (result, carry) = cpu.registers.a.overflowing_add(value);
    //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#l55
    let half_carry = (cpu.registers.a & 0x0f).checked_add(value | 0xf0).is_none();
    cpu.registers.a = result;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

pub fn sub(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    let result = cpu.registers.a.wrapping_sub(value);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(
        // mooneye
        (cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
    );
    cpu.registers
        .set_cf((cpu.registers.a as u16) < (value as u16));
    cpu.registers.a = result;
}

pub fn addhl(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let hl = cpu.registers.hl();
    let value = cpu.read_from(location, bus);
    if location.is_dual_register() {
        bus.generic_cycle();
    }
    let (result, overflow) = hl.overflowing_add(value);
    let [h, l] = result.to_be_bytes();
    cpu.registers.h = h;
    cpu.registers.l = l;
    cpu.registers.set_nf(false);
    cpu.registers
        .set_hf((hl & 0xfff) + (value & 0xfff) > 0x0fff);
    cpu.registers.set_cf(overflow);
}

pub fn adc(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    let carry = cpu.registers.flg_c() as u8;
    let result = cpu.registers.a.wrapping_add(value).wrapping_add(carry);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    // maybe: see https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#l55
    cpu.registers
        .set_hf((cpu.registers.a & 0xf) + (value & 0xf) + carry > 0xf);
    cpu.registers
        .set_cf(cpu.registers.a as u16 + value as u16 + carry as u16 > 0xff);
    cpu.registers.a = result;
}

pub fn and(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a &= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(true);
    cpu.registers.set_cf(false);
}
pub fn xor(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a ^= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}
pub fn orr(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a |= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}
pub fn not(location: Location, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a = !value;
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(true);
}

pub fn ccf(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(!cpu.registers.flg_c());
}
pub fn scf(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(true);
}

pub fn sbc(l: Location, cpu: &mut CPU, bus: &mut Bus) {
    let a = cpu.registers.a;
    let value: u8 = cpu.read_from(l, bus).into();
    let cy = cpu.registers.flg_c() as u8;
    let result = a.wrapping_sub(value).wrapping_sub(cy);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(
        // mooneye
        (cpu.registers.a & 0xf)
            .wrapping_sub(value & 0xf)
            .wrapping_sub(cy)
            & (0xf + 1)
            != 0,
    );
    cpu.registers
        .set_cf((cpu.registers.a as u16) < (value as u16) + (cy as u16));
    cpu.registers.a = result;
}

pub fn rra(cpu: &mut CPU, _bus: &mut Bus) {
    let carry = cpu.registers.a & 1 != 0;
    cpu.registers.a >>= 1;
    if cpu.registers.flg_c() {
        cpu.registers.a |= 0b1000_0000;
    }
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(carry);
}
pub fn rrca(cpu: &mut CPU, _bus: &mut Bus) {
    let carry = cpu.registers.a & 1 != 0;
    cpu.registers.a >>= 1;
    if carry {
        cpu.registers.a |= 0b1000_0000;
    }
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(carry);
}
pub fn rla(cpu: &mut CPU, _bus: &mut Bus) {
    let overflow = cpu.registers.a & 0x80 != 0;
    let result = cpu.registers.a << 1;
    cpu.registers.a = result | (cpu.registers.flg_c() as u8);
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(overflow);
}
pub fn rlca(cpu: &mut CPU, _bus: &mut Bus) {
    let carry = cpu.registers.a & 0x80 != 0;
    let result = cpu.registers.a << 1 | carry as u8;
    cpu.registers.a = result;
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(carry);
}

pub fn addsp(cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.next_u8(bus) as i8 as i16 as u16;
    let sp = cpu.registers.sp;
    let result = cpu.registers.sp.wrapping_add(offset);
    bus.generic_cycle();
    bus.generic_cycle();
    let half_carry = ((sp & 0x0f) + (offset & 0x0f)) > 0x0f;
    let overflow = ((sp & 0xff) + (offset & 0xff)) > 0xff;
    cpu.registers.sp = result;
    cpu.registers.set_zf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(overflow);
}
