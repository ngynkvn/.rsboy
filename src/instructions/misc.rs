use crate::{
    bus::Bus,
    cpu::{CPU, value::Writable},
};

use super::Register;

pub const fn daa(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.a = cpu.bcd_adjust(cpu.registers.a);
}
pub fn push(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.registers.fetch_u16(register);
    cpu.push_stack(value, bus);
    bus.generic_cycle();
}
pub fn pop(register: Register, cpu: &mut CPU, bus: &mut Bus) {
    let addr = cpu.pop_stack(bus);
    addr.to_register(&mut cpu.registers, register);
}

pub const fn halt(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.halt = true;
}
