mod alu;
mod cb;
mod jp;
mod ld;
mod misc;

use self::{Flag::*, Instr::*, Register::*, location::Address::*};
use crate::{bus::Bus, cpu::CPU, instructions::location::Address};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Flag {
    FlagNZ,
    FlagZ,
    FlagC,
    FlagNC,
}

pub mod location {

    use tap::Pipe;

    use crate::{
        bus::Bus,
        cpu::{CPU, value::Writable},
        instructions::Register,
    };

    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub enum Address {
        Memory(Register),
        Register(Register),
        ImmediateByte, // Bytes
        ImmediateWord, // Words
        MemOffsetImm,
        MemoryImmediate,
        MemOffsetC,
    }

    impl Address {
        pub fn read(self, cpu: &mut CPU, bus: &mut Bus) -> Read {
            use Register::*;
            match self {
                Self::ImmediateByte => Read::Byte(cpu.next_u8(bus)),
                Self::ImmediateWord => Read::Word(cpu.next_u16(bus)),
                Self::MemoryImmediate => Read::Byte(cpu.next_u16(bus).pipe(|x| bus.read_cycle(x))),
                Self::MemOffsetImm => Read::Byte(cpu.next_u8(bus).pipe(|x| bus.read_cycle_high(x))),
                Self::MemOffsetC => Read::Byte(cpu.registers.c.pipe(|x| bus.read_cycle_high(x))),
                Self::Memory(reg) => {
                    Read::Byte(cpu.registers.fetch_u16(reg).pipe(|x| bus.read_cycle(x)))
                }
                Self::Register(reg @ (A | B | C | D | E | H | L | F)) => {
                    Read::Byte(cpu.registers.fetch_u8(reg))
                }
                Self::Register(reg @ (AF | BC | DE | HL | SP | PC)) => {
                    Read::Word(cpu.registers.fetch_u16(reg))
                }
            }
        }
        pub fn write<T>(self, cpu: &mut CPU, bus: &mut Bus, write_value: T)
        where
            T: Writable,
        {
            match self {
                Self::ImmediateWord | Self::MemoryImmediate => {
                    let address = cpu.next_u16(bus);
                    write_value.to_memory_address(address, bus);
                }
                Self::Memory(r) => {
                    let address = cpu
                        .registers
                        .get_dual_reg(r)
                        .expect("I tried to access a u8 as a bus address.");
                    write_value.to_memory_address(address, bus);
                }
                Self::MemOffsetImm => {
                    let next = cpu.next_u8(bus);
                    write_value.to_memory_address(0xFF00 + u16::from(next), bus);
                }
                Self::MemOffsetC => {
                    write_value.to_memory_address(0xFF00 + u16::from(cpu.registers.c), bus);
                }
                Self::Register(r) => write_value.to_register(&mut cpu.registers, r),
                Self::ImmediateByte => unimplemented!("{:?}", self),
            }
        }
    }

    /// Result of a read operation.
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    pub enum Read {
        Byte(u8),
        Word(u16),
    }

    impl From<Read> for u8 {
        fn from(val: Read) -> Self {
            #[allow(clippy::cast_possible_truncation)]
            match val {
                Read::Byte(x) => x,
                Read::Word(x) => x as Self,
            }
        }
    }

    impl From<Read> for u16 {
        fn from(val: Read) -> Self {
            match val {
                Read::Byte(x) => Self::from(x),
                Read::Word(x) => x,
            }
        }
    }

    impl From<u8> for Read {
        fn from(val: u8) -> Self {
            Self::Byte(val)
        }
    }
    impl From<u16> for Read {
        fn from(val: u16) -> Self {
            Self::Word(val)
        }
    }

    impl Address {
        pub const fn is_word_register(self) -> bool {
            match self {
                Register(r) => r.is_word_register(),
                _ => false,
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

pub trait Executable {
    fn execute(self, cpu: &mut CPU, bus: &mut Bus);
}

impl Register {
    pub const fn is_word_register(self) -> bool {
        matches!(self, HL | BC | DE | SP)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Instr {
    #[default]
    NOOP,
    UNIMPLEMENTED,
    LD(Address, Address), // (To, From)
    LDD(Address, Address),
    LDI(Address, Address),
    LDSP,
    INC(Address),
    DEC(Address),
    ADD(Address),
    ADDHL(Address),
    ADC(Address),
    SUB(Address),
    AND(Address),
    XOR(Address),
    OR(Address),
    CP(Address),
    SBC(Address),
    CB,
    JR(Option<Flag>),
    STOP,
    DisableInterrupts,
    EnableInterrupts,
    JP(Option<Flag>),
    JpHl,
    RET(Option<Flag>),
    RETI,
    DAA,
    POP(Register),
    PUSH(Register),
    NOT(Address),
    CALL(Option<Flag>),
    RLCA,
    RRCA,
    RLA,
    RRA,
    SCF,
    CCF,
    ADDSP,
    HALT,
    RST(u8),
}

impl From<u8> for Instr {
    fn from(op: u8) -> Self {
        INSTR_TABLE[op as usize]
    }
}

impl Instr {
    pub fn run(self, cpu: &mut CPU, bus: &mut Bus) {
        match self {
            NOOP | STOP => {} // empty / TODO
            LD(from, to) => ld::ld((from, to), cpu, bus),
            LDI(from, to) => ld::ldi((from, to), cpu, bus),
            LDD(from, to) => ld::ldd((from, to), cpu, bus),
            LDSP => ld::ldsp(cpu, bus),
            INC(location) => alu::inc(location, cpu, bus),
            DEC(location) => alu::dec(location, cpu, bus),
            SUB(location) => alu::sub(location, cpu, bus),
            ADD(location) => alu::add(location, cpu, bus),
            CP(location) => alu::cp(location, cpu, bus),
            ADDHL(location) => alu::addhl(location, cpu, bus),
            ADC(location) => alu::adc(location, cpu, bus),
            AND(location) => alu::and(location, cpu, bus),
            XOR(location) => alu::xor(location, cpu, bus),
            OR(location) => alu::orr(location, cpu, bus),
            SBC(location) => alu::sbc(location, cpu, bus),
            NOT(location) => alu::not(location, cpu, bus),
            RLCA => alu::rlca(cpu, bus),
            RRCA => alu::rrca(cpu, bus),
            RLA => alu::rla(cpu, bus),
            RRA => alu::rra(cpu, bus),
            CCF => alu::ccf(cpu, bus),
            ADDSP => alu::addsp(cpu, bus),
            SCF => alu::scf(cpu, bus),
            RST(addr) => jp::rst(u16::from(addr), cpu, bus),
            JP(flag) => jp::jp(flag, cpu, bus),
            JR(flag) => jp::jr(flag, cpu, bus),
            JpHl => jp::jp_hl(cpu, bus),
            RET(flag) => jp::ret(flag, cpu, bus),
            RETI => jp::reti(cpu, bus),
            CALL(flag) => jp::call(flag, cpu, bus),
            CB => cb::cb(cpu, bus),
            DisableInterrupts => bus.disable_interrupts(),
            EnableInterrupts => bus.enable_interrupts(),
            DAA => misc::daa(cpu, bus),
            POP(r) => misc::pop(r, cpu, bus),
            PUSH(r) => misc::push(r, cpu, bus),
            HALT => misc::halt(cpu, bus),
            UNIMPLEMENTED => unimplemented!(),
        }
    }
}
pub const INSTR_TABLE: [Instr; 256] = [
    NOOP,                             //0x00
    LD(Register(BC), ImmediateWord),  //0x01
    LD(Memory(BC), Register(A)),      //0x02
    INC(Register(BC)),                //0x03
    INC(Register(B)),                 //0x04
    DEC(Register(B)),                 //0x05
    LD(Register(B), ImmediateByte),   //0x06
    RLCA,                             //0x07
    LD(ImmediateWord, Register(SP)),  //0x08
    ADDHL(Register(BC)),              //0x09
    LD(Register(A), Memory(BC)),      //0x0A
    DEC(Register(BC)),                //0x0B
    INC(Register(C)),                 //0x0C
    DEC(Register(C)),                 //0x0D
    LD(Register(C), ImmediateByte),   //0x0E
    RRCA,                             //0x0F
    STOP,                             //0x10
    LD(Register(DE), ImmediateWord),  //0x11
    LD(Memory(DE), Register(A)),      //0x12
    INC(Register(DE)),                //0x13
    INC(Register(D)),                 //0x14
    DEC(Register(D)),                 //0x15
    LD(Register(D), ImmediateByte),   //0x16
    RLA,                              //0x17
    JR(None),                         //0x18
    ADDHL(Register(DE)),              //0x19
    LD(Register(A), Memory(DE)),      //0x1A
    DEC(Register(DE)),                //0x1B
    INC(Register(E)),                 //0x1C
    DEC(Register(E)),                 //0x1D
    LD(Register(E), ImmediateByte),   //0x1E
    RRA,                              //0x1F
    JR(Some(FlagNZ)),                 //0x20
    LD(Register(HL), ImmediateWord),  //0x21
    LDI(Memory(HL), Register(A)),     //0x22
    INC(Register(HL)),                //0x23
    INC(Register(H)),                 //0x24
    DEC(Register(H)),                 //0x25
    LD(Register(H), ImmediateByte),   //0x26
    DAA,                              //0x27
    JR(Some(FlagZ)),                  //0x28
    ADDHL(Register(HL)),              //0x29
    LDI(Register(A), Memory(HL)),     //0x2A
    DEC(Register(HL)),                //0x2B
    INC(Register(L)),                 //0x2C
    DEC(Register(L)),                 //0x2D
    LD(Register(L), ImmediateByte),   //0x2E
    NOT(Register(A)),                 //0x2F
    JR(Some(FlagNC)),                 //0x30
    LD(Register(SP), ImmediateWord),  //0x31
    LDD(Memory(HL), Register(A)),     //0x32
    INC(Register(SP)),                //0x33
    INC(Memory(HL)),                  //0x34
    DEC(Memory(HL)),                  //0x35
    LD(Memory(HL), ImmediateByte),    //0x36
    SCF,                              //0x37
    JR(Some(FlagC)),                  //0x38
    ADDHL(Register(SP)),              //0x39
    LDD(Register(A), Memory(HL)),     //0x3A
    DEC(Register(SP)),                //0x3B
    INC(Register(A)),                 //0x3C
    DEC(Register(A)),                 //0x3D
    LD(Register(A), ImmediateByte),   //0x3E
    CCF,                              //0x3F
    LD(Register(B), Register(B)),     //0x40
    LD(Register(B), Register(C)),     //0x41
    LD(Register(B), Register(D)),     //0x42
    LD(Register(B), Register(E)),     //0x43
    LD(Register(B), Register(H)),     //0x44
    LD(Register(B), Register(L)),     //0x45
    LD(Register(B), Memory(HL)),      //0x46
    LD(Register(B), Register(A)),     //0x47
    LD(Register(C), Register(B)),     //0x48
    LD(Register(C), Register(C)),     //0x49
    LD(Register(C), Register(D)),     //0x4A
    LD(Register(C), Register(E)),     //0x4B
    LD(Register(C), Register(H)),     //0x4C
    LD(Register(C), Register(L)),     //0x4D
    LD(Register(C), Memory(HL)),      //0x4E
    LD(Register(C), Register(A)),     //0x4F
    LD(Register(D), Register(B)),     //0x50
    LD(Register(D), Register(C)),     //0x51
    LD(Register(D), Register(D)),     //0x52
    LD(Register(D), Register(E)),     //0x53
    LD(Register(D), Register(H)),     //0x54
    LD(Register(D), Register(L)),     //0x55
    LD(Register(D), Memory(HL)),      //0x56
    LD(Register(D), Register(A)),     //0x57
    LD(Register(E), Register(B)),     //0x58
    LD(Register(E), Register(C)),     //0x59
    LD(Register(E), Register(D)),     //0x5A
    LD(Register(E), Register(E)),     //0x5B
    LD(Register(E), Register(H)),     //0x5C
    LD(Register(E), Register(L)),     //0x5D
    LD(Register(E), Memory(HL)),      //0x5E
    LD(Register(E), Register(A)),     //0x5F
    LD(Register(H), Register(B)),     //0x60
    LD(Register(H), Register(C)),     //0x61
    LD(Register(H), Register(D)),     //0x62
    LD(Register(H), Register(E)),     //0x63
    LD(Register(H), Register(H)),     //0x64
    LD(Register(H), Register(L)),     //0x65
    LD(Register(H), Memory(HL)),      //0x66
    LD(Register(H), Register(A)),     //0x67
    LD(Register(L), Register(B)),     //0x68
    LD(Register(L), Register(C)),     //0x69
    LD(Register(L), Register(D)),     //0x6A
    LD(Register(L), Register(E)),     //0x6B
    LD(Register(L), Register(H)),     //0x6C
    LD(Register(L), Register(L)),     //0x6D
    LD(Register(L), Memory(HL)),      //0x6E
    LD(Register(L), Register(A)),     //0x6F
    LD(Memory(HL), Register(B)),      //0x70
    LD(Memory(HL), Register(C)),      //0x71
    LD(Memory(HL), Register(D)),      //0x72
    LD(Memory(HL), Register(E)),      //0x73
    LD(Memory(HL), Register(H)),      //0x74
    LD(Memory(HL), Register(L)),      //0x75
    HALT,                             //0x76
    LD(Memory(HL), Register(A)),      //0x77
    LD(Register(A), Register(B)),     //0x78
    LD(Register(A), Register(C)),     //0x79
    LD(Register(A), Register(D)),     //0x7A
    LD(Register(A), Register(E)),     //0x7B
    LD(Register(A), Register(H)),     //0x7C
    LD(Register(A), Register(L)),     //0x7D
    LD(Register(A), Memory(HL)),      //0x7E
    LD(Register(A), Register(A)),     //0x7F
    ADD(Register(B)),                 //0x80
    ADD(Register(C)),                 //0x81
    ADD(Register(D)),                 //0x82
    ADD(Register(E)),                 //0x83
    ADD(Register(H)),                 //0x84
    ADD(Register(L)),                 //0x85
    ADD(Memory(HL)),                  //0x86
    ADD(Register(A)),                 //0x87
    ADC(Register(B)),                 //0x88
    ADC(Register(C)),                 //0x89
    ADC(Register(D)),                 //0x8A
    ADC(Register(E)),                 //0x8B
    ADC(Register(H)),                 //0x8C
    ADC(Register(L)),                 //0x8D
    ADC(Memory(HL)),                  //0x8E
    ADC(Register(A)),                 //0x8F
    SUB(Register(B)),                 //0x90
    SUB(Register(C)),                 //0x91
    SUB(Register(D)),                 //0x92
    SUB(Register(E)),                 //0x93
    SUB(Register(H)),                 //0x94
    SUB(Register(L)),                 //0x95
    SUB(Memory(HL)),                  //0x96
    SUB(Register(A)),                 //0x97
    SBC(Register(B)),                 //0x98
    SBC(Register(C)),                 //0x99
    SBC(Register(D)),                 //0x92
    SBC(Register(E)),                 //0x93
    SBC(Register(H)),                 //0x94
    SBC(Register(L)),                 //0x9D
    SBC(Memory(HL)),                  //0x9E
    SBC(Register(A)),                 //0x9F
    AND(Register(B)),                 //0xA0
    AND(Register(C)),                 //0xA1
    AND(Register(D)),                 //0xA2
    AND(Register(E)),                 //0xA3
    AND(Register(H)),                 //0xA4
    AND(Register(L)),                 //0xA5
    AND(Memory(HL)),                  //0xA6
    AND(Register(A)),                 //0xA7
    XOR(Register(B)),                 //0xA8
    XOR(Register(C)),                 //0xA9
    XOR(Register(D)),                 //0xAA
    XOR(Register(E)),                 //0xAB
    XOR(Register(H)),                 //0xAC
    XOR(Register(L)),                 //0xAD
    XOR(Memory(HL)),                  //0xAE
    XOR(Register(A)),                 //0xAF
    OR(Register(B)),                  //0xB0
    OR(Register(C)),                  //0xB1
    OR(Register(D)),                  //0xB2
    OR(Register(E)),                  //0xB3
    OR(Register(H)),                  //0xB4
    OR(Register(L)),                  //0xB5
    OR(Memory(HL)),                   //0xB6
    OR(Register(A)),                  //0xB7
    CP(Register(B)),                  //0xB8
    CP(Register(C)),                  //0xB9
    CP(Register(D)),                  //0xBA
    CP(Register(E)),                  //0xBB
    CP(Register(H)),                  //0xBC
    CP(Register(L)),                  //0xBD
    CP(Memory(HL)),                   //0xBE
    CP(Register(A)),                  //0xBF
    RET(Some(FlagNZ)),                //0xC0
    POP(BC),                          //0xC1
    JP(Some(FlagNZ)),                 //0xC2
    JP(None),                         //0xC3
    CALL(Some(FlagNZ)),               //0xC4
    PUSH(BC),                         //0xC5
    ADD(ImmediateByte),               //0xC6
    RST(0x0),                         //0xC7
    RET(Some(FlagZ)),                 //0xC8
    RET(None),                        //0xC9
    JP(Some(FlagZ)),                  //0xCA
    CB,                               //0xCB
    CALL(Some(FlagZ)),                //0xCC
    CALL(None),                       //0xCD
    ADC(ImmediateByte),               //0xCE
    RST(0x8),                         //0xCF
    RET(Some(FlagNC)),                //0xD0
    POP(DE),                          //0xD1
    JP(Some(FlagNC)),                 //0xD2
    UNIMPLEMENTED,                    //0xD3
    CALL(Some(FlagNC)),               //0xD4
    PUSH(DE),                         //0xD5
    SUB(ImmediateByte),               //0xD6
    RST(0x10),                        //0xD7
    RET(Some(FlagC)),                 //0xD8
    RETI,                             //0xD9
    JP(Some(FlagC)),                  //0xDA
    UNIMPLEMENTED,                    //0xDB
    CALL(Some(FlagC)),                //0xDC
    UNIMPLEMENTED,                    //0xDD
    SBC(ImmediateByte),               //0xDE
    RST(0x18),                        //0xDF
    LD(MemOffsetImm, Register(A)),    //0xE0
    POP(HL),                          //0xE1
    LD(MemOffsetC, Register(A)),      //0xE2
    UNIMPLEMENTED,                    //0xE3
    UNIMPLEMENTED,                    //0xE4
    PUSH(HL),                         //0xE5
    AND(ImmediateByte),               //0xE6
    RST(0x20),                        //0xE7
    ADDSP,                            //0xE8
    JpHl,                             //0xE9
    LD(MemoryImmediate, Register(A)), //0xEA
    UNIMPLEMENTED,                    //0xEB
    UNIMPLEMENTED,                    //0xEC
    UNIMPLEMENTED,                    //0xED
    XOR(ImmediateByte),               //0xEE
    RST(0x28),                        //0xEF
    LD(Register(A), MemOffsetImm),    //0xF0
    POP(AF),                          //0xF1
    LD(Register(A), MemOffsetC),      //0xF2
    DisableInterrupts,                //0xF3
    UNIMPLEMENTED,                    //0xF4
    PUSH(AF),                         //0xF5
    OR(ImmediateByte),                //0xF6
    RST(0x30),                        //0xF7
    LDSP,                             //0xF8
    LD(Register(SP), Register(HL)),   //0xF9
    LD(Register(A), MemoryImmediate), //0xFA
    EnableInterrupts,                 //0xFB
    UNIMPLEMENTED,                    //0xFC
    UNIMPLEMENTED,                    //0xFD
    CP(ImmediateByte),                //0xFE
    RST(0x38),                        //0xFF
];

pub const INSTR_DATA_LENGTHS: [usize; 256] = [
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
