//! JS bindings for `src/features/shared/policyTarget.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `findPolicyHub`.
fn find_policy_hub_inner(groups: String, fallback_name: String) -> Result<String, String> {
    let groups = crate::js::from_json(&groups)?;
    let out = port::find_policy_hub(groups, fallback_name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "findPolicyHub")]
pub fn find_policy_hub(groups: String, fallback_name: String) -> Result<String, JsError> {
    find_policy_hub_inner(groups, fallback_name).map_err(|e| JsError::new(&e))
}

/// `resolvePolicyState`.
fn resolve_policy_state_inner(arg0: String) -> Result<String, String> {
    let arg0 = crate::js::from_json(&arg0)?;
    let out = port::resolve_policy_state(arg0)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "resolvePolicyState")]
pub fn resolve_policy_state(arg0: String) -> Result<String, JsError> {
    resolve_policy_state_inner(arg0).map_err(|e| JsError::new(&e))
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

/// `findPolicyHub` over the binary wire.
#[wasm_bindgen(js_name = "findPolicyHubWire")]
pub fn find_policy_hub_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let groups: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let fallback_name: String = r.string16()?;
        let out = &port::find_policy_hub(groups, fallback_name)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_RESOLVE_POLICY_STATE_0: wire::Schema = wire::Schema::Dyn;
static S_RESOLVE_POLICY_STATE_OUT: wire::Schema = wire::Schema::Obj(&[
    ("hubId", wire::Schema::OptJs(&wire::Schema::Str16)),
    (
        "hubMembers",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("id", wire::Schema::Str16),
            ("name", wire::Schema::Str16),
        ])),
    ),
    (
        "options",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("id", wire::Schema::Str16),
            ("label", wire::Schema::Str16),
        ])),
    ),
    ("activeTarget", wire::Schema::Str16),
    ("usesHub", wire::Schema::Bool),
]);

/// `resolvePolicyState` over the binary wire.
#[wasm_bindgen(js_name = "resolvePolicyStateWire")]
pub fn resolve_policy_state_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let arg0: serde_json::Value = wire::read_value(r, &S_RESOLVE_POLICY_STATE_0)?;
        let out = &port::resolve_policy_state(arg0)?;
        wire::write_value(w, out, &S_RESOLVE_POLICY_STATE_OUT)?;
        Ok(())
    })
}
