#![macro_use]

/**
 * MACROS: LOAD
 */
#[macro_export]
macro_rules! LD {
    ($self: ident, $dest: ident, $src: expr, $n: literal) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            $dest: $src,
            ..$self.registers
        }
    }};

    // LD (r1), r2, to MEM
    ($self: ident, LOAD_MEM, $r1: ident, $r2: ident) => {{
        $self.set_byte($self.registers.$r1(), $self.registers.$r2());
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};

    ($self: ident, LOAD_MEM_OFFSET, $r1: ident) => {{
        let offset = $self.next_u8();
        if(offset == 0x01) {
            println!("{}", $self.registers.$r1() as char);
        }
        $self.set_byte(0xFF00 + offset as u16, $self.registers.$r1());
        $self.registers = RegisterState {
            pc: $self.registers.pc + 2,
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! LD16 {
    ($self: ident, IMMEDIATE, $r1: ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 3,
            $r1: $self.next_u16(),
            ..$self.registers
        }
    }};
    ($self: ident, IMMEDIATE, $r1: ident, $r2: ident) => {{
        let value = $self.next_u16();
        $self.registers = RegisterState {
            pc: $self.registers.pc + 3,
            $r1: (value >> 8) as u8,
            $r2: (value & 0x00FF) as u8,
            ..$self.registers
        }
    }};

    ($self: ident, sp, $r1: ident, $r2: ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            sp: (($self.registers.$r1 as u16) << 8) | $self.registers.$r2 as u16,
            ..$self.registers
        }
    }};
}

/**
 * MACROS: SUBROUTINES
 */

#[macro_export]
macro_rules! CALL {
    ($self: ident) => {{
        $self.push_stack($self.registers.pc + 3);
        $self.registers = RegisterState {
            pc: $self.next_u16(),
            ..$self.registers
        };
        $self.clock += 1;
    }};

    ($self: ident, $condition: ident) => {{
        let value = $self.next_u16();
        $self.clock += 1;
        if $self.registers.$condition() {
            $self.push_stack($self.registers.pc + 3);
            $self.registers = RegisterState {
                pc: value,
                ..$self.registers
            }
        } else {
            $self.registers = $self.inc_pc(1);
        }
    }};
}

#[macro_export]
macro_rules! JP {
    ($self: ident, IMMEDIATE) => {{
        let addr = $self.next_u16();
        log::trace!("[JP] Jump from {} to {}", $self.registers.pc, addr);
        $self.registers = RegisterState {
            pc: addr,
            ..$self.registers
        }
    }};

    ($self: ident, IF, $flag: ident) => {{
        let n = $self.next_u16();
        if $self.registers.$flag() {
            log::trace!("[JP] Jump from {} to {}", $self.registers.pc, n);
            $self.clock += 1;
            $self.registers = RegisterState {
                pc: n,
                ..$self.registers
            };
        } else {
            log::trace!("[JP] Jump at {} not taken.", $self.registers.pc);
            $self.registers = RegisterState {
                pc: $self.registers.pc + 2,
                ..$self.registers
            };
        }
    }};

    ($self: ident, $r1: ident) => {{
        log::trace!("[JP] Jump from {} to {}", $self.registers.pc, $self.registers.hl());
        $self.registers = RegisterState {
            pc: $self.registers.hl(),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! JR {
    ($self: ident, IF, $flag: ident) => {{
        let n = $self.next_u8() as i8;
        if $self.registers.$flag() {
            log::trace!("[JR] Jump from {} to {}", $self.registers.pc, n);
            $self.clock += 1;
            $self.registers = RegisterState {
                pc: (($self.registers.pc as u32 as i32) + (n as i32) + (2 as i32)) as u16,
                ..$self.registers
            };
        } else {
            log::trace!("[JR] Jump at {} not taken.", $self.registers.pc);
            $self.registers = RegisterState {
                pc: $self.registers.pc + 2,
                ..$self.registers
            };
        }
    }};
}

#[macro_export]
macro_rules! INC {
    ($self: ident, NN, $r1: ident, $r2: ident) => {{
        let n = (($self.registers.$r1 as u16) << 8) | ($self.registers.$r2 as u16) + 1;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: (n >> 8) as u8,
            $r2: (n & 0x00FF) as u8,
            ..$self.registers
        }
    }};
    ($self: ident, NN, $r1: ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: $self.registers.$r1() + 1,
            ..$self.registers
        }
    }};
    ($self: ident, hl) => {{
        let n = $self.memory[$self.registers.hl()];
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_add(1);
        $self.set_byte($self.registers.hl(), n);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            f: flags(n == 0, false, half_carry, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident) => {{
        let n = $self.registers.$r1;
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_add(1);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: n,
            f: flags(n == 0, false, half_carry, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
}

/**
 * MACROS: ALU / ARITHMETIC
 */

#[macro_export]
macro_rules! AND {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let result = $self.registers.a & value;

        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            a: result,
            f: flags(result == 0, false, true, false),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! OR {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let result = $self.registers.a | value;

        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            a: result,
            f: flags(result == 0, false, false, false),
            ..$self.registers
        }
    }};
}

macro_rules! RET {
    ($self: ident, $flag: ident) => {{
        if $self.registers.$flag() {
            let ret_addr = $self.pop_u16();
            $self.registers = RegisterState {
                pc: ret_addr,
                ..$self.registers
            };
        } else {
            $self.clock += 1;
            $self.registers = RegisterState {
                pc: $self.registers.pc + 1,
                ..$self.registers
            };
        }
    }};
}

#[macro_export]
macro_rules! XOR {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let result = $self.registers.a ^ value;
        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            a: result,
            f: flags(result == 0, false, false, false),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! ADC {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let result = $self.registers.a.wrapping_add(value).wrapping_add($self.registers.flg_c() as u8);
        let h = (($self.registers.a & 0xf) + (value & 0xf)) & 0x10 != 0;
        let c = ($self.registers.a as u16 + value as u16) & 0xFF00 != 0;
        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            a: result,
            f: flags(result == 0, false, h, c),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! SBC {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let result = $self.registers.a.wrapping_sub(value).wrapping_sub($self.registers.flg_c() as u8);
        let h = ($self.registers.a & 0x0f) > 0x0f;
        let c = value > $self.registers.a;
        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            a: result,
            f: flags(result == 0, true, h, c),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! ADD {
    ($self: ident, hl, $r1: ident, $r2: ident ) => {{
        let hl = $self.registers.hl();
        let value = (($self.registers.$r1 as u16) << 8) | ($self.registers.$r2 as u16);
        let h = (hl & 0xfff) + (value & 0xfff) & 0x1000 == 0x1000;
        let c = (hl as usize) + (value as usize) & 0x10000 == 0x10000;
        let n = hl.wrapping_add(value);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            h: (n >> 8) as u8,
            l: (n & 0x00FF) as u8,
            f: flags($self.registers.flg_z(), false, h, c),
            ..$self.registers
        }
    }};

    ($self: ident, hl) => {{
        let value = $self.read_byte($self.registers.hl());
        let result = $self.registers.a.wrapping_add(value);
        let z = result == 0;
        let n = false;
        let h = ($self.registers.a & 0x0f) + (value & 0x0f) > 0x0f;
        let c = ($self.registers.a as usize) + (value as usize) > 0xFF;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            a: result,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
    ($self: ident, IMMEDIATE) => {{
        let value = $self.next_u8();
        let result = $self.registers.a.wrapping_add(value);
        let z = value == $self.registers.a;
        let n = false;
        let h = ($self.registers.a & 0x0f) + (value & 0x0f) > 0x0f;
        let c = ($self.registers.a as usize) + (value as usize) > 0xFF;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 2,
            a: result,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident) => {{
        let value = $self.registers.$r1;
        let result = $self.registers.a.wrapping_add(value);
        let z = value == $self.registers.a;
        let n = false;
        let h = ($self.registers.a & 0x0f) + (value & 0x0f) > 0x0f;
        let c = ($self.registers.a as usize) + (value as usize) > 0xFF;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            a: result,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
}
#[macro_export]
macro_rules! SUB {
    ($self: ident, IMMEDIATE) => {{
        let value = $self.next_u8();
        let z = value == $self.registers.a;
        let n = true;
        let h = ($self.registers.a & 0x0f) > 0x0f;
        let c = value > $self.registers.a;
        let value = $self.registers.a.wrapping_sub(value);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 2,
            a: value,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident) => {{
        let value = $self.registers.$r1;
        let z = value == $self.registers.a;
        let n = true;
        let h = ($self.registers.a & 0x0f) > 0x0f;
        let c = value > $self.registers.a;
        let value = $self.registers.a.wrapping_sub(value);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            a: value,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
}
#[macro_export]
macro_rules! DEC {
    ($self: ident, hl) => {{
        let n = $self.memory[$self.registers.hl()];
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_sub(1);
        $self.set_byte($self.registers.hl(), n);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            f: flags(n == 0, true, half_carry, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident) => {{
        let n = $self.registers.$r1;
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_sub(1);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: n,
            f: flags(n == 0, true, half_carry, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident, $r2: ident, $r3: ident) => {{
        let n = $self.registers.$r1();
        let half_carry = (n & 0x0f) == 0x0f;
        let n = n.wrapping_sub(1);
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r2: (n >> 8) as u8,
            $r3: (n & 0x00FF) as u8,
            f: flags(n == 0, true, half_carry, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
}

/**
 * MACROS: STACK
 */
#[macro_export]
macro_rules! PUSH {
    ($self: ident, $r1: ident) => {{
        $self.push_stack($self.registers.$r1());
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! POP {
    ($self: ident, $r1: ident, $r2: ident) => {{
        let n = $self.pop_u16();
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: (n >> 8) as u8,
            $r2: (n & 0x00FF) as u8,
            ..$self.registers
        }
    }};
}

// CB +2 PC
#[macro_export]
macro_rules! SWAP {
    ($self: ident, hl) => {{
        let addr = $self.registers.hl();
        let byte = $self.read_byte(addr);
        $self.set_byte(addr, swap_nibbles(byte));
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            ..$self.registers
        }
    }};
    ($self: ident, $r1: ident) => {{
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: swap_nibbles($self.registers.$r1),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! ROT_THRU_CARRY {
    ($self: ident, LEFT, $r1: ident) => {{
        let leftmost = $self.registers.$r1 & 0b1000_0000 != 0;
        let carry = $self.registers.flg_c() as u8;
        let n = ($self.registers.$r1 << 1) + carry;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: n,
            f: flags(n == 0, false, false, leftmost),
            ..$self.registers
        }
    }};
    ($self: ident, RIGHT, $r1: ident) => {{
        let rightmost = $self.registers.$r1 & 0b0000_0001 != 0;
        let carry = $self.registers.flg_c() as u8;
        let n = ($self.registers.$r1 >> 1) + carry;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: n,
            f: flags(n == 0, false, false, rightmost),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! SRL {
    ($self: ident, $r1: ident) => {{
        let rightmost = $self.registers.$r1 & 0b0000_0001 != 0;
        let n = ($self.registers.$r1 >> 1) & 0b1000_0000;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            $r1: n,
            f: flags(n == 0, false, false, rightmost),
            ..$self.registers
        }
    }};
}

//Stole H logic from Cinoop again :)
#[macro_export]
macro_rules! CP {
    ($self: ident, $getter: expr, $n: literal) => {{
        let value = $getter;
        let z = $self.registers.a == value;
        let n = true;
        let h = (value & 0x0f) > ($self.registers.a & 0x0f);
        let c = $self.registers.a < value;
        $self.registers = RegisterState {
            pc: $self.registers.pc + $n,
            f: flags(z, n, h, c),
            ..$self.registers
        }
    }};
}

#[macro_export]
macro_rules! TEST_BIT {
    ($self: ident, $r1: ident, $bit: expr) => {{
        let r = $self.registers.$r1 & (1 << ($bit)) == 0;
        $self.registers = RegisterState {
            pc: $self.registers.pc + 1,
            f: flags(r, false, true, $self.registers.flg_c()),
            ..$self.registers
        }
    }};
}

pub fn swap_nibbles(value: u8) -> u8 {
    ((value & 0x0F as u8) << 4) | (value >> 4) as u8
}
