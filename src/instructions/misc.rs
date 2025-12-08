//! Miscellaneous instructions (PUSH, POP)

use crate::{bus::Bus, cpu::CPU, operand::Reg16};

/// PUSH rr - Push 16-bit register to stack
pub fn push(reg: Reg16, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.registers.get_r16(reg);
    cpu.push_stack(value, bus);
    bus.generic_cycle();
}

/// POP rr - Pop 16-bit register from stack
pub fn pop(reg: Reg16, cpu: &mut CPU, bus: &mut Bus) {
    let value = cpu.pop_stack(bus);
    cpu.registers.set_r16(reg, value);
}
