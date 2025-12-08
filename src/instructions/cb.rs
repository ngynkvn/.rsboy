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
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::CPU;

    fn setup() -> (CPU, Bus) {
        let cpu = CPU::new();
        let mut bus = Bus::new(&[], None);
        bus.in_bios = 1; // Skip bootrom
        (cpu, bus)
    }

    /// Helper to execute a CB instruction with given opcode
    fn execute_cb(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
        // Put the CB opcode at PC location
        let pc = cpu.registers.pc;
        bus.memory[pc as usize] = opcode;
        cb(cpu, bus);
    }

    #[test]
    fn test_swap_nibbles() {
        assert_eq!(swap_nibbles(0x12), 0x21);
        assert_eq!(swap_nibbles(0xAB), 0xBA);
        assert_eq!(swap_nibbles(0x00), 0x00);
        assert_eq!(swap_nibbles(0xFF), 0xFF);
    }

    // RLC - Rotate Left Circular (0x00-0x07)
    #[test]
    fn rlc_rotates_left_with_bit7_to_carry_and_bit0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::B, 0b1000_0001);
        execute_cb(&mut cpu, &mut bus, 0x00); // RLC B
        assert_eq!(cpu.registers.get_r8(Reg8::B), 0b0000_0011);
        assert!(cpu.registers.flg_c());
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
    }

    #[test]
    fn rlc_sets_zero_flag_when_result_is_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::B, 0x00);
        execute_cb(&mut cpu, &mut bus, 0x00); // RLC B
        assert_eq!(cpu.registers.get_r8(Reg8::B), 0x00);
        assert!(cpu.registers.flg_z());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn rlc_a_register() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b0100_0000);
        execute_cb(&mut cpu, &mut bus, 0x07); // RLC A
        assert_eq!(cpu.registers.get_r8(Reg8::A), 0b1000_0000);
        assert!(!cpu.registers.flg_c());
    }

    // RRC - Rotate Right Circular (0x08-0x0F)
    #[test]
    fn rrc_rotates_right_with_bit0_to_carry_and_bit7() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::C, 0b1000_0001);
        execute_cb(&mut cpu, &mut bus, 0x09); // RRC C
        assert_eq!(cpu.registers.get_r8(Reg8::C), 0b1100_0000);
        assert!(cpu.registers.flg_c());
        assert!(!cpu.registers.flg_z());
    }

    #[test]
    fn rrc_no_carry_when_bit0_is_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::D, 0b1000_0000);
        execute_cb(&mut cpu, &mut bus, 0x0A); // RRC D
        assert_eq!(cpu.registers.get_r8(Reg8::D), 0b0100_0000);
        assert!(!cpu.registers.flg_c());
    }

    // RL - Rotate Left through carry (0x10-0x17)
    #[test]
    fn rl_rotates_left_through_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::E, 0b0100_0000);
        cpu.registers.set_cf(true);
        execute_cb(&mut cpu, &mut bus, 0x13); // RL E
        assert_eq!(cpu.registers.get_r8(Reg8::E), 0b1000_0001);
        assert!(!cpu.registers.flg_c()); // Old bit 7 was 0
    }

    #[test]
    fn rl_sets_carry_from_bit7() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::H, 0b1000_0000);
        cpu.registers.set_cf(false);
        execute_cb(&mut cpu, &mut bus, 0x14); // RL H
        assert_eq!(cpu.registers.get_r8(Reg8::H), 0b0000_0000);
        assert!(cpu.registers.flg_c());
        assert!(cpu.registers.flg_z());
    }

    // RR - Rotate Right through carry (0x18-0x1F)
    #[test]
    fn rr_rotates_right_through_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::L, 0b0000_0010);
        cpu.registers.set_cf(true);
        execute_cb(&mut cpu, &mut bus, 0x1D); // RR L
        assert_eq!(cpu.registers.get_r8(Reg8::L), 0b1000_0001);
        assert!(!cpu.registers.flg_c()); // Old bit 0 was 0
    }

    #[test]
    fn rr_sets_carry_from_bit0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b0000_0001);
        cpu.registers.set_cf(false);
        execute_cb(&mut cpu, &mut bus, 0x1F); // RR A
        assert_eq!(cpu.registers.get_r8(Reg8::A), 0b0000_0000);
        assert!(cpu.registers.flg_c());
        assert!(cpu.registers.flg_z());
    }

    // SLA - Shift Left Arithmetic (0x20-0x27)
    #[test]
    fn sla_shifts_left_and_sets_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::B, 0b1100_0001);
        execute_cb(&mut cpu, &mut bus, 0x20); // SLA B
        assert_eq!(cpu.registers.get_r8(Reg8::B), 0b1000_0010);
        assert!(cpu.registers.flg_c()); // Bit 7 was set
    }

    #[test]
    fn sla_clears_carry_when_bit7_is_zero() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::C, 0b0100_0000);
        execute_cb(&mut cpu, &mut bus, 0x21); // SLA C
        assert_eq!(cpu.registers.get_r8(Reg8::C), 0b1000_0000);
        assert!(!cpu.registers.flg_c());
    }

    // SRA - Shift Right Arithmetic (0x28-0x2F)
    #[test]
    fn sra_preserves_sign_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::D, 0b1000_0100);
        execute_cb(&mut cpu, &mut bus, 0x2A); // SRA D
        assert_eq!(cpu.registers.get_r8(Reg8::D), 0b1100_0010);
        assert!(!cpu.registers.flg_c()); // Bit 0 was 0
    }

    #[test]
    fn sra_sets_carry_from_bit0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::E, 0b0000_0011);
        execute_cb(&mut cpu, &mut bus, 0x2B); // SRA E
        assert_eq!(cpu.registers.get_r8(Reg8::E), 0b0000_0001);
        assert!(cpu.registers.flg_c()); // Bit 0 was 1
    }

    // SWAP - Swap nibbles (0x30-0x37)
    #[test]
    fn swap_exchanges_nibbles() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::H, 0x12);
        execute_cb(&mut cpu, &mut bus, 0x34); // SWAP H
        assert_eq!(cpu.registers.get_r8(Reg8::H), 0x21);
        assert!(!cpu.registers.flg_z());
        assert!(!cpu.registers.flg_n());
        assert!(!cpu.registers.flg_h());
        assert!(!cpu.registers.flg_c());
    }

    #[test]
    fn swap_zero_sets_zero_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::L, 0x00);
        execute_cb(&mut cpu, &mut bus, 0x35); // SWAP L
        assert_eq!(cpu.registers.get_r8(Reg8::L), 0x00);
        assert!(cpu.registers.flg_z());
    }

    // SRL - Shift Right Logical (0x38-0x3F)
    #[test]
    fn srl_shifts_right_and_clears_bit7() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b1000_0010);
        execute_cb(&mut cpu, &mut bus, 0x3F); // SRL A
        assert_eq!(cpu.registers.get_r8(Reg8::A), 0b0100_0001);
        assert!(!cpu.registers.flg_c()); // Bit 0 was 0
    }

    #[test]
    fn srl_sets_carry_from_bit0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::B, 0b0000_0001);
        execute_cb(&mut cpu, &mut bus, 0x38); // SRL B
        assert_eq!(cpu.registers.get_r8(Reg8::B), 0b0000_0000);
        assert!(cpu.registers.flg_c());
        assert!(cpu.registers.flg_z());
    }

    // BIT - Test bit (0x40-0x7F)
    #[test]
    fn bit_0_tests_lowest_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::C, 0b0000_0001);
        execute_cb(&mut cpu, &mut bus, 0x41); // BIT 0,C
        assert!(!cpu.registers.flg_z()); // Bit is set
        assert!(!cpu.registers.flg_n());
        assert!(cpu.registers.flg_h());
    }

    #[test]
    fn bit_7_tests_highest_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::D, 0b0111_1111);
        execute_cb(&mut cpu, &mut bus, 0x7A); // BIT 7,D
        assert!(cpu.registers.flg_z()); // Bit 7 is not set
    }

    #[test]
    fn bit_preserves_carry_flag() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::E, 0x00);
        cpu.registers.set_cf(true);
        execute_cb(&mut cpu, &mut bus, 0x43); // BIT 0,E
        assert!(cpu.registers.flg_z());
        assert!(cpu.registers.flg_c()); // Carry unchanged
    }

    #[test]
    fn bit_4_checks_correct_position() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b0001_0000);
        execute_cb(&mut cpu, &mut bus, 0x67); // BIT 4,A
        assert!(!cpu.registers.flg_z()); // Bit 4 is set
    }

    // RES - Reset bit (0x80-0xBF)
    #[test]
    fn res_clears_bit_0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::B, 0b1111_1111);
        execute_cb(&mut cpu, &mut bus, 0x80); // RES 0,B
        assert_eq!(cpu.registers.get_r8(Reg8::B), 0b1111_1110);
    }

    #[test]
    fn res_clears_bit_7() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::C, 0b1111_1111);
        execute_cb(&mut cpu, &mut bus, 0xB9); // RES 7,C
        assert_eq!(cpu.registers.get_r8(Reg8::C), 0b0111_1111);
    }

    #[test]
    fn res_on_already_clear_bit_is_noop() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::D, 0b0000_0000);
        execute_cb(&mut cpu, &mut bus, 0x92); // RES 2,D
        assert_eq!(cpu.registers.get_r8(Reg8::D), 0b0000_0000);
    }

    #[test]
    fn res_4_clears_correct_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b1111_1111);
        execute_cb(&mut cpu, &mut bus, 0xA7); // RES 4,A
        assert_eq!(cpu.registers.get_r8(Reg8::A), 0b1110_1111);
    }

    // SET - Set bit (0xC0-0xFF)
    #[test]
    fn set_sets_bit_0() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::E, 0b0000_0000);
        execute_cb(&mut cpu, &mut bus, 0xC3); // SET 0,E
        assert_eq!(cpu.registers.get_r8(Reg8::E), 0b0000_0001);
    }

    #[test]
    fn set_sets_bit_7() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::H, 0b0000_0000);
        execute_cb(&mut cpu, &mut bus, 0xFC); // SET 7,H
        assert_eq!(cpu.registers.get_r8(Reg8::H), 0b1000_0000);
    }

    #[test]
    fn set_on_already_set_bit_is_noop() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::L, 0b1111_1111);
        execute_cb(&mut cpu, &mut bus, 0xED); // SET 5,L
        assert_eq!(cpu.registers.get_r8(Reg8::L), 0b1111_1111);
    }

    #[test]
    fn set_3_sets_correct_bit() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r8(Reg8::A, 0b0000_0000);
        execute_cb(&mut cpu, &mut bus, 0xDF); // SET 3,A
        assert_eq!(cpu.registers.get_r8(Reg8::A), 0b0000_1000);
    }

    // Memory (HL) operations
    #[test]
    fn rlc_hl_indirect() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r16(Reg16::HL, 0xC000);
        bus.memory[0xC000] = 0b1000_0001;
        execute_cb(&mut cpu, &mut bus, 0x06); // RLC (HL)
        assert_eq!(bus.memory[0xC000], 0b0000_0011);
        assert!(cpu.registers.flg_c());
    }

    #[test]
    fn bit_hl_indirect() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r16(Reg16::HL, 0xC000);
        bus.memory[0xC000] = 0b0000_1000;
        execute_cb(&mut cpu, &mut bus, 0x5E); // BIT 3,(HL)
        assert!(!cpu.registers.flg_z()); // Bit 3 is set
    }

    #[test]
    fn res_hl_indirect() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r16(Reg16::HL, 0xC000);
        bus.memory[0xC000] = 0b1111_1111;
        execute_cb(&mut cpu, &mut bus, 0x96); // RES 2,(HL)
        assert_eq!(bus.memory[0xC000], 0b1111_1011);
    }

    #[test]
    fn set_hl_indirect() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.set_r16(Reg16::HL, 0xC000);
        bus.memory[0xC000] = 0b0000_0000;
        execute_cb(&mut cpu, &mut bus, 0xFE); // SET 7,(HL)
        assert_eq!(bus.memory[0xC000], 0b1000_0000);
    }

    // Test decode_operand
    #[test]
    fn decode_operand_maps_correctly() {
        assert!(matches!(decode_operand(0x00), RmwOperand8::Reg(Reg8::B)));
        assert!(matches!(decode_operand(0x01), RmwOperand8::Reg(Reg8::C)));
        assert!(matches!(decode_operand(0x02), RmwOperand8::Reg(Reg8::D)));
        assert!(matches!(decode_operand(0x03), RmwOperand8::Reg(Reg8::E)));
        assert!(matches!(decode_operand(0x04), RmwOperand8::Reg(Reg8::H)));
        assert!(matches!(decode_operand(0x05), RmwOperand8::Reg(Reg8::L)));
        assert!(matches!(decode_operand(0x06), RmwOperand8::Indirect(Reg16::HL)));
        assert!(matches!(decode_operand(0x07), RmwOperand8::Reg(Reg8::A)));
    }
}
