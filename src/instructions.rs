use self::Flag::*;
use self::Instr::*;
use self::Location::*;
use self::Register::*;
use crate::{
    bus::Bus,
    cpu::{value::Value, value::Value::*, value::Writable, CPU},
};
use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F, //FLAGS
    H,
    L,
    SP,
    PC,
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Flag {
    FlagNZ,
    FlagZ,
    FlagC,
    FlagNC,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Location {
    Memory(Register),
    Immediate(usize), // Bytes
    Register(Register),
    MemOffsetImm,
    MemoryImmediate,
    MemOffsetRegister(Register),
    Literal(Value),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

type Condition = Option<Flag>;

pub trait Executable {
    fn execute(self, cpu: &mut CPU, bus: &mut Bus);
}

impl Register {
    pub fn is_dual_register(self) -> bool {
        match self {
            HL => true,
            BC => true,
            DE => true,
            SP => true,
            _ => false,
        }
    }
}

impl Location {
    pub fn is_dual_register(self) -> bool {
        if let Register(r) = self {
            r.is_dual_register()
        } else {
            false
        }
    }
}

impl Executable for Instr {
    fn execute(self, cpu: &mut CPU, bus: &mut Bus) {
        match self {
            LD(Register(SP), Register(HL)) => {
                cpu.registers.sp = cpu.registers.hl();
                bus.generic_cycle();
            }
            LD(into, from) => cpu.load(into, from, bus),
            LDD(into, from) => {
                cpu.load(into, from, bus);
                cpu.registers.dec(Register::HL);
            }
            LDI(into, from) => {
                cpu.load(into, from, bus);
                cpu.registers.inc(Register::HL);
            }
            LDSP => {
                let offset = cpu.next_u8(bus) as i8 as u16;
                let result = cpu.registers.sp.wrapping_add(offset); // todo ?
                let half_carry = (cpu.registers.sp & 0x0F).wrapping_add(offset & 0x0F) > 0x0F;
                let carry = (cpu.registers.sp & 0xFF).wrapping_add(offset & 0xFF) > 0xFF;
                cpu.write_into(Location::Register(HL), U16(result), bus);
                bus.generic_cycle();
                cpu.registers.set_zf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(half_carry);
                cpu.registers.set_cf(carry);
            }
            STOP => {
                println!("STOP: {:04x}", cpu.registers.pc - 1); // TODO ?
            }
            NOOP => (),
            RST(size) => {
                bus.generic_cycle();
                cpu.push_stack(cpu.registers.pc, bus);
                cpu.registers.pc = size as u16;
            }
            CP(location) => {
                let value = cpu.read_from(location, bus).into();
                cpu.registers.set_zf(cpu.registers.a == value);
                cpu.registers.set_nf(true);
                //https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu.rs#L156
                cpu.registers
                    .set_hf((cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0);
                cpu.registers.set_cf(cpu.registers.a < value);
            }
            ADD(location) => {
                let value = cpu.read_from(location, bus).into();
                let (result, carry) = cpu.registers.a.overflowing_add(value);
                //https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#L55
                let half_carry = (cpu.registers.a & 0x0f).checked_add(value | 0xf0).is_none();
                cpu.registers.a = result;
                cpu.registers.set_zf(cpu.registers.a == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(half_carry);
                cpu.registers.set_cf(carry);
            }
            SUB(location) => {
                let value = cpu.read_from(location, bus).into();
                let result = cpu.registers.a.wrapping_sub(value);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(true);
                cpu.registers.set_hf(
                    // Mooneye
                    (cpu.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
                );
                cpu.registers
                    .set_cf((cpu.registers.a as u16) < (value as u16));
                cpu.registers.a = result;
            }
            ADC(location) => {
                let value = cpu.read_from(location, bus).into();
                let carry = cpu.registers.flg_c() as u8;
                let result = cpu.registers.a.wrapping_add(value).wrapping_add(carry);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                // Maybe: See https://github.com/Gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/cpu/execute.rs#L55
                cpu.registers
                    .set_hf((cpu.registers.a & 0xf) + (value & 0xf) + carry > 0xf);
                cpu.registers
                    .set_cf(cpu.registers.a as u16 + value as u16 + carry as u16 > 0xff);
                cpu.registers.a = result;
            }
            ADDHL(location) => {
                let hl = cpu.registers.hl();
                if let U16(value) = cpu.read_from(location, bus) {
                    if location.is_dual_register() {
                        bus.generic_cycle();
                    }
                    let (result, overflow) = hl.overflowing_add(value);
                    let [h, l] = result.to_be_bytes();
                    cpu.registers.h = h;
                    cpu.registers.l = l;
                    cpu.registers.set_nf(false);
                    cpu.registers
                        .set_hf((hl & 0xfff) + (value & 0xfff) > 0x0fff);
                    cpu.registers.set_cf(overflow);
                } else {
                    unimplemented!()
                }
            }
            AND(location) => {
                let value: u8 = cpu.read_from(location, bus).into();
                cpu.registers.a &= value;
                cpu.registers.set_zf(cpu.registers.a == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(true);
                cpu.registers.set_cf(false);
            }
            XOR(location) => {
                let value: u8 = cpu.read_from(location, bus).into();
                cpu.registers.a ^= value;
                cpu.registers.set_zf(cpu.registers.a == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(false);
            }
            OR(location) => {
                let value: u8 = cpu.read_from(location, bus).into();
                cpu.registers.a |= value;
                cpu.registers.set_zf(cpu.registers.a == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(false);
            }
            NOT(location) => {
                let value: u8 = cpu.read_from(location, bus).into();
                cpu.registers.a = !value;
                cpu.registers.set_nf(true);
                cpu.registers.set_hf(true);
            }
            CCF => {
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(!cpu.registers.flg_c());
            }
            SCF => {
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(true);
            }
            HALT => {
                //TODO
            }
            CB => cpu.handle_cb(bus),
            JP(jump_type) => {
                let address = cpu.next_u16(bus);
                cpu.jumping(jump_type, bus, |cpu, _| cpu.registers.pc = address);
            }
            JP_HL => {
                cpu.registers.pc = cpu.registers.hl();
            }
            JR(jump_type) => {
                let offset = cpu.next_u8(bus) as i8;
                let address = cpu.registers.pc.wrapping_add(offset as u16);
                cpu.jumping(jump_type, bus, |cpu, _| {
                    cpu.registers.pc = address;
                })
            }
            CALL(jump_type) => {
                let address = cpu.next_u16(bus);
                cpu.jumping(jump_type, bus, |cpu, bus| {
                    cpu.push_stack(cpu.registers.pc, bus);
                    cpu.registers.pc = address;
                });
            }
            DEC(Location::Memory(r)) => {
                let address = cpu.registers.fetch_u16(r);
                let value = bus.read_cycle(address);
                let result = value.wrapping_sub(1);
                bus.write_cycle(address, result);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(true);
                cpu.registers.set_hf(result & 0x0f == 0x0f);
            }
            DEC(Location::Register(r)) => {
                cpu.registers.dec(r);
                if r.is_dual_register() {
                    bus.generic_cycle();
                }
            }
            INC(Location::Memory(r)) => {
                let address = cpu.registers.fetch_u16(r);
                let value = bus.read_cycle(address);
                let result = value.wrapping_add(1);
                bus.write_cycle(address, result);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(value & 0x0f == 0x0f);
            }
            INC(Location::Register(r)) => {
                cpu.registers.inc(r);
                if r.is_dual_register() {
                    bus.generic_cycle();
                }
            }
            PUSH(Location::Register(r)) => {
                let addr = cpu.registers.fetch_u16(r);
                cpu.push_stack(addr, bus);
                bus.generic_cycle();
            }
            POP(Location::Register(r)) => {
                let addr = cpu.pop_stack(bus);
                addr.to_register(&mut cpu.registers, r);
            }
            RET(None) => cpu.jumping(None, bus, |cpu, bus| {
                cpu.registers.pc = cpu.pop_stack(bus);
            }),
            RET(jump_type) => {
                cpu.jumping(jump_type, bus, |cpu, bus| {
                    cpu.registers.pc = cpu.pop_stack(bus);
                });
                bus.generic_cycle();
            }
            RRA => {
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
            RRCA => {
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
            RLA => {
                let overflow = cpu.registers.a & 0x80 != 0;
                let result = cpu.registers.a << 1;
                cpu.registers.a = result | (cpu.registers.flg_c() as u8);
                cpu.registers.set_zf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_cf(overflow);
            }
            RLCA => {
                let carry = cpu.registers.a & 0x80 != 0;
                let result = cpu.registers.a << 1 | carry as u8;
                cpu.registers.a = result;
                cpu.registers.set_zf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_cf(carry);
            }
            ADDSP => {
                let offset = cpu.next_u8(bus) as i8 as i16 as u16;
                let sp = cpu.registers.sp;
                let result = cpu.registers.sp.wrapping_add(offset);
                bus.generic_cycle();
                bus.generic_cycle();
                let half_carry = ((sp & 0x0F) + (offset & 0x0F)) > 0x0F;
                let overflow = ((sp & 0xff) + (offset & 0xff)) > 0xff;
                cpu.registers.sp = result;
                cpu.registers.set_zf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(half_carry);
                cpu.registers.set_cf(overflow);
            }
            RETI => {
                bus.enable_interrupts();
                let addr = cpu.pop_stack(bus);
                cpu.registers.pc = addr;
                bus.generic_cycle();
            }
            DAA => {
                cpu.registers.a = cpu.bcd_adjust(cpu.registers.a);
            }
            EnableInterrupts => {
                bus.enable_interrupts();
            }
            DisableInterrupts => {
                bus.disable_interrupts();
            }
            UNIMPLEMENTED => unimplemented!(),
            SBC(l) => {
                let a = cpu.registers.a;
                let value: u8 = cpu.read_from(l, bus).into();
                let cy = cpu.registers.flg_c() as u8;
                let result = a.wrapping_sub(value).wrapping_sub(cy);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(true);
                cpu.registers.set_hf(
                    // Mooneye
                    (cpu.registers.a & 0xf)
                        .wrapping_sub(value & 0xf)
                        .wrapping_sub(cy)
                        & (0xf + 1)
                        != 0,
                );
                cpu.registers
                    .set_cf((cpu.registers.a as u16) < (value as u16) + (cy as u16));
                cpu.registers.a = result;
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instr {
    NOOP,
    UNIMPLEMENTED,
    LD(Location, Location), // (To, From)
    LDD(Location, Location),
    LDI(Location, Location),
    LDSP,
    INC(Location),
    DEC(Location),
    ADD(Location),
    ADDHL(Location),
    ADC(Location),
    SUB(Location),
    AND(Location),
    XOR(Location),
    OR(Location),
    CP(Location),
    SBC(Location),
    CB,
    JR(Condition),
    STOP,
    DisableInterrupts,
    EnableInterrupts,
    JP(Condition),
    JP_HL,
    RET(Condition),
    RETI,
    DAA,
    POP(Location),
    PUSH(Location),
    NOT(Location),
    CALL(Condition),
    RLCA,
    RRCA,
    RLA,
    RRA,
    SCF,
    CCF,
    ADDSP,
    HALT,
    RST(usize),
}

pub const INSTR_TABLE: [Instr; 256] = [
    NOOP,                                  //0x00
    LD(Register(BC), Immediate(2)),        //0x01
    LD(Memory(BC), Register(A)),           //0x02
    INC(Register(BC)),                     //0x03
    INC(Register(B)),                      //0x04
    DEC(Register(B)),                      //0x05
    LD(Register(B), Immediate(1)),         //0x06
    RLCA,                                  //0x07
    LD(Immediate(2), Register(SP)),        //0x08
    ADDHL(Register(BC)),                   //0x09
    LD(Register(A), Memory(BC)),           //0x0A
    DEC(Register(BC)),                     //0x0B
    INC(Register(C)),                      //0x0C
    DEC(Register(C)),                      //0x0D
    LD(Register(C), Immediate(1)),         //0x0E
    RRCA,                                  //0x0F
    STOP,                                  //0x10
    LD(Register(DE), Immediate(2)),        //0x11
    LD(Memory(DE), Register(A)),           //0x12
    INC(Register(DE)),                     //0x13
    INC(Register(D)),                      //0x14
    DEC(Register(D)),                      //0x15
    LD(Register(D), Immediate(1)),         //0x16
    RLA,                                   //0x17
    JR(None),                              //0x18
    ADDHL(Register(DE)),                   //0x19
    LD(Register(A), Memory(DE)),           //0x1A
    DEC(Register(DE)),                     //0x1B
    INC(Register(E)),                      //0x1C
    DEC(Register(E)),                      //0x1D
    LD(Register(E), Immediate(1)),         //0x1E
    RRA,                                   //0x1F
    JR(Some(FlagNZ)),                      //0x20
    LD(Register(HL), Immediate(2)),        //0x21
    LDI(Memory(HL), Register(A)),          //0x22
    INC(Register(HL)),                     //0x23
    INC(Register(H)),                      //0x24
    DEC(Register(H)),                      //0x25
    LD(Register(H), Immediate(1)),         //0x26
    DAA,                                   //0x27
    JR(Some(FlagZ)),                       //0x28
    ADDHL(Register(HL)),                   //0x29
    LDI(Register(A), Memory(HL)),          //0x2A
    DEC(Register(HL)),                     //0x2B
    INC(Register(L)),                      //0x2C
    DEC(Register(L)),                      //0x2D
    LD(Register(L), Immediate(1)),         //0x2E
    NOT(Register(A)),                      //0x2F
    JR(Some(FlagNC)),                      //0x30
    LD(Register(SP), Immediate(2)),        //0x31
    LDD(Memory(HL), Register(A)),          //0x32
    INC(Register(SP)),                     //0x33
    INC(Memory(HL)),                       //0x34
    DEC(Memory(HL)),                       //0x35
    LD(Memory(HL), Immediate(1)),          //0x36
    SCF,                                   //0x37
    JR(Some(FlagC)),                       //0x38
    ADDHL(Register(SP)),                   //0x39
    LDD(Register(A), Memory(HL)),          //0x3A
    DEC(Register(SP)),                     //0x3B
    INC(Register(A)),                      //0x3C
    DEC(Register(A)),                      //0x3D
    LD(Register(A), Immediate(1)),         //0x3E
    CCF,                                   //0x3F
    LD(Register(B), Register(B)),          //0x40
    LD(Register(B), Register(C)),          //0x41
    LD(Register(B), Register(D)),          //0x42
    LD(Register(B), Register(E)),          //0x43
    LD(Register(B), Register(H)),          //0x44
    LD(Register(B), Register(L)),          //0x45
    LD(Register(B), Memory(HL)),           //0x46
    LD(Register(B), Register(A)),          //0x47
    LD(Register(C), Register(B)),          //0x48
    LD(Register(C), Register(C)),          //0x49
    LD(Register(C), Register(D)),          //0x4A
    LD(Register(C), Register(E)),          //0x4B
    LD(Register(C), Register(H)),          //0x4C
    LD(Register(C), Register(L)),          //0x4D
    LD(Register(C), Memory(HL)),           //0x4E
    LD(Register(C), Register(A)),          //0x4F
    LD(Register(D), Register(B)),          //0x50
    LD(Register(D), Register(C)),          //0x51
    LD(Register(D), Register(D)),          //0x52
    LD(Register(D), Register(E)),          //0x53
    LD(Register(D), Register(H)),          //0x54
    LD(Register(D), Register(L)),          //0x55
    LD(Register(D), Memory(HL)),           //0x56
    LD(Register(D), Register(A)),          //0x57
    LD(Register(E), Register(B)),          //0x58
    LD(Register(E), Register(C)),          //0x59
    LD(Register(E), Register(D)),          //0x5A
    LD(Register(E), Register(E)),          //0x5B
    LD(Register(E), Register(H)),          //0x5C
    LD(Register(E), Register(L)),          //0x5D
    LD(Register(E), Memory(HL)),           //0x5E
    LD(Register(E), Register(A)),          //0x5F
    LD(Register(H), Register(B)),          //0x60
    LD(Register(H), Register(C)),          //0x61
    LD(Register(H), Register(D)),          //0x62
    LD(Register(H), Register(E)),          //0x63
    LD(Register(H), Register(H)),          //0x64
    LD(Register(H), Register(L)),          //0x65
    LD(Register(H), Memory(HL)),           //0x66
    LD(Register(H), Register(A)),          //0x67
    LD(Register(L), Register(B)),          //0x68
    LD(Register(L), Register(C)),          //0x69
    LD(Register(L), Register(D)),          //0x6A
    LD(Register(L), Register(E)),          //0x6B
    LD(Register(L), Register(H)),          //0x6C
    LD(Register(L), Register(L)),          //0x6D
    LD(Register(L), Memory(HL)),           //0x6E
    LD(Register(L), Register(A)),          //0x6F
    LD(Memory(HL), Register(B)),           //0x70
    LD(Memory(HL), Register(C)),           //0x71
    LD(Memory(HL), Register(D)),           //0x72
    LD(Memory(HL), Register(E)),           //0x73
    LD(Memory(HL), Register(H)),           //0x74
    LD(Memory(HL), Register(L)),           //0x75
    HALT,                                  //0x76
    LD(Memory(HL), Register(A)),           //0x77
    LD(Register(A), Register(B)),          //0x78
    LD(Register(A), Register(C)),          //0x79
    LD(Register(A), Register(D)),          //0x7A
    LD(Register(A), Register(E)),          //0x7B
    LD(Register(A), Register(H)),          //0x7C
    LD(Register(A), Register(L)),          //0x7D
    LD(Register(A), Memory(HL)),           //0x7E
    LD(Register(A), Register(A)),          //0x7F
    ADD(Register(B)),                      //0x80
    ADD(Register(C)),                      //0x81
    ADD(Register(D)),                      //0x82
    ADD(Register(E)),                      //0x83
    ADD(Register(H)),                      //0x84
    ADD(Register(L)),                      //0x85
    ADD(Memory(HL)),                       //0x86
    ADD(Register(A)),                      //0x87
    ADC(Register(B)),                      //0x88
    ADC(Register(C)),                      //0x89
    ADC(Register(D)),                      //0x8A
    ADC(Register(E)),                      //0x8B
    ADC(Register(H)),                      //0x8C
    ADC(Register(L)),                      //0x8D
    ADC(Memory(HL)),                       //0x8E
    ADC(Register(A)),                      //0x8F
    SUB(Register(B)),                      //0x90
    SUB(Register(C)),                      //0x91
    SUB(Register(D)),                      //0x92
    SUB(Register(E)),                      //0x93
    SUB(Register(H)),                      //0x94
    SUB(Register(L)),                      //0x95
    SUB(Memory(HL)),                       //0x96
    SUB(Register(A)),                      //0x97
    SBC(Register(B)),                      //0x98
    SBC(Register(C)),                      //0x99
    SBC(Register(D)),                      //0x92
    SBC(Register(E)),                      //0x93
    SBC(Register(H)),                      //0x94
    SBC(Register(L)),                      //0x9D
    SBC(Memory(HL)),                       //0x9E
    SBC(Register(A)),                      //0x9F
    AND(Register(B)),                      //0xA0
    AND(Register(C)),                      //0xA1
    AND(Register(D)),                      //0xA2
    AND(Register(E)),                      //0xA3
    AND(Register(H)),                      //0xA4
    AND(Register(L)),                      //0xA5
    AND(Memory(HL)),                       //0xA6
    AND(Register(A)),                      //0xA7
    XOR(Register(B)),                      //0xA8
    XOR(Register(C)),                      //0xA9
    XOR(Register(D)),                      //0xAA
    XOR(Register(E)),                      //0xAB
    XOR(Register(H)),                      //0xAC
    XOR(Register(L)),                      //0xAD
    XOR(Memory(HL)),                       //0xAE
    XOR(Register(A)),                      //0xAF
    OR(Register(B)),                       //0xB0
    OR(Register(C)),                       //0xB1
    OR(Register(D)),                       //0xB2
    OR(Register(E)),                       //0xB3
    OR(Register(H)),                       //0xB4
    OR(Register(L)),                       //0xB5
    OR(Memory(HL)),                        //0xB6
    OR(Register(A)),                       //0xB7
    CP(Register(B)),                       //0xB8
    CP(Register(C)),                       //0xB9
    CP(Register(D)),                       //0xBA
    CP(Register(E)),                       //0xBB
    CP(Register(H)),                       //0xBC
    CP(Register(L)),                       //0xBD
    CP(Memory(HL)),                        //0xBE
    CP(Register(A)),                       //0xBF
    RET(Some(FlagNZ)),                     //0xC0
    POP(Register(BC)),                     //0xC1
    JP(Some(FlagNZ)),                      //0xC2
    JP(None),                              //0xC3
    CALL(Some(FlagNZ)),                    //0xC4
    PUSH(Register(BC)),                    //0xC5
    ADD(Immediate(1)),                     //0xC6
    RST(0x0),                              //0xC7
    RET(Some(FlagZ)),                      //0xC8
    RET(None),                             //0xC9
    JP(Some(FlagZ)),                       //0xCA
    CB,                                    //0xCB
    CALL(Some(FlagZ)),                     //0xCC
    CALL(None),                            //0xCD
    ADC(Immediate(1)),                     //0xCE
    RST(0x8),                              //0xCF
    RET(Some(FlagNC)),                     //0xD0
    POP(Register(DE)),                     //0xD1
    JP(Some(FlagNC)),                      //0xD2
    UNIMPLEMENTED,                         //0xD3
    CALL(Some(FlagNC)),                    //0xD4
    PUSH(Register(DE)),                    //0xD5
    SUB(Immediate(1)),                     //0xD6
    RST(0x10),                             //0xD7
    RET(Some(FlagC)),                      //0xD8
    RETI,                                  //0xD9
    JP(Some(FlagC)),                       //0xDA
    UNIMPLEMENTED,                         //0xDB
    CALL(Some(FlagC)),                     //0xDC
    UNIMPLEMENTED,                         //0xDD
    SBC(Immediate(1)),                     //0xDE
    RST(0x18),                             //0xDF
    LD(MemOffsetImm, Register(A)),         //0xE0
    POP(Register(HL)),                     //0xE1
    LD(MemOffsetRegister(C), Register(A)), //0xE2
    UNIMPLEMENTED,                         //0xE3
    UNIMPLEMENTED,                         //0xE4
    PUSH(Register(HL)),                    //0xE5
    AND(Immediate(1)),                     //0xE6
    RST(0x20),                             //0xE7
    ADDSP,                                 //0xE8
    JP_HL,                                 //0xE9
    LD(MemoryImmediate, Register(A)),      //0xEA
    UNIMPLEMENTED,                         //0xEB
    UNIMPLEMENTED,                         //0xEC
    UNIMPLEMENTED,                         //0xED
    XOR(Immediate(1)),                     //0xEE
    RST(0x28),                             //0xEF
    LD(Register(A), MemOffsetImm),         //0xF0
    POP(Register(AF)),                     //0xF1
    LD(Register(A), MemOffsetRegister(C)), //0xF2
    DisableInterrupts,                     //0xF3
    UNIMPLEMENTED,                         //0xF4
    PUSH(Register(AF)),                    //0xF5
    OR(Immediate(1)),                      //0xF6
    RST(0x30),                             //0xF7
    LDSP,                                  //0xF8
    LD(Register(SP), Register(HL)),        //0xF9
    LD(Register(A), MemoryImmediate),      //0xFA
    EnableInterrupts,                      //0xFB
    UNIMPLEMENTED,                         //0xFC
    UNIMPLEMENTED,                         //0xFD
    CP(Immediate(1)),                      //0xFE
    RST(0x38),                             //0xFF
];

pub const INSTR_LENGTHS: [usize; 256] = [
    0, // 0x00
    2, // 0x01
    0, // 0x02
    0, // 0x03
    0, // 0x04
    0, // 0x05
    1, // 0x06
    0, // 0x07
    2, // 0x08
    0, // 0x09
    0, // 0x0a
    0, // 0x0b
    0, // 0x0c
    0, // 0x0d
    1, // 0x0e
    0, // 0x0f
    1, // 0x10
    2, // 0x11
    0, // 0x12
    0, // 0x13
    0, // 0x14
    0, // 0x15
    1, // 0x16
    0, // 0x17
    1, // 0x18
    0, // 0x19
    0, // 0x1a
    0, // 0x1b
    0, // 0x1c
    0, // 0x1d
    1, // 0x1e
    0, // 0x1f
    1, // 0x20
    2, // 0x21
    0, // 0x22
    0, // 0x23
    0, // 0x24
    0, // 0x25
    1, // 0x26
    0, // 0x27
    1, // 0x28
    0, // 0x29
    0, // 0x2a
    0, // 0x2b
    0, // 0x2c
    0, // 0x2d
    1, // 0x2e
    0, // 0x2f
    1, // 0x30
    2, // 0x31
    0, // 0x32
    0, // 0x33
    0, // 0x34
    0, // 0x35
    1, // 0x36
    0, // 0x37
    1, // 0x38
    0, // 0x39
    0, // 0x3a
    0, // 0x3b
    0, // 0x3c
    0, // 0x3d
    1, // 0x3e
    0, // 0x3f
    0, // 0x40
    0, // 0x41
    0, // 0x42
    0, // 0x43
    0, // 0x44
    0, // 0x45
    0, // 0x46
    0, // 0x47
    0, // 0x48
    0, // 0x49
    0, // 0x4a
    0, // 0x4b
    0, // 0x4c
    0, // 0x4d
    0, // 0x4e
    0, // 0x4f
    0, // 0x50
    0, // 0x51
    0, // 0x52
    0, // 0x53
    0, // 0x54
    0, // 0x55
    0, // 0x56
    0, // 0x57
    0, // 0x58
    0, // 0x59
    0, // 0x5a
    0, // 0x5b
    0, // 0x5c
    0, // 0x5d
    0, // 0x5e
    0, // 0x5f
    0, // 0x60
    0, // 0x61
    0, // 0x62
    0, // 0x63
    0, // 0x64
    0, // 0x65
    0, // 0x66
    0, // 0x67
    0, // 0x68
    0, // 0x69
    0, // 0x6a
    0, // 0x6b
    0, // 0x6c
    0, // 0x6d
    0, // 0x6e
    0, // 0x6f
    0, // 0x70
    0, // 0x71
    0, // 0x72
    0, // 0x73
    0, // 0x74
    0, // 0x75
    0, // 0x76
    0, // 0x77
    0, // 0x78
    0, // 0x79
    0, // 0x7a
    0, // 0x7b
    0, // 0x7c
    0, // 0x7d
    0, // 0x7e
    0, // 0x7f
    0, // 0x80
    0, // 0x81
    0, // 0x82
    0, // 0x83
    0, // 0x84
    0, // 0x85
    0, // 0x86
    0, // 0x87
    0, // 0x88
    0, // 0x89
    0, // 0x8a
    0, // 0x8b
    0, // 0x8c
    0, // 0x8d
    0, // 0x8e
    0, // 0x8f
    0, // 0x90
    0, // 0x91
    0, // 0x92
    0, // 0x93
    0, // 0x94
    0, // 0x95
    0, // 0x96
    0, // 0x97
    0, // 0x98
    0, // 0x99
    0, // 0x9a
    0, // 0x9b
    0, // 0x9c
    0, // 0x9d
    0, // 0x9e
    0, // 0x9f
    0, // 0xa0
    0, // 0xa1
    0, // 0xa2
    0, // 0xa3
    0, // 0xa4
    0, // 0xa5
    0, // 0xa6
    0, // 0xa7
    0, // 0xa8
    0, // 0xa9
    0, // 0xaa
    0, // 0xab
    0, // 0xac
    0, // 0xad
    0, // 0xae
    0, // 0xaf
    0, // 0xb0
    0, // 0xb1
    0, // 0xb2
    0, // 0xb3
    0, // 0xb4
    0, // 0xb5
    0, // 0xb6
    0, // 0xb7
    0, // 0xb8
    0, // 0xb9
    0, // 0xba
    0, // 0xbb
    0, // 0xbc
    0, // 0xbd
    0, // 0xbe
    0, // 0xbf
    0, // 0xc0
    0, // 0xc1
    2, // 0xc2
    2, // 0xc3
    2, // 0xc4
    0, // 0xc5
    1, // 0xc6
    0, // 0xc7
    0, // 0xc8
    0, // 0xc9
    2, // 0xca
    1, // 0xcb
    2, // 0xcc
    2, // 0xcd
    1, // 0xce
    0, // 0xcf
    0, // 0xd0
    0, // 0xd1
    2, // 0xd2
    0, // 0xd3
    2, // 0xd4
    0, // 0xd5
    1, // 0xd6
    0, // 0xd7
    0, // 0xd8
    0, // 0xd9
    2, // 0xda
    0, // 0xdb
    2, // 0xdc
    0, // 0xdd
    1, // 0xde
    0, // 0xdf
    1, // 0xe0
    0, // 0xe1
    0, // 0xe2
    0, // 0xe3
    0, // 0xe4
    0, // 0xe5
    1, // 0xe6
    0, // 0xe7
    1, // 0xe8
    0, // 0xe9
    2, // 0xea
    0, // 0xeb
    0, // 0xec
    0, // 0xed
    1, // 0xee
    0, // 0xef
    1, // 0xf0
    0, // 0xf1
    0, // 0xf2
    0, // 0xf3
    0, // 0xf4
    0, // 0xf5
    1, // 0xf6
    0, // 0xf7
    1, // 0xf8
    0, // 0xf9
    2, // 0xfa
    0, // 0xfb
    0, // 0xfc
    0, // 0xfd
    1, // 0xfe
    0, // 0xff
];
