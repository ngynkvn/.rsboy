use crate::instructions::{Bus, CPU, Register, location::Address};

pub fn ld((into, from): (Address, Address), cpu: &mut CPU, bus: &mut Bus) {
    let from_value = cpu.read_from(from, bus);
    cpu.write_into(into, from_value, bus);
}

pub fn ldi(location: (Address, Address), cpu: &mut CPU, bus: &mut Bus) {
    ld(location, cpu, bus);
    cpu.registers.inc(Register::HL);
}

pub fn ldd(location: (Address, Address), cpu: &mut CPU, bus: &mut Bus) {
    ld(location, cpu, bus);
    cpu.registers.dec(Register::HL);
}

pub fn ldsp(cpu: &mut CPU, bus: &mut Bus) {
    let offset = i16::from(cpu.next_u8(bus) as i8).cast_unsigned();
    let result = cpu.registers.sp.wrapping_add(offset); // todo ?
    let half_carry = (cpu.registers.sp & 0x0F).wrapping_add(offset & 0x0F) > 0x0F;
    let carry = (cpu.registers.sp & 0xFF).wrapping_add(offset & 0xFF) > 0xFF;
    cpu.write_into(Address::Register(Register::HL), result, bus);
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
        instructions::{Register, Register::*, ld},
    };

    #[test]
    fn _ld() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        cpu.registers.a = 5;
        cpu.registers.b = 8;
        assert_eq!(cpu.registers.a, 0x5);
        ld::ld((Register(A), Register(B)), &mut cpu, &mut bus);
        assert_eq!(cpu.registers.a, 0x8);
    }
}
