use crate::bus::Bus;
use crate::cpu::value::Value::*;
use crate::cpu::value::Writable;
use crate::cpu::CPU;
use crate::instructions::Flag;
use crate::instructions::Location;
use crate::instructions::Register;
use crate::instructions::Register::*;
impl CPU {
    pub fn noop(&mut self, bus: &mut Bus) {}

    pub fn ld(&mut self, into: Location, from: Location, bus: &mut Bus) {
        let from_value = self.read_from(from, bus);
        self.write_into(into, from_value, bus)
    }

    pub fn inc_mem(&mut self, r: Register, bus: &mut Bus) {
        let address = self.registers.fetch_u16(r);
        let value = bus.read_cycle(address);
        let result = value.wrapping_add(1);
        bus.write_cycle(address, result);
        self.registers.set_zf(result == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(value & 0x0f == 0x0f);
    }
    pub fn inc_reg(&mut self, r: Register, bus: &mut Bus) {
        self.registers.inc(r);
        if r.is_dual_register() {
            bus.generic_cycle();
        }
    }
    pub fn dec_mem(&mut self, r: Register, bus: &mut Bus) {
        let address = self.registers.fetch_u16(r);
        let value = bus.read_cycle(address);
        let result = value.wrapping_sub(1);
        bus.write_cycle(address, result);
        self.registers.set_zf(result == 0);
        self.registers.set_nf(true);
        self.registers.set_hf(result & 0x0f == 0x0f);
    }
    pub fn dec_reg(&mut self, r: Register, bus: &mut Bus) {
        self.registers.dec(r);
        if r.is_dual_register() {
            bus.generic_cycle();
        }
    }

    pub fn ldi(&mut self, into: Location, from: Location, bus: &mut Bus) {
        self.ld(into, from, bus);
        self.registers.inc(Register::HL);
    }
    pub fn ldd(&mut self, into: Location, from: Location, bus: &mut Bus) {
        self.ld(into, from, bus);
        self.registers.dec(Register::HL);
    }
    pub fn rst(&mut self, size: u16, bus: &mut Bus) {
        bus.generic_cycle();
        self.push_stack(self.registers.pc, bus);
        self.registers.pc = size;
    }
    pub fn ldsp(&mut self, bus: &mut Bus) {
        let offset = self.next_u8(bus) as i8 as u16;
        let result = self.registers.sp.wrapping_add(offset); // todo ?
        let half_carry = (self.registers.sp & 0x0F).wrapping_add(offset & 0x0F) > 0x0F;
        let carry = (self.registers.sp & 0xFF).wrapping_add(offset & 0xFF) > 0xFF;
        self.write_into(Location::Register(HL), U16(result), bus);
        bus.generic_cycle();
        self.registers.set_zf(false);
        self.registers.set_nf(false);
        self.registers.set_hf(half_carry);
        self.registers.set_cf(carry);
    }
    pub fn stop(&mut self, bus: &mut Bus) {
        println!("stop: {:04x}", self.registers.pc - 1); // todo ?
    }
    pub fn cp(&mut self, location: Location, bus: &mut Bus) {
        let value = self.read_from(location, bus).into();
        self.registers.set_zf(self.registers.a == value);
        self.registers.set_nf(true);
        //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/self.rs#l156
        self.registers
            .set_hf((self.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0);
        self.registers.set_cf(self.registers.a < value);
    }
    pub fn add(&mut self, location: Location, bus: &mut Bus) {
        let value = self.read_from(location, bus).into();
        let (result, carry) = self.registers.a.overflowing_add(value);
        //https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/self/execute.rs#l55
        let half_carry = (self.registers.a & 0x0f)
            .checked_add(value | 0xf0)
            .is_none();
        self.registers.a = result;
        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(half_carry);
        self.registers.set_cf(carry);
    }
    pub fn sub(&mut self, location: Location, bus: &mut Bus) {
        let value = self.read_from(location, bus).into();
        let result = self.registers.a.wrapping_sub(value);
        self.registers.set_zf(result == 0);
        self.registers.set_nf(true);
        self.registers.set_hf(
            // mooneye
            (self.registers.a & 0xf).wrapping_sub(value & 0xf) & (0xf + 1) != 0,
        );
        self.registers
            .set_cf((self.registers.a as u16) < (value as u16));
        self.registers.a = result;
    }
    pub fn adc(&mut self, location: Location, bus: &mut Bus) {
        let value = self.read_from(location, bus).into();
        let carry = self.registers.flg_c() as u8;
        let result = self.registers.a.wrapping_add(value).wrapping_add(carry);
        self.registers.set_zf(result == 0);
        self.registers.set_nf(false);
        // maybe: see https://github.com/gekkio/mooneye-gb/blob/ca7ff30b52fd3de4f1527397f27a729ffd848dfa/core/src/self/execute.rs#l55
        self.registers
            .set_hf((self.registers.a & 0xf) + (value & 0xf) + carry > 0xf);
        self.registers
            .set_cf(self.registers.a as u16 + value as u16 + carry as u16 > 0xff);
        self.registers.a = result;
    }
    pub fn addhl(&mut self, location: Location, bus: &mut Bus) {
        let hl = self.registers.hl();
        if let U16(value) = self.read_from(location, bus) {
            if location.is_dual_register() {
                bus.generic_cycle();
            }
            let (result, overflow) = hl.overflowing_add(value);
            let [h, l] = result.to_be_bytes();
            self.registers.h = h;
            self.registers.l = l;
            self.registers.set_nf(false);
            self.registers
                .set_hf((hl & 0xfff) + (value & 0xfff) > 0x0fff);
            self.registers.set_cf(overflow);
        } else {
            unimplemented!()
        }
    }
    pub fn and(&mut self, location: Location, bus: &mut Bus) {
        let value: u8 = self.read_from(location, bus).into();
        self.registers.a &= value;
        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(true);
        self.registers.set_cf(false);
    }
    pub fn xor(&mut self, location: Location, bus: &mut Bus) {
        let value: u8 = self.read_from(location, bus).into();
        self.registers.a ^= value;
        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(false);
    }
    pub fn orr(&mut self, location: Location, bus: &mut Bus) {
        let value: u8 = self.read_from(location, bus).into();
        self.registers.a |= value;
        self.registers.set_zf(self.registers.a == 0);
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(false);
    }
    pub fn not(&mut self, location: Location, bus: &mut Bus) {
        let value: u8 = self.read_from(location, bus).into();
        self.registers.a = !value;
        self.registers.set_nf(true);
        self.registers.set_hf(true);
    }
    pub fn ccf(&mut self, bus: &mut Bus) {
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(!self.registers.flg_c());
    }
    pub fn scf(&mut self, bus: &mut Bus) {
        self.registers.set_nf(false);
        self.registers.set_hf(false);
        self.registers.set_cf(true);
    }
    pub fn halt(&mut self, bus: &mut Bus) {
        //todo
        self.halt = true;
    }
    pub fn jumping<F: FnOnce(&mut Self, &mut Bus)>(
        &mut self,
        jt: Option<Flag>,
        bus: &mut Bus,
        f: F,
    ) {
        if let Some(false) = jt.map(|flag| self.check_flag(flag)) {
            return;
        }
        f(self, bus);
        bus.generic_cycle();

    }
    pub fn jp(&mut self, jump_type: Option<Flag>, bus: &mut Bus) {
        let address = self.next_u16(bus);
        self.jumping(jump_type, bus, |cpu, _| cpu.registers.pc = address);
    }
    pub fn jp_hl(&mut self, bus: &mut Bus) {
        self.registers.pc = self.registers.hl();
    }
    pub fn jr(&mut self, jump_type: Option<Flag>, bus: &mut Bus) {
        let offset = self.next_u8(bus) as i8;
        let address = self.registers.pc.wrapping_add(offset as u16);
        self.jumping(jump_type, bus, |cpu, _| {
            cpu.registers.pc = address;
        });
    }
    pub fn call(&mut self, jump_type: Option<Flag>, bus: &mut Bus) {
        let address = self.next_u16(bus);
        self.jumping(jump_type, bus, |cpu, bus| {
            cpu.push_stack(cpu.registers.pc, bus);
            cpu.registers.pc = address;
        });
    }
    pub fn push(&mut self, register: Register, bus: &mut Bus) {
        let value = self.registers.fetch_u16(register);
        self.push_stack(value, bus);
        bus.generic_cycle();
    }
    pub fn pop(&mut self, register: Register, bus: &mut Bus) {
        let addr = self.pop_stack(bus);
        addr.to_register(&mut self.registers, register);
    }
    pub fn ret(&mut self, jump_type: Option<Flag>, bus: &mut Bus) {
        self.jumping(jump_type, bus, |cpu, bus| {
            cpu.registers.pc = cpu.pop_stack(bus);
        });
        if jump_type.is_some() {
            bus.generic_cycle();
        }
    }
    pub fn enableinterrupts(&mut self, bus: &mut Bus) {
        bus.enable_interrupts();
    }
    pub fn disableinterrupts(&mut self, bus: &mut Bus) {
        bus.disable_interrupts();
    }
    pub fn rra(&mut self, bus: &mut Bus) {
        let carry = self.registers.a & 1 != 0;
        self.registers.a >>= 1;
        if self.registers.flg_c() {
            self.registers.a |= 0b1000_0000;
        }
        self.registers.set_zf(false);
        self.registers.set_hf(false);
        self.registers.set_nf(false);
        self.registers.set_cf(carry);
    }
    pub fn rrca(&mut self, bus: &mut Bus) {
        let carry = self.registers.a & 1 != 0;
        self.registers.a >>= 1;
        if carry {
            self.registers.a |= 0b1000_0000;
        }
        self.registers.set_zf(false);
        self.registers.set_hf(false);
        self.registers.set_nf(false);
        self.registers.set_cf(carry);
    }
    pub fn rla(&mut self, bus: &mut Bus) {
        let overflow = self.registers.a & 0x80 != 0;
        let result = self.registers.a << 1;
        self.registers.a = result | (self.registers.flg_c() as u8);
        self.registers.set_zf(false);
        self.registers.set_hf(false);
        self.registers.set_nf(false);
        self.registers.set_cf(overflow);
    }
    pub fn rlca(&mut self, bus: &mut Bus) {
        let carry = self.registers.a & 0x80 != 0;
        let result = self.registers.a << 1 | carry as u8;
        self.registers.a = result;
        self.registers.set_zf(false);
        self.registers.set_hf(false);
        self.registers.set_nf(false);
        self.registers.set_cf(carry);
    }
    pub fn addsp(&mut self, bus: &mut Bus) {
        let offset = self.next_u8(bus) as i8 as i16 as u16;
        let sp = self.registers.sp;
        let result = self.registers.sp.wrapping_add(offset);
        bus.generic_cycle();
        bus.generic_cycle();
        let half_carry = ((sp & 0x0f) + (offset & 0x0f)) > 0x0f;
        let overflow = ((sp & 0xff) + (offset & 0xff)) > 0xff;
        self.registers.sp = result;
        self.registers.set_zf(false);
        self.registers.set_nf(false);
        self.registers.set_hf(half_carry);
        self.registers.set_cf(overflow);
    }
    pub fn reti(&mut self, bus: &mut Bus) {
        bus.enable_interrupts();
        let addr = self.pop_stack(bus);
        self.registers.pc = addr;
        bus.generic_cycle();
    }
    pub fn daa(&mut self, bus: &mut Bus) {
        self.registers.a = self.bcd_adjust(self.registers.a);
    }
    pub fn sbc(&mut self, l: Location, bus: &mut Bus) {
        let a = self.registers.a;
        let value: u8 = self.read_from(l, bus).into();
        let cy = self.registers.flg_c() as u8;
        let result = a.wrapping_sub(value).wrapping_sub(cy);
        self.registers.set_zf(result == 0);
        self.registers.set_nf(true);
        self.registers.set_hf(
            // mooneye
            (self.registers.a & 0xf)
                .wrapping_sub(value & 0xf)
                .wrapping_sub(cy)
                & (0xf + 1)
                != 0,
        );
        self.registers
            .set_cf((self.registers.a as u16) < (value as u16) + (cy as u16));
        self.registers.a = result;
    }

    pub fn handle_cb(&mut self, bus: &mut Bus) {
        let opcode = self.next_u8(bus);
        let target = CPU::cb_location(opcode);
        if let U8(value) = self.read_from(target, bus) {
            match opcode {
                0x00..=0x07 => {
                    //RLC
                    let carry = value & 0x80 != 0;
                    let result = value << 1 | carry as u8;
                    self.registers.set_zf(result == 0);
                    self.registers.set_hf(false);
                    self.registers.set_nf(false);
                    self.registers.set_cf(carry);
                    self.write_into(target, result, bus);
                }
                0x08..=0x0F => {
                    //RRC
                    let carry = value & 0x01 != 0;
                    let result = ((carry as u8) << 7) | (value >> 1);
                    self.registers.set_zf(result == 0);
                    self.registers.set_hf(false);
                    self.registers.set_nf(false);
                    self.registers.set_cf(carry);
                    self.write_into(target, result, bus);
                }
                0x10..=0x17 => {
                    //RL
                    let result = value << 1 | self.registers.flg_c() as u8;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x80 != 0);
                    self.write_into(target, result, bus);
                }
                0x18..=0x1F => {
                    //RR
                    let result = (value >> 1) | ((self.registers.flg_c() as u8) << 7);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x01 != 0);
                    self.write_into(target, result, bus);
                }
                0x30..=0x37 => {
                    // SWAP
                    let result = swapped_nibbles(value);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(false);
                    self.write_into(target, result, bus);
                }
                0x40..=0x7F => {
                    // BIT
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 4) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let check_zero = value & (1 << bit_index) == 0;
                    self.registers.set_zf(check_zero);
                    self.registers.set_nf(false);
                    self.registers.set_hf(true);
                }
                0xC0..=0xFF => {
                    // SET
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 0xC) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let result = value | (1 << bit_index);
                    self.write_into(target, result, bus);
                }
                0x38..=0x3F => {
                    let result = value >> 1;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 1 != 0);
                    self.write_into(target, result, bus);
                }
                0x20..=0x27 => {
                    // SLA
                    let result = value << 1;
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x80 != 0);
                    self.write_into(target, result, bus);
                }
                0x28..=0x2F => {
                    // SRA
                    let result = value >> 1 | (value & 0x80);
                    self.registers.set_zf(result == 0);
                    self.registers.set_nf(false);
                    self.registers.set_hf(false);
                    self.registers.set_cf(value & 0x1 != 0);
                    self.write_into(target, result, bus);
                }
                0x80..=0xBF => {
                    // RES
                    let mut bit_index = (((opcode & 0xF0) >> 4) - 8) * 2;
                    if opcode & 0x08 != 0 {
                        bit_index += 1;
                    }
                    let result = value & !(1 << bit_index);
                    self.write_into(target, result, bus);
                }
            };
        } else {
            unreachable!();
        }
    }

}

#[inline]
pub fn swapped_nibbles(byte: u8) -> u8 {
    let [hi, lo] = [byte >> 4, byte & 0xF];
    (lo << 4) | hi
}