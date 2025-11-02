pub mod value;

use std::fmt::Display;

use crate::bus::{Bus, Memory};

use crate::{
    instructions::{
        Instr,
        location::{Address, Read},
    },
    registers::RegisterState,
};
use value::Writable;

#[derive(Debug, Clone)]
pub enum CPUState {
    Running,
    Interrupted,
    Halted,
}
// Global emu struct.
#[derive(Debug, Clone)]
pub struct CPU {
    pub registers: RegisterState,
    pub state: CPUState,
    pub opcode: u8,
    pub op_addr: u16,
    pub halt: bool,
}

pub const VBLANK: u8 = 0b1;
pub const LCDSTAT: u8 = 0b10;
pub const TIMER: u8 = 0b100;
pub const SERIAL: u8 = 0b1000;
pub const JOYPAD: u8 = 0b10000;

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    #[must_use]
    pub fn new() -> Self {
        // TODO
        Self {
            registers: RegisterState::new(),
            opcode: 0,
            op_addr: 0,
            state: CPUState::Running,
            halt: false,
        }
    }

    fn execute_op(&mut self, bus: &mut Bus) {
        Instr::from(self.opcode).run(self, bus);
    }

    pub fn prefetch_op(&mut self, bus: &mut Bus, addr: u16) -> CPUState {
        let opcode = bus.read_cycle(addr);
        self.op_addr = addr;
        self.opcode = opcode;
        if self.interrupt_detected(bus) {
            return CPUState::Interrupted;
        }
        self.registers.pc = self.registers.pc.wrapping_add(1);
        CPUState::Running
    }
    pub fn next_u8(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.registers.pc;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        bus.read_cycle(addr)
    }

    pub fn next_u16(&mut self, bus: &mut Bus) -> u16 {
        // Little endianess means LSB comes first.
        let lo = self.next_u8(bus);
        let hi = self.next_u8(bus);
        u16::from_le_bytes([lo, hi])
    }

    // ld a, b
    // cpu.parse_op | read_from(location) | write_to(location)

    /// # Panics
    /// - If the address is not a valid address.
    pub fn read_from(&mut self, location: Address, bus: &mut Bus) -> Read {
        location.read(self, bus)
    }

    /// # Panics
    /// - If the address is not a valid address.
    pub fn write_into<T>(&mut self, into: Address, write_value: T, bus: &mut Bus)
    where
        T: Writable,
    {
        into.write(self, bus, write_value);
    }

    pub fn push_stack(&mut self, value: u16, bus: &mut Bus) {
        let [lo, hi] = value.to_le_bytes();
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_cycle(self.registers.sp, hi);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write_cycle(self.registers.sp, lo);
    }

    pub fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        let lo = bus.read_cycle(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let hi = bus.read_cycle(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        u16::from_le_bytes([lo, hi])
    }

    pub const fn bcd_adjust(&mut self, value: u8) -> u8 {
        let mut value = value;
        if self.registers.flg_nn() {
            if self.registers.flg_c() || value > 0x99 {
                value = value.wrapping_add(0x60);
                self.registers.set_cf(true);
            }
            if self.registers.flg_h() || (value & 0x0F) > 0x09 {
                value = value.wrapping_add(0x6);
            }
        } else {
            if self.registers.flg_c() {
                value = value.wrapping_sub(0x60);
            }
            if self.registers.flg_h() {
                value = value.wrapping_sub(0x6);
            }
        }
        self.registers.set_zf(value == 0);
        self.registers.set_hf(false);
        value
    }

    pub const fn interrupt_detected(&mut self, bus: &mut Bus) -> bool {
        bus.ime != 0 && (bus.int_enabled & bus.int_flags) != 0
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        let fired = bus.int_enabled & bus.int_flags;
        bus.generic_cycle();
        self.push_stack(self.registers.pc, bus);
        if fired & VBLANK != 0 {
            bus.ack_interrupt(VBLANK);
            self.registers.pc = 0x40;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & LCDSTAT != 0 {
            bus.ack_interrupt(LCDSTAT);
            self.registers.pc = 0x48;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & TIMER != 0 {
            bus.ack_interrupt(TIMER);
            self.registers.pc = 0x50;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & SERIAL != 0 {
            bus.ack_interrupt(SERIAL);
            self.registers.pc = 0x58;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        } else if fired & JOYPAD != 0 {
            bus.ack_interrupt(JOYPAD);
            self.registers.pc = 0x60;
            let opcode = self.next_u8(bus);
            self.opcode = opcode;
        }
    }

    // TODO hide this
    fn load_start_values(&mut self, bus: &mut Bus) {
        self.registers.a = 0x11;
        self.registers.f = 0xb0;
        self.registers.b = 0x00;
        self.registers.c = 0x13;
        self.registers.d = 0x00;
        self.registers.e = 0xd8;
        self.registers.h = 0x01;
        self.registers.l = 0x4d;
        self.registers.sp = 0xfffe;
        self.registers.pc = 0x100;
        bus.in_bios = 1;
        bus.timer.internal = 0x1ea0;
        bus.write(0xFF06, 0x00); // TMA
        bus.write(0xFF07, 0x00); // TAC
        bus.write(0xFF10, 0x80); // NR10
        bus.write(0xFF11, 0xBF); // NR11
        bus.write(0xFF12, 0xF3); // NR12
        bus.write(0xFF14, 0xBF); // NR14
        bus.write(0xFF16, 0x3F); // NR21
        bus.write(0xFF17, 0x00); // NR22
        bus.write(0xFF19, 0xBF); // NR24
        bus.write(0xFF1A, 0x7F); // NR30
        bus.write(0xFF1B, 0xFF); // NR31
        bus.write(0xFF1C, 0x9F); // NR32
        bus.write(0xFF1E, 0xBF); // NR33
        bus.write(0xFF20, 0xFF); // NR41
        bus.write(0xFF21, 0x00); // NR42
        bus.write(0xFF22, 0x00); // NR43
        bus.write(0xFF23, 0xBF); // NR30
        bus.write(0xFF24, 0x77); // NR50
        bus.write(0xFF25, 0xF3); // NR51
        bus.write(0xFF26, 0xF1); // NR52
        bus.write(0xFF40, 0x91); // LCDC
        bus.write(0xFF42, 0x00); // SCY
        bus.write(0xFF43, 0x00); // SCX
        bus.write(0xFF45, 0x00); // LYC
        bus.write(0xFF47, 0xFC); // BGP
        bus.write(0xFF48, 0xFF); // OBP0
        bus.write(0xFF49, 0xFF); // OBP1
        bus.write(0xFF4A, 0x00); // WY
        bus.write(0xFF4B, 0x00); // WX
        bus.write(0xFFFF, 0x00); // IE
        // assert_eq!(bus.memory[0xFF04], 0xAB);
    }

    /// # Panics
    pub fn step(&mut self, bus: &mut Bus) {
        if bus.rom_start_signal {
            bus.rom_start_signal = false;
            self.load_start_values(bus);
        }
        match &self.state {
            CPUState::Running => {
                // self.opcode.execute(self, bus);
                self.execute_op(bus);
                self.state = self.prefetch_op(bus, self.registers.pc);
            }
            CPUState::Interrupted => {
                self.handle_interrupts(bus);
                self.state = CPUState::Running;
            }
            CPUState::Halted => {
                panic!();
            }
        }
    }
}

impl Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#}", self.registers))
    }
}

#[cfg(test)]
mod test;
