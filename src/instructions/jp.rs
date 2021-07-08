use crate::instructions::Bus;
use crate::instructions::Location;
use crate::instructions::CPU;

use super::Flag;

pub fn rst(size: u16, cpu: &mut CPU, bus: &mut Bus) {
    bus.generic_cycle();
    cpu.push_stack(cpu.registers.pc, bus);
    cpu.registers.pc = size;
}

fn check_flag(cpu: &mut CPU, flag: Flag) -> bool {
    match flag {
        Flag::FlagC => cpu.registers.flg_c(),
        Flag::FlagNC => cpu.registers.flg_nc(),
        Flag::FlagZ => cpu.registers.flg_z(),
        Flag::FlagNZ => cpu.registers.flg_nz(),
    }
}

pub fn jumping<F: FnOnce(&mut CPU, &mut Bus)>(
    jt: Option<Flag>,
    cpu: &mut CPU,
    bus: &mut Bus,
    f: F,
) {
    if let Some(false) = jt.map(|flag| check_flag(cpu, flag)) {
        return;
    }
    f(cpu, bus);
    bus.generic_cycle();
}

pub fn jp(jump_type: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.next_u16(bus);
    jumping(jump_type, cpu, bus, |cpu, _| cpu.registers.pc = address);
}

pub fn jr(jump_type: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.next_u8(bus) as i8;
    let address = cpu.registers.pc.wrapping_add(offset as u16);
    jumping(jump_type, cpu, bus, |cpu, _| {
        cpu.registers.pc = address;
    });
}

pub fn jp_hl(cpu: &mut CPU, _bus: &mut Bus) {
    cpu.registers.pc = cpu.registers.hl();
}

pub fn ret(jump_type: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    jumping(jump_type, cpu, bus, |cpu, bus| {
        cpu.registers.pc = cpu.pop_stack(bus);
    });
    if jump_type.is_some() {
        bus.generic_cycle();
    }
}
pub fn reti(cpu: &mut CPU, bus: &mut Bus) {
    bus.enable_interrupts();
    let addr = cpu.pop_stack(bus);
    cpu.registers.pc = addr;
    bus.generic_cycle();
}

pub fn call(jump_type: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.next_u16(bus);
    jumping(jump_type, cpu, bus, |cpu, bus| {
        cpu.push_stack(cpu.registers.pc, bus);
        cpu.registers.pc = address;
    });
}
