use crate::{bus::Bus, cpu::CPU, instructions::Register, location::Address};

pub fn inc(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    match location {
        Address::Memory(reg) => inc_mem(reg, cpu, bus),
        Address::Register(reg) => inc_reg(reg, cpu, bus),
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
    if register.is_word_register() {
        bus.generic_cycle();
    }
}

pub fn dec(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    match location {
        Address::Memory(reg) => dec_mem(reg, cpu, bus),
        Address::Register(reg) => dec_reg(reg, cpu, bus),
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
    if register.is_word_register() {
        bus.generic_cycle();
    }
}

pub fn cp(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    cpu.registers.set_zf(cpu.registers.a == value);
    cpu.registers.set_nf(true);
    //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu.rs#l156
    cpu.registers.set_hf((cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0);
    cpu.registers.set_cf(cpu.registers.a < value);
}

pub fn add(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    let (result, carry) = cpu.registers.a.overflowing_add(value);
    //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#l55
    let half_carry = (cpu.registers.a & 0x0f).checked_add(value | 0xf0).is_none();
    cpu.registers.a = result;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(half_carry);
    cpu.registers.set_cf(carry);
}

pub fn sub(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    let result = cpu.registers.a.wrapping_sub(value);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(
        // mooneye
        (cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
    );
    cpu.registers.set_cf(u16::from(cpu.registers.a) < u16::from(value));
    cpu.registers.a = result;
}

pub fn addhl(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let hl = cpu.registers.hl();
    let value: u16 = cpu.read_from(location, bus).into();
    if location.is_word_register() {
        bus.generic_cycle();
    }
    let (result, overflow) = hl.overflowing_add(value);
    let [h, l] = result.to_be_bytes();
    cpu.registers.h = h;
    cpu.registers.l = l;
    cpu.registers.set_nf(false);
    cpu.registers.set_hf((hl & 0xfff) + (value & 0xfff) > 0x0fff);
    cpu.registers.set_cf(overflow);
}

pub fn adc(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.read_from(location, bus).into();
    let carry = u8::from(cpu.registers.flg_c());
    let result = cpu.registers.a.wrapping_add(value).wrapping_add(carry);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(false);
    // maybe: see https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#l55
    cpu.registers.set_hf((cpu.registers.a & 0xf) + (value & 0xf) + carry > 0xf);
    cpu.registers.set_cf(u16::from(cpu.registers.a) + u16::from(value) + u16::from(carry) > 0xff);
    cpu.registers.a = result;
}

pub fn and(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a &= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(true);
    cpu.registers.set_cf(false);
}
pub fn xor(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a ^= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}
pub fn orr(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a |= value;
    cpu.registers.set_zf(cpu.registers.a == 0);
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(false);
}
pub fn not(location: Address, cpu: &mut CPU, bus: &mut Bus) {
    let value: u8 = cpu.read_from(location, bus).into();
    cpu.registers.a = !value;
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(true);
}

pub const fn ccf(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(!cpu.registers.flg_c());
}
pub const fn scf(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.set_nf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_cf(true);
}

pub fn sbc(l: Address, cpu: &mut CPU, bus: &mut Bus) {
    let a = cpu.registers.a;
    let value: u8 = cpu.read_from(l, bus).into();
    let cy = u8::from(cpu.registers.flg_c());
    let result = a.wrapping_sub(value).wrapping_sub(cy);
    cpu.registers.set_zf(result == 0);
    cpu.registers.set_nf(true);
    cpu.registers.set_hf(
        // mooneye
        (cpu.registers.a & 0xf).wrapping_sub(value & 0xf).wrapping_sub(cy) & (0xf + 1) != 0,
    );
    cpu.registers.set_cf(u16::from(cpu.registers.a) < u16::from(value) + u16::from(cy));
    cpu.registers.a = result;
}

pub const fn rra(cpu: &mut CPU, _bus: &mut Bus) {
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
pub const fn rrca(cpu: &mut CPU, _bus: &mut Bus) {
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
pub const fn rla(cpu: &mut CPU, _bus: &mut Bus) {
    let overflow = cpu.registers.a & 0x80 != 0;
    let result = cpu.registers.a << 1;
    cpu.registers.a = result | (cpu.registers.flg_c() as u8);
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(overflow);
}
pub const fn rlca(cpu: &mut CPU, _bus: &mut Bus) {
    let carry = cpu.registers.a & 0x80 != 0;
    let result = cpu.registers.a << 1 | carry as u8;
    cpu.registers.a = result;
    cpu.registers.set_zf(false);
    cpu.registers.set_hf(false);
    cpu.registers.set_nf(false);
    cpu.registers.set_cf(carry);
}

pub fn addsp(cpu: &mut CPU, bus: &mut Bus) {
    let offset = i16::from(cpu.next_u8(bus) as i8).cast_unsigned();
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
