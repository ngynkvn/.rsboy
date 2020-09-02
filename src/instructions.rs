use self::Flag::*;
use self::Instr::*;
use self::JumpType::*;
use self::Location::*;
use self::Register::*;
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
    Literal(u16),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum JumpType {
    Always,
    If(Flag),
    To(Location),
}

#[derive(Debug, Copy, Clone)]
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
    JR(JumpType),
    STOP,
    DisableInterrupts,
    EnableInterrupts,
    JP(JumpType),
    RET(JumpType),
    RETI,
    DAA,
    POP(Location),
    PUSH(Location),
    NOT(Location),
    CALL(JumpType),
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
    JR(Always),                            //0x18
    ADDHL(Register(DE)),                   //0x19
    LD(Register(A), Memory(DE)),           //0x1A
    DEC(Register(DE)),                     //0x1B
    INC(Register(E)),                      //0x1C
    DEC(Register(E)),                      //0x1D
    LD(Register(E), Immediate(1)),         //0x1E
    RRA,                                   //0x1F
    JR(If(FlagNZ)),                        //0x20
    LD(Register(HL), Immediate(2)),        //0x21
    LDI(Memory(HL), Register(A)),          //0x22
    INC(Register(HL)),                     //0x23
    INC(Register(H)),                      //0x24
    DEC(Register(H)),                      //0x25
    LD(Register(H), Immediate(1)),         //0x26
    DAA,                                   //0x27
    JR(If(FlagZ)),                         //0x28
    ADDHL(Register(HL)),                   //0x29
    LDI(Register(A), Memory(HL)),          //0x2A
    DEC(Register(HL)),                     //0x2B
    INC(Register(L)),                      //0x2C
    DEC(Register(L)),                      //0x2D
    LD(Register(L), Immediate(1)),         //0x2E
    NOT(Register(A)),                      //0x2F
    JR(If(FlagNC)),                        //0x30
    LD(Register(SP), Immediate(2)),        //0x31
    LDD(Memory(HL), Register(A)),          //0x32
    INC(Register(SP)),                     //0x33
    INC(Memory(HL)),                       //0x34
    DEC(Memory(HL)),                       //0x35
    LD(Memory(HL), Immediate(1)),          //0x36
    SCF,                                   //0x37
    JR(If(FlagC)),                         //0x38
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
    RET(If(FlagNZ)),                       //0xC0
    POP(Register(BC)),                     //0xC1
    JP(If(FlagNZ)),                        //0xC2
    JP(Always),                            //0xC3
    CALL(If(FlagNZ)),                      //0xC4
    PUSH(Register(BC)),                    //0xC5
    ADD(Immediate(1)),                     //0xC6
    RST(0x0),                              //0xC7
    RET(If(FlagZ)),                        //0xC8
    RET(Always),                           //0xC9
    JP(If(FlagZ)),                         //0xCA
    CB,                                    //0xCB
    CALL(If(FlagZ)),                       //0xCC
    CALL(Always),                          //0xCD
    ADC(Immediate(1)),                     //0xCE
    RST(0x8),                              //0xCF
    RET(If(FlagNC)),                       //0xD0
    POP(Register(DE)),                     //0xD1
    JP(If(FlagNC)),                        //0xD2
    UNIMPLEMENTED,                         //0xD3
    CALL(If(FlagNC)),                      //0xD4
    PUSH(Register(DE)),                    //0xD5
    SUB(Immediate(1)),                     //0xD6
    RST(0x10),                             //0xD7
    RET(If(FlagC)),                        //0xD8
    RETI,                                  //0xD9
    JP(If(FlagC)),                         //0xDA
    UNIMPLEMENTED,                         //0xDB
    CALL(If(FlagC)),                       //0xDC
    UNIMPLEMENTED,                         //0xDD
    SBC(Immediate(1)),                     //0xDE
    RST(0x18), //0xDF Push present address onto stack. Jump to address $0000 + n.
    LD(MemOffsetImm, Register(A)), //0xE0
    POP(Register(HL)), //0xE1
    LD(MemOffsetRegister(C), Register(A)), //0xE2
    UNIMPLEMENTED, //0xE3
    UNIMPLEMENTED, //0xE4
    PUSH(Register(HL)), //0xE5
    AND(Immediate(1)), //0xE6
    RST(0x20), //0xE7
    ADDSP,     //0xE8
    JP(To(Register(HL))), //0xE9
    LD(MemoryImmediate, Register(A)), //0xEA
    UNIMPLEMENTED, //0xEB
    UNIMPLEMENTED, //0xEC
    UNIMPLEMENTED, //0xED
    XOR(Immediate(1)), //0xEE
    RST(0x28), //0xEF
    LD(Register(A), MemOffsetImm), //0xF0
    POP(Register(AF)), //0xF1
    LD(Register(A), MemOffsetRegister(C)), //0xF2
    DisableInterrupts, //0xF3
    UNIMPLEMENTED, //0xF4
    PUSH(Register(AF)), //0xF5
    OR(Immediate(1)), //0xF6
    RST(0x30), //0xF7
    LDSP,      //0xF8
    LD(Register(SP), Register(HL)), //0xF9
    LD(Register(A), MemoryImmediate), //0xFA
    EnableInterrupts, //0xFB
    UNIMPLEMENTED, //0xFC
    UNIMPLEMENTED, //0xFD
    CP(Immediate(1)), //0xFE
    RST(0x38), //0xFF
];
// (InstrLen, ASM String)
#[derive(Debug, Copy, Clone)]
pub struct Instruction(pub usize, pub &'static str);

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}
