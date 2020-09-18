use super::*;
use crate::instructions::{Instr, Location::*};

//https://github.com/CTurt/Cinoop/blob/990e7d92b759892e98a450b4979e887865d6757f/source/cpu.c
// TODO, Add tests that have variable tick timings.
// A value of 0 means that instruction is ignored in testing.
pub const EXPECTED_TICKS: [usize; 256] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, 4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4,
    8, 4, 0, 12, 8, 8, 4, 4, 8, 4, 0, 8, 8, 8, 4, 4, 8, 4, 0, 12, 8, 8, 12, 12, 12, 4, 0, 8, 8, 8,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4,
    4, 4, 8, 4, 0, 12, 0, 16, 0, 16, 8, 16, 0, 16, 0, 0, 0, 24, 8, 16, 0, 12, 0, 0, 0, 16, 8, 16,
    0, 16, 0, 0, 0, 0, 8, 16, 12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16, 12, 12, 8, 4,
    0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16,
];

#[test]
fn ticks_expected() {
    let mut cpu = CPU::new();
    let mut bus = Bus::new(vec![]);
    let mut i = 0;
    while i < INSTR_TABLE.len() {
        if EXPECTED_TICKS[i] == 0 {
            i += 1;
            continue;
        }
        let instr = INSTR_TABLE[i];
        let time = time_instr(instr, &mut cpu, &mut bus);
        assert_eq!(
            time,
            EXPECTED_TICKS[i] / 4,
            "{:02x} {:?} was {} ticks, but expected {}",
            i,
            instr,
            time,
            EXPECTED_TICKS[i] / 4
        );
        i += 1
    }
}

fn time_instr(instr: Instr, cpu: &mut CPU, bus: &mut Bus) -> usize {
    let before = bus.clock;
    bus.generic_cycle();
    instr.execute(cpu, bus);
    let after = bus.clock;
    return after - before;
}

#[test]
fn ticks_cb_instr() {
    for instr in 0x00..=0xFF {
        let mut cpu = CPU::new();
        let mut bus = Bus::new(vec![]);
        let before = bus.clock;
        cpu.registers.pc = 0;
        bus.in_bios = 1;
        bus.memory[0x00] = instr;
        bus.generic_cycle();
        Instr::CB.execute(&mut cpu, &mut bus);
        let after = bus.clock;
        if let Location::Register(_) = CPU::cb_location(instr) {
            assert_eq!(after - before, 2, "Opcode failed: {:02x}", instr);
        } else {
            assert_eq!(after - before, 4, "Opcode failed: {:02x}", instr);
        }
    }
}

#[test]
fn ticks_expected_jumps() {
    let mut cpu = CPU::new();
    let mut bus = Bus::new(vec![]);
    let time = time_instr(Instr::JP(None), &mut cpu, &mut bus);
    assert_eq!(time, 4);

    let time = time_instr(Instr::CALL(None), &mut cpu, &mut bus);
    assert_eq!(time, 6);

    let time = time_instr(Instr::RET(None), &mut cpu, &mut bus);
    assert_eq!(time, 4);

    let pos_flags = [Flag::FlagZ, Flag::FlagC];
    for flag in &pos_flags {
        let time = time_instr(Instr::JP(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 3);
    }

    let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
    for flag in &neg_flags {
        let time = time_instr(Instr::JP(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 4);
    }

    let pos_flags = [Flag::FlagZ, Flag::FlagC];
    for flag in &pos_flags {
        let time = time_instr(Instr::CALL(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 3);
    }

    let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
    for flag in &neg_flags {
        let time = time_instr(Instr::CALL(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 6);
    }

    let pos_flags = [Flag::FlagZ, Flag::FlagC];
    for flag in &pos_flags {
        let time = time_instr(Instr::RET(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 2);
    }

    let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
    for flag in &neg_flags {
        let time = time_instr(Instr::RET(Some(*flag)), &mut cpu, &mut bus);
        assert_eq!(time, 5);
    }
}

#[test]
fn ld() {
    let mut cpu = CPU::new();
    cpu.registers.a = 5;
    cpu.registers.b = 8;
    let mut bus = Bus::new(vec![]);
    assert_eq!(cpu.registers.a, 0x5);
    Instr::LD(Register(A), Register(B)).execute(&mut cpu, &mut bus);
    assert_eq!(cpu.registers.a, 0x8);
}

#[test]
fn pop_af() {
    let mut cpu = CPU::new();
    let mut bus = Bus::new(vec![]);
    cpu.registers.b = 0x12; //      ld   bc,$1200
    cpu.registers.c = 0x00;
    for i in 0..0xFF {
        // -    push bc
        Instr::PUSH(Register(BC)).execute(&mut cpu, &mut bus);
        //      pop  af
        Instr::POP(Register(AF)).execute(&mut cpu, &mut bus);
        //      push af
        Instr::PUSH(Register(AF)).execute(&mut cpu, &mut bus);
        //      pop  de
        Instr::POP(Register(DE)).execute(&mut cpu, &mut bus);
        //      ld   a,c
        Instr::LD(Register(A), Register(C)).execute(&mut cpu, &mut bus);
        //      and  $F0
        Instr::AND(Literal(U8(0xF0))).execute(&mut cpu, &mut bus);
        cpu.dump_state();
        //      cp   e
        Instr::CP(Register(E)).execute(&mut cpu, &mut bus);
        assert!(
            !cpu.registers.flg_nz(),
            "Test {}: State: {:#}",
            i,
            cpu.registers
        );
        Instr::INC(Register(B)).execute(&mut cpu, &mut bus);
        Instr::INC(Register(C)).execute(&mut cpu, &mut bus);
    }
}
