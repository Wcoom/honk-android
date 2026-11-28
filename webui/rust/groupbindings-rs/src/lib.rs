//! JS bindings for `apps/lite/src/groupBindings.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;

/// `regexLiterals`.
fn regex_literals_inner(pattern: String) -> Result<String, String> {
    let out = port::regex_literals(pattern)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "regexLiterals")]
pub fn regex_literals(pattern: String) -> Result<String, JsError> {
    regex_literals_inner(pattern).map_err(|e| JsError::new(&e))
}

/// `nodeBinding`.
fn node_binding_inner(node: String, providers: String, manual: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let providers = crate::js::from_json(&providers)?;
    let out = port::node_binding(node, providers, manual)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeBinding")]
pub fn node_binding(node: String, providers: String, manual: String) -> Result<String, JsError> {
    node_binding_inner(node, providers, manual).map_err(|e| JsError::new(&e))
}

/// `subscriptionFilter`.
fn subscription_filter_inner(name: String, regex: String) -> Result<String, String> {
    let out = port::subscription_filter(name, regex)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "subscriptionFilter")]
pub fn subscription_filter(name: String, regex: String) -> Result<String, JsError> {
    subscription_filter_inner(name, regex).map_err(|e| JsError::new(&e))
}

/// `honkPolicy`.
fn honk_policy_inner(value: String) -> Result<String, String> {
    let out = port::honk_policy(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "honkPolicy")]
pub fn honk_policy(value: String) -> Result<String, JsError> {
    honk_policy_inner(value).map_err(|e| JsError::new(&e))
}

/// `regionPatterns`.
fn region_patterns_inner(iso: String) -> Result<String, String> {
    let out = port::region_patterns(iso)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "regionPatterns")]
pub fn region_patterns(iso: String) -> Result<String, JsError> {
    region_patterns_inner(iso).map_err(|e| JsError::new(&e))
}

/// `subscriptionBinding`.
fn subscription_binding_inner(
    name: String,
    regions: Vec<String>,
    words: Vec<String>,
) -> Result<String, String> {
    let out = port::subscription_binding(name, regions, words)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "subscriptionBinding")]
pub fn subscription_binding(
    name: String,
    regions: Vec<String>,
    words: Vec<String>,
) -> Result<String, JsError> {
    subscription_binding_inner(name, regions, words).map_err(|e| JsError::new(&e))
}

/// `readSubscriptionBinding`.
fn read_subscription_binding_inner(filter: String) -> Result<String, String> {
    let out = port::read_subscription_binding(filter)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readSubscriptionBinding")]
pub fn read_subscription_binding(filter: String) -> Result<String, JsError> {
    read_subscription_binding_inner(filter).map_err(|e| JsError::new(&e))
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

/// `regexLiterals` over the binary wire.
#[wasm_bindgen(js_name = "regexLiteralsWire")]
pub fn regex_literals_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let pattern: String = r.string16()?;
        let out = &port::regex_literals(pattern)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.tag(1);
                w.count(x0.len());
                for x1 in x0.iter() {
                    w.string16(x1);
                }
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_NODE_BINDING_0: wire::Schema = wire::Schema::Dyn;
static S_NODE_BINDING_1: wire::Schema = wire::Schema::Dyn;
static S_NODE_BINDING_OUT: wire::Schema = wire::Schema::Obj(&[
    ("id", wire::Schema::Str16),
    ("label", wire::Schema::Str16),
    ("description", wire::Schema::Str16),
    ("filter", wire::Schema::Str16),
    ("provider", wire::Schema::OptJs(&wire::Schema::Str16)),
]);

/// `nodeBinding` over the binary wire.
#[wasm_bindgen(js_name = "nodeBindingWire")]
pub fn node_binding_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: Value = wire::read_value(r, &S_NODE_BINDING_0)?;
        let providers: Value = wire::read_value(r, &S_NODE_BINDING_1)?;
        let manual: String = r.string16()?;
        let out = &port::node_binding(node, providers, manual)?;
        wire::write_value(w, out, &S_NODE_BINDING_OUT)?;
        Ok(())
    })
}

static S_SUBSCRIPTION_FILTER_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `subscriptionFilter` over the binary wire.
#[wasm_bindgen(js_name = "subscriptionFilterWire")]
pub fn subscription_filter_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let regex: String = r.string16()?;
        let out = &port::subscription_filter(name, regex)?;
        wire::write_value(w, out, &S_SUBSCRIPTION_FILTER_OUT)?;
        Ok(())
    })
}

static S_REGION_PATTERNS_OUT: wire::Schema = wire::Schema::Dyn;

/// `regionPatterns` over the binary wire.
#[wasm_bindgen(js_name = "regionPatternsWire")]
pub fn region_patterns_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let iso: String = r.string16()?;
        let out = &port::region_patterns(iso)?;
        wire::write_value(w, out, &S_REGION_PATTERNS_OUT)?;
        Ok(())
    })
}

static S_READ_SUBSCRIPTION_BINDING_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("source", wire::Schema::Str16),
    ("regions", wire::Schema::List(&wire::Schema::Str16)),
    ("words", wire::Schema::List(&wire::Schema::Str16)),
]));

/// `readSubscriptionBinding` over the binary wire.
#[wasm_bindgen(js_name = "readSubscriptionBindingWire")]
pub fn read_subscription_binding_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filter: String = r.string16()?;
        let out = &port::read_subscription_binding(filter)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_READ_SUBSCRIPTION_BINDING_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}
