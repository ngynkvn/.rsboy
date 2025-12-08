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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_timer_is_zeroed() {
        let timer = Timer::new();
        assert_eq!(timer.tima, 0);
        assert_eq!(timer.tma, 0);
        assert_eq!(timer.div(), 0);
        assert_eq!(timer.internal, 0);
        assert!(!timer.is_enabled());
    }

    #[test]
    fn div_is_upper_8_bits_of_internal() {
        let mut timer = Timer::new();
        timer.internal = 0x1234;
        assert_eq!(timer.div(), 0x12);

        timer.internal = 0xABCD;
        assert_eq!(timer.div(), 0xAB);

        timer.internal = 0x00FF;
        assert_eq!(timer.div(), 0x00);
    }

    #[test]
    fn write_div_resets_internal_counter() {
        let mut timer = Timer::new();
        timer.internal = 0xFFFF;
        assert_eq!(timer.div(), 0xFF);

        // Any write to DIV resets internal to 0
        timer.write_div(0x42);
        assert_eq!(timer.internal, 0);
        assert_eq!(timer.div(), 0);
    }

    #[test]
    fn tac_enable_bit() {
        let mut timer = Timer::new();
        assert!(!timer.is_enabled());

        timer.write_tac(0b100); // Enable
        assert!(timer.is_enabled());

        timer.write_tac(0b000); // Disable
        assert!(!timer.is_enabled());
    }

    #[test]
    fn tac_clock_select() {
        let mut timer = Timer::new();

        timer.write_tac(0b00);
        assert_eq!(timer.clock_speed(), ClockSpeed::Hz4096);

        timer.write_tac(0b01);
        assert_eq!(timer.clock_speed(), ClockSpeed::Hz262144);

        timer.write_tac(0b10);
        assert_eq!(timer.clock_speed(), ClockSpeed::Hz65536);

        timer.write_tac(0b11);
        assert_eq!(timer.clock_speed(), ClockSpeed::Hz16384);
    }

    #[test]
    fn tac_read_has_upper_bits_set() {
        let mut timer = Timer::new();
        timer.write_tac(0b000);
        // Upper 5 bits always read as 1
        assert_eq!(timer.read_tac(), 0b1111_1000);

        timer.write_tac(0b111);
        assert_eq!(timer.read_tac(), 0b1111_1111);
    }

    #[test]
    fn tac_ignores_invalid_bits() {
        let mut timer = Timer::new();
        // Writing 0xFF should only keep bits 0-2
        timer.write_tac(0xFF);
        assert!(timer.is_enabled());
        assert_eq!(timer.clock_speed(), ClockSpeed::Hz16384);
    }

    #[test]
    fn tick_increments_internal_counter() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        // One M-cycle = 4 T-cycles = internal counter + 4
        timer.tick(&mut flags);
        assert_eq!(timer.internal, 4);

        timer.tick(&mut flags);
        assert_eq!(timer.internal, 8);
    }

    #[test]
    fn div_increments_every_256_t_cycles() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        // DIV increments every 256 T-cycles (64 M-cycles)
        for _ in 0..63 {
            timer.tick(&mut flags);
        }
        assert_eq!(timer.div(), 0);

        timer.tick(&mut flags);
        assert_eq!(timer.div(), 1);

        for _ in 0..64 {
            timer.tick(&mut flags);
        }
        assert_eq!(timer.div(), 2);
    }

    #[test]
    fn tima_does_not_increment_when_disabled() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        // Timer disabled, fastest clock
        timer.write_tac(0b001); // Disabled, 262144 Hz

        for _ in 0..1000 {
            timer.tick(&mut flags);
        }

        assert_eq!(timer.tima, 0);
        assert_eq!(flags, 0);
    }

    #[test]
    fn tima_increments_at_correct_rate_262144hz() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        // Enable timer at 262144 Hz (fastest, bit 3 of internal)
        // Bit 3 of internal counter falls every 16 T-cycles = 4 M-cycles
        // (bit 3 is set for values 8-15, then falls to 0 at value 16)
        timer.write_tac(0b101);

        // After 3 ticks (12 T-cycles), bit 3 has risen but not fallen yet
        for _ in 0..3 {
            timer.tick(&mut flags);
        }
        assert_eq!(timer.tima, 0);

        // After 4th tick (16 T-cycles), internal goes 12->16
        // During this, internal passes through 15->16 where bit 3 falls
        timer.tick(&mut flags);
        assert_eq!(timer.tima, 1);

        // Another 4 ticks for the next increment
        for _ in 0..4 {
            timer.tick(&mut flags);
        }
        assert_eq!(timer.tima, 2);
    }

    #[test]
    fn tima_overflow_triggers_interrupt_and_reloads_tma() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        // Set up timer close to overflow
        timer.tima = 0xFF;
        timer.tma = 0x42;
        timer.write_tac(0b101); // Enable at 262144 Hz

        // Need 4 M-cycles for falling edge at 262144 Hz
        for _ in 0..4 {
            timer.tick(&mut flags);
        }

        // Should have triggered interrupt and reloaded from TMA
        assert_eq!(timer.tima, 0x42);
        assert_ne!(flags & cpu::interrupts::TIMER, 0);
    }

    #[test]
    fn clock_speed_frequencies() {
        assert_eq!(ClockSpeed::Hz4096.frequency(), 4096);
        assert_eq!(ClockSpeed::Hz262144.frequency(), 262_144);
        assert_eq!(ClockSpeed::Hz65536.frequency(), 65536);
        assert_eq!(ClockSpeed::Hz16384.frequency(), 16384);
    }

    #[test]
    fn internal_counter_wraps() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        timer.internal = 0xFFFC;
        timer.tick(&mut flags);

        // Should wrap to 0
        assert_eq!(timer.internal, 0);
    }

    #[test]
    fn mclock_tracks_m_cycles() {
        let mut timer = Timer::new();
        let mut flags = 0u8;

        assert_eq!(timer.mclock, 0);
        timer.tick(&mut flags);
        assert_eq!(timer.mclock, 1);
        timer.tick(&mut flags);
        timer.tick(&mut flags);
        assert_eq!(timer.mclock, 3);
    }
}
