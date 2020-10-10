use std::fmt::Display;

use crate::cpu;

pub const DIV: usize = 0xFF04;
pub const TIMA: usize = 0xFF05;
pub const TMA: usize = 0xFF06;
pub const TAC: usize = 0xFF07;

pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub clock: usize,
    // TODO, move to using internal register for div
    pub internal: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            clock: 0,
            internal: 0,
        }
    }

    pub fn tick_timer_counter(&mut self, flags: &mut u8) {
        let control = self.tac;
        let clock_select = control & 0b11;

        self.clock += 1;
        let was_one = match clock_select {
            0b00 => self.internal & (1 << 9),
            0b01 => self.internal & (1 << 3),
            0b10 => self.internal & (1 << 5),
            0b11 => self.internal & (1 << 7),
            _ => unreachable!(),
        } != 0;
        self.internal = self.internal.wrapping_add(1);
        let enable = (control & 0b100) != 0;
        let now_zero = match clock_select {
            0b00 => self.internal & (1 << 9),
            0b01 => self.internal & (1 << 3),
            0b10 => self.internal & (1 << 5),
            0b11 => self.internal & (1 << 7),
            _ => unreachable!(),
        } == 0;
        if self.clock % 256 == 0 {
            self.div = self.div.wrapping_add(1);
        }
        if enable && was_one && now_zero {//(was_one && now_zero) {
            let (value, overflow) = self.tima.overflowing_add(1);
            if overflow {
                *flags |= cpu::TIMER;
                self.tima = self.tma;
            } else {
                self.tima = value;
            }
        }
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DIV:{:02x}\nTIMA:{:02x}\nTMA:{:02x}\nTAC:{:08b}\n{:016b}",
            self.div, self.tima, self.tma, self.tac, self.internal,
        ))
    }
}
