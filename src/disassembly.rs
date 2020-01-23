use crate::instructions::INSTRUCTION_TABLE;
use crate::instructions::Instruction;

pub fn print(rom: &Vec<u8>, from: usize, to: usize) {
    let mut pc = from;
    println!(" PC  OP Instr");
    while pc < to {
        let Instruction(length, string) = INSTRUCTION_TABLE[rom[pc] as usize]; 
        let word = match length {
            1 => rom[pc+1] as u16,
            2 => ((rom[pc+2] as u16) << 8) + rom[pc+1] as u16,
            _ => 0,
        };
        let word_str = if length == 1 {
            format!("{:02X}", word)
        } else if length == 2 {
            format!("{:04X}", word)
        } else {
            String::from("")
        };
        println!("{:04X} {:02X} {} ", pc, rom[pc], string.replace("??", &word_str));
        pc += length + 1;
    }
}
pub fn print_all(rom: &Vec<u8>) {
    print(rom, 0, rom.len());
}