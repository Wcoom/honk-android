//! JS bindings for `apps/lite/src/ruleReview.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `ruleReview`.
fn rule_review_inner(
    before: String,
    after: String,
    kind: String,
    t: String,
) -> Result<String, String> {
    let t = crate::js::from_json(&t)?;
    let out = port::rule_review(before, after, kind, t)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "ruleReview")]
pub fn rule_review(
    before: String,
    after: String,
    kind: String,
    t: String,
) -> Result<String, JsError> {
    rule_review_inner(before, after, kind, t).map_err(|e| JsError::new(&e))
}

/// `dnsReview`.
fn dns_review_inner(before: String, after: String, t: String) -> Result<String, String> {
    let t = crate::js::from_json(&t)?;
    let out = port::dns_review(before, after, t)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "dnsReview")]
pub fn dns_review(before: String, after: String, t: String) -> Result<String, JsError> {
    dns_review_inner(before, after, t).map_err(|e| JsError::new(&e))
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

static S_RULE_REVIEW_3: wire::Schema = wire::Schema::Dyn;
static S_RULE_REVIEW_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Str16);

/// `ruleReview` over the binary wire.
#[wasm_bindgen(js_name = "ruleReviewWire")]
pub fn rule_review_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let before: String = r.string16()?;
        let after: String = r.string16()?;
        let kind: String = r.string16()?;
        let t: Value = wire::read_value(r, &S_RULE_REVIEW_3)?;
        let out = &port::rule_review(before, after, kind, t)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_RULE_REVIEW_OUT.inner())?;
        }
        Ok(())
    })
}

static S_DNS_REVIEW_2: wire::Schema = wire::Schema::Dyn;
static S_DNS_REVIEW_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Str16);

/// `dnsReview` over the binary wire.
#[wasm_bindgen(js_name = "dnsReviewWire")]
pub fn dns_review_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let before: String = r.string16()?;
        let after: String = r.string16()?;
        let t: Value = wire::read_value(r, &S_DNS_REVIEW_2)?;
        let out = &port::dns_review(before, after, t)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_DNS_REVIEW_OUT.inner())?;
        }
        Ok(())
    })
}
