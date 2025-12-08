//! Type-safe operand system for Game Boy CPU instructions.
//!
//! This module provides compile-time type safety for 8-bit vs 16-bit operations,
//! eliminating runtime type checks and preventing silent truncation bugs.

use crate::{bus::Bus, cpu::CPU};

/// 8-bit registers: A, B, C, D, E, H, L, F
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reg8 {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    H = 5,
    L = 6,
    F = 7,
}

/// 16-bit register pairs: BC, DE, HL, SP, AF
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
    AF,
}

/// 8-bit operand sources (for reading values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Src8 {
    /// Register direct: A, B, C, D, E, H, L
    Reg(Reg8),
    /// Memory indirect via 16-bit register: [BC], [DE], [HL]
    Indirect(Reg16),
    /// Immediate byte from instruction stream
    Imm,
    /// High memory via C register: [0xFF00 + C]
    HighMemC,
    /// High memory via immediate: [0xFF00 + n]
    HighMemImm,
    /// Memory at immediate 16-bit address: [nn]
    MemImm,
}

/// 8-bit operand destinations (for writing values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dst8 {
    /// Register direct: A, B, C, D, E, H, L
    Reg(Reg8),
    /// Memory indirect via 16-bit register: [BC], [DE], [HL]
    Indirect(Reg16),
    /// High memory via C register: [0xFF00 + C]
    HighMemC,
    /// High memory via immediate: [0xFF00 + n]
    HighMemImm,
    /// Memory at immediate 16-bit address: [nn]
    MemImm,
}

/// 16-bit operand sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Src16 {
    /// Register pair: BC, DE, HL, SP, AF
    Reg(Reg16),
    /// Immediate word from instruction stream
    Imm,
}

/// 16-bit operand destinations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dst16 {
    /// Register pair: BC, DE, HL, SP
    Reg(Reg16),
    /// Memory at immediate address (for SP store)
    MemImm,
}

// ============================================================================
// Reading operands
// ============================================================================

impl Src8 {
    /// Read an 8-bit value from this operand source
    pub fn read(self, cpu: &mut CPU, bus: &mut Bus) -> u8 {
        match self {
            Self::Reg(r) => cpu.registers.get_r8(r),
            Self::Indirect(r) => {
                let addr = cpu.registers.get_r16(r);
                bus.read_cycle(addr)
            }
            Self::Imm => cpu.next_u8(bus),
            Self::HighMemC => {
                let addr = 0xFF00 + u16::from(cpu.registers.get_r8(Reg8::C));
                bus.read_cycle(addr)
            }
            Self::HighMemImm => {
                let offset = cpu.next_u8(bus);
                bus.read_cycle(0xFF00 + u16::from(offset))
            }
            Self::MemImm => {
                let addr = cpu.next_u16(bus);
                bus.read_cycle(addr)
            }
        }
    }
}

impl Src16 {
    /// Read a 16-bit value from this operand source
    pub fn read(self, cpu: &mut CPU, bus: &mut Bus) -> u16 {
        match self {
            Self::Reg(r) => cpu.registers.get_r16(r),
            Self::Imm => cpu.next_u16(bus),
        }
    }
}

// ============================================================================
// Writing operands
// ============================================================================

impl Dst8 {
    /// Write an 8-bit value to this operand destination
    pub fn write(self, cpu: &mut CPU, bus: &mut Bus, value: u8) {
        match self {
            Self::Reg(r) => cpu.registers.set_r8(r, value),
            Self::Indirect(r) => {
                let addr = cpu.registers.get_r16(r);
                bus.write_cycle(addr, value);
            }
            Self::HighMemC => {
                let addr = 0xFF00 + u16::from(cpu.registers.get_r8(Reg8::C));
                bus.write_cycle(addr, value);
            }
            Self::HighMemImm => {
                let offset = cpu.next_u8(bus);
                bus.write_cycle(0xFF00 + u16::from(offset), value);
            }
            Self::MemImm => {
                let addr = cpu.next_u16(bus);
                bus.write_cycle(addr, value);
            }
        }
    }
}

impl Dst16 {
    /// Write a 16-bit value to this operand destination
    pub fn write(self, cpu: &mut CPU, bus: &mut Bus, value: u16) {
        match self {
            Self::Reg(r) => cpu.registers.set_r16(r, value),
            Self::MemImm => {
                let addr = cpu.next_u16(bus);
                let [lo, hi] = value.to_le_bytes();
                bus.write_cycle(addr, lo);
                bus.write_cycle(addr.wrapping_add(1), hi);
            }
        }
    }
}

// ============================================================================
// Read-modify-write operands (for INC, DEC, rotate, etc.)
// ============================================================================

/// Operand that can be both read and written (8-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RmwOperand8 {
    Reg(Reg8),
    Indirect(Reg16),
}

impl RmwOperand8 {
    pub fn read(self, cpu: &mut CPU, bus: &mut Bus) -> u8 {
        match self {
            Self::Reg(r) => cpu.registers.get_r8(r),
            Self::Indirect(r) => {
                let addr = cpu.registers.get_r16(r);
                bus.read_cycle(addr)
            }
        }
    }

    pub fn write(self, cpu: &mut CPU, bus: &mut Bus, value: u8) {
        match self {
            Self::Reg(r) => cpu.registers.set_r8(r, value),
            Self::Indirect(r) => {
                let addr = cpu.registers.get_r16(r);
                bus.write_cycle(addr, value);
            }
        }
    }

    /// Returns true if this is a memory operand (affects cycle timing)
    pub const fn is_memory(self) -> bool {
        matches!(self, Self::Indirect(_))
    }
}

// ============================================================================
// Instruction length calculation
// ============================================================================

impl Src8 {
    /// Number of immediate bytes this operand consumes
    pub const fn imm_bytes(self) -> usize {
        match self {
            Self::Imm | Self::HighMemImm => 1,
            Self::MemImm => 2,
            Self::Reg(_) | Self::Indirect(_) | Self::HighMemC => 0,
        }
    }
}

impl Dst8 {
    /// Number of immediate bytes this operand consumes
    pub const fn imm_bytes(self) -> usize {
        match self {
            Self::HighMemImm => 1,
            Self::MemImm => 2,
            Self::Reg(_) | Self::Indirect(_) | Self::HighMemC => 0,
        }
    }
}

impl Src16 {
    /// Number of immediate bytes this operand consumes
    pub const fn imm_bytes(self) -> usize {
        match self {
            Self::Imm => 2,
            Self::Reg(_) => 0,
        }
    }
}

impl Dst16 {
    /// Number of immediate bytes this operand consumes
    pub const fn imm_bytes(self) -> usize {
        match self {
            Self::MemImm => 2,
            Self::Reg(_) => 0,
        }
    }
}

// ============================================================================
// Display implementations
// ============================================================================

impl std::fmt::Display for Reg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::H => write!(f, "H"),
            Self::L => write!(f, "L"),
            Self::F => write!(f, "F"),
        }
    }
}

impl std::fmt::Display for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BC => write!(f, "BC"),
            Self::DE => write!(f, "DE"),
            Self::HL => write!(f, "HL"),
            Self::SP => write!(f, "SP"),
            Self::AF => write!(f, "AF"),
        }
    }
}

impl std::fmt::Display for Src8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{r}"),
            Self::Indirect(r) => write!(f, "[{r}]"),
            Self::Imm => write!(f, "n"),
            Self::HighMemC => write!(f, "[FF00+C]"),
            Self::HighMemImm => write!(f, "[FF00+n]"),
            Self::MemImm => write!(f, "[nn]"),
        }
    }
}

impl std::fmt::Display for Dst8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{r}"),
            Self::Indirect(r) => write!(f, "[{r}]"),
            Self::HighMemC => write!(f, "[FF00+C]"),
            Self::HighMemImm => write!(f, "[FF00+n]"),
            Self::MemImm => write!(f, "[nn]"),
        }
    }
}

impl std::fmt::Display for Src16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{r}"),
            Self::Imm => write!(f, "nn"),
        }
    }
}

impl std::fmt::Display for Dst16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{r}"),
            Self::MemImm => write!(f, "[nn]"),
        }
    }
}

impl std::fmt::Display for RmwOperand8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reg(r) => write!(f, "{r}"),
            Self::Indirect(r) => write!(f, "[{r}]"),
        }
    }
}
