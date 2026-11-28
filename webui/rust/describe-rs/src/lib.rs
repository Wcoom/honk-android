//! JS bindings for `apps/lite/src/describe.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `describeGroupFilter`.
fn describe_group_filter_inner(filter: String) -> Result<String, String> {
    let out = port::describe_group_filter(filter)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeGroupFilter")]
pub fn describe_group_filter(filter: String) -> Result<String, JsError> {
    describe_group_filter_inner(filter).map_err(|e| JsError::new(&e))
}

/// `describeGroupFilters`.
fn describe_group_filters_inner(filters: Vec<String>) -> Result<String, String> {
    let out = port::describe_group_filters(filters)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeGroupFilters")]
pub fn describe_group_filters(filters: Vec<String>) -> Result<String, JsError> {
    describe_group_filters_inner(filters).map_err(|e| JsError::new(&e))
}

/// `policyKey`.
fn policy_key_inner(keyword: Option<String>) -> Result<String, String> {
    let out = port::policy_key(keyword)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "policyKey")]
pub fn policy_key(keyword: Option<String>) -> Result<String, JsError> {
    policy_key_inner(keyword).map_err(|e| JsError::new(&e))
}

/// `describeCondition`.
fn describe_condition_inner(condition: String) -> Result<String, String> {
    let condition = crate::js::from_json(&condition)?;
    let out = port::describe_condition(condition)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeCondition")]
pub fn describe_condition(condition: String) -> Result<String, JsError> {
    describe_condition_inner(condition).map_err(|e| JsError::new(&e))
}

/// `describeTarget`.
fn describe_target_inner(target: String) -> Result<String, String> {
    let out = port::describe_target(target)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeTarget")]
pub fn describe_target(target: String) -> Result<String, JsError> {
    describe_target_inner(target).map_err(|e| JsError::new(&e))
}

/// `describeRule`.
fn describe_rule_inner(conditions_opt: String, target: String) -> Result<String, String> {
    let conditions_opt = crate::js::from_json(&conditions_opt)?;
    let out = port::describe_rule(conditions_opt, target)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeRule")]
pub fn describe_rule(conditions_opt: String, target: String) -> Result<String, JsError> {
    describe_rule_inner(conditions_opt, target).map_err(|e| JsError::new(&e))
}

/// `phraseText`.
fn phrase_text_inner(value: String, t: String) -> Result<String, String> {
    let value = crate::js::from_json(&value)?;
    let t = crate::js::from_json(&t)?;
    let out = port::phrase_text(value, t)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "phraseText")]
pub fn phrase_text(value: String, t: String) -> Result<String, JsError> {
    phrase_text_inner(value, t).map_err(|e| JsError::new(&e))
}

/// `describeGroupChanges`.
fn describe_group_changes_inner(changes: String) -> Result<String, String> {
    let changes = crate::js::from_json(&changes)?;
    let out = port::describe_group_changes(changes)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeGroupChanges")]
pub fn describe_group_changes(changes: String) -> Result<String, JsError> {
    describe_group_changes_inner(changes).map_err(|e| JsError::new(&e))
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

static S_DESCRIBE_GROUP_FILTER_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("key", wire::Schema::Dyn),
    (
        "values",
        wire::Schema::OptJs(&wire::Schema::Map16(&wire::Schema::Dyn)),
    ),
]));

/// `describeGroupFilter` over the binary wire.
#[wasm_bindgen(js_name = "describeGroupFilterWire")]
pub fn describe_group_filter_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filter: String = r.string16()?;
        let out = &port::describe_group_filter(filter)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_DESCRIBE_GROUP_FILTER_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_DESCRIBE_GROUP_FILTERS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::Dyn),
    (
        "values",
        wire::Schema::OptJs(&wire::Schema::Map16(&wire::Schema::Dyn)),
    ),
]));

/// `describeGroupFilters` over the binary wire.
#[wasm_bindgen(js_name = "describeGroupFiltersWire")]
pub fn describe_group_filters_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filters: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::describe_group_filters(filters)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_DESCRIBE_GROUP_FILTERS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_POLICY_KEY_OUT: wire::Schema = wire::Schema::Dyn;

/// `policyKey` over the binary wire.
#[wasm_bindgen(js_name = "policyKeyWire")]
pub fn policy_key_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let keyword: Option<String> = r.opt_js(|r| r.string16())?;
        let out = &port::policy_key(keyword)?;
        wire::write_value(w, out, &S_POLICY_KEY_OUT)?;
        Ok(())
    })
}

static S_DESCRIBE_CONDITION_0: wire::Schema = wire::Schema::Dyn;
static S_DESCRIBE_CONDITION_OUT: wire::Schema = wire::Schema::Dyn;

/// `describeCondition` over the binary wire.
#[wasm_bindgen(js_name = "describeConditionWire")]
pub fn describe_condition_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let condition: Value = wire::read_value(r, &S_DESCRIBE_CONDITION_0)?;
        let out = &port::describe_condition(condition)?;
        wire::write_value(w, out, &S_DESCRIBE_CONDITION_OUT)?;
        Ok(())
    })
}

static S_DESCRIBE_TARGET_OUT: wire::Schema = wire::Schema::Dyn;

/// `describeTarget` over the binary wire.
#[wasm_bindgen(js_name = "describeTargetWire")]
pub fn describe_target_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let target: String = r.string16()?;
        let out = &port::describe_target(target)?;
        wire::write_value(w, out, &S_DESCRIBE_TARGET_OUT)?;
        Ok(())
    })
}

static S_DESCRIBE_RULE_0: wire::Schema =
    wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
        ("kind", wire::Schema::Str16),
        ("value", wire::Schema::Str16),
        ("negate", wire::Schema::Bool),
    ])));
static S_DESCRIBE_RULE_OUT: wire::Schema = wire::Schema::Obj(&[
    ("key", wire::Schema::Dyn),
    (
        "values",
        wire::Schema::OptJs(&wire::Schema::Map16(&wire::Schema::Dyn)),
    ),
]);

/// `describeRule` over the binary wire.
#[wasm_bindgen(js_name = "describeRuleWire")]
pub fn describe_rule_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let conditions_opt: Option<Vec<Value>> =
            r.opt_js(|r| r.list(|r| wire::read_value(r, S_DESCRIBE_RULE_0.inner().inner())))?;
        let target: String = r.string16()?;
        let out = &port::describe_rule(conditions_opt, target)?;
        wire::write_value(w, out, &S_DESCRIBE_RULE_OUT)?;
        Ok(())
    })
}

static S_PHRASE_TEXT_0: wire::Schema = wire::Schema::Dyn;
static S_PHRASE_TEXT_1: wire::Schema = wire::Schema::Dyn;

/// `phraseText` over the binary wire.
#[wasm_bindgen(js_name = "phraseTextWire")]
pub fn phrase_text_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: Value = wire::read_value(r, &S_PHRASE_TEXT_0)?;
        let t: Value = wire::read_value(r, &S_PHRASE_TEXT_1)?;
        let out = &port::phrase_text(value, t)?;
        w.string16(out);
        Ok(())
    })
}

static S_DESCRIBE_GROUP_CHANGES_0: wire::Schema = wire::Schema::Dyn;
static S_DESCRIBE_GROUP_CHANGES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::Dyn),
    (
        "values",
        wire::Schema::OptJs(&wire::Schema::Map16(&wire::Schema::Dyn)),
    ),
]));

/// `describeGroupChanges` over the binary wire.
#[wasm_bindgen(js_name = "describeGroupChangesWire")]
pub fn describe_group_changes_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let changes: Value = wire::read_value(r, &S_DESCRIBE_GROUP_CHANGES_0)?;
        let out = &port::describe_group_changes(changes)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_DESCRIBE_GROUP_CHANGES_OUT.inner())?;
        }
        Ok(())
    })
}
