use crate::cpu::CPU;
use crate::{bus::Bus, instructions::Register, registers::RegisterState};

#[derive(Debug, PartialEq, Clone, Copy)]
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

trait Incrementable {
    fn wrapping_add<T: Copy>(a: T, b: T) -> T;
}

impl Writable for Value {
    fn to_memory_address(self, address: u16, b: &mut Bus) {
        if let Value::U16(value) = self {
            value.to_memory_address(address, b)
        } else if let Value::U8(value) = self {
            value.to_memory_address(address, b)
        }
    }

    fn to_register(self, registers: &mut RegisterState, r: Register) {
        if let Value::U16(value) = self {
            value.to_register(registers, r);
        } else if let Value::U8(value) = self {
            value.to_register(registers, r);
        }
    }
}

pub trait Writable {
    fn to_memory_address(self, address: u16, b: &mut Bus);
    fn to_register(self, registers: &mut RegisterState, r: Register);
}
impl Writable for u8 {
    fn to_memory_address(self, address: u16, b: &mut Bus) {
        b.write_cycle(address, self);
    }

    fn to_register(self, registers: &mut RegisterState, r: Register) {
        match r {
            Register::A => {
                registers.a = self;
            }
            Register::B => {
                registers.b = self;
            }
            Register::C => {
                registers.c = self;
            }
            Register::D => {
                registers.d = self;
            }
            Register::E => {
                registers.e = self;
            }
            Register::H => {
                registers.h = self;
            }
            Register::L => {
                registers.l = self;
            }
            _ => unreachable!("{:?}", r),
        }
    }
}
impl Writable for u16 {
    fn to_memory_address(self, address: u16, b: &mut Bus) {
        let [lo, hi] = self.to_le_bytes();
        b.write_cycle(address, lo);
        b.write_cycle(address + 1, hi);
    }

    fn to_register(self, registers: &mut RegisterState, r: Register) {
        match r {
            Register::SP => {
                registers.sp = self;
            }
            Register::HL => {
                let [h, l] = self.to_be_bytes();
                registers.h = h;
                registers.l = l;
            }
            Register::DE => {
                let [d, e] = self.to_be_bytes();
                registers.d = d;
                registers.e = e;
            }
            Register::BC => {
                let [b, c] = self.to_be_bytes();
                registers.b = b;
                registers.c = c;
            }
            Register::AF => {
                let [a, f] = (self & 0b1111_1111_1111_0000).to_be_bytes();
                registers.a = a;
                registers.f = f;
            }
            _ => unreachable!(),
        }
    }
}
