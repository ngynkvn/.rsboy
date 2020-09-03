use super::*;
use crate::instructions::{Instr, Location::*};

#[test]
fn ld() -> Result<(), String> {
    let mut cpu = CPU::new();
    cpu.registers.a = 5;
    cpu.registers.b = 8;
    let mut bus = Bus::new(vec![]);
    assert_eq!(cpu.registers.a, 0x5);
    cpu.perform_instruction(Instr::LD(Register(A), Register(B)), &mut bus)?;
    assert_eq!(cpu.registers.a, 0x8);
    Ok(())
}

#[test]
fn pop_af() -> Result<(), String> {
    let mut cpu = CPU::new();
    let mut bus = Bus::new(vec![]);
    cpu.registers.b = 0x12; //      ld   bc,$1200
    cpu.registers.c = 0x00;
    for i in 0..0xFF {
        // -    push bc
        cpu.perform_instruction(Instr::PUSH(Register(BC)), &mut bus)?;
        //      pop  af
        cpu.perform_instruction(Instr::POP(Register(AF)), &mut bus)?;
        //      push af
        cpu.perform_instruction(Instr::PUSH(Register(AF)), &mut bus)?;
        //      pop  de
        cpu.perform_instruction(Instr::POP(Register(DE)), &mut bus)?;
        //      ld   a,c
        cpu.perform_instruction(Instr::LD(Register(A), Register(C)), &mut bus)?;
        //      and  $F0
        cpu.perform_instruction(Instr::AND(Literal(0xF0)), &mut bus)?;
        cpu.dump_state();
        //      cp   e
        cpu.perform_instruction(Instr::CP(Register(E)), &mut bus)?;
        assert!(
            !cpu.registers.flg_nz(),
            "Test {}: State: {:#}",
            i,
            cpu.registers
        );
        cpu.perform_instruction(Instr::INC(Register(B)), &mut bus)?;
        cpu.perform_instruction(Instr::INC(Register(C)), &mut bus)?;
    }
    Ok(())
}

#[test]
fn ldbc() -> Result<(), String> {
    let mut cpu = CPU::new();
    cpu.registers.b = 0x21;
    cpu.registers.c = 0x21;
    assert_eq!(cpu.registers.bc(), 0x2121);
    let mut bus = Bus::new(vec![]); // LD BC, d16
                                    // TODO, make Bus a trait that I can inherit from so I can mock it.
    bus.bootrom[0] = 0x01;
    bus.bootrom[1] = 0x22;
    bus.bootrom[2] = 0x11;

    cpu.read_instruction(&mut bus)?;
    assert_eq!(cpu.registers.bc(), 0x1122);
    Ok(())
}

// #[test]
// fn test_ld16() {
//     let mut cpu = CPU::new(false);
//     cpu.registers.sp = 0xFFFF;
//     cpu.registers.b = 0x21;
//     cpu.registers.c = 0x21;
//     assert_eq!(cpu.registers.bc(), 0x2121);
// }
// }
