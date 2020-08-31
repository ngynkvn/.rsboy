extern crate wasm_bindgen;

use js_sys::Uint16Array;
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
pub struct WasmEmu {
    emu: Emu,
}

// #[wasm_bindgen]
// pub fn frame(w: &mut WasmEmu) -> Uint16Array {
//     let mut i = 0;
//     while i < 17476 {
//         match w.emu.cycle() {
//             Ok(c) => i += c,
//             Err(s) => panic!(s),
//         }
//     }
//     w.emu.bus.gpu.render(&mut w.emu.framebuffer);
//     unsafe {
//         let b = std::mem::transmute(w.emu.framebuffer);
//         Uint16Array::view(b)
//     }
// }

#[wasm_bindgen]
pub fn init_emu() -> WasmEmu {
    WasmEmu {
        emu: Emu::new(vec![]),
    }
}
