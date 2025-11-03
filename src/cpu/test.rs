#[allow(dead_code)]
use tracing::info_span;

use crate::{
    bus::Bus,
    cpu::{CPU, CPUState, Stage},
    instructions::Instr,
};

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
    use super::*;
    use crate::{cpu::CPU, instructions::INSTR_TABLE};
    unsafe { std::env::set_var("RUST_LOG", "trace") };
    crate::constants::setup_logger().unwrap();
    for (i, instr) in INSTR_TABLE.iter().enumerate() {
        let mut cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        info!("Memory: {:?}", &bus.memory[0..10]);
        bus.in_bios = 1;
        bus.memory[0] = i as u8;
        if EXPECTED_TICKS[i] == 0 {
            continue;
        }
        info!(
            "Testing {}: {}: {}? {:?}",
            instr,
            i,
            cpu.registers.pc,
            &bus.memory[0..10]
        );
        let time = time_instr(i as u8, &mut cpu, &mut bus);
        assert_eq!(
            time,
            EXPECTED_TICKS[i] / 4,
            "{:02x} {} was {} ticks, but expected {}",
            i,
            instr,
            time,
            EXPECTED_TICKS[i] / 4
        );
        tracing::info!("OK");
    }
}

#[allow(dead_code)]
fn time_instr(instr: u8, cpu: &mut CPU, bus: &mut Bus) -> usize {
    const UPPER_LIMIT: usize = 10;
    let before = bus.clock;
    let mut prev = cpu.state;
    let _span = info_span!("Instr", instr = %Instr::from(instr)).entered();
    for _ in 0..UPPER_LIMIT {
        if matches!(
            (&prev, &cpu.state),
            (
                &CPUState::Running(Stage::Execute),
                &CPUState::Running(Stage::Fetch)
            )
        ) {
            break;
        }
        prev = cpu.state;
        cpu.step(bus);
        tracing::info!(
            clock = bus.clock,
            prev = ?prev,
            cpu.state = ?cpu.state,
            "Memory: {:?}",
            &bus.memory[0..10]
        );
    }
    let after = bus.clock;
    after - before
}

// #[test]
// fn ticks_expected_jumps() {
//     let mut cpu = CPU::new();
//     let mut bus = Bus::new(vec![], None);
//     let time = time_instr(Instr::JP(None), &mut cpu, &mut bus);
//     assert_eq!(time, 4);

//     let time = time_instr(Instr::CALL(None), &mut cpu, &mut bus);
//     assert_eq!(time, 6);

//     let time = time_instr(Instr::RET(None), &mut cpu, &mut bus);
//     assert_eq!(time, 4);

//     let time = time_instr(Instr::JR(None), &mut cpu, &mut bus);
//     assert_eq!(time, 3);

//     let pos_flags = [Flag::FlagZ, Flag::FlagC];
//     for flag in &pos_flags {
//         let time = time_instr(Instr::JP(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 3);
//     }

//     let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
//     for flag in &neg_flags {
//         let time = time_instr(Instr::JP(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 4);
//     }

//     let pos_flags = [Flag::FlagZ, Flag::FlagC];
//     for flag in &pos_flags {
//         let time = time_instr(Instr::CALL(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 3);
//     }

//     let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
//     for flag in &neg_flags {
//         let time = time_instr(Instr::CALL(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 6);
//     }

//     let pos_flags = [Flag::FlagZ, Flag::FlagC];
//     for flag in &pos_flags {
//         let time = time_instr(Instr::RET(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 2);
//     }

//     let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
//     for flag in &neg_flags {
//         let time = time_instr(Instr::RET(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 5);
//     }

//     let pos_flags = [Flag::FlagZ, Flag::FlagC];
//     for flag in &pos_flags {
//         let time = time_instr(Instr::JR(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 2);
//     }

//     let neg_flags = [Flag::FlagNZ, Flag::FlagNC];
//     for flag in &neg_flags {
//         let time = time_instr(Instr::JR(Some(*flag)), &mut cpu, &mut bus);
//         assert_eq!(time, 3);
//     }
// }

// #[test]
// fn pop_af() {
//     let mut cpu = CPU::new();
//     let mut bus = Bus::new(vec![], None);
//     cpu.registers.b = 0x12; //      ld   bc,$1200
//     cpu.registers.c = 0x00;
//     cpu.registers.h = 0xF0;
//     for i in 0..0xFF {
//         // -    push bc
//         let opcode = Instr::PUSH(Register(BC)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      pop  af
//         let opcode = Instr::POP(Register(AF)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      push af
//         let opcode = Instr::PUSH(Register(AF)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      pop  de
//         let opcode = Instr::POP(Register(DE)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      ld   a,c
//         let opcode = Instr::LD(Register(A), Register(C)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      and  $F0
//         let opcode = Instr::AND(Register(H)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         //      cp   e
//         let opcode = Instr::CP(Register(E)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         assert!(
//             !cpu.registers.flg_nz(),
//             "Test {}: State: {:#}",
//             i,
//             cpu.registers
//         );
//         let opcode = Instr::INC(Register(B)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//         let opcode = Instr::INC(Register(C)).into();
//         cpu.opcode = opcode;
//         cpu.execute_op(&mut bus);
//     }
// }

#[test]
fn fetch_execute_overlap() {
    use crate::{cpu::CPU, instructions::Register, location::Address};
    unsafe { std::env::set_var("RUST_LOG", "trace") };
    crate::constants::setup_logger().unwrap();
    let mut cpu = CPU::new();
    let mem = vec![
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::NOOP.into(),
        Instr::INC(Address::Register(Register::A)).into(),
        Instr::LD(Address::Register(Register::A), Address::MemOffsetImm).into(),
        10,
        Instr::RST(0x08).into(),
    ];
    cpu.registers.pc = 0x0A;
    let mut bus = Bus::new(&mem, None);
    bus.in_bios = 1;
    for _ in 0..12 {
        cpu.step(&mut bus);
    }
}
