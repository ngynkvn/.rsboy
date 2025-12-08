//! CB-prefixed bit manipulation instructions

use crate::{
    bus::Bus,
    cpu::CPU,
    operand::{Reg8, Reg16, RmwOperand8},
};

/// Decode operand from CB opcode lower nibble
fn decode_operand(opcode: u8) -> RmwOperand8 {
    match opcode & 0x07 {
        0x00 => RmwOperand8::Reg(Reg8::B),
        0x01 => RmwOperand8::Reg(Reg8::C),
        0x02 => RmwOperand8::Reg(Reg8::D),
        0x03 => RmwOperand8::Reg(Reg8::E),
        0x04 => RmwOperand8::Reg(Reg8::H),
        0x05 => RmwOperand8::Reg(Reg8::L),
        0x06 => RmwOperand8::Indirect(Reg16::HL),
        0x07 => RmwOperand8::Reg(Reg8::A),
        _ => unreachable!(),
    }
}

/// Execute CB-prefixed instruction
#[allow(clippy::too_many_lines)]
pub fn cb(cpu: &mut CPU, bus: &mut Bus) {
    let opcode = cpu.next_u8(bus);
    let target = decode_operand(opcode);
    let value = target.read(cpu, bus);

    match opcode {
        // RLC - Rotate Left Circular
        0x00..=0x07 => {
            let carry = value & 0x80 != 0;
            let result = (value << 1) | u8::from(carry);
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(carry);
            target.write(cpu, bus, result);
        }

        // RRC - Rotate Right Circular
        0x08..=0x0F => {
            let carry = value & 0x01 != 0;
            let result = (u8::from(carry) << 7) | (value >> 1);
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(carry);
            target.write(cpu, bus, result);
        }

        // RL - Rotate Left through carry
        0x10..=0x17 => {
            let result = (value << 1) | u8::from(cpu.registers.flg_c());
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(value & 0x80 != 0);
            target.write(cpu, bus, result);
        }

        // RR - Rotate Right through carry
        0x18..=0x1F => {
            let result = (value >> 1) | (u8::from(cpu.registers.flg_c()) << 7);
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(value & 0x01 != 0);
            target.write(cpu, bus, result);
        }

        // SLA - Shift Left Arithmetic
        0x20..=0x27 => {
            let result = value << 1;
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(value & 0x80 != 0);
            target.write(cpu, bus, result);
        }

        // SRA - Shift Right Arithmetic (preserves sign)
        0x28..=0x2F => {
            let result = (value >> 1) | (value & 0x80);
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(value & 0x01 != 0);
            target.write(cpu, bus, result);
        }

        // SWAP - Swap nibbles
        0x30..=0x37 => {
            let result = swap_nibbles(value);
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(false);
            target.write(cpu, bus, result);
        }

        // SRL - Shift Right Logical
        0x38..=0x3F => {
            let result = value >> 1;
            cpu.registers.set_zf(result == 0);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(false);
            cpu.registers.set_cf(value & 0x01 != 0);
            target.write(cpu, bus, result);
        }

        // BIT - Test bit
        0x40..=0x7F => {
            let bit_index = (opcode - 0x40) / 8;
            let check_zero = value & (1 << bit_index) == 0;
            cpu.registers.set_zf(check_zero);
            cpu.registers.set_nf(false);
            cpu.registers.set_hf(true);
            // BIT doesn't write back, but memory operations need extra cycle
            if target.is_memory() {
                bus.generic_cycle();
            }
        }

        // RES - Reset bit
        0x80..=0xBF => {
            let bit_index = (opcode - 0x80) / 8;
            let result = value & !(1 << bit_index);
            target.write(cpu, bus, result);
        }

        // SET - Set bit
        0xC0..=0xFF => {
            let bit_index = (opcode - 0xC0) / 8;
            let result = value | (1 << bit_index);
            target.write(cpu, bus, result);
        }
    }
}

/// Swap upper and lower nibbles of a byte
#[inline]
pub const fn swap_nibbles(byte: u8) -> u8 {
    byte.rotate_left(4)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_swap_nibbles() {
        assert_eq!(swap_nibbles(0x12), 0x21);
        assert_eq!(swap_nibbles(0xAB), 0xBA);
        assert_eq!(swap_nibbles(0x00), 0x00);
        assert_eq!(swap_nibbles(0xFF), 0xFF);
    }
}
