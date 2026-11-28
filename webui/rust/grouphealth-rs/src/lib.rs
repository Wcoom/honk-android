//! JS bindings for `apps/lite/src/groupHealth.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;

/// `memberName`.
fn member_name_inner(group: String, id: String, unknown: String) -> Result<String, String> {
    let group = crate::js::from_json(&group)?;
    let id = crate::js::from_json(&id)?;
    let unknown = crate::js::from_json(&unknown)?;
    let out = port::member_name(group, id, unknown)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "memberName")]
pub fn member_name(group: String, id: String, unknown: String) -> Result<String, JsError> {
    member_name_inner(group, id, unknown).map_err(|e| JsError::new(&e))
}

/// `healthFallback`.
fn health_fallback_inner(nodes: String, groups: String) -> Result<String, String> {
    let nodes = crate::js::from_json(&nodes)?;
    let groups = crate::js::from_json(&groups)?;
    let out = port::health_fallback(nodes, groups)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "healthFallback")]
pub fn health_fallback(nodes: String, groups: String) -> Result<String, JsError> {
    health_fallback_inner(nodes, groups).map_err(|e| JsError::new(&e))
}

/// `observedMembers`.
fn observed_members_inner(group: String, fallback: String) -> Result<String, String> {
    let group = crate::js::from_json(&group)?;
    let fallback = crate::js::from_json(&fallback)?;
    let out = port::observed_members(group, fallback)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "observedMembers")]
pub fn observed_members(group: String, fallback: String) -> Result<String, JsError> {
    observed_members_inner(group, fallback).map_err(|e| JsError::new(&e))
}

/// `groupStatus`.
fn group_status_inner(group: String, fallback: String) -> Result<String, String> {
    let group = crate::js::from_json(&group)?;
    let fallback = crate::js::from_json(&fallback)?;
    let out = port::group_status(group, fallback)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupStatus")]
pub fn group_status(group: String, fallback: String) -> Result<String, JsError> {
    group_status_inner(group, fallback).map_err(|e| JsError::new(&e))
}

// ---- the binary wire ------------------------------------------------------------------------

thread_local! {
    static WIRE_IN: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    static WIRE_OUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// This module's memory, where `index.cjs` reads and writes the wire buffers.
#[wasm_bindgen(js_name = "rustAnythingWireMemory")]
pub fn rust_anything_wire_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// The address of an argument buffer of `len` bytes.
#[wasm_bindgen(js_name = "rustAnythingWireIn")]
pub fn rust_anything_wire_in(len: u32) -> u32 {
    WIRE_IN.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr() as usize as u32
    })
}

/// The address of the last result.
#[wasm_bindgen(js_name = "rustAnythingWireOut")]
pub fn rust_anything_wire_out() -> u32 {
    WIRE_OUT.with(|b| b.borrow().as_ptr() as usize as u32)
}

/// Reads `len` argument bytes, runs `f`, and keeps its result for `rustAnythingWireOut`.
fn wire_io(
    len: u32,
    f: impl FnOnce(&mut wire::Reader, &mut wire::Writer) -> Result<(), String>,
) -> Result<u32, JsError> {
    let input = WIRE_IN.with(|b| std::mem::take(&mut *b.borrow_mut()));
    let mut w = wire::Writer::default();
    let result = f(
        &mut wire::Reader::new(&input[..(len as usize).min(input.len())]),
        &mut w,
    );
    WIRE_IN.with(|b| *b.borrow_mut() = input);
    match result {
        // A result of another shape than the TS type: JS runs the call through JSON.
        Err(e) if e.starts_with(wire::MISMATCH) => return Ok(u32::MAX),
        other => other.map_err(|e| JsError::new(&e))?,
    }
    let n = w.0.len();
    WIRE_OUT.with(|b| *b.borrow_mut() = w.0);
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

static S_MEMBER_NAME_0: wire::Schema = wire::Schema::Dyn;
static S_MEMBER_NAME_1: wire::Schema = wire::Schema::Dyn;
static S_MEMBER_NAME_2: wire::Schema = wire::Schema::Dyn;
static S_MEMBER_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `memberName` over the binary wire.
#[wasm_bindgen(js_name = "memberNameWire")]
pub fn member_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let group: Value = wire::read_value(r, &S_MEMBER_NAME_0)?;
        let id: Value = wire::read_value(r, &S_MEMBER_NAME_1)?;
        let unknown: Value = wire::read_value(r, &S_MEMBER_NAME_2)?;
        let out = &port::member_name(group, id, unknown)?;
        wire::write_value(w, out, &S_MEMBER_NAME_OUT)?;
        Ok(())
    })
}

static S_HEALTH_FALLBACK_0: wire::Schema = wire::Schema::Dyn;
static S_HEALTH_FALLBACK_1: wire::Schema = wire::Schema::Dyn;
static S_HEALTH_FALLBACK_OUT: wire::Schema = wire::Schema::Dyn;

/// `healthFallback` over the binary wire.
#[wasm_bindgen(js_name = "healthFallbackWire")]
pub fn health_fallback_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let nodes: Value = wire::read_value(r, &S_HEALTH_FALLBACK_0)?;
        let groups: Value = wire::read_value(r, &S_HEALTH_FALLBACK_1)?;
        let out = &port::health_fallback(nodes, groups)?;
        wire::write_value(w, out, &S_HEALTH_FALLBACK_OUT)?;
        Ok(())
    })
}

static S_OBSERVED_MEMBERS_0: wire::Schema = wire::Schema::Dyn;
static S_OBSERVED_MEMBERS_1: wire::Schema = wire::Schema::Dyn;
static S_OBSERVED_MEMBERS_OUT: wire::Schema = wire::Schema::Dyn;

/// `observedMembers` over the binary wire.
#[wasm_bindgen(js_name = "observedMembersWire")]
pub fn observed_members_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let group: Value = wire::read_value(r, &S_OBSERVED_MEMBERS_0)?;
        let fallback: Value = wire::read_value(r, &S_OBSERVED_MEMBERS_1)?;
        let out = &port::observed_members(group, fallback)?;
        wire::write_value(w, out, &S_OBSERVED_MEMBERS_OUT)?;
        Ok(())
    })
}

static S_GROUP_STATUS_0: wire::Schema = wire::Schema::Dyn;
static S_GROUP_STATUS_1: wire::Schema = wire::Schema::Dyn;
static S_GROUP_STATUS_OUT: wire::Schema = wire::Schema::Obj(&[
    ("status", wire::Schema::Str16),
    ("latency", wire::Schema::OptJs(&wire::Schema::Num)),
]);

/// `groupStatus` over the binary wire.
#[wasm_bindgen(js_name = "groupStatusWire")]
pub fn group_status_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let group: Value = wire::read_value(r, &S_GROUP_STATUS_0)?;
        let fallback: Value = wire::read_value(r, &S_GROUP_STATUS_1)?;
        let out = &port::group_status(group, fallback)?;
        wire::write_value(w, out, &S_GROUP_STATUS_OUT)?;
        Ok(())
    })
}
