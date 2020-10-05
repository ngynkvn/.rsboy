use crate::cpu;

const DIV_TIMER_HZ: usize = 16384;
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
}

impl Timer {
    pub fn new() -> Self {
        return Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            clock: 0,
        };
    }
    pub fn dump_timer_info(&self) {
        println!(
            "DIV:{:02x}\nTIMA:{:02x}\nTMA:{:02x}\nTAC:{:02x}",
            self.div, self.tima, self.tma, self.tac
        );
    }

    pub fn tick_timer_counter(&mut self, flags: &mut u8) {
        if self.clock % DIV_TIMER_HZ == 0 {
            self.div = self.div.wrapping_add(1);
        }
        let control = self.tac;
        let clock_select = control & 0b11;
        let enable = (control & 0b100) != 0;
        let clock_speed = match clock_select {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => unreachable!(),
        };
        if enable && self.clock % clock_speed == 0 {
            let (value, overflow) = self.tima.overflowing_add(1);
            if overflow {
                *flags |= cpu::TIMER;
                self.tima = self.tma
            } else {
                self.tima = value;
            }
        }
    }

}
