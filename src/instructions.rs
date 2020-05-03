use std::fmt;
use self::Instr::*;
use self::Location::*;
use self::Register::*;
use self::Direction::*;
use self::JumpType::*;
use self::Flag::*;

#[derive(Debug)]
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
    MEM,

    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum Flag {
    FlagNZ,
    FlagZ,
    FlagC,
    FlagNC,
}

#[derive(Debug)]
pub enum Location {
    Memory(Register),
    Immediate(usize), // Bytes
    Register(Register),
    MemoryOffset
}

#[derive(Debug)]
pub enum Direction {
    LEFT, RIGHT
}

#[derive(Debug)]
pub enum JumpType {
    Imm,
    If(Flag),
    To(Location)
}

#[derive(Debug)]
pub enum Instr {
    NOOP,
    UNIMPLEMENTED,
    LD(Location, Location), // (From, To)
    INC(Location),
    DEC(Location),
    ADD(Location),
    ADD2(Location, Location),
    ADC(Location),
    SUB(Location),
    AND(Location),
    XOR(Location),
    OR(Location),
    CP(Location),
    SDC(Location),
    JR,
    JP(JumpType),
    RET(JumpType),
    POP(Location),
    PUSH(Location),
    NOT(Location),
    CALL(JumpType),
    RotThruCarry(Direction, Location),
}

pub const INSTR_TABLE: [Instr; 256] = [
    NOOP, //0x00
    LD(Immediate(2), Register(BC)),
    LD(Register(A), Memory(BC)),
    INC(Register(BC)),
    INC(Register(B)),
    DEC(Register(B)),
    LD(Immediate(1), Register(B)),
    RotThruCarry(LEFT, Register(A)),
    LD(Immediate(2), Register(SP)),
    UNIMPLEMENTED,
    LD(Register(A), Memory(BC)),
    DEC(Register(BC)),
    INC(Register(C)),
    DEC(Register(C)),
    LD(Immediate(1), Register(C)),
    UNIMPLEMENTED, 
    UNIMPLEMENTED, //0x10
    LD(Immediate(2), Register(DE)),
    LD(Register(A), Memory(DE)),
    INC(Register(DE)),
    INC(Register(D)),
    DEC(Register(D)),
    LD(Immediate(1), Register(D)),
    RotThruCarry(LEFT, Register(A)),
    NOOP,// TODO JMP ADD
    ADD2(Register(HL), Register(DE)),
    LD(Register(A), Memory(DE)),
    UNIMPLEMENTED,
    INC(Register(E)),
    DEC(Register(E)),
    LD(Immediate(1), Register(E)),
    RotThruCarry(RIGHT, Register(A)),
    JR, //0x20
    LD(Immediate(2), Register(HL)),
    NOOP, // LD A (HL), INC(HL) TODO: Composite OP?
    INC(Register(HL)),
    INC(Register(H)),
    DEC(Register(H)),
    LD(Immediate(1), Register(H)),
    UNIMPLEMENTED,
    JR,
    ADD2(Register(HL), Register(HL)),
    NOOP, // TODO LD!(self, a, self.read_byte(self.registers.hl()), 1);self.registers = self.inc_hl();
    UNIMPLEMENTED,
    INC(Register(L)),
    DEC(Register(L)),
    LD(Immediate(1), Register(L)),
    NOT(Register(A)),
    JR, // 0x30
    LD(Immediate(2), Register(SP)),
    NOOP, // LD A (HL), DEC(HL) TODO: Composite OP?
    INC(Register(SP)),
    INC(Register(HL)),
    DEC(Register(HL)),
    NOOP, // TODO
    UNIMPLEMENTED,
    JR,
    UNIMPLEMENTED, 
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    INC(Register(A)),
    DEC(Register(A)),
    LD(Immediate(1), Register(A)),
    UNIMPLEMENTED,
    LD(Register(B), Register(B)), //0x40 -- LD B's
    LD(Register(B), Register(C)),
    LD(Register(B), Register(D)),
    LD(Register(B), Register(E)),
    LD(Register(B), Register(H)),
    LD(Register(B), Register(L)),
    LD(Register(B), Memory(HL)),
    LD(Register(B), Register(A)),
    LD(Register(C), Register(B)),
    LD(Register(C), Register(C)),
    LD(Register(C), Register(D)),
    LD(Register(C), Register(E)),
    LD(Register(C), Register(H)),
    LD(Register(C), Register(L)),
    LD(Register(C), Memory(HL)),
    LD(Register(C), Register(A)),
    LD(Register(D), Register(B)), //0x50
    LD(Register(D), Register(C)),
    LD(Register(D), Register(D)),
    LD(Register(D), Register(E)),
    LD(Register(D), Register(H)),
    LD(Register(D), Register(L)),
    LD(Register(D), Memory(HL)),
    LD(Register(D), Register(A)),
    LD(Register(E), Register(B)),
    LD(Register(E), Register(C)),
    LD(Register(E), Register(D)),
    LD(Register(E), Register(E)),
    LD(Register(E), Register(H)),
    LD(Register(E), Register(L)),
    LD(Register(E), Memory(HL)),
    LD(Register(E), Register(A)),
    LD(Register(H), Register(B)), //0x60
    LD(Register(H), Register(C)),
    LD(Register(H), Register(D)),
    LD(Register(H), Register(E)),
    LD(Register(H), Register(H)),
    LD(Register(H), Register(L)),
    LD(Register(H), Memory(HL)),
    LD(Register(H), Register(A)),
    LD(Register(L), Register(B)),
    LD(Register(L), Register(C)),
    LD(Register(L), Register(D)),
    LD(Register(L), Register(E)),
    LD(Register(L), Register(H)),
    LD(Register(L), Register(L)),
    LD(Register(L), Memory(HL)),
    LD(Register(L), Register(A)),
    LD( Register(B), Memory(HL)), // 0x70
    LD( Register(C), Memory(HL)),
    LD( Register(D), Memory(HL)),
    LD( Register(E), Memory(HL)),
    LD( Register(H), Memory(HL)),
    LD( Register(L), Memory(HL)),
    UNIMPLEMENTED, //0x76
    LD(Register(A), Memory(HL)),
    LD(Register(A), Register(B)),
    LD(Register(A), Register(C)),
    LD(Register(A), Register(D)),
    LD(Register(A), Register(E)),
    LD(Register(A), Register(H)),
    LD(Register(A), Register(L)),
    LD(Register(A), Memory(HL)),
    LD(Register(A), Register(A)),
    ADD(Register(B)), // 0x80
    ADD(Register(C)),
    ADD(Register(D)),
    ADD(Register(E)),
    ADD(Register(H)),
    ADD(Register(L)),
    ADD(Register(HL)),
    ADD(Register(A)),
    ADC(Register(B)),
    ADC(Register(C)),
    ADC(Register(D)),
    ADC(Register(E)),
    ADC(Register(H)),
    ADC(Register(L)),
    ADC(Memory(HL)),
    ADC(Register(A)),
    SUB(Register(B)), //0x90
    SUB(Register(C)),
    SUB(Register(D)),
    SUB(Register(E)),
    SUB(Register(H)),
    SUB(Register(L)),
    SUB(Memory(HL)),
    SUB(Register(A)),
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    AND(Register(B)), //0xA0
    AND(Register(C)),
    AND(Register(D)),
    AND(Register(E)),
    AND(Register(H)),
    AND(Register(L)),
    AND(Memory(HL)),
    AND(Register(A)),
    XOR(Register(B)),
    XOR(Register(C)),
    XOR(Register(D)),
    XOR(Register(E)),
    XOR(Register(H)),
    XOR(Register(L)),
    XOR(Memory(HL)),
    XOR(Register(A)),
    OR(Register(B)), //0xB0
    OR(Register(C)),
    OR(Register(D)),
    OR(Register(E)),
    OR(Register(H)),
    OR(Register(L)),
    OR(Memory(HL)),
    OR(Register(A)),
    CP(Register(B)),
    CP(Register(C)),
    CP(Register(D)),
    CP(Register(E)),
    CP(Register(H)),
    CP(Register(L)),
    CP(Memory(HL)),
    CP(Register(A)),
    RET(If(FlagNZ)), //0xC0
    POP(Register(BC)),
    JP(If(FlagNZ)),
    JP(Imm),
    CALL(If(FlagNZ)),
    PUSH(Register(BC)),
    ADD(Immediate(2)), // TODO guessing
    UNIMPLEMENTED,
    RET(If(FlagZ)),
    UNIMPLEMENTED, // TODO JP Pop
    UNIMPLEMENTED, 
    UNIMPLEMENTED,  // TODO CB INSTRUCTIONS ********
    CALL(If(FlagNZ)),
    CALL(Imm),
    ADC(Immediate(1)),
    UNIMPLEMENTED, 
            // 0xD0 => RET!(self, flg_c),
            // 0xD1 => POP!(self, d, e),
            // 0xD2 => unimplemented!(),
//             0xD3
            // 0xD4 => CALL!(self, flg_nz),
            // 0xD5 => PUSH!(self, de),
            // 0xD6 => SUB!(self, IMMEDIATE),
//             0xD7
            // 0xD8 => RET!(self, flg_nc),
            // 0xD9 => unimplemented!(),
            // 0xDA
            // 0xDB => unimplemented!(),
            // 0xDC
            // 0xDD => unimplemented!(),
            // 0xDE => SBC!(self, self.next_u8(), 2),
            // 0xDF
            // 0xF0
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED,  
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED,  //0xE0
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    UNIMPLEMENTED, 
    JP(To(Register(HL))),
    RET(If(FlagC)),
    POP(Register(DE)),
    UNIMPLEMENTED,
    CALL(If(FlagZ)),
    PUSH(Register(DE)),
    SUB(Immediate(2)),
    RET(If(FlagNC)),
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    SDC(Immediate(2)),
    LD(Register(A), MemoryOffset),
    POP(Register(HL)),
    UNIMPLEMENTED, // TODO 0xE2 //self.set_byte(0xFF00 + self.registers.c() as u16, self.registers.a());self.registers = self.inc_pc(1);
    UNIMPLEMENTED,
    PUSH(Register(HL)),
    AND(Immediate(2)),
    UNIMPLEMENTED,
    UNIMPLEMENTED, // TODO EA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
];
// (InstrLen, ASM String)
#[derive(Debug, Clone, Copy)]
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
