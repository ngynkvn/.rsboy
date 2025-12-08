use strum_macros::IntoStaticStr;
use tap::{Pipe, Tap};
use tracing::info;

use crate::{
    bus::Bus,
    cpu::{CPU, value::Writable},
    instructions::Register,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone, IntoStaticStr, Hash)]
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
            Self::Memory(reg) => Read::Byte(cpu.registers.fetch_u16(reg).pipe(|x| bus.read_cycle(x))),
            Self::Register(reg @ (A | B | C | D | E | H | L | F)) => Read::Byte(cpu.registers.fetch_u8(reg)),
            Self::Register(reg @ (AF | BC | DE | HL | SP | PC)) => Read::Word(cpu.registers.fetch_u16(reg)),
        }
        .tap(|x| info!("Read {self}: {:?} at clock {}", x, bus.mclock()))
    }
    pub fn write<T>(self, cpu: &mut CPU, bus: &mut Bus, write_value: T)
    where
        T: Writable,
    {
        info!("Writing to {self} at clock {}", bus.mclock());
        match self {
            Self::ImmediateWord | Self::MemoryImmediate => {
                let address = cpu.next_u16(bus);
                write_value.to_memory_address(address, bus);
            }
            Self::Memory(r) => {
                let address = cpu.registers.get_dual_reg(r).expect("I tried to access a u8 as a bus address.");
                write_value.to_memory_address(address, bus);
            }
            Self::MemOffsetImm => {
                let next = cpu.next_u8(bus);
                write_value.to_memory_address(0xFF00 + u16::from(next), bus);
            }
            Self::MemOffsetC => {
                write_value.to_memory_address(0xFF00 + u16::from(cpu.registers.c), bus);
            }
            Self::Register(r @ Register::SP) => {
                // bus.generic_cycle();
                write_value.to_register(&mut cpu.registers, r);
            }
            Self::Register(r) => write_value.to_register(&mut cpu.registers, r),
            Self::ImmediateByte => unimplemented!("{:?}", self),
        }
        info!("Wrote to {self} at clock {}", bus.mclock());
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
            Self::Register(r) => r.is_word_register(),
            _ => false,
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(reg) => write!(f, "Mem({reg})"),
            Self::Register(reg) => write!(f, "{reg}"),
            Self::ImmediateByte => write!(f, "ImmByte"),
            Self::ImmediateWord => write!(f, "ImmWord"),
            Self::MemOffsetImm => write!(f, "MemOffset"),
            Self::MemoryImmediate => write!(f, "MemImm"),
            Self::MemOffsetC => write!(f, "MemOffC"),
        }
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&str>::from(self))
    }
}
