#![allow(clippy::used_underscore_binding)]
use std::fmt::Display;

use tracing::trace;

use crate::cpu;

pub const DIV: usize = 0xFF04;
pub const TIMA: usize = 0xFF05;
pub const TMA: usize = 0xFF06;
pub const TAC: usize = 0xFF07;

#[derive(Default, Debug)]
pub struct Timer {
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub mclock: usize,
    pub internal: u16,
    pub _debug: usize,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0,
            mclock: 0,
            internal: 0,
            _debug: 0,
        }
    }

    pub fn read_tac(&self) -> u8 {
        trace!("Reading TAC: {:03b}", self.tac & 0b111);
        self.tac & 0b111
    }

    pub fn write_tac(&mut self, value: u8) {
        trace!("Writing TAC: {:03b}", value & 0b111);
        self.tac = 0b1111_1000 | (value & 0b111);
    }

    pub fn read_div(&self) -> u8 {
        trace!("Reading DIV: {:02x}", self.div());
        self.div()
    }

    pub fn write_div(&mut self, value: u8) {
        trace!("Writing DIV: {:02x}", self.div());
        self.internal = u16::from(value);
    }

    pub const fn div(&self) -> u8 {
        (self.internal >> 8) as u8
    }

    pub const fn speed(tac: u8) -> Speed {
        match tac & 0b11 {
            0b00 => Speed::Hz4096,
            0b01 => Speed::Hz262144,
            0b10 => Speed::Hz65536,
            0b11 => Speed::Hz16384,
            _ => unreachable!(),
        }
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
                *flags |= cpu::interrupts::TIMER;
                self.tima = self.tma;
            } else {
                self.tima = value;
            }
        }
    }

    pub fn tick_timer_counter(&mut self, flags: &mut u8) {
        self.mclock += 1;
        for _ in 0..4 {
            self.update_internal(flags, self.internal.wrapping_add(1));
        }
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DIV:{:02x}\nTIMA:{:02x}\nTMA:{:02x}\nTAC:{:?}\n{:016b}",
            self.div(),
            self.tima,
            self.tma,
            Self::speed(self.tac),
            self.internal,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Speed {
    Hz4096,
    Hz262144,
    Hz65536,
    Hz16384,
}
