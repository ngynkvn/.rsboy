#![feature(vec_deque_truncate_front)]
pub mod bus;
pub mod cpu;
pub mod emu;
pub mod gpu;
pub mod instructions;
pub mod location;
pub mod registers;
pub mod texture;
// pub mod tui;
pub mod constants;
pub mod debugger;
pub mod timer;
// extern crate cfg_if;
// extern crate wasm_bindgen;

mod utils;
pub mod prelude {
    pub use tap::Tap;
    pub use tracing::*;
    pub use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
}

// use cfg_if::cfg_if;
// use wasm_bindgen::prelude::*;

// cfg_if! {
//     // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
//     // allocator.
//     if #[cfg(feature = "wee_alloc")] {
//         extern crate wee_alloc;
//         #[global_allocator]
//         static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
//     }
// }

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet() {
//     alert("Hello, wasm-game-of-life!");
// }
