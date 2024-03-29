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

// #[test]
// fn ticks_expected() {
//     let mut i = 0;
//     while i < INSTR_TABLE.len() {
//         let mut cpu = CPU::new();
//         let mut bus = Bus::new(vec![], None);
//         bus.in_bios = 1;
//         if EXPECTED_TICKS[i] == 0 {
//             i += 1;
//             continue;
//         }
//         let instr = INSTR_TABLE[i];
//         print!("Testing {:?}? ", instr);
//         let time = time_instr(instr, &mut cpu, &mut bus);
//         assert_eq!(
//             time,
//             EXPECTED_TICKS[i] / 4,
//             "{:02x} {:?} was {} ticks, but expected {}",
//             i,
//             instr,
//             time,
//             EXPECTED_TICKS[i] / 4
//         );
//         println!("OK");
//         i += 1
//     }
// }

// fn time_instr(instr: Instr, cpu: &mut CPU, bus: &mut Bus) -> usize {
//     let before = bus.clock;
//     bus.generic_cycle();
//     let opcode = instr.into();
//     cpu.opcode = opcode;
//     cpu.execute_op(bus);
//     let after = bus.clock;
//     after - before
// }

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
