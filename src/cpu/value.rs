use crate::{bus::Bus, instructions::Register, location::Read, registers::RegisterState};

// #[repr(u8)]
// #[derive(Clone, Copy)]
// enum Tag {W, B}

// #[derive(Clone, Copy)]
// union U {
//     word: u16,
//     byte: u8,
// }

// pub struct Value {
//     u: U,
//     tag: Tag
// }

pub struct Val<T>(pub T);
pub trait Writable {
    fn to_memory_address(self, address: u16, b: &mut Bus);
    fn to_register(self, registers: &mut RegisterState, r: Register);
}

impl Writable for Read {
    fn to_memory_address(self, address: u16, b: &mut Bus) {
        match self {
            Self::Byte(x) => x.to_memory_address(address, b),
            Self::Word(x) => x.to_memory_address(address, b),
        }
    }
    fn to_register(self, registers: &mut RegisterState, r: Register) {
        match self {
            Self::Byte(x) => x.to_register(registers, r),
            Self::Word(x) => x.to_register(registers, r),
        }
    }
}

impl Writable for Val<u8> {
    fn to_memory_address(self, address: u16, b: &mut Bus) {
        self.0.to_memory_address(address, b);
    }

    fn to_register(self, registers: &mut RegisterState, r: Register) {
        self.0.to_register(registers, r);
    }
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
