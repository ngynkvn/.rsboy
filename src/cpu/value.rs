use super::CPU;
use crate::{
    bus::{Bus, Memory},
    instructions::Register,
};

pub enum Value {
    U16(u16),
    U8(u8),
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::U8(v)
    }
}
impl Into<u8> for Value {
    fn into(self) -> u8 {
        if let Value::U8(value) = self {
            value
        } else {
            panic!("Tried to convert U16 into U8.")
        }
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::U16(v)
    }
}
impl Into<u16> for Value {
    fn into(self) -> u16 {
        if let Value::U16(value) = self {
            value
        } else {
            panic!("Tried to convert U16 into U8.")
        }
    }
}

impl Writable for Value {
    fn to_memory_address(self, cpu: &mut CPU, address: u16, b: &mut Bus) {
        if let Value::U16(value) = self {
            value.to_memory_address(cpu, address, b)
        } else if let Value::U8(value) = self {
            value.to_memory_address(cpu, address, b)
        }
    }

    fn to_register(self, cpu: &mut CPU, r: Register) {
        if let Value::U16(value) = self {
            value.to_register(cpu, r);
        } else if let Value::U8(value) = self {
            value.to_register(cpu, r);
        }
    }
}

pub trait Writable {
    fn to_memory_address(self, cpu: &mut CPU, address: u16, b: &mut Bus);
    fn to_register(self, cpu: &mut CPU, r: Register);
}
impl Writable for u8 {
    fn to_memory_address(self, cpu: &mut CPU, address: u16, b: &mut Bus) {
        b.write(address, self);
        cpu.tick(b);
    }

    fn to_register(self, cpu: &mut CPU, r: Register) {
        match r {
            Register::A => {
                cpu.registers.a = self;
            }
            Register::B => {
                cpu.registers.b = self;
            }
            Register::C => {
                cpu.registers.c = self;
            }
            Register::D => {
                cpu.registers.d = self;
            }
            Register::E => {
                cpu.registers.e = self;
            }
            Register::H => {
                cpu.registers.h = self;
            }
            Register::L => {
                cpu.registers.l = self;
            }
            _ => unreachable!(),
        }
    }
}
impl Writable for u16 {
    fn to_memory_address(self, cpu: &mut CPU, address: u16, b: &mut Bus) {
        let [lo, hi] = self.to_le_bytes();
        b.write(address, lo);
        cpu.tick(b);
        b.write(address + 1, hi);
        cpu.tick(b);
    }

    fn to_register(self, cpu: &mut CPU, r: Register) {
        match r {
            Register::SP => {
                cpu.registers.sp = self;
            }
            Register::HL => {
                let [h, l] = self.to_be_bytes();
                cpu.registers.h = h;
                cpu.registers.l = l;
            }
            Register::DE => {
                let [d, e] = self.to_be_bytes();
                cpu.registers.d = d;
                cpu.registers.e = e;
            }
            Register::BC => {
                let [b, c] = self.to_be_bytes();
                cpu.registers.b = b;
                cpu.registers.c = c;
            }
            Register::AF => {
                let [a, f] = (self & 0b1111_1111_1111_0000).to_be_bytes();
                cpu.registers.a = a;
                cpu.registers.f = f;
            }
            _ => unreachable!(),
        }
    }
}
