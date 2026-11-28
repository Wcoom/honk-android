//! Pure-Rust rlib exposing all 141 `src/dae/` functions from `webui/rust/dae-rs/src/port.rs`
//! without any `#[wasm_bindgen]` wire exports (so downstream WASM crates link cleanly with zero symbol collisions).

#![forbid(unsafe_code)]
#![allow(dead_code)]

#[path = "../../../webui/rust/dae-rs/src/js.rs"]
pub mod js;

#[path = "../../../webui/rust/dae-rs/src/utf16.rs"]
pub mod utf16;

pub mod port;
