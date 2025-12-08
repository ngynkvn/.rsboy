// use bitflags::bitflags;
use std::fmt::Display;

use crate::bus::{Bus, Memory};

use crate::{
    cpu,
    instructions::Instr,
    prelude::*,
    registers::RegisterState,
};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum CPUState {
    #[default]
    Boot,
    Running(Stage),
    Interrupted(Stage),
    Halted,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    #[default]
    Fetch,
    Execute,
}
// Global emu struct.
#[derive(Debug, Clone)]
pub struct CPU {
    pub registers: RegisterState,
    pub state: CPUState,
    pub opcode: u8,
    pub op_addr: u16,
}

// bitflags! {
//     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//     pub struct Interrupts: u8 {
//         const VBLANK  = 0b0000_0001;
//         const LCDSTAT = 0b0000_0010;
//         const TIMER   = 0b0000_0100;
//         const SERIAL  = 0b0000_1000;
//         const JOYPAD = 0b0001_0000;
//     }
// }
pub mod interrupts {
    pub const VBLANK: u8 = 0b0000_0001;
    pub const LCDSTAT: u8 = 0b0000_0010;
    pub const TIMER: u8 = 0b0000_0100;
    pub const SERIAL: u8 = 0b0000_1000;
    pub const JOYPAD: u8 = 0b0001_0000;
}
use interrupts::*;

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
            state: CPUState::Running(Stage::Fetch),
        }
    }

    pub fn execute_op(&mut self, bus: &mut Bus) -> CPUState {
        // M2: execute
        let instr = Instr::from(self.opcode).run(self, bus);
        match instr {
            Instr::Halt => CPUState::Halted,
            _ => CPUState::Running(Stage::Fetch),
        }
    }

    pub fn fetch_op(&mut self, bus: &mut Bus) -> CPUState {
        // M1: fetch
        let opcode = bus.read_cycle(self.registers.pc);
        self.op_addr = self.registers.pc;
        self.opcode = opcode;
        if self.interrupt_detected(bus) {
            CPUState::Interrupted(Stage::Fetch)
        } else {
            self.registers.pc = self.registers.pc.wrapping_add(1);
            CPUState::Running(Stage::Execute)
        }
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

    pub const fn interrupt_detected(&mut self, bus: &mut Bus) -> bool {
        bus.ime != 0 && (bus.int_enabled & bus.int_flags) != 0
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) -> CPUState {
        bus.disable_interrupts();
        bus.generic_cycle();
        let fired = bus.int_enabled & bus.int_flags;
        self.push_stack(self.registers.pc, bus);

        if fired & VBLANK != 0 {
            bus.ack_interrupt(VBLANK);
            self.registers.pc = 0x40;
        } else if fired & LCDSTAT != 0 {
            bus.ack_interrupt(LCDSTAT);
            self.registers.pc = 0x48;
        } else if fired & TIMER != 0 {
            bus.ack_interrupt(TIMER);
            self.registers.pc = 0x50;
        } else if fired & SERIAL != 0 {
            bus.ack_interrupt(SERIAL);
            self.registers.pc = 0x58;
        } else if fired & JOYPAD != 0 {
            bus.ack_interrupt(JOYPAD);
            self.registers.pc = 0x60;
        }
        bus.generic_cycle();
        match self.state {
            CPUState::Interrupted(Stage::Fetch) | CPUState::Halted => self.fetch_op(bus),
            _ => unreachable!(),
        }
    }

    /// # Panics
    pub fn step(&mut self, bus: &mut Bus) {
        let _span = info_span!("Step", state = ?self.state, clock = bus.mclock()).entered();
        assert_eq!(bus.mclock(), bus.timer.mclock, "Clock mismatch: {} != {}", bus.mclock(), bus.timer.mclock);
        debug!(
            "{: ^32} [{: >4}] {: >16}",
            format!("{}", Instr::from(self.opcode)),
            cpu::test::EXPECTED_TICKS[self.opcode as usize],
            bus.mclock(),
        );
        'state: {
            self.state = match &mut self.state {
                CPUState::Boot => {
                    self.state = CPUState::Running(Stage::Fetch);
                    break 'state;
                }
                CPUState::Running(Stage::Fetch) => {
                    if bus.rom_start_signal {
                        bus.rom_start_signal = false;
                        self.print_start_values(bus);
                        self.load_start_values(bus);
                    }
                    self.fetch_op(bus)
                }
                CPUState::Running(Stage::Execute) => self.execute_op(bus),
                CPUState::Interrupted(_) => self.handle_interrupts(bus),
                CPUState::Halted => {
                    if (bus.int_enabled & bus.int_flags) != 0 {
                        if bus.ime != 0 { self.handle_interrupts(bus) } else { self.fetch_op(bus) }
                    } else {
                        bus.generic_cycle();
                        CPUState::Halted
                    }
                }
            }
        }
    }
}

impl Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#}", self.registers))
    }
}

// #[cfg(test)]
mod test;

impl CPU {
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
        bus.write(0xFF0F, 0xE1);
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

    fn print_start_values(&self, bus: &Bus) {
        error!(
            r"
            A: {:02x}, F: {:02x},
            B: {:02x}, C: {:02x},
            D: {:02x}, E: {:02x},
            H: {:02x}, L: {:02x},
            SP: {:02x}, PC: {:02x},
            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} 
            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}
            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}
            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}
            ",
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.registers.sp,
            self.registers.pc,
            bus.read(0xFF06),
            bus.read(0xFF07),
            bus.read(0xFF10),
            bus.read(0xFF11),
            bus.read(0xFF12),
            bus.read(0xFF14),
            bus.read(0xFF16),
            bus.read(0xFF17),
            bus.read(0xFF19),
            bus.read(0xFF1A),
            bus.read(0xFF1B),
            bus.read(0xFF1C),
            bus.read(0xFF1E),
            bus.read(0xFF20),
            bus.read(0xFF21),
            bus.read(0xFF22),
            bus.read(0xFF23),
            bus.read(0xFF24),
            bus.read(0xFF25),
            bus.read(0xFF26),
            bus.read(0xFF40),
            bus.read(0xFF42),
            bus.read(0xFF43),
            bus.read(0xFF45),
            bus.read(0xFF47),
            bus.read(0xFF48),
            bus.read(0xFF49),
            bus.read(0xFF4A),
            bus.read(0xFF4B),
            bus.read(0xFFFF),
        );
    }
}
