//! Top-level emulator coordinator

use color_eyre::Result;
use std::{fs::File, io::Read, path::PathBuf};

use crate::{
    bus::Bus,
    cpu::CPU,
    instructions::{INSTR_DATA_LENGTHS, INSTR_TABLE, Instr},
};

/// A disassembled instruction with its address and operand data
#[derive(Clone, Debug, Default)]
pub struct InstrListing {
    pub instr: Instr,
    pub data: Option<u16>,
    pub addr: u16,
}

impl std::fmt::Display for InstrListing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            Some(d) => write!(f, "{:04X}: {} ({:04X})", self.addr, self.instr, d),
            None => write!(f, "{:04X}: {}", self.addr, self.instr),
        }
    }
}

/// Generate instruction listing from memory bytes
#[must_use]
pub fn disassemble(mem: &[u8]) -> Vec<InstrListing> {
    let mut listings = Vec::new();
    let mut offset = 0;

    while offset < mem.len() {
        let opcode = mem[offset];
        let instr = INSTR_TABLE[opcode as usize];
        let data_len = INSTR_DATA_LENGTHS[opcode as usize];

        let data = match data_len {
            0 => None,
            1 => mem.get(offset + 1).map(|&b| u16::from(b)),
            2 => mem
                .get(offset + 1..offset + 3)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])),
            _ => unreachable!("Invalid data length"),
        };

        listings.push(InstrListing {
            instr,
            data,
            addr: offset as u16,
        });

        offset += 1 + data_len;
    }

    listings
}

/// Format instruction listings as a string
#[must_use]
pub fn format_listings(listings: &[InstrListing]) -> String {
    listings
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Top-level emulator state
pub struct Emu {
    pub cpu: CPU,
    pub bus: Bus,
}

impl Emu {
    /// Create a new emulator from ROM data
    #[must_use]
    pub fn new(rom: &[u8], bootrom: Option<PathBuf>) -> Self {
        Self {
            cpu: CPU::new(),
            bus: Bus::new(rom, bootrom),
        }
    }

    /// Create a new emulator from a ROM file path
    ///
    /// # Errors
    /// Returns an error if the file cannot be read
    pub fn from_path(input: PathBuf, bootrom: Option<PathBuf>) -> Result<Self> {
        let mut file = File::open(input)?;
        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;
        Ok(Self::new(&rom, bootrom))
    }

    /// Execute one CPU step
    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    /// Execute until reaching target clock cycle
    pub fn run_until(&mut self, target_clock: usize) -> usize {
        while self.bus.mclock() < target_clock {
            self.step();
        }
        self.bus.mclock()
    }

    /// Get current memory view (bootrom or main memory)
    fn current_memory(&self) -> &[u8] {
        if self.bus.in_bios == 0 {
            &self.bus.bootrom[..]
        } else {
            &self.bus.memory[..]
        }
    }

    /// Get instruction listing around current PC
    #[must_use]
    pub fn view(&self) -> Vec<InstrListing> {
        let pc = self.cpu.op_addr;
        let mem = self.current_memory();
        let listings = disassemble(mem);

        // Find chunk containing current PC
        listings
            .chunks(10)
            .find(|chunk| chunk.iter().any(|il| il.addr == pc))
            .map(<[InstrListing]>::to_vec)
            .unwrap_or_default()
    }
}

// Keep old name as alias for backwards compatibility
#[deprecated(note = "Use `disassemble` instead")]
pub fn gen_il(mem: &[u8]) -> Vec<InstrListing> {
    disassemble(mem)
}

#[deprecated(note = "Use `format_listings` instead")]
pub fn str_il(il: &[InstrListing]) -> String {
    format_listings(il)
}
