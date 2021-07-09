use crate::cpu::value::Value::U8;
use crate::{bus::Bus, cpu::CPU, instructions::Location, instructions::Register::*};

pub fn cb(cpu: &mut CPU, bus: &mut Bus) {
    let opcode = cpu.next_u8(bus);
    let target = {
        let opcode = opcode;
        match opcode & 0x0F {
            0x00 | 0x08 => Location::Register(B),
            0x01 | 0x09 => Location::Register(C),
            0x02 | 0x0a => Location::Register(D),
            0x03 | 0x0b => Location::Register(E),
            0x04 | 0x0c => Location::Register(H),
            0x05 | 0x0d => Location::Register(L),
            0x06 | 0x0e => Location::Memory(HL),
            0x07 | 0x0f => Location::Register(A),
            _ => panic!(),
        }
    };
    if let U8(value) = cpu.read_from(target, bus) {
        match opcode {
            0x00..=0x07 => {
                //RLC
                let carry = value & 0x80 != 0;
                let result = value << 1 | carry as u8;
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_hf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_cf(carry);
                cpu.write_into(target, result, bus);
            }
            0x08..=0x0F => {
                //RRC
                let carry = value & 0x01 != 0;
                let result = ((carry as u8) << 7) | (value >> 1);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_hf(false);
                cpu.registers.set_nf(false);
                cpu.registers.set_cf(carry);
                cpu.write_into(target, result, bus);
            }
            0x10..=0x17 => {
                //RL
                let result = value << 1 | cpu.registers.flg_c() as u8;
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(value & 0x80 != 0);
                cpu.write_into(target, result, bus);
            }
            0x18..=0x1F => {
                //RR
                let result = (value >> 1) | ((cpu.registers.flg_c() as u8) << 7);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(value & 0x01 != 0);
                cpu.write_into(target, result, bus);
            }
            0x30..=0x37 => {
                // SWAP
                let result = swapped_nibbles(value);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(false);
                cpu.write_into(target, result, bus);
            }
            0x40..=0x7F => {
                // BIT
                let mut bit_index = (((opcode & 0xF0) >> 4) - 4) * 2;
                if opcode & 0x08 != 0 {
                    bit_index += 1;
                }
                let check_zero = value & (1 << bit_index) == 0;
                cpu.registers.set_zf(check_zero);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(true);
                if let Location::Memory(_) = target {
                    bus.generic_cycle();
                }
            }
            0xC0..=0xFF => {
                // SET
                let mut bit_index = (((opcode & 0xF0) >> 4) - 0xC) * 2;
                if opcode & 0x08 != 0 {
                    bit_index += 1;
                }
                let result = value | (1 << bit_index);
                cpu.write_into(target, result, bus);
            }
            0x38..=0x3F => {
                let result = value >> 1;
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(value & 1 != 0);
                cpu.write_into(target, result, bus);
            }
            0x20..=0x27 => {
                // SLA
                let result = value << 1;
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(value & 0x80 != 0);
                cpu.write_into(target, result, bus);
            }
            0x28..=0x2F => {
                // SRA
                let result = value >> 1 | (value & 0x80);
                cpu.registers.set_zf(result == 0);
                cpu.registers.set_nf(false);
                cpu.registers.set_hf(false);
                cpu.registers.set_cf(value & 0x1 != 0);
                cpu.write_into(target, result, bus);
            }
            0x80..=0xBF => {
                // RES
                let mut bit_index = (((opcode & 0xF0) >> 4) - 8) * 2;
                if opcode & 0x08 != 0 {
                    bit_index += 1;
                }
                let result = value & !(1 << bit_index);
                cpu.write_into(target, result, bus);
            }
        };
    } else {
        unreachable!();
    }
}

#[inline]
pub fn swapped_nibbles(byte: u8) -> u8 {
    let [hi, lo] = [byte >> 4, byte & 0xF];
    (lo << 4) | hi
}

#[cfg(test)]
mod test {
    use crate::{
        bus::Bus,
        cpu::CPU,
        instructions::{Instr, Location},
    };

    // #[test]
    // fn ticks_cb_instr() {
    //     for instr in 0x00..=0xFF {
    //         let mut cpu = CPU::new();
    //         let mut bus = Bus::new(vec![], None);
    //         let before = bus.clock;
    //         cpu.registers.pc = 0;
    //         bus.in_bios = 1;
    //         bus.memory[0x00] = instr;
    //         bus.generic_cycle();
    //         cpu.opcode = Instr::CB.into();
    //         cpu.execute_op(&mut bus);
    //         let after = bus.clock;
    //         if let Location::Register(_) = cb_location(instr) {
    //             assert_eq!(after - before, 2, "Opcode failed: {:02x}", instr);
    //         } else {
    //             assert_eq!(after - before, 4, "Opcode failed: {:02x}", instr);
    //         }
    //     }
    // }
}
