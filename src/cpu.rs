use std::fmt::Display;

use bitflags::bitflags;

use crate::bus::{Bus, Memory};
use crate::{instructions::Instr, prelude::*, registers::RegisterState};

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

bitflags! {
    /// Interrupt flags for the Game Boy's interrupt system.
    /// Each bit corresponds to a specific interrupt source.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Interrupt: u8 {
        /// V-Blank interrupt (triggered at start of V-Blank period)
        const VBLANK  = 0b0000_0001;
        /// LCD STAT interrupt (triggered by LCD status conditions)
        const LCDSTAT = 0b0000_0010;
        /// Timer interrupt (triggered when TIMA overflows)
        const TIMER   = 0b0000_0100;
        /// Serial interrupt (triggered after serial transfer)
        const SERIAL  = 0b0000_1000;
        /// Joypad interrupt (triggered on button press)
        const JOYPAD  = 0b0001_0000;
    }
}

/// Interrupt handler entry: (interrupt flag, handler address)
const INTERRUPT_HANDLERS: [(Interrupt, u16); 5] = [
    (Interrupt::VBLANK, 0x40),
    (Interrupt::LCDSTAT, 0x48),
    (Interrupt::TIMER, 0x50),
    (Interrupt::SERIAL, 0x58),
    (Interrupt::JOYPAD, 0x60),
];

#[derive(Debug, Clone)]
pub struct CPU {
    pub registers: RegisterState,
    pub state: CPUState,
    /// Current opcode being executed (for debugging)
    opcode: u8,
    /// Address of current opcode (for debugging)
    op_addr: u16,
}

impl CPU {
    /// Returns the current opcode being executed
    #[inline]
    pub const fn opcode(&self) -> u8 {
        self.opcode
    }

    /// Returns the address of the current opcode
    #[inline]
    pub const fn op_addr(&self) -> u16 {
        self.op_addr
    }
}

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

    /// Check if any enabled interrupts are pending
    pub fn interrupt_detected(&self, bus: &Bus) -> bool {
        bus.ime && bus.pending_interrupts().intersects(bus.enabled_interrupts())
    }

    pub fn handle_interrupts(&mut self, bus: &mut Bus) -> CPUState {
        bus.disable_interrupts();
        bus.generic_cycle();

        let pending = bus.pending_interrupts();
        let enabled = bus.enabled_interrupts();
        let fired = pending & enabled;

        self.push_stack(self.registers.pc, bus);

        // Handle highest priority interrupt using the handler table
        for (interrupt, handler_addr) in INTERRUPT_HANDLERS {
            if fired.contains(interrupt) {
                bus.ack_interrupt(interrupt);
                self.registers.pc = handler_addr;
                break;
            }
        }

        bus.generic_cycle();
        match self.state {
            CPUState::Interrupted(Stage::Fetch) | CPUState::Halted => self.fetch_op(bus),
            _ => unreachable!("handle_interrupts called from invalid state"),
        }
    }

    /// # Panics
    pub fn step(&mut self, bus: &mut Bus) {
        let _span = info_span!("Step", state = ?self.state, clock = bus.mclock()).entered();
        assert_eq!(bus.mclock(), bus.timer.mclock, "Clock mismatch: {} != {}", bus.mclock(), bus.timer.mclock);
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
                    let has_pending = bus.pending_interrupts().intersects(bus.enabled_interrupts());
                    if has_pending {
                        if bus.ime { self.handle_interrupts(bus) } else { self.fetch_op(bus) }
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
        bus.in_bios = true;
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
