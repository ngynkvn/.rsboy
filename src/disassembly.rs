use crate::instructions::Instruction;
use crate::instructions::INSTRUCTION_TABLE;
use crate::instructions::INSTR_TABLE;

pub fn print(rom: &[u8], from: usize, to: usize) {
    let mut pc = from;
    println!(" PC  OP Instr");
    while pc < to {
        let Instruction(length, string) = INSTRUCTION_TABLE[rom[pc] as usize];
        let word = match length {
            1 => rom[pc + 1] as u16,
            2 => ((rom[pc + 2] as u16) << 8) + rom[pc + 1] as u16,
            _ => 0,
        };
        let word_str = match length {
            1 => format!("{:02X}", word),
            2 => format!("{:04X}", word),
            0 => String::from(""),
            _ => panic!("Length was unexpected.."),
        };
        println!(
            "{:04X} {:02X} {} {:?}",
            pc,
            rom[pc],
            string.replace("??", &word_str),
            INSTR_TABLE[rom[pc] as usize]
        );
        pc += length + 1;
    }
}
pub fn print_all(rom: &[u8]) {
    print(rom, 0, rom.len());
}
