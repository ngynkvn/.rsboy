use self::Direction::*;
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

#[derive(Debug, Copy, Clone)]
pub enum Flag {
    FlagNZ,
    FlagZ,
    FlagC,
    FlagNC,
}

#[derive(Debug, Copy, Clone)]
pub enum Location {
    Memory(Register),
    Immediate(usize), // Bytes
    Register(Register),
    MemOffsetImm,
    MemoryImmediate,
    MemOffsetRegister(Register),
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Debug, Copy, Clone)]
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
    RLCA,       //0x07
    LD(Immediate(2), Register(SP)),        //0x08
    ADDHL( Register(BC)),      //0x09
    LD(Register(A), Memory(BC)),           //0x0A
    DEC(Register(BC)),                     //0x0B
    INC(Register(C)),                      //0x0C
    DEC(Register(C)),                      //0x0D
    LD(Register(C), Immediate(1)),         //0x0E
    RRCA,                         //0x0F
    STOP,                                  //0x10
    LD(Register(DE), Immediate(2)),        //0x11
    LD(Memory(DE), Register(A)),           //0x12
    INC(Register(DE)),                     //0x13
    INC(Register(D)),                      //0x14
    DEC(Register(D)),                      //0x15
    LD(Register(D), Immediate(1)),         //0x16
    RLA,       //0x17
    JR(Always),                            //0x18
    ADDHL( Register(DE)),      //0x19
    LD(Register(A), Memory(DE)),           //0x1A
    DEC(Register(DE)),                         //0x1B
    INC(Register(E)),                      //0x1C
    DEC(Register(E)),                      //0x1D
    LD(Register(E), Immediate(1)),         //0x1E
    RRA,      //0x1F
    JR(If(FlagNZ)),                        //0x20
    LD(Register(HL), Immediate(2)),        //0x21
    LDI(Memory(HL), Register(A)),          //0x22
    INC(Register(HL)),                     //0x23
    INC(Register(H)),                      //0x24
    DEC(Register(H)),                      //0x25
    LD(Register(H), Immediate(1)),         //0x26
    DAA,                                   //0x27
    JR(If(FlagZ)),                         //0x28
    ADDHL( Register(HL)),      //0x29
    LDI(Register(A), Memory(HL)),                                  //0x2A
    DEC(Register(HL)),                         //0x2B
    INC(Register(L)),                      //0x2C
    DEC(Register(L)),                      //0x2D
    LD(Register(L), Immediate(1)),         //0x2E
    NOT(Register(A)),                      //0x2F
    JR(If(FlagNC)),                        //0x30
    LD(Register(SP), Immediate(2)),        //0x31
    LDD(Memory(HL), Register(A)),          //0x32
    INC(Register(SP)),                     //0x33
    INC(Memory(HL)),                     //0x34
    DEC(Memory(HL)),                     //0x35
    LD(Memory(HL), Immediate(1)),                                  //0x36
    SCF,                         //0x37
    JR(If(FlagC)),                         //0x38
    ADDHL( Register(SP)),      //0x39
    LDD(Register(A), Memory(HL)),          //0x3A
    DEC(Register(SP)),                     //0x3B
    INC(Register(A)),                      //0x3C
    DEC(Register(A)),                      //0x3D
    LD(Register(A), Immediate(1)),         //0x3E
    CCF,                         //0x3F
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
    UNIMPLEMENTED,                         //0x76
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
    ADD(Memory(HL)),                     //0x86
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
    SBC(Register(B)),                         //0x98
    SBC(Register(C)),                      //0x99
    SBC(Register(D)),                      //0x92
    SBC(Register(E)),                      //0x93
    SBC(Register(H)),                      //0x94
    SBC(Register(L)),                      //0x9D
    SBC(Memory(HL)),                         //0x9E
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
    JP(Always),                  //0xC3
    CALL(If(FlagNZ)),                      //0xC4
    PUSH(Register(BC)),                    //0xC5
    ADD(Immediate(1)),                     //0xC6
    RST(0x0),                         //0xC7
    RET(If(FlagZ)),                        //0xC8
    RET(Always),                           //0xC9
    JP(If(FlagZ)),                         //0xCA
    CB,                                    //0xCB
    CALL(If(FlagZ)),                       //0xCC
    CALL(Always),                          //0xCD
    ADC(Immediate(1)),                     //0xCE
    RST(0x8),                                //0xCF
    RET(If(FlagNC)),                         //0xD0
    POP(Register(DE)),                     //0xD1
    JP(If(FlagNC)),                         //0xD2
    UNIMPLEMENTED,                         //0xD3
    CALL(If(FlagNC)),                      //0xD4
    PUSH(Register(DE)),                    //0xD5
    SUB(Immediate(1)),                         //0xD6
    RST(0x10),                               //0xD7
    RET(If(FlagC)),                         //0xD8
    RETI,                                  //0xD9
    JP(If(FlagC)),                         //0xDA
    UNIMPLEMENTED,                         //0xDB
    CALL(If(FlagC)),                         //0xDC
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
    ADDSP, //0xE8
    JP(To(Memory(HL))), //0xE9
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
    UNIMPLEMENTED, //0xF8
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
// Lifted from CINOOP's cpu.c
// https://github.com/CTurt/Cinoop/blob/master/source/cpu.c

pub const INSTRUCTION_TABLE: [Instruction; 256] = [
    Instruction(0, "NOP"),                   // 0x00
    Instruction(2, "LD BC, 0x??"),           // 0x01
    Instruction(0, "LD (HL), A"),            // 0x02
    Instruction(0, "INC BC"),                // 0x03
    Instruction(0, "INC B"),                 // 0x04
    Instruction(0, "DEC B"),                 // 0x05
    Instruction(1, "LD B, 0x??"),            // 0x06
    Instruction(0, "RLCA"),                  // 0x07
    Instruction(2, "LD (0x??), SP"),         // 0x08
    Instruction(0, "ADD HL, BC"),            // 0x09
    Instruction(0, "LD A, (BC)"),            // 0x0a
    Instruction(0, "DEC BC"),                // 0x0b
    Instruction(0, "INC C"),                 // 0x0c
    Instruction(0, "DEC C"),                 // 0x0d
    Instruction(1, "LD C, 0x??"),            // 0x0e
    Instruction(0, "RRCA"),                  // 0x0f
    Instruction(1, "STOP"),                  // 0x10
    Instruction(2, "LD DE, 0x??"),           // 0x11
    Instruction(0, "LD (DE), A"),            // 0x12
    Instruction(0, "INC DE"),                // 0x13
    Instruction(0, "INC D"),                 // 0x14
    Instruction(0, "DEC D"),                 // 0x15
    Instruction(1, "LD D, 0x??"),            // 0x16
    Instruction(0, "RLA"),                   // 0x17
    Instruction(1, "JR 0x??"),               // 0x18
    Instruction(0, "ADD HL, DE"),            // 0x19
    Instruction(0, "LD A, (DE)"),            // 0x1a
    Instruction(0, "DEC DE"),                // 0x1b
    Instruction(0, "INC E"),                 // 0x1c
    Instruction(0, "DEC E"),                 // 0x1d
    Instruction(1, "LD E, 0x??"),            // 0x1e
    Instruction(0, "RRA"),                   // 0x1f
    Instruction(1, "JR NZ, 0x??"),           // 0x20
    Instruction(2, "LD HL, 0x??"),           // 0x21
    Instruction(0, "LDI ), A"),              // 0x22
    Instruction(0, "INC HL"),                // 0x23
    Instruction(0, "INC H"),                 // 0x24
    Instruction(0, "DEC H"),                 // 0x25
    Instruction(1, "LD H, 0x??"),            // 0x26
    Instruction(0, "DAA"),                   // 0x27
    Instruction(1, "JR Z, 0x??"),            // 0x28
    Instruction(0, "ADD HL, HL"),            // 0x29
    Instruction(0, "LDI A, )"),              // 0x2a
    Instruction(0, "DEC HL"),                // 0x2b
    Instruction(0, "INC L"),                 // 0x2c
    Instruction(0, "DEC L"),                 // 0x2d
    Instruction(1, "LD L, 0x??"),            // 0x2e
    Instruction(0, "CPL"),                   // 0x2f
    Instruction(1, "JR NC, 0x??"),           // 0x30
    Instruction(2, "LD SP, 0x??"),           // 0x31
    Instruction(0, "LDD (HL), A"),           // 0x32
    Instruction(0, "INC SP"),                // 0x33
    Instruction(0, "INC (HL)"),              // 0x34
    Instruction(0, "DEC (HL)"),              // 0x35
    Instruction(1, "LD (HL), 0x??"),         // 0x36
    Instruction(0, "SCF"),                   // 0x37
    Instruction(1, "JR C, 0x??"),            // 0x38
    Instruction(0, "ADD HL, SP"),            // 0x39
    Instruction(0, "LDD A, )"),              // 0x3a
    Instruction(0, "DEC SP"),                // 0x3b
    Instruction(0, "INC A"),                 // 0x3c
    Instruction(0, "DEC A"),                 // 0x3d
    Instruction(1, "LD A, 0x??"),            // 0x3e
    Instruction(0, "CCF"),                   // 0x3f
    Instruction(0, "LD B, B"),               // 0x40
    Instruction(0, "LD B, C"),               // 0x41
    Instruction(0, "LD B, D"),               // 0x42
    Instruction(0, "LD B, E"),               // 0x43
    Instruction(0, "LD B, H"),               // 0x44
    Instruction(0, "LD B, L"),               // 0x45
    Instruction(0, "LD B, )"),               // 0x46
    Instruction(0, "LD B, A"),               // 0x47
    Instruction(0, "LD C, B"),               // 0x48
    Instruction(0, "LD C, C"),               // 0x49
    Instruction(0, "LD C, D"),               // 0x4a
    Instruction(0, "LD C, E"),               // 0x4b
    Instruction(0, "LD C, H"),               // 0x4c
    Instruction(0, "LD C, L"),               // 0x4d
    Instruction(0, "LD C, )"),               // 0x4e
    Instruction(0, "LD C, A"),               // 0x4f
    Instruction(0, "LD D, B"),               // 0x50
    Instruction(0, "LD D, C"),               // 0x51
    Instruction(0, "LD D, D"),               // 0x52
    Instruction(0, "LD D, E"),               // 0x53
    Instruction(0, "LD D, H"),               // 0x54
    Instruction(0, "LD D, L"),               // 0x55
    Instruction(0, "LD D, )"),               // 0x56
    Instruction(0, "LD D, A"),               // 0x57
    Instruction(0, "LD E, B"),               // 0x58
    Instruction(0, "LD E, C"),               // 0x59
    Instruction(0, "LD E, D"),               // 0x5a
    Instruction(0, "LD E, E"),               // 0x5b
    Instruction(0, "LD E, H"),               // 0x5c
    Instruction(0, "LD E, L"),               // 0x5d
    Instruction(0, "LD E, )"),               // 0x5e
    Instruction(0, "LD E, A"),               // 0x5f
    Instruction(0, "LD H, B"),               // 0x60
    Instruction(0, "LD H, C"),               // 0x61
    Instruction(0, "LD H, D"),               // 0x62
    Instruction(0, "LD H, E"),               // 0x63
    Instruction(0, "LD H, H"),               // 0x64
    Instruction(0, "LD H, L"),               // 0x65
    Instruction(0, "LD H, )"),               // 0x66
    Instruction(0, "LD H, A"),               // 0x67
    Instruction(0, "LD L, B"),               // 0x68
    Instruction(0, "LD L, C"),               // 0x69
    Instruction(0, "LD L, D"),               // 0x6a
    Instruction(0, "LD L, E"),               // 0x6b
    Instruction(0, "LD L, H"),               // 0x6c
    Instruction(0, "LD L, L"),               // 0x6d
    Instruction(0, "LD L, )"),               // 0x6e
    Instruction(0, "LD L, A"),               // 0x6f
    Instruction(0, "LD ), B"),               // 0x70
    Instruction(0, "LD ), C"),               // 0x71
    Instruction(0, "LD ), D"),               // 0x72
    Instruction(0, "LD ), E"),               // 0x73
    Instruction(0, "LD ), H"),               // 0x74
    Instruction(0, "LD ), L"),               // 0x75
    Instruction(0, "HALT"),                  // 0x76
    Instruction(0, "LD ), A"),               // 0x77
    Instruction(0, "LD A, B"),               // 0x78
    Instruction(0, "LD A, C"),               // 0x79
    Instruction(0, "LD A, D"),               // 0x7a
    Instruction(0, "LD A, E"),               // 0x7b
    Instruction(0, "LD A, H"),               // 0x7c
    Instruction(0, "LD A, L"),               // 0x7d
    Instruction(0, "LD A, )"),               // 0x7e
    Instruction(0, "LD A, A"),               // 0x7f
    Instruction(0, "ADD A, B"),              // 0x80
    Instruction(0, "ADD A, C"),              // 0x81
    Instruction(0, "ADD A, D"),              // 0x82
    Instruction(0, "ADD A, E"),              // 0x83
    Instruction(0, "ADD A, H"),              // 0x84
    Instruction(0, "ADD A, L"),              // 0x85
    Instruction(0, "ADD A, )"),              // 0x86
    Instruction(0, "ADD A"),                 // 0x87
    Instruction(0, "ADC B"),                 // 0x88
    Instruction(0, "ADC C"),                 // 0x89
    Instruction(0, "ADC D"),                 // 0x8a
    Instruction(0, "ADC E"),                 // 0x8b
    Instruction(0, "ADC H"),                 // 0x8c
    Instruction(0, "ADC L"),                 // 0x8d
    Instruction(0, "ADC )"),                 // 0x8e
    Instruction(0, "ADC A"),                 // 0x8f
    Instruction(0, "SUB B"),                 // 0x90
    Instruction(0, "SUB C"),                 // 0x91
    Instruction(0, "SUB D"),                 // 0x92
    Instruction(0, "SUB E"),                 // 0x93
    Instruction(0, "SUB H"),                 // 0x94
    Instruction(0, "SUB L"),                 // 0x95
    Instruction(0, "SUB )"),                 // 0x96
    Instruction(0, "SUB A"),                 // 0x97
    Instruction(0, "SBC B"),                 // 0x98
    Instruction(0, "SBC C"),                 // 0x99
    Instruction(0, "SBC D"),                 // 0x9a
    Instruction(0, "SBC E"),                 // 0x9b
    Instruction(0, "SBC H"),                 // 0x9c
    Instruction(0, "SBC L"),                 // 0x9d
    Instruction(0, "SBC )"),                 // 0x9e
    Instruction(0, "SBC A"),                 // 0x9f
    Instruction(0, "AND B"),                 // 0xa0
    Instruction(0, "AND C"),                 // 0xa1
    Instruction(0, "AND D"),                 // 0xa2
    Instruction(0, "AND E"),                 // 0xa3
    Instruction(0, "AND H"),                 // 0xa4
    Instruction(0, "AND L"),                 // 0xa5
    Instruction(0, "AND )"),                 // 0xa6
    Instruction(0, "AND A"),                 // 0xa7
    Instruction(0, "XOR B"),                 // 0xa8
    Instruction(0, "XOR C"),                 // 0xa9
    Instruction(0, "XOR D"),                 // 0xaa
    Instruction(0, "XOR E"),                 // 0xab
    Instruction(0, "XOR H"),                 // 0xac
    Instruction(0, "XOR L"),                 // 0xad
    Instruction(0, "XOR )"),                 // 0xae
    Instruction(0, "XOR A"),                 // 0xaf
    Instruction(0, "OR B"),                  // 0xb0
    Instruction(0, "OR C"),                  // 0xb1
    Instruction(0, "OR D"),                  // 0xb2
    Instruction(0, "OR E"),                  // 0xb3
    Instruction(0, "OR H"),                  // 0xb4
    Instruction(0, "OR L"),                  // 0xb5
    Instruction(0, "OR )"),                  // 0xb6
    Instruction(0, "OR A"),                  // 0xb7
    Instruction(0, "CP B"),                  // 0xb8
    Instruction(0, "CP C"),                  // 0xb9
    Instruction(0, "CP D"),                  // 0xba
    Instruction(0, "CP E"),                  // 0xbb
    Instruction(0, "CP H"),                  // 0xbc
    Instruction(0, "CP L"),                  // 0xbd
    Instruction(0, "CP )"),                  // 0xbe
    Instruction(0, "CP A"),                  // 0xbf
    Instruction(0, "RET NZ"),                // 0xc0
    Instruction(0, "POP BC"),                // 0xc1
    Instruction(2, "JP NZ, 0x??"),           // 0xc2
    Instruction(2, "JP 0x??"),               // 0xc3
    Instruction(2, "CALL NZ, 0x??"),         // 0xc4
    Instruction(0, "PUSH BC"),               // 0xc5
    Instruction(1, "ADD A, 0x??"),           // 0xc6
    Instruction(0, "RST 0x00"),              // 0xc7
    Instruction(0, "RET Z"),                 // 0xc8
    Instruction(0, "RET"),                   // 0xc9
    Instruction(2, "JP Z, 0x??"),            // 0xca
    Instruction(1, "CB ??"),                 // 0xcb
    Instruction(2, "CALL Z, 0x??"),          // 0xcc
    Instruction(2, "CALL 0x??"),             // 0xcd
    Instruction(1, "ADC 0x??"),              // 0xce
    Instruction(0, "RST 0x08"),              // 0xcf
    Instruction(0, "RET NC"),                // 0xd0
    Instruction(0, "POP DE"),                // 0xd1
    Instruction(2, "JP NC, 0x??"),           // 0xd2
    Instruction(0, "UNKNOWN"),               // 0xd3
    Instruction(2, "CALL NC, 0x??"),         // 0xd4
    Instruction(0, "PUSH DE"),               // 0xd5
    Instruction(1, "SUB 0x??"),              // 0xd6
    Instruction(0, "RST 0x10"),              // 0xd7
    Instruction(0, "RET C"),                 // 0xd8
    Instruction(0, "RETI"),                  // 0xd9
    Instruction(2, "JP C, 0x??"),            // 0xda
    Instruction(0, "UNKNOWN"),               // 0xdb
    Instruction(2, "CALL C, 0x??"),          // 0xdc
    Instruction(0, "UNKNOWN"),               // 0xdd
    Instruction(1, "SBC 0x??"),              // 0xde
    Instruction(0, "RST 0x18"),              // 0xdf
    Instruction(1, "LD (0xFF00 + 0x??), A"), // 0xe0
    Instruction(0, "POP HL"),                // 0xe1
    Instruction(0, "LD (0xFF00 ), A"),       // 0xe2
    Instruction(0, "UNKNOWN"),               // 0xe3
    Instruction(0, "UNKNOWN"),               // 0xe4
    Instruction(0, "PUSH HL"),               // 0xe5
    Instruction(1, "AND 0x??"),              // 0xe6
    Instruction(0, "RST 0x20"),              // 0xe7
    Instruction(1, "ADD SP,0x??"),           // 0xe8
    Instruction(0, "JP HL"),                 // 0xe9
    Instruction(0, "LD (0x??), A"),          // 0xea
    Instruction(0, "UNKNOWN"),               // 0xeb
    Instruction(0, "UNKNOWN"),               // 0xec
    Instruction(0, "UNKNOWN"),               // 0xed
    Instruction(1, "XOR 0x??"),              // 0xee
    Instruction(0, "RST 0x28"),              // 0xef
    Instruction(1, "LD A, (0xFF00 + 0x??)"), // 0xf0
    Instruction(0, "POP AF"),                // 0xf1
    Instruction(0, "LD A, (0xFF00 )"),       // 0xf2
    Instruction(0, "DI"),                    // 0xf3
    Instruction(0, "UNKNOWN"),               // 0xf4
    Instruction(0, "PUSH AF"),               // 0xf5
    Instruction(1, "OR 0x??"),               // 0xf6
    Instruction(0, "RST 0x30"),              // 0xf7
    Instruction(1, "LD HL, SP+0x??"),        // 0xf8
    Instruction(0, "LD SP, HL"),             // 0xf9
    Instruction(2, "LD A, (0x??)"),          // 0xfa
    Instruction(0, "EI"),                    // 0xfb
    Instruction(0, "UNKNOWN"),               // 0xfc
    Instruction(0, "UNKNOWN"),               // 0xfd
    Instruction(1, "CP 0x??"),               // 0xfe
    Instruction(0, "RST 0x38"),              // 0xff
];
