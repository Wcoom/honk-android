//! JS bindings for `apps/lite/src/ruleDraft.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `kindsFor`.
fn kinds_for_inner(kind: String) -> Result<Vec<String>, String> {
    let out = port::kinds_for(kind)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "kindsFor")]
pub fn kinds_for(kind: String) -> Result<Vec<String>, JsError> {
    kinds_for_inner(kind).map_err(|e| JsError::new(&e))
}

/// `serializeBuilder`.
fn serialize_builder_inner(rows: String) -> Result<Option<String>, String> {
    let rows = crate::js::from_json(&rows)?;
    let out = port::serialize_builder(rows)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "serializeBuilder")]
pub fn serialize_builder(rows: String) -> Result<Option<String>, JsError> {
    serialize_builder_inner(rows).map_err(|e| JsError::new(&e))
}

/// `findBlocks`.
fn find_blocks_inner(text: String, path: Vec<String>) -> Result<String, String> {
    let out = port::find_blocks(text, path)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "findBlocks")]
pub fn find_blocks(text: String, path: Vec<String>) -> Result<String, JsError> {
    find_blocks_inner(text, path).map_err(|e| JsError::new(&e))
}

/// `parseRuleList`.
fn parse_rule_list_inner(text: String, kind: String) -> Result<String, String> {
    let out = port::parse_rule_list(text, kind)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseRuleList")]
pub fn parse_rule_list(text: String, kind: String) -> Result<String, JsError> {
    parse_rule_list_inner(text, kind).map_err(|e| JsError::new(&e))
}

/// `writeRuleList`.
fn write_rule_list_inner(doc: String, rows: String) -> Result<String, String> {
    let doc = crate::js::from_json(&doc)?;
    let rows = crate::js::from_json(&rows)?;
    let out = port::write_rule_list(doc, rows)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeRuleList")]
pub fn write_rule_list(doc: String, rows: String) -> Result<String, JsError> {
    write_rule_list_inner(doc, rows).map_err(|e| JsError::new(&e))
}

/// `moveRule`.
fn move_rule_inner(text: String, kind: String, from: f64, to: f64) -> Result<String, String> {
    let out = port::move_rule(text, kind, from, to)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "moveRule")]
pub fn move_rule(text: String, kind: String, from: f64, to: f64) -> Result<String, JsError> {
    move_rule_inner(text, kind, from, to).map_err(|e| JsError::new(&e))
}

/// `ensureBlock`.
fn ensure_block_inner(text: String, path: Vec<String>) -> Result<String, String> {
    let out = port::ensure_block(text, path)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "ensureBlock")]
pub fn ensure_block(text: String, path: Vec<String>) -> Result<String, JsError> {
    ensure_block_inner(text, path).map_err(|e| JsError::new(&e))
}

/// `addDraftRule`.
fn add_draft_rule_inner(
    text: String,
    kind: String,
    condition: String,
    target: String,
    position: Option<String>,
) -> Result<String, String> {
    let out = port::add_draft_rule(text, kind, condition, target, position)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "addDraftRule")]
pub fn add_draft_rule(
    text: String,
    kind: String,
    condition: String,
    target: String,
    position: Option<String>,
) -> Result<String, JsError> {
    add_draft_rule_inner(text, kind, condition, target, position).map_err(|e| JsError::new(&e))
}

/// `editDraftRule`.
fn edit_draft_rule_inner(
    text: String,
    rule: String,
    conditions: String,
    target: String,
) -> Result<String, String> {
    let rule = crate::js::from_json(&rule)?;
    let conditions = crate::js::from_json(&conditions)?;
    let out = port::edit_draft_rule(text, rule, conditions, target)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "editDraftRule")]
pub fn edit_draft_rule(
    text: String,
    rule: String,
    conditions: String,
    target: String,
) -> Result<String, JsError> {
    edit_draft_rule_inner(text, rule, conditions, target).map_err(|e| JsError::new(&e))
}

/// `setFallback`.
fn set_fallback_inner(text: String, kind: String, target: String) -> Result<String, String> {
    let out = port::set_fallback(text, kind, target)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "setFallback")]
pub fn set_fallback(text: String, kind: String, target: String) -> Result<String, JsError> {
    set_fallback_inner(text, kind, target).map_err(|e| JsError::new(&e))
}

/// `changeCount`.
fn change_count_inner(before: String, after: String) -> Result<f64, String> {
    let out = port::change_count(before, after)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "changeCount")]
pub fn change_count(before: String, after: String) -> Result<f64, JsError> {
    change_count_inner(before, after).map_err(|e| JsError::new(&e))
}

/// `stageAuthoredRules`.
fn stage_authored_rules_inner(before: String, after: String) -> Result<String, String> {
    let out = port::stage_authored_rules(before, after)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "stageAuthoredRules")]
pub fn stage_authored_rules(before: String, after: String) -> Result<String, JsError> {
    stage_authored_rules_inner(before, after).map_err(|e| JsError::new(&e))
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

static S_SERIALIZE_BUILDER_0: wire::Schema = wire::Schema::Dyn;

/// `serializeBuilder` over the binary wire.
#[wasm_bindgen(js_name = "serializeBuilderWire")]
pub fn serialize_builder_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let rows: Value = wire::read_value(r, &S_SERIALIZE_BUILDER_0)?;
        let out = &port::serialize_builder(rows)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_FIND_BLOCKS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    ("from", wire::Schema::Num),
    ("open", wire::Schema::Num),
    ("close", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("line", wire::Schema::Num),
    ("endLine", wire::Schema::Num),
    ("depth", wire::Schema::Num),
    (
        "children",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("name", wire::Schema::Str16),
            ("from", wire::Schema::Num),
            ("open", wire::Schema::Num),
            ("close", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("line", wire::Schema::Num),
            ("endLine", wire::Schema::Num),
            ("depth", wire::Schema::Num),
            (
                "children",
                wire::Schema::List(&wire::Schema::Obj(&[
                    ("name", wire::Schema::Str16),
                    ("from", wire::Schema::Num),
                    ("open", wire::Schema::Num),
                    ("close", wire::Schema::Num),
                    ("to", wire::Schema::Num),
                    ("line", wire::Schema::Num),
                    ("endLine", wire::Schema::Num),
                    ("depth", wire::Schema::Num),
                    ("children", wire::Schema::List(&wire::Schema::Dyn)),
                ])),
            ),
        ])),
    ),
]));

/// `findBlocks` over the binary wire.
#[wasm_bindgen(js_name = "findBlocksWire")]
pub fn find_blocks_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let path: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::find_blocks(text, path)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_FIND_BLOCKS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_PARSE_RULE_LIST_OUT: wire::Schema = wire::Schema::Obj(&[
    ("text", wire::Schema::Str16),
    ("kind", wire::Schema::Str16),
    (
        "block",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("name", wire::Schema::Str16),
            ("from", wire::Schema::Num),
            ("open", wire::Schema::Num),
            ("close", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("line", wire::Schema::Num),
            ("endLine", wire::Schema::Num),
            ("depth", wire::Schema::Num),
            (
                "children",
                wire::Schema::List(&wire::Schema::Obj(&[
                    ("name", wire::Schema::Str16),
                    ("from", wire::Schema::Num),
                    ("open", wire::Schema::Num),
                    ("close", wire::Schema::Num),
                    ("to", wire::Schema::Num),
                    ("line", wire::Schema::Num),
                    ("endLine", wire::Schema::Num),
                    ("depth", wire::Schema::Num),
                    (
                        "children",
                        wire::Schema::List(&wire::Schema::Obj(&[
                            ("name", wire::Schema::Dyn),
                            ("from", wire::Schema::Dyn),
                            ("open", wire::Schema::Dyn),
                            ("close", wire::Schema::Dyn),
                            ("to", wire::Schema::Dyn),
                            ("line", wire::Schema::Dyn),
                            ("endLine", wire::Schema::Dyn),
                            ("depth", wire::Schema::Dyn),
                            ("children", wire::Schema::Dyn),
                        ])),
                    ),
                ])),
            ),
        ])),
    ),
    (
        "rows",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("raw", wire::Schema::Str16),
            ("note", wire::Schema::Str16),
            ("expression", wire::Schema::Str16),
            ("target", wire::Schema::Str16),
            ("fallback", wire::Schema::Bool),
            (
                "conditions",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("kind", wire::Schema::Str16),
                    ("value", wire::Schema::Str16),
                    ("negate", wire::Schema::Bool),
                ]))),
            ),
            (
                "anchor",
                wire::Schema::Obj(&[
                    ("from", wire::Schema::Num),
                    ("to", wire::Schema::Num),
                    ("indent", wire::Schema::Str16),
                    ("text", wire::Schema::Str16),
                    (
                        "target",
                        wire::Schema::OptJs(&wire::Schema::Obj(&[
                            ("from", wire::Schema::Num),
                            ("to", wire::Schema::Num),
                            ("foldCase", wire::Schema::OptJs(&wire::Schema::Bool)),
                        ])),
                    ),
                    (
                        "condition",
                        wire::Schema::OptJs(&wire::Schema::Obj(&[
                            ("from", wire::Schema::Num),
                            ("to", wire::Schema::Num),
                        ])),
                    ),
                    ("open", wire::Schema::OptJs(&wire::Schema::Str16)),
                    ("close", wire::Schema::OptJs(&wire::Schema::Str16)),
                ]),
            ),
        ])),
    ),
    ("ambiguous", wire::Schema::Bool),
]);

/// `parseRuleList` over the binary wire.
#[wasm_bindgen(js_name = "parseRuleListWire")]
pub fn parse_rule_list_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let kind: String = r.string16()?;
        let out = &port::parse_rule_list(text, kind)?;
        wire::write_value(w, out, &S_PARSE_RULE_LIST_OUT)?;
        Ok(())
    })
}

static S_WRITE_RULE_LIST_0: wire::Schema = wire::Schema::Dyn;
static S_WRITE_RULE_LIST_1: wire::Schema = wire::Schema::Dyn;

/// `writeRuleList` over the binary wire.
#[wasm_bindgen(js_name = "writeRuleListWire")]
pub fn write_rule_list_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let doc: Value = wire::read_value(r, &S_WRITE_RULE_LIST_0)?;
        let rows: Value = wire::read_value(r, &S_WRITE_RULE_LIST_1)?;
        let out = &port::write_rule_list(doc, rows)?;
        w.string16(out);
        Ok(())
    })
}

static S_EDIT_DRAFT_RULE_1: wire::Schema = wire::Schema::Dyn;
static S_EDIT_DRAFT_RULE_2: wire::Schema = wire::Schema::Dyn;

/// `editDraftRule` over the binary wire.
#[wasm_bindgen(js_name = "editDraftRuleWire")]
pub fn edit_draft_rule_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let rule: Value = wire::read_value(r, &S_EDIT_DRAFT_RULE_1)?;
        let conditions: Value = wire::read_value(r, &S_EDIT_DRAFT_RULE_2)?;
        let target: String = r.string16()?;
        let out = &port::edit_draft_rule(text, rule, conditions, target)?;
        w.string16(out);
        Ok(())
    })
}
