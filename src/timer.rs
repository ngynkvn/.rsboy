use std::fmt::Display;

use crate::cpu;

pub const DIV: usize = 0xFF04;
pub const TIMA: usize = 0xFF05;
pub const TMA: usize = 0xFF06;
pub const TAC: usize = 0xFF07;

#[derive(Default)]
pub struct Timer {
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub clock: usize,
    pub internal: u16,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0,
            clock: 0,
            internal: 0,
        }
    }

    pub const fn div(&self) -> u8 {
        (self.internal >> 8) as u8
    }

    pub fn update_internal(&mut self, flags: &mut u8, new: u16) {
        //Falling edge detector
        let control = self.tac;
        let clock_select = control & 0b11;

        let mask = match clock_select {
            0b00 => 1 << 9,
            0b01 => 1 << 3,
            0b10 => 1 << 5,
            0b11 => 1 << 7,
            _ => unreachable!(),
        };

        let was_one = self.internal & mask != 0;
        self.internal = new;
        let now_zero = self.internal & mask == 0;
        let enable = (control & 0b100) != 0;
        if enable && was_one && now_zero {
            let (value, overflow) = self.tima.overflowing_add(1);
            if overflow {
                *flags |= cpu::TIMER;
                self.tima = self.tma;
            } else {
                self.tima = value;
            }
        }
    }

    pub fn tick_timer_counter(&mut self, flags: &mut u8) {
        self.clock += 1;
        self.update_internal(flags, self.internal.wrapping_add(1));
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DIV:{:02x}\nTIMA:{:02x}\nTMA:{:02x}\nTAC:{:08b}\n{:016b}",
            self.div(),
            self.tima,
            self.tma,
            self.tac,
            self.internal,
        ))
    }
}
