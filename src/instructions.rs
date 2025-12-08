//! Game Boy CPU instruction definitions using typed operands.

mod alu;
mod cb;
mod jp;
mod ld;
mod misc;

use strum_macros::IntoStaticStr;

use crate::{
    bus::Bus,
    cpu::CPU,
    operand::{Dst8, Dst16, Reg8, Reg16, RmwOperand8, Src8, Src16},
};

// Re-export operand types for convenience
pub use crate::operand::{Reg8 as R8, Reg16 as R16};

#[derive(Debug, PartialEq, Eq, Copy, Clone, IntoStaticStr, Hash)]
pub enum Flag {
    FlagNZ,
    FlagZ,
    FlagC,
    FlagNC,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

/// Game Boy CPU instructions with typed operands
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub enum Instr {
    #[default]
    Nop,

    // 8-bit loads
    Ld8(Dst8, Src8),
    LdInc(Dst8, Src8),  // LD with HL++
    LdDec(Dst8, Src8),  // LD with HL--

    // 16-bit loads
    Ld16(Dst16, Src16),
    LdSpHl,             // LD SP, HL
    LdHlSpOffset,       // LD HL, SP+e (signed offset)

    // 8-bit ALU (operate on A register)
    Add(Src8),
    Adc(Src8),
    Sub(Src8),
    Sbc(Src8),
    And(Src8),
    Xor(Src8),
    Or(Src8),
    Cp(Src8),

    // 8-bit INC/DEC
    Inc8(RmwOperand8),
    Dec8(RmwOperand8),

    // 16-bit INC/DEC
    Inc16(Reg16),
    Dec16(Reg16),

    // 16-bit ADD HL
    AddHl(Reg16),

    // ADD SP, e (signed offset)
    AddSp,

    // Rotates on A (no zero flag set)
    Rlca,
    Rrca,
    Rla,
    Rra,

    // Misc ALU
    Daa,
    Cpl,    // Complement A (NOT A)
    Scf,    // Set carry flag
    Ccf,    // Complement carry flag

    // CB prefix (bit operations)
    Cb,

    // Jumps
    Jr(Option<Flag>),
    Jp(Option<Flag>),
    JpHl,

    // Calls and returns
    Call(Option<Flag>),
    Ret(Option<Flag>),
    Reti,
    Rst(u8),

    // Stack ops
    Push(Reg16),
    Pop(Reg16),

    // Interrupts
    Di,
    Ei,

    // Control
    Halt,
    Stop,

    // Invalid/unimplemented opcode
    Invalid,
}

impl Instr {
    /// Execute this instruction
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn run(self, cpu: &mut CPU, bus: &mut Bus) -> Self {
        match self {
            Self::Nop => {}
            Self::Invalid => panic!("Invalid opcode executed"),
            Self::Stop => {} // STOP halts until button press - treated as NOP
            Self::Halt => {} // Handled by CPU state machine

            // 8-bit loads
            Self::Ld8(dst, src) => ld::ld8(dst, src, cpu, bus),
            Self::LdInc(dst, src) => ld::ld_inc(dst, src, cpu, bus),
            Self::LdDec(dst, src) => ld::ld_dec(dst, src, cpu, bus),

            // 16-bit loads
            Self::Ld16(dst, src) => ld::ld16(dst, src, cpu, bus),
            Self::LdSpHl => ld::ld_sp_hl(cpu, bus),
            Self::LdHlSpOffset => ld::ld_hl_sp_offset(cpu, bus),

            // 8-bit ALU
            Self::Add(src) => alu::add(src, cpu, bus),
            Self::Adc(src) => alu::adc(src, cpu, bus),
            Self::Sub(src) => alu::sub(src, cpu, bus),
            Self::Sbc(src) => alu::sbc(src, cpu, bus),
            Self::And(src) => alu::and(src, cpu, bus),
            Self::Xor(src) => alu::xor(src, cpu, bus),
            Self::Or(src) => alu::or(src, cpu, bus),
            Self::Cp(src) => alu::cp(src, cpu, bus),

            // 8-bit INC/DEC
            Self::Inc8(op) => alu::inc8(op, cpu, bus),
            Self::Dec8(op) => alu::dec8(op, cpu, bus),

            // 16-bit INC/DEC
            Self::Inc16(r) => alu::inc16(r, cpu, bus),
            Self::Dec16(r) => alu::dec16(r, cpu, bus),

            // 16-bit ADD
            Self::AddHl(r) => alu::add_hl(r, cpu, bus),
            Self::AddSp => alu::add_sp(cpu, bus),

            // Rotates on A
            Self::Rlca => alu::rlca(cpu),
            Self::Rrca => alu::rrca(cpu),
            Self::Rla => alu::rla(cpu),
            Self::Rra => alu::rra(cpu),

            // Misc ALU
            Self::Daa => alu::daa(cpu),
            Self::Cpl => alu::cpl(cpu),
            Self::Scf => alu::scf(cpu),
            Self::Ccf => alu::ccf(cpu),

            // CB prefix
            Self::Cb => cb::cb(cpu, bus),

            // Jumps
            Self::Jr(flag) => jp::jr(flag, cpu, bus),
            Self::Jp(flag) => jp::jp(flag, cpu, bus),
            Self::JpHl => jp::jp_hl(cpu),

            // Calls and returns
            Self::Call(flag) => jp::call(flag, cpu, bus),
            Self::Ret(flag) => jp::ret(flag, cpu, bus),
            Self::Reti => jp::reti(cpu, bus),
            Self::Rst(addr) => jp::rst(addr, cpu, bus),

            // Stack
            Self::Push(r) => misc::push(r, cpu, bus),
            Self::Pop(r) => misc::pop(r, cpu, bus),

            // Interrupts
            Self::Di => bus.disable_interrupts(),
            Self::Ei => bus.enable_interrupts(),
        }
        self
    }

    /// Calculate instruction length (including opcode byte)
    #[allow(clippy::match_same_arms)]
    pub const fn length(&self) -> usize {
        1 + match self {
            Self::Ld8(dst, src)
            | Self::LdInc(dst, src)
            | Self::LdDec(dst, src) => dst.imm_bytes() + src.imm_bytes(),
            Self::Ld16(dst, src) => dst.imm_bytes() + src.imm_bytes(),
            Self::LdHlSpOffset | Self::AddSp | Self::Jr(_) | Self::Cb => 1,
            Self::Jp(_) | Self::Call(_) => 2,
            Self::Add(src) | Self::Adc(src) | Self::Sub(src) | Self::Sbc(src)
            | Self::And(src) | Self::Xor(src) | Self::Or(src) | Self::Cp(src) => src.imm_bytes(),
            _ => 0,
        }
    }
}

// ============================================================================
// Instruction table - maps opcodes 0x00-0xFF to instructions
// ============================================================================

use Dst8 as D8;
use Src8 as S8;
use Dst16 as D16;
use Src16 as S16;
use Reg8::*;
use Reg16::*;
use RmwOperand8 as Rmw;
use Flag::*;
use Instr::*;

pub const INSTR_TABLE: [Instr; 256] = [
    // 0x00-0x0F
    Nop,                                           // 0x00
    Ld16(D16::Reg(BC), S16::Imm),                  // 0x01 LD BC, nn
    Ld8(D8::Indirect(BC), S8::Reg(A)),             // 0x02 LD [BC], A
    Inc16(BC),                                     // 0x03 INC BC
    Inc8(Rmw::Reg(B)),                             // 0x04 INC B
    Dec8(Rmw::Reg(B)),                             // 0x05 DEC B
    Ld8(D8::Reg(B), S8::Imm),                      // 0x06 LD B, n
    Rlca,                                          // 0x07 RLCA
    Ld16(D16::MemImm, S16::Reg(SP)),               // 0x08 LD [nn], SP
    AddHl(BC),                                     // 0x09 ADD HL, BC
    Ld8(D8::Reg(A), S8::Indirect(BC)),             // 0x0A LD A, [BC]
    Dec16(BC),                                     // 0x0B DEC BC
    Inc8(Rmw::Reg(C)),                             // 0x0C INC C
    Dec8(Rmw::Reg(C)),                             // 0x0D DEC C
    Ld8(D8::Reg(C), S8::Imm),                      // 0x0E LD C, n
    Rrca,                                          // 0x0F RRCA

    // 0x10-0x1F
    Stop,                                          // 0x10 STOP
    Ld16(D16::Reg(DE), S16::Imm),                  // 0x11 LD DE, nn
    Ld8(D8::Indirect(DE), S8::Reg(A)),             // 0x12 LD [DE], A
    Inc16(DE),                                     // 0x13 INC DE
    Inc8(Rmw::Reg(D)),                             // 0x14 INC D
    Dec8(Rmw::Reg(D)),                             // 0x15 DEC D
    Ld8(D8::Reg(D), S8::Imm),                      // 0x16 LD D, n
    Rla,                                           // 0x17 RLA
    Jr(None),                                      // 0x18 JR e
    AddHl(DE),                                     // 0x19 ADD HL, DE
    Ld8(D8::Reg(A), S8::Indirect(DE)),             // 0x1A LD A, [DE]
    Dec16(DE),                                     // 0x1B DEC DE
    Inc8(Rmw::Reg(E)),                             // 0x1C INC E
    Dec8(Rmw::Reg(E)),                             // 0x1D DEC E
    Ld8(D8::Reg(E), S8::Imm),                      // 0x1E LD E, n
    Rra,                                           // 0x1F RRA

    // 0x20-0x2F
    Jr(Some(FlagNZ)),                              // 0x20 JR NZ, e
    Ld16(D16::Reg(HL), S16::Imm),                  // 0x21 LD HL, nn
    LdInc(D8::Indirect(HL), S8::Reg(A)),           // 0x22 LD [HL+], A
    Inc16(HL),                                     // 0x23 INC HL
    Inc8(Rmw::Reg(H)),                             // 0x24 INC H
    Dec8(Rmw::Reg(H)),                             // 0x25 DEC H
    Ld8(D8::Reg(H), S8::Imm),                      // 0x26 LD H, n
    Daa,                                           // 0x27 DAA
    Jr(Some(FlagZ)),                               // 0x28 JR Z, e
    AddHl(HL),                                     // 0x29 ADD HL, HL
    LdInc(D8::Reg(A), S8::Indirect(HL)),           // 0x2A LD A, [HL+]
    Dec16(HL),                                     // 0x2B DEC HL
    Inc8(Rmw::Reg(L)),                             // 0x2C INC L
    Dec8(Rmw::Reg(L)),                             // 0x2D DEC L
    Ld8(D8::Reg(L), S8::Imm),                      // 0x2E LD L, n
    Cpl,                                           // 0x2F CPL

    // 0x30-0x3F
    Jr(Some(FlagNC)),                              // 0x30 JR NC, e
    Ld16(D16::Reg(SP), S16::Imm),                  // 0x31 LD SP, nn
    LdDec(D8::Indirect(HL), S8::Reg(A)),           // 0x32 LD [HL-], A
    Inc16(SP),                                     // 0x33 INC SP
    Inc8(Rmw::Indirect(HL)),                       // 0x34 INC [HL]
    Dec8(Rmw::Indirect(HL)),                       // 0x35 DEC [HL]
    Ld8(D8::Indirect(HL), S8::Imm),                // 0x36 LD [HL], n
    Scf,                                           // 0x37 SCF
    Jr(Some(FlagC)),                               // 0x38 JR C, e
    AddHl(SP),                                     // 0x39 ADD HL, SP
    LdDec(D8::Reg(A), S8::Indirect(HL)),           // 0x3A LD A, [HL-]
    Dec16(SP),                                     // 0x3B DEC SP
    Inc8(Rmw::Reg(A)),                             // 0x3C INC A
    Dec8(Rmw::Reg(A)),                             // 0x3D DEC A
    Ld8(D8::Reg(A), S8::Imm),                      // 0x3E LD A, n
    Ccf,                                           // 0x3F CCF

    // 0x40-0x4F: LD B/C, r
    Ld8(D8::Reg(B), S8::Reg(B)),                   // 0x40 LD B, B
    Ld8(D8::Reg(B), S8::Reg(C)),                   // 0x41 LD B, C
    Ld8(D8::Reg(B), S8::Reg(D)),                   // 0x42 LD B, D
    Ld8(D8::Reg(B), S8::Reg(E)),                   // 0x43 LD B, E
    Ld8(D8::Reg(B), S8::Reg(H)),                   // 0x44 LD B, H
    Ld8(D8::Reg(B), S8::Reg(L)),                   // 0x45 LD B, L
    Ld8(D8::Reg(B), S8::Indirect(HL)),             // 0x46 LD B, [HL]
    Ld8(D8::Reg(B), S8::Reg(A)),                   // 0x47 LD B, A
    Ld8(D8::Reg(C), S8::Reg(B)),                   // 0x48 LD C, B
    Ld8(D8::Reg(C), S8::Reg(C)),                   // 0x49 LD C, C
    Ld8(D8::Reg(C), S8::Reg(D)),                   // 0x4A LD C, D
    Ld8(D8::Reg(C), S8::Reg(E)),                   // 0x4B LD C, E
    Ld8(D8::Reg(C), S8::Reg(H)),                   // 0x4C LD C, H
    Ld8(D8::Reg(C), S8::Reg(L)),                   // 0x4D LD C, L
    Ld8(D8::Reg(C), S8::Indirect(HL)),             // 0x4E LD C, [HL]
    Ld8(D8::Reg(C), S8::Reg(A)),                   // 0x4F LD C, A

    // 0x50-0x5F: LD D/E, r
    Ld8(D8::Reg(D), S8::Reg(B)),                   // 0x50 LD D, B
    Ld8(D8::Reg(D), S8::Reg(C)),                   // 0x51 LD D, C
    Ld8(D8::Reg(D), S8::Reg(D)),                   // 0x52 LD D, D
    Ld8(D8::Reg(D), S8::Reg(E)),                   // 0x53 LD D, E
    Ld8(D8::Reg(D), S8::Reg(H)),                   // 0x54 LD D, H
    Ld8(D8::Reg(D), S8::Reg(L)),                   // 0x55 LD D, L
    Ld8(D8::Reg(D), S8::Indirect(HL)),             // 0x56 LD D, [HL]
    Ld8(D8::Reg(D), S8::Reg(A)),                   // 0x57 LD D, A
    Ld8(D8::Reg(E), S8::Reg(B)),                   // 0x58 LD E, B
    Ld8(D8::Reg(E), S8::Reg(C)),                   // 0x59 LD E, C
    Ld8(D8::Reg(E), S8::Reg(D)),                   // 0x5A LD E, D
    Ld8(D8::Reg(E), S8::Reg(E)),                   // 0x5B LD E, E
    Ld8(D8::Reg(E), S8::Reg(H)),                   // 0x5C LD E, H
    Ld8(D8::Reg(E), S8::Reg(L)),                   // 0x5D LD E, L
    Ld8(D8::Reg(E), S8::Indirect(HL)),             // 0x5E LD E, [HL]
    Ld8(D8::Reg(E), S8::Reg(A)),                   // 0x5F LD E, A

    // 0x60-0x6F: LD H/L, r
    Ld8(D8::Reg(H), S8::Reg(B)),                   // 0x60 LD H, B
    Ld8(D8::Reg(H), S8::Reg(C)),                   // 0x61 LD H, C
    Ld8(D8::Reg(H), S8::Reg(D)),                   // 0x62 LD H, D
    Ld8(D8::Reg(H), S8::Reg(E)),                   // 0x63 LD H, E
    Ld8(D8::Reg(H), S8::Reg(H)),                   // 0x64 LD H, H
    Ld8(D8::Reg(H), S8::Reg(L)),                   // 0x65 LD H, L
    Ld8(D8::Reg(H), S8::Indirect(HL)),             // 0x66 LD H, [HL]
    Ld8(D8::Reg(H), S8::Reg(A)),                   // 0x67 LD H, A
    Ld8(D8::Reg(L), S8::Reg(B)),                   // 0x68 LD L, B
    Ld8(D8::Reg(L), S8::Reg(C)),                   // 0x69 LD L, C
    Ld8(D8::Reg(L), S8::Reg(D)),                   // 0x6A LD L, D
    Ld8(D8::Reg(L), S8::Reg(E)),                   // 0x6B LD L, E
    Ld8(D8::Reg(L), S8::Reg(H)),                   // 0x6C LD L, H
    Ld8(D8::Reg(L), S8::Reg(L)),                   // 0x6D LD L, L
    Ld8(D8::Reg(L), S8::Indirect(HL)),             // 0x6E LD L, [HL]
    Ld8(D8::Reg(L), S8::Reg(A)),                   // 0x6F LD L, A

    // 0x70-0x7F: LD [HL]/A, r
    Ld8(D8::Indirect(HL), S8::Reg(B)),             // 0x70 LD [HL], B
    Ld8(D8::Indirect(HL), S8::Reg(C)),             // 0x71 LD [HL], C
    Ld8(D8::Indirect(HL), S8::Reg(D)),             // 0x72 LD [HL], D
    Ld8(D8::Indirect(HL), S8::Reg(E)),             // 0x73 LD [HL], E
    Ld8(D8::Indirect(HL), S8::Reg(H)),             // 0x74 LD [HL], H
    Ld8(D8::Indirect(HL), S8::Reg(L)),             // 0x75 LD [HL], L
    Halt,                                          // 0x76 HALT
    Ld8(D8::Indirect(HL), S8::Reg(A)),             // 0x77 LD [HL], A
    Ld8(D8::Reg(A), S8::Reg(B)),                   // 0x78 LD A, B
    Ld8(D8::Reg(A), S8::Reg(C)),                   // 0x79 LD A, C
    Ld8(D8::Reg(A), S8::Reg(D)),                   // 0x7A LD A, D
    Ld8(D8::Reg(A), S8::Reg(E)),                   // 0x7B LD A, E
    Ld8(D8::Reg(A), S8::Reg(H)),                   // 0x7C LD A, H
    Ld8(D8::Reg(A), S8::Reg(L)),                   // 0x7D LD A, L
    Ld8(D8::Reg(A), S8::Indirect(HL)),             // 0x7E LD A, [HL]
    Ld8(D8::Reg(A), S8::Reg(A)),                   // 0x7F LD A, A

    // 0x80-0x8F: ADD/ADC A, r
    Add(S8::Reg(B)),                               // 0x80 ADD A, B
    Add(S8::Reg(C)),                               // 0x81 ADD A, C
    Add(S8::Reg(D)),                               // 0x82 ADD A, D
    Add(S8::Reg(E)),                               // 0x83 ADD A, E
    Add(S8::Reg(H)),                               // 0x84 ADD A, H
    Add(S8::Reg(L)),                               // 0x85 ADD A, L
    Add(S8::Indirect(HL)),                         // 0x86 ADD A, [HL]
    Add(S8::Reg(A)),                               // 0x87 ADD A, A
    Adc(S8::Reg(B)),                               // 0x88 ADC A, B
    Adc(S8::Reg(C)),                               // 0x89 ADC A, C
    Adc(S8::Reg(D)),                               // 0x8A ADC A, D
    Adc(S8::Reg(E)),                               // 0x8B ADC A, E
    Adc(S8::Reg(H)),                               // 0x8C ADC A, H
    Adc(S8::Reg(L)),                               // 0x8D ADC A, L
    Adc(S8::Indirect(HL)),                         // 0x8E ADC A, [HL]
    Adc(S8::Reg(A)),                               // 0x8F ADC A, A

    // 0x90-0x9F: SUB/SBC A, r
    Sub(S8::Reg(B)),                               // 0x90 SUB A, B
    Sub(S8::Reg(C)),                               // 0x91 SUB A, C
    Sub(S8::Reg(D)),                               // 0x92 SUB A, D
    Sub(S8::Reg(E)),                               // 0x93 SUB A, E
    Sub(S8::Reg(H)),                               // 0x94 SUB A, H
    Sub(S8::Reg(L)),                               // 0x95 SUB A, L
    Sub(S8::Indirect(HL)),                         // 0x96 SUB A, [HL]
    Sub(S8::Reg(A)),                               // 0x97 SUB A, A
    Sbc(S8::Reg(B)),                               // 0x98 SBC A, B
    Sbc(S8::Reg(C)),                               // 0x99 SBC A, C
    Sbc(S8::Reg(D)),                               // 0x9A SBC A, D
    Sbc(S8::Reg(E)),                               // 0x9B SBC A, E
    Sbc(S8::Reg(H)),                               // 0x9C SBC A, H
    Sbc(S8::Reg(L)),                               // 0x9D SBC A, L
    Sbc(S8::Indirect(HL)),                         // 0x9E SBC A, [HL]
    Sbc(S8::Reg(A)),                               // 0x9F SBC A, A

    // 0xA0-0xAF: AND/XOR A, r
    And(S8::Reg(B)),                               // 0xA0 AND A, B
    And(S8::Reg(C)),                               // 0xA1 AND A, C
    And(S8::Reg(D)),                               // 0xA2 AND A, D
    And(S8::Reg(E)),                               // 0xA3 AND A, E
    And(S8::Reg(H)),                               // 0xA4 AND A, H
    And(S8::Reg(L)),                               // 0xA5 AND A, L
    And(S8::Indirect(HL)),                         // 0xA6 AND A, [HL]
    And(S8::Reg(A)),                               // 0xA7 AND A, A
    Xor(S8::Reg(B)),                               // 0xA8 XOR A, B
    Xor(S8::Reg(C)),                               // 0xA9 XOR A, C
    Xor(S8::Reg(D)),                               // 0xAA XOR A, D
    Xor(S8::Reg(E)),                               // 0xAB XOR A, E
    Xor(S8::Reg(H)),                               // 0xAC XOR A, H
    Xor(S8::Reg(L)),                               // 0xAD XOR A, L
    Xor(S8::Indirect(HL)),                         // 0xAE XOR A, [HL]
    Xor(S8::Reg(A)),                               // 0xAF XOR A, A

    // 0xB0-0xBF: OR/CP A, r
    Or(S8::Reg(B)),                                // 0xB0 OR A, B
    Or(S8::Reg(C)),                                // 0xB1 OR A, C
    Or(S8::Reg(D)),                                // 0xB2 OR A, D
    Or(S8::Reg(E)),                                // 0xB3 OR A, E
    Or(S8::Reg(H)),                                // 0xB4 OR A, H
    Or(S8::Reg(L)),                                // 0xB5 OR A, L
    Or(S8::Indirect(HL)),                          // 0xB6 OR A, [HL]
    Or(S8::Reg(A)),                                // 0xB7 OR A, A
    Cp(S8::Reg(B)),                                // 0xB8 CP A, B
    Cp(S8::Reg(C)),                                // 0xB9 CP A, C
    Cp(S8::Reg(D)),                                // 0xBA CP A, D
    Cp(S8::Reg(E)),                                // 0xBB CP A, E
    Cp(S8::Reg(H)),                                // 0xBC CP A, H
    Cp(S8::Reg(L)),                                // 0xBD CP A, L
    Cp(S8::Indirect(HL)),                          // 0xBE CP A, [HL]
    Cp(S8::Reg(A)),                                // 0xBF CP A, A

    // 0xC0-0xCF
    Ret(Some(FlagNZ)),                             // 0xC0 RET NZ
    Pop(BC),                                       // 0xC1 POP BC
    Jp(Some(FlagNZ)),                              // 0xC2 JP NZ, nn
    Jp(None),                                      // 0xC3 JP nn
    Call(Some(FlagNZ)),                            // 0xC4 CALL NZ, nn
    Push(BC),                                      // 0xC5 PUSH BC
    Add(S8::Imm),                                  // 0xC6 ADD A, n
    Rst(0x00),                                     // 0xC7 RST 00H
    Ret(Some(FlagZ)),                              // 0xC8 RET Z
    Ret(None),                                     // 0xC9 RET
    Jp(Some(FlagZ)),                               // 0xCA JP Z, nn
    Cb,                                            // 0xCB CB prefix
    Call(Some(FlagZ)),                             // 0xCC CALL Z, nn
    Call(None),                                    // 0xCD CALL nn
    Adc(S8::Imm),                                  // 0xCE ADC A, n
    Rst(0x08),                                     // 0xCF RST 08H

    // 0xD0-0xDF
    Ret(Some(FlagNC)),                             // 0xD0 RET NC
    Pop(DE),                                       // 0xD1 POP DE
    Jp(Some(FlagNC)),                              // 0xD2 JP NC, nn
    Invalid,                                       // 0xD3 (invalid)
    Call(Some(FlagNC)),                            // 0xD4 CALL NC, nn
    Push(DE),                                      // 0xD5 PUSH DE
    Sub(S8::Imm),                                  // 0xD6 SUB A, n
    Rst(0x10),                                     // 0xD7 RST 10H
    Ret(Some(FlagC)),                              // 0xD8 RET C
    Reti,                                          // 0xD9 RETI
    Jp(Some(FlagC)),                               // 0xDA JP C, nn
    Invalid,                                       // 0xDB (invalid)
    Call(Some(FlagC)),                             // 0xDC CALL C, nn
    Invalid,                                       // 0xDD (invalid)
    Sbc(S8::Imm),                                  // 0xDE SBC A, n
    Rst(0x18),                                     // 0xDF RST 18H

    // 0xE0-0xEF
    Ld8(D8::HighMemImm, S8::Reg(A)),               // 0xE0 LDH [n], A
    Pop(HL),                                       // 0xE1 POP HL
    Ld8(D8::HighMemC, S8::Reg(A)),                 // 0xE2 LD [C], A
    Invalid,                                       // 0xE3 (invalid)
    Invalid,                                       // 0xE4 (invalid)
    Push(HL),                                      // 0xE5 PUSH HL
    And(S8::Imm),                                  // 0xE6 AND A, n
    Rst(0x20),                                     // 0xE7 RST 20H
    AddSp,                                         // 0xE8 ADD SP, e
    JpHl,                                          // 0xE9 JP HL
    Ld8(D8::MemImm, S8::Reg(A)),                   // 0xEA LD [nn], A
    Invalid,                                       // 0xEB (invalid)
    Invalid,                                       // 0xEC (invalid)
    Invalid,                                       // 0xED (invalid)
    Xor(S8::Imm),                                  // 0xEE XOR A, n
    Rst(0x28),                                     // 0xEF RST 28H

    // 0xF0-0xFF
    Ld8(D8::Reg(A), S8::HighMemImm),               // 0xF0 LDH A, [n]
    Pop(AF),                                       // 0xF1 POP AF
    Ld8(D8::Reg(A), S8::HighMemC),                 // 0xF2 LD A, [C]
    Di,                                            // 0xF3 DI
    Invalid,                                       // 0xF4 (invalid)
    Push(AF),                                      // 0xF5 PUSH AF
    Or(S8::Imm),                                   // 0xF6 OR A, n
    Rst(0x30),                                     // 0xF7 RST 30H
    LdHlSpOffset,                                  // 0xF8 LD HL, SP+e
    LdSpHl,                                        // 0xF9 LD SP, HL
    Ld8(D8::Reg(A), S8::MemImm),                   // 0xFA LD A, [nn]
    Ei,                                            // 0xFB EI
    Invalid,                                       // 0xFC (invalid)
    Invalid,                                       // 0xFD (invalid)
    Cp(S8::Imm),                                   // 0xFE CP A, n
    Rst(0x38),                                     // 0xFF RST 38H
];

/// Instruction data lengths (derived from instruction definitions)
pub const INSTR_DATA_LENGTHS: [usize; 256] = {
    let mut lengths = [0usize; 256];
    let mut i = 0;
    while i < 256 {
        lengths[i] = INSTR_TABLE[i].length() - 1; // subtract 1 for opcode byte
        i += 1;
    }
    lengths
};

impl From<u8> for Instr {
    fn from(value: u8) -> Self {
        INSTR_TABLE[value as usize]
    }
}

impl std::fmt::Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nop => write!(f, "NOP"),
            Self::Invalid => write!(f, "INVALID"),
            Self::Stop => write!(f, "STOP"),
            Self::Halt => write!(f, "HALT"),

            Self::Ld8(dst, src) => write!(f, "LD {dst}, {src}"),
            Self::LdInc(dst, src) => write!(f, "LD {dst}, {src} (HL+)"),
            Self::LdDec(dst, src) => write!(f, "LD {dst}, {src} (HL-)"),
            Self::Ld16(dst, src) => write!(f, "LD {dst}, {src}"),
            Self::LdSpHl => write!(f, "LD SP, HL"),
            Self::LdHlSpOffset => write!(f, "LD HL, SP+e"),

            Self::Add(src) => write!(f, "ADD A, {src}"),
            Self::Adc(src) => write!(f, "ADC A, {src}"),
            Self::Sub(src) => write!(f, "SUB A, {src}"),
            Self::Sbc(src) => write!(f, "SBC A, {src}"),
            Self::And(src) => write!(f, "AND A, {src}"),
            Self::Xor(src) => write!(f, "XOR A, {src}"),
            Self::Or(src) => write!(f, "OR A, {src}"),
            Self::Cp(src) => write!(f, "CP A, {src}"),

            Self::Inc8(op) => write!(f, "INC {op}"),
            Self::Dec8(op) => write!(f, "DEC {op}"),
            Self::Inc16(r) => write!(f, "INC {r}"),
            Self::Dec16(r) => write!(f, "DEC {r}"),

            Self::AddHl(r) => write!(f, "ADD HL, {r}"),
            Self::AddSp => write!(f, "ADD SP, e"),

            Self::Rlca => write!(f, "RLCA"),
            Self::Rrca => write!(f, "RRCA"),
            Self::Rla => write!(f, "RLA"),
            Self::Rra => write!(f, "RRA"),
            Self::Daa => write!(f, "DAA"),
            Self::Cpl => write!(f, "CPL"),
            Self::Scf => write!(f, "SCF"),
            Self::Ccf => write!(f, "CCF"),

            Self::Cb => write!(f, "CB"),

            Self::Jr(None) => write!(f, "JR e"),
            Self::Jr(Some(flag)) => write!(f, "JR {flag:?}, e"),
            Self::Jp(None) => write!(f, "JP nn"),
            Self::Jp(Some(flag)) => write!(f, "JP {flag:?}, nn"),
            Self::JpHl => write!(f, "JP HL"),

            Self::Call(None) => write!(f, "CALL nn"),
            Self::Call(Some(flag)) => write!(f, "CALL {flag:?}, nn"),
            Self::Ret(None) => write!(f, "RET"),
            Self::Ret(Some(flag)) => write!(f, "RET {flag:?}"),
            Self::Reti => write!(f, "RETI"),
            Self::Rst(addr) => write!(f, "RST {addr:02X}H"),

            Self::Push(r) => write!(f, "PUSH {r}"),
            Self::Pop(r) => write!(f, "POP {r}"),

            Self::Di => write!(f, "DI"),
            Self::Ei => write!(f, "EI"),
        }
    }
}

