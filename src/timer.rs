//! Game Boy Timer subsystem
//!
//! The timer uses a falling edge detector on an internal 16-bit counter.
//! DIV is the upper 8 bits of this counter. TIMA increments when the
//! selected bit of the internal counter transitions from 1 to 0.

use std::fmt::Display;

use tracing::trace;

use crate::cpu;

/// I/O register addresses
pub mod addr {
    pub const DIV: u16 = 0xFF04;
    pub const TIMA: u16 = 0xFF05;
    pub const TMA: u16 = 0xFF06;
    pub const TAC: u16 = 0xFF07;
}

/// TAC register bit definitions
mod tac {
    /// Timer enable bit (bit 2)
    pub const ENABLE: u8 = 0b0000_0100;
    /// Clock select mask (bits 0-1)
    pub const CLOCK_SELECT_MASK: u8 = 0b0000_0011;
    /// Valid bits mask (bits 0-2)
    pub const VALID_MASK: u8 = 0b0000_0111;
    /// Upper bits always read as 1
    pub const UPPER_BITS: u8 = 0b1111_1000;
}

/// Internal counter bit positions for falling edge detection
/// These correspond to the TAC clock select values
mod clock_bit {
    pub const HZ_4096: u16 = 1 << 9;   // Clock select 0b00
    pub const HZ_262144: u16 = 1 << 3; // Clock select 0b01
    pub const HZ_65536: u16 = 1 << 5;  // Clock select 0b10
    pub const HZ_16384: u16 = 1 << 7;  // Clock select 0b11
}

/// Timer clock speed selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClockSpeed {
    #[default]
    Hz4096,
    Hz262144,
    Hz65536,
    Hz16384,
}

impl ClockSpeed {
    /// Get the actual frequency in Hz
    pub const fn frequency(self) -> u32 {
        match self {
            Self::Hz4096 => 4096,
            Self::Hz262144 => 262_144,
            Self::Hz65536 => 65536,
            Self::Hz16384 => 16384,
        }
    }

    /// Get the bit mask for falling edge detection
    const fn bit_mask(self) -> u16 {
        match self {
            Self::Hz4096 => clock_bit::HZ_4096,
            Self::Hz262144 => clock_bit::HZ_262144,
            Self::Hz65536 => clock_bit::HZ_65536,
            Self::Hz16384 => clock_bit::HZ_16384,
        }
    }

    /// Create from TAC clock select bits
    const fn from_tac(tac: u8) -> Self {
        match tac & tac::CLOCK_SELECT_MASK {
            0b00 => Self::Hz4096,
            0b01 => Self::Hz262144,
            0b10 => Self::Hz65536,
            0b11 => Self::Hz16384,
            _ => unreachable!(),
        }
    }
}

/// Game Boy Timer
#[derive(Debug, Default)]
pub struct Timer {
    /// TIMA - Timer counter (0xFF05)
    pub tima: u8,
    /// TMA - Timer modulo (0xFF06)
    pub tma: u8,
    /// TAC - Timer control (0xFF07)
    tac: u8,
    /// M-cycle counter (for synchronization)
    pub mclock: usize,
    /// Internal 16-bit counter (DIV is upper 8 bits)
    pub internal: u16,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0,
            mclock: 0,
            internal: 0,
        }
    }

    /// Check if timer is enabled (TAC bit 2)
    pub const fn is_enabled(&self) -> bool {
        self.tac & tac::ENABLE != 0
    }

    /// Get the current clock speed selection
    pub const fn clock_speed(&self) -> ClockSpeed {
        ClockSpeed::from_tac(self.tac)
    }

    /// Read TAC register
    pub fn read_tac(&self) -> u8 {
        let value = tac::UPPER_BITS | (self.tac & tac::VALID_MASK);
        trace!("Reading TAC: {value:03b}");
        value
    }

    /// Write TAC register
    pub fn write_tac(&mut self, value: u8) {
        trace!("Writing TAC: {:03b}", value & tac::VALID_MASK);
        self.tac = value & tac::VALID_MASK;
    }

    /// Read DIV register (upper 8 bits of internal counter)
    pub fn read_div(&self) -> u8 {
        let value = self.div();
        trace!("Reading DIV: {value:02x}");
        value
    }

    /// Write to DIV register (any write resets internal counter to 0)
    pub fn write_div(&mut self, _value: u8) {
        trace!("Writing DIV: resetting internal counter");
        self.internal = 0;
    }

    /// Get DIV value (upper 8 bits of internal counter)
    #[allow(clippy::cast_possible_truncation)]
    pub const fn div(&self) -> u8 {
        (self.internal >> 8) as u8
    }

    /// Update internal counter with falling edge detection
    ///
    /// This implements the Game Boy's timer behavior where TIMA increments
    /// when the selected bit of the internal counter falls from 1 to 0.
    fn update_internal(&mut self, flags: &mut u8, new_value: u16) {
        let speed = self.clock_speed();
        let mask = speed.bit_mask();

        let was_high = self.internal & mask != 0;
        self.internal = new_value;
        let now_low = self.internal & mask == 0;

        // Falling edge: was 1, now 0
        if self.is_enabled() && was_high && now_low {
            let (value, overflow) = self.tima.overflowing_add(1);
            if overflow {
                *flags |= cpu::interrupts::TIMER;
                self.tima = self.tma;
            } else {
                self.tima = value;
            }
        }
    }

    /// Advance timer by one M-cycle (4 T-cycles)
    pub fn tick(&mut self, flags: &mut u8) {
        self.mclock += 1;
        // Internal counter runs at 4MHz (4 T-cycles per M-cycle)
        for _ in 0..4 {
            self.update_internal(flags, self.internal.wrapping_add(1));
        }
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DIV:{:02x} TIMA:{:02x} TMA:{:02x} TAC:{:?} ({}) internal:{:016b}",
            self.div(),
            self.tima,
            self.tma,
            self.clock_speed(),
            if self.is_enabled() { "ON" } else { "OFF" },
            self.internal,
        )
    }
}
