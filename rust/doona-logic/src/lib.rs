//! Pure-Rust direct access to all `rust-anything`-gated Doona & Doona Lite logic ports.
//!
//! Each `webui/rust/*-rs` crate is built independently as a standalone `wasm-bindgen` cdylib for
//! `rust-anything diff` / `gate`, where every crate exports `rustAnythingWireMemory`. Mounting the
//! exact gated `src/port.rs` modules here via `#[path]` gives `doona-wasm` and `doona-lite-wasm`
//! zero-duplication, zero-collision access to the exact gated Rust implementations.

#![forbid(unsafe_code)]
#![allow(dead_code)]

#[path = "../../../webui/rust/selectors-rs/src/js.rs"]
pub mod js;

#[path = "../../../webui/rust/selectors-rs/src/utf16.rs"]
pub mod utf16;

#[path = "../../../webui/rust/policytarget-rs/src/port.rs"]
pub mod policy_target;

#[path = "../../../webui/rust/selectors-rs/src/port.rs"]
pub mod selectors;

#[path = "../../../webui/rust/rings-rs/src/port.rs"]
pub mod rings;

#[path = "../../../webui/rust/hash-rs/src/port.rs"]
pub mod hash;

pub use dae_core::port as dae;

#[path = "../../../webui/rust/fetchfailure-rs/src/port.rs"]
pub mod fetch_failure;

#[path = "../../../webui/rust/grouphealth-rs/src/port.rs"]
pub mod group_health;

#[path = "../../../webui/rust/groupbindings-rs/src/port.rs"]
pub mod group_bindings;

#[path = "../../../webui/rust/resourcedraft-rs/src/port.rs"]
pub mod resource_draft;

#[path = "../../../webui/rust/ruledraft-rs/src/port.rs"]
pub mod rule_draft;

#[path = "../../../webui/rust/groupdraft-rs/src/port.rs"]
pub mod group_draft;

#[path = "../../../webui/rust/describe-rs/src/port.rs"]
pub mod describe;

#[path = "../../../webui/rust/rulereview-rs/src/port.rs"]
pub mod rule_review;

#[path = "../../../webui/rust/searchindex-rs/src/port.rs"]
pub mod search_index;
