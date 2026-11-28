//! JS bindings for `apps/lite/src/groupDraft.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `groupDraftEntry`.
fn group_draft_entry_inner(content: String, name: String) -> Result<String, String> {
    let out = port::group_draft_entry(content, name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupDraftEntry")]
pub fn group_draft_entry(content: String, name: String) -> Result<String, JsError> {
    group_draft_entry_inner(content, name).map_err(|e| JsError::new(&e))
}

/// `changeGroupText`.
fn change_group_text_inner(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
    body: String,
    duplicate: bool,
) -> Result<String, String> {
    let out = port::change_group_text(content, others, original, name, body, duplicate)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "changeGroupText")]
pub fn change_group_text(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
    body: String,
    duplicate: bool,
) -> Result<String, JsError> {
    change_group_text_inner(content, others, original, name, body, duplicate)
        .map_err(|e| JsError::new(&e))
}

/// `deleteGroup`.
fn delete_group_inner(
    content: String,
    others: Vec<String>,
    name: String,
) -> Result<String, String> {
    let out = port::delete_group(content, others, name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "deleteGroup")]
pub fn delete_group(content: String, others: Vec<String>, name: String) -> Result<String, JsError> {
    delete_group_inner(content, others, name).map_err(|e| JsError::new(&e))
}

/// `renameGroup`.
fn rename_group_inner(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
) -> Result<String, String> {
    let out = port::rename_group(content, others, original, name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "renameGroup")]
pub fn rename_group(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
) -> Result<String, JsError> {
    rename_group_inner(content, others, original, name).map_err(|e| JsError::new(&e))
}

/// `groupChanges`.
fn group_changes_inner(before: String, after: String) -> Result<String, String> {
    let out = port::group_changes(before, after)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupChanges")]
pub fn group_changes(before: String, after: String) -> Result<String, JsError> {
    group_changes_inner(before, after).map_err(|e| JsError::new(&e))
}

/// `emptyGroups`.
fn empty_groups_inner(content: String) -> Result<String, String> {
    let out = port::empty_groups(content)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "emptyGroups")]
pub fn empty_groups(content: String) -> Result<String, JsError> {
    empty_groups_inner(content).map_err(|e| JsError::new(&e))
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

static S_GROUP_DRAFT_ENTRY_OUT: wire::Schema = wire::Schema::Dyn;

/// `groupDraftEntry` over the binary wire.
#[wasm_bindgen(js_name = "groupDraftEntryWire")]
pub fn group_draft_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let content: String = r.string16()?;
        let name: String = r.string16()?;
        let out = &port::group_draft_entry(content, name)?;
        wire::write_value(w, out, &S_GROUP_DRAFT_ENTRY_OUT)?;
        Ok(())
    })
}

static S_CHANGE_GROUP_TEXT_OUT: wire::Schema = wire::Schema::Dyn;

/// `changeGroupText` over the binary wire.
#[wasm_bindgen(js_name = "changeGroupTextWire")]
pub fn change_group_text_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let content: String = r.string16()?;
        let others: Vec<String> = r.list(|r| r.string16())?;
        let original: String = r.string16()?;
        let name: String = r.string16()?;
        let body: String = r.string16()?;
        let duplicate: bool = r.bool()?;
        let out = &port::change_group_text(content, others, original, name, body, duplicate)?;
        wire::write_value(w, out, &S_CHANGE_GROUP_TEXT_OUT)?;
        Ok(())
    })
}

static S_DELETE_GROUP_OUT: wire::Schema = wire::Schema::Dyn;

/// `deleteGroup` over the binary wire.
#[wasm_bindgen(js_name = "deleteGroupWire")]
pub fn delete_group_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let content: String = r.string16()?;
        let others: Vec<String> = r.list(|r| r.string16())?;
        let name: String = r.string16()?;
        let out = &port::delete_group(content, others, name)?;
        wire::write_value(w, out, &S_DELETE_GROUP_OUT)?;
        Ok(())
    })
}

static S_RENAME_GROUP_OUT: wire::Schema = wire::Schema::Dyn;

/// `renameGroup` over the binary wire.
#[wasm_bindgen(js_name = "renameGroupWire")]
pub fn rename_group_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let content: String = r.string16()?;
        let others: Vec<String> = r.list(|r| r.string16())?;
        let original: String = r.string16()?;
        let name: String = r.string16()?;
        let out = &port::rename_group(content, others, original, name)?;
        wire::write_value(w, out, &S_RENAME_GROUP_OUT)?;
        Ok(())
    })
}

static S_GROUP_CHANGES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Dyn);

/// `groupChanges` over the binary wire.
#[wasm_bindgen(js_name = "groupChangesWire")]
pub fn group_changes_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let before: String = r.string16()?;
        let after: String = r.string16()?;
        let out = &port::group_changes(before, after)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_GROUP_CHANGES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_EMPTY_GROUPS_OUT: wire::Schema = wire::Schema::Dyn;

/// `emptyGroups` over the binary wire.
#[wasm_bindgen(js_name = "emptyGroupsWire")]
pub fn empty_groups_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let content: String = r.string16()?;
        let out = &port::empty_groups(content)?;
        wire::write_value(w, out, &S_EMPTY_GROUPS_OUT)?;
        Ok(())
    })
}
