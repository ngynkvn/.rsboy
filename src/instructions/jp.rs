//! Jump, Call, and Return instructions

use crate::{bus::Bus, cpu::CPU, operand::Reg16};

use super::Flag;

/// Check if flag condition is met
const fn check_flag(cpu: &CPU, flag: Flag) -> bool {
    match flag {
        Flag::FlagC => cpu.registers.flg_c(),
        Flag::FlagNC => cpu.registers.flg_nc(),
        Flag::FlagZ => cpu.registers.flg_z(),
        Flag::FlagNZ => cpu.registers.flg_nz(),
    }
}

/// Execute a jump operation with optional flag condition
fn jumping<F: FnOnce(&mut CPU, &mut Bus)>(
    condition: Option<Flag>,
    cpu: &mut CPU,
    bus: &mut Bus,
    f: F,
) {
    if condition.is_some_and(|flag| !check_flag(cpu, flag)) {
        return;
    }
    f(cpu, bus);
    bus.generic_cycle();
}

/// JP nn or JP cc,nn - absolute jump
pub fn jp(condition: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.next_u16(bus);
    jumping(condition, cpu, bus, |cpu, _| cpu.registers.pc = address);
}

/// JR e or JR cc,e - relative jump
pub fn jr(condition: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.next_u8(bus) as i8;
    let address = cpu.registers.pc.wrapping_add(i16::from(offset) as u16);
    jumping(condition, cpu, bus, |cpu, _| {
        cpu.registers.pc = address;
    });
}

/// JP HL - jump to address in HL
pub fn jp_hl(cpu: &mut CPU) {
    cpu.registers.pc = cpu.registers.get_r16(Reg16::HL);
}

/// RET or RET cc - return from subroutine
pub fn ret(condition: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    jumping(condition, cpu, bus, |cpu, bus| {
        cpu.registers.pc = cpu.pop_stack(bus);
    });
    if condition.is_some() {
        bus.generic_cycle();
    }
}

/// RETI - return from interrupt
pub fn reti(cpu: &mut CPU, bus: &mut Bus) {
    bus.enable_interrupts();
    let addr = cpu.pop_stack(bus);
    cpu.registers.pc = addr;
    bus.generic_cycle();
}

/// CALL nn or CALL cc,nn - call subroutine
pub fn call(condition: Option<Flag>, cpu: &mut CPU, bus: &mut Bus) {
    let address = cpu.next_u16(bus);
    jumping(condition, cpu, bus, |cpu, bus| {
        cpu.push_stack(cpu.registers.pc, bus);
        cpu.registers.pc = address;
    });
}

/// RST vec - restart to fixed address
pub fn rst(vector: u8, cpu: &mut CPU, bus: &mut Bus) {
    bus.generic_cycle();
    cpu.push_stack(cpu.registers.pc, bus);
    cpu.registers.pc = u16::from(vector);
}

#[cfg(test)]
mod test {
    use crate::{
        bus::Bus,
        cpu::CPU,
        instructions::{jp::jr, Flag},
    };

    #[test]
    fn jr_negative_offset() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        cpu.registers.pc = 0x000A + 1;
        bus.bootrom[0x0007] = 0x76;
        bus.bootrom[0x000A] = 0x20;
        bus.bootrom[0x000B] = 0xFB; // -5

        assert_eq!(cpu.registers.pc, 0x000B);
        jr(Some(Flag::FlagNZ), &mut cpu, &mut bus);
        assert_eq!(cpu.registers.pc, 0x0007);
    }
}
