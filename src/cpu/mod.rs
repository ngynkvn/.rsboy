use crate::instructions::*;
use crate::memory::Memory;
use crate::registers::RegisterState;
use crate::emu::Emu;
use std::convert::TryInto;

const HISTORY_SIZE: usize = 10;

trait Controller {
    fn cycle(&mut self, emu: Emu) -> usize;
}

pub struct CPU {
    registers: RegisterState,
    pub clock: usize,
}

impl CPU {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: RegisterState::new(),
            clock: 0,
        }
    }
    fn curr_u8(&mut self, memory: Memory) -> u8 {
        self.clock += 1;
        memory[self.registers.pc]
    }
    fn next_u8(&mut self, memory: Memory) -> u8 {
        self.clock += 1;
        memory[(self.registers.pc + 1)]
    }
    fn next_u16(&mut self, memory: Memory) -> u16 {
        // Little endianess means LSB comes first.
        self.clock += 1;
        (memory[self.registers.pc + 2] as u16) << 8 | memory[self.registers.pc + 1] as u16
    }
    fn read_byte(&mut self, address: u16, memory: Memory) -> u8 {
        self.clock += 1;
        memory[address]
    }
    fn read_io(&mut self, offset: u8, memory: Memory) -> u8 {
        self.read_byte(0xFF00 + offset as u16, memory)
    }
    fn set_byte(&mut self, address: u16, value: u8, memory: Memory) {
        self.clock += 1;
        memory[address] = value;
    }

    fn read_location(&mut self, location: &Location, memory: Memory) -> u16 {
        match location {
            Location::Immediate(1) => self.next_u8(memory).into(),
            Location::Immediate(2) => self.next_u16(memory),
            Location::Immediate(_) => panic!(),
            Location::Register(r) => self.registers.fetch(r),
            Location::Memory(r) => self.read_byte(self.registers.fetch(r), memory).into(),
            Location::MemOffsetImm => {
                let next = self.next_u8(memory);
                self.read_io(next, memory).into()
            }
            Location::MemOffsetRegister(r) => self
                .read_io(
                    self.registers
                        .fetch(r)
                        .try_into()
                        .expect("Offset was too large."),
                        memory
                )
                .into(),
        }
    }

    fn load(&mut self, into: &Location, from: &Location, memory: Memory) {
        let from_value = self.read_location(from, memory);
        match into {
            Location::Immediate(_) => panic!("Tried to write into ROM?"),
            Location::Register(r) => {
                self.registers = self.registers.put(from_value, r);
            },
            Location::Memory(r) => {
                match self.registers.get_dual_reg(r) {
                    Some(address) => self.set_byte(address, from_value.try_into().unwrap(), memory),
                    None => panic!("I tried to access a u8 as a memory address.")
                }
            },
            Location::MemOffsetImm => {
                let next = self.next_u8(memory);
                self.set_byte((0xFF00 + next).into(), from_value
                                                        .try_into()
                                                        .unwrap(), 
                                                        memory);
            }
            Location::MemOffsetRegister(r) => {
                let offset = self.registers.fetch_u8(r);
                self.set_byte((0xFF00 + offset).into(), from_value.try_into().unwrap(), memory);
            }
        };
    }

    fn read_instruction(&mut self, emu: Emu) -> Result<(), String> {
        let curr_byte = self.curr_u8(emu.memory);
        println!("{}", curr_byte);
        match &INSTR_TABLE[curr_byte as usize] {
            Instr::LD(into, from) => self.load(into, from, emu.memory),
            Instr::NOOP => {}
            Instr::UNIMPLEMENTED => panic!("Unimplemented"),
            x => panic!(format!("Not done {:?}", x)),
        }
        Ok(())
    }
}

impl Controller for CPU {
    fn cycle(&mut self, emu: Emu) -> usize {
        self.read_instruction(emu);
        0
    }
}