extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
pub mod bus;
pub mod cpu;
pub mod disassembly;
pub mod emu;
pub mod gpu;
pub mod instructions;
pub mod registers;
pub mod texture;
use crate::emu::Emu;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    unsafe {
        alert(&format!("Hello, {}!", name));
    }
}

pub fn tetris(rom: Vec<u8>) -> Emu {
    Emu::new(rom)
}
