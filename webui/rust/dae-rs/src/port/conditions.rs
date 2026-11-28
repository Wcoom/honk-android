//! Port of `src/dae/conditions.ts`, `src/dae/groupConditions.ts`,
//! `src/dae/groupFilters.ts`, and `src/dae/rustRegex.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::groups::{rule_condition_str, split_top_vec};
use crate::port::js_str::{is_js_ws, is_truthy, js_slice, js_trim, js_trim_start, obj_defined};
use crate::port::text::{
    TokenKind, is_fragment_bool, is_quotable_bool, quote_str, scan_config_raw, unquote_str,
};
use crate::port::vocab::is_builtin_outbound_str;
use crate::utf16;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

// --- conditions.ts ---

fn has_nonempty_token(value: &str) -> bool {
    value
        .split(|c: char| c == ',' || is_js_ws(c))
        .any(|s| !s.is_empty())
}

pub fn serialize_conditions(rows: Value) -> Result<Option<String>, String> {
    if rows.is_null() || js::is_undefined(&rows) {
        return Err("TypeError: Cannot read properties of null (reading 'length')".to_string());
    }
    let Some(arr) = rows.as_array() else {
        if let Some(s) = rows.as_str() {
            if s.is_empty() {
                return Ok(None);
            }
            return Err("TypeError: rows.map is not a function".to_string());
        }
        return Ok(None);
    };
    if arr.is_empty() {
        return Ok(None);
    }
    let mut calls: Vec<(bool, Option<String>)> = Vec::with_capacity(arr.len());
    for row in arr {
        let Some(o) = row.as_object() else {
            return Err("TypeError: Cannot read properties of null (reading 'value')".to_string());
        };
        let Some(val_v) = obj_defined(o, "value") else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'split')".to_string(),
            );
        };
        let Some(val_s) = val_v.as_str() else {
            return Err("TypeError: row.value.split is not a function".to_string());
        };
        let kind_s = obj_defined(o, "kind").and_then(Value::as_str).unwrap_or("");
        let negate = obj_defined(o, "negate").is_some_and(is_truthy);
        let call = if has_nonempty_token(val_s) {
            rule_condition_str(kind_s, val_s)?
        } else {
            None
        };
        calls.push((negate, call));
    }
    if calls.iter().any(|(_, c)| c.is_none()) {
        return Ok(None);
    }
    let joined = calls
        .into_iter()
        .map(|(neg, c)| {
            let call = c.unwrap_or_default();
            if neg { format!("!{call}") } else { call }
        })
        .collect::<Vec<_>>()
        .join(" && ");
    Ok(Some(joined))
}

/// Match `/\s*(!?)\s*(\w+)\s*\(([^()]*)\)\s*/y` starting at UTF-16 offset `offset`.
fn match_condition_call(units: &[u16], offset: usize) -> Option<(bool, String, String, usize)> {
    let mut i = offset;
    let len = units.len();
    while i < len && crate::port::js_str::is_js_ws_u16(units[i]) {
        i += 1;
    }
    let negate = if i < len && units[i] == b'!' as u16 {
        i += 1;
        true
    } else {
        false
    };
    while i < len && crate::port::js_str::is_js_ws_u16(units[i]) {
        i += 1;
    }
    let name_start = i;
    while i < len && u8::try_from(units[i]).is_ok_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
        i += 1;
    }
    if i == name_start {
        return None;
    }
    let name = js_slice(units, name_start, i);
    while i < len && crate::port::js_str::is_js_ws_u16(units[i]) {
        i += 1;
    }
    if i >= len || units[i] != b'(' as u16 {
        return None;
    }
    i += 1;
    let body_start = i;
    while i < len && units[i] != b'(' as u16 && units[i] != b')' as u16 {
        i += 1;
    }
    if i >= len || units[i] != b')' as u16 {
        return None;
    }
    let body = js_slice(units, body_start, i);
    i += 1;
    while i < len && crate::port::js_str::is_js_ws_u16(units[i]) {
        i += 1;
    }
    Some((negate, name, body, i))
}

pub fn parse_conditions(text: String, kinds: Vec<String>) -> Result<Option<Vec<Value>>, String> {
    let units = utf16::units(&text);
    let mut rows: Vec<Value> = Vec::new();
    let mut offset = 0usize;
    while offset < units.len() {
        let Some((negate, name, body, next_offset)) = match_condition_call(&units, offset) else {
            return Ok(None);
        };
        let tokens = scan_config_raw(&body).tokens;
        let body_u = utf16::units(&body);
        let raw = |n: usize| -> String { js_slice(&body_u, tokens[n].from, tokens[n].to) };
        let mut args: Vec<(String, String)> = Vec::new();
        let mut i = 0usize;
        while i < tokens.len() {
            let mut prefix = String::new();
            if i + 1 < tokens.len() && raw(i + 1) == ":" {
                prefix = raw(i);
                i += 2;
            }
            let Some(token) = tokens.get(i) else {
                return Ok(None);
            };
            if token.kind != TokenKind::Text && token.kind != TokenKind::Quoted {
                return Ok(None);
            }
            let atom = raw(i);
            i += 1;
            let atom_u = utf16::units(&atom);
            if token.kind == TokenKind::Quoted
                && (atom_u.len() < 2 || atom_u.last() != atom_u.first())
            {
                return Ok(None);
            }
            if token.kind == TokenKind::Quoted
                && prefix.is_empty()
                && (name == "domain" || name == "qname")
            {
                return Ok(None);
            }
            let value = unquote_str(&atom);
            if value.is_empty() || value.chars().any(|c| c == ',' || is_js_ws(c)) {
                return Ok(None);
            }
            let eff_prefix = if !prefix.is_empty() {
                prefix
            } else if name == "domain" || name == "qname" {
                "suffix".to_string()
            } else {
                String::new()
            };
            args.push((eff_prefix, value));
            if i < tokens.len() {
                let sep = raw(i);
                i += 1;
                if sep != "," || i == tokens.len() {
                    return Ok(None);
                }
            }
        }
        if args.is_empty() || args.iter().any(|(p, _)| *p != args[0].0) {
            return Ok(None);
        }
        let expected_sample = if args[0].0.is_empty() {
            format!("{name}(value)")
        } else {
            format!("{name}({}: value)", args[0].0)
        };
        let mut matched_kind: Option<String> = None;
        for k in &kinds {
            if let Ok(Some(sample)) = rule_condition_str(k, "value")
                && sample == expected_sample
            {
                matched_kind = Some(k.clone());
                break;
            }
        }
        let Some(kind) = matched_kind else {
            return Ok(None);
        };
        let val_joined = args
            .iter()
            .map(|(_, v)| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let row = json!({
            "kind": kind,
            "value": val_joined,
            "negate": negate,
        });
        if serialize_conditions(Value::Array(vec![row.clone()]))?.is_none() {
            return Ok(None);
        }
        rows.push(row);
        offset = next_offset;
        if offset == units.len() {
            break;
        }
        if js_slice(&units, offset, offset + 2) != "&&" {
            return Ok(None);
        }
        offset += 2;
        if js_trim(&js_slice(&units, offset, units.len())).is_empty() {
            return Ok(None);
        }
    }
    if rows.is_empty() {
        Ok(None)
    } else {
        Ok(Some(rows))
    }
}

// --- groupConditions.ts ---

const GROUP_CALLS: [(&str, &str, &str); 7] = [
    ("nameKeyword", "name", "keyword:"),
    ("nameRegex", "name", "regex:"),
    ("nameExact", "name", ""),
    ("subtag", "subtag", ""),
    ("subtagKeyword", "subtag", "keyword:"),
    ("subtagRegex", "subtag", "regex:"),
    ("group", "group", ""),
];

fn lookup_group_call(kind: &str) -> Result<(&'static str, &'static str), String> {
    GROUP_CALLS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map(|(_, f, p)| (*f, *p))
        .ok_or_else(|| "TypeError: Cannot read properties of undefined (reading '0')".to_string())
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn alloc_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn condition_family(kind: Value) -> Result<Value, String> {
    let k = kind.as_str().unwrap_or("");
    let (fam, _) = lookup_group_call(k)?;
    Ok(Value::String(fam.to_string()))
}

pub fn new_group_condition() -> Result<Value, String> {
    Ok(json!({
        "id": alloc_id(),
        "kind": "nameKeyword",
        "value": "",
        "negate": false,
    }))
}

fn needs_csv_quote(value: &str) -> bool {
    value
        .bytes()
        .any(|b| matches!(b, b',' | b'"' | b'\r' | b'\n'))
        || value.chars().next().is_some_and(is_js_ws)
        || value.chars().next_back().is_some_and(is_js_ws)
}

pub fn condition_values_str(values: &[String]) -> String {
    values
        .iter()
        .map(|v| {
            if needs_csv_quote(v) {
                serde_json::to_string(v).unwrap_or_else(|_| format!("\"{v}\""))
            } else {
                v.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn condition_values(values: Vec<String>) -> Result<Value, String> {
    Ok(Value::String(condition_values_str(&values)))
}

/// Match JS `/^"(?:[^"\\]|\\.)*"/` where `.` excludes `\n`, `\r`, `\u{2028}`, `\u{2029}`.
fn match_quoted_json_string(s: &str) -> Option<&str> {
    let mut chars = s.char_indices();
    let (_, first) = chars.next()?;
    if first != '"' {
        return None;
    }
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '"' => return Some(&s[..idx + 1]),
            '\\' => {
                let (_, escaped) = chars.next()?;
                if matches!(escaped, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

pub fn parse_condition_values_vec(text: &str) -> Option<Vec<String>> {
    let mut values: Vec<String> = Vec::new();
    let mut rest = js_trim(text);
    while !rest.is_empty() {
        let value: String;
        if rest.starts_with('"') {
            let m = match_quoted_json_string(rest)?;
            let parsed: Value = serde_json::from_str(m).ok()?;
            value = parsed.as_str()?.to_string();
            rest = js_trim_start(&rest[m.len()..]);
            if !rest.is_empty() && !rest.starts_with(',') {
                return None;
            }
        } else if let Some(end) = rest.find(',') {
            value = js_trim(&rest[..end]).to_string();
            rest = &rest[end..];
        } else {
            value = js_trim(rest).to_string();
            rest = "";
        }
        if value.is_empty() || !is_quotable_bool(&value) {
            return None;
        }
        values.push(value);
        if !rest.is_empty() {
            rest = js_trim_start(&rest[1..]);
            if rest.is_empty() {
                return None;
            }
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

pub fn parse_condition_values(text: String) -> Result<Option<Vec<String>>, String> {
    Ok(parse_condition_values_vec(&text))
}

#[derive(Clone, Debug)]
struct GroupTerm {
    id: u64,
    kind: &'static str,
    value: String,
}

#[derive(Clone, Debug)]
struct GroupRow {
    id: u64,
    kind: &'static str,
    value: String,
    negate: bool,
    alternatives: Option<Vec<GroupTerm>>,
}

impl GroupRow {
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".to_string(), json!(self.id));
        m.insert("kind".to_string(), json!(self.kind));
        m.insert("value".to_string(), json!(self.value));
        m.insert("negate".to_string(), json!(self.negate));
        if let Some(alts) = &self.alternatives {
            let list: Vec<Value> = alts
                .iter()
                .map(|a| json!({"id": a.id, "kind": a.kind, "value": a.value}))
                .collect();
            m.insert("alternatives".to_string(), Value::Array(list));
        }
        Value::Object(m)
    }
}

fn parse_group_conditions_rows(source: &str) -> Option<Vec<GroupRow>> {
    if js_trim(source).is_empty() || !is_fragment_bool(source) {
        return None;
    }
    let tokens = scan_config_raw(source).tokens;
    let units = utf16::units(source);
    let mut rows: Vec<GroupRow> = Vec::new();
    let mut index = 0usize;
    let raw_at = |idx: usize| -> String {
        tokens
            .get(idx)
            .map(|t| js_slice(&units, t.from, t.to))
            .unwrap_or_default()
    };
    while index < tokens.len() {
        let mut head = raw_at(index);
        let mut negate = false;
        if let Some(stripped) = head.strip_prefix('!') {
            negate = true;
            head = stripped.to_string();
            if head.is_empty() {
                index += 1;
                head = raw_at(index);
            }
        }
        if !matches!(head.as_str(), "name" | "subtag" | "group") {
            return None;
        }
        index += 1;
        if raw_at(index) != "(" {
            return None;
        }
        index += 1;
        let mut terms: Vec<GroupTerm> = Vec::new();
        while index < tokens.len() && raw_at(index) != ")" {
            let mut arg_prefix = String::new();
            let cur = raw_at(index);
            if (cur == "keyword" || cur == "regex") && raw_at(index + 1) == ":" {
                arg_prefix = format!("{cur}:");
                index += 2;
            }
            let token = tokens.get(index)?;
            if token.kind != TokenKind::Text && token.kind != TokenKind::Quoted {
                return None;
            }
            let value = unquote_str(&raw_at(index));
            if value.is_empty() || value.contains('\r') || value.contains('\n') {
                return None;
            }
            let kind = GROUP_CALLS
                .iter()
                .find(|(_, f, p)| *f == head && *p == arg_prefix)
                .map(|(k, _, _)| *k)?;
            let values: Vec<String> = if kind == "group" {
                value
                    .split(['|', ','])
                    .map(|v| js_trim(v).to_string())
                    .filter(|v| !v.is_empty())
                    .collect()
            } else {
                vec![value]
            };
            if values.is_empty() || !values.iter().all(|v| is_quotable_bool(v)) {
                return None;
            }
            if let Some(prev) = terms.last_mut()
                && prev.kind == kind
            {
                let mut combined = parse_condition_values_vec(&prev.value)?;
                combined.extend(values);
                prev.value = condition_values_str(&combined);
            } else {
                terms.push(GroupTerm {
                    id: alloc_id(),
                    kind,
                    value: condition_values_str(&values),
                });
            }
            index += 1;
            let sep = raw_at(index);
            if sep == "," {
                index += 1;
                if raw_at(index) == ")" {
                    return None;
                }
            } else if sep != ")" {
                return None;
            }
        }
        if raw_at(index) != ")" || terms.is_empty() {
            return None;
        }
        index += 1;
        let mut iter = terms.into_iter();
        let first = iter.next()?;
        let alternatives: Vec<GroupTerm> = iter.collect();
        if first.kind == "group" && (negate || !rows.is_empty() || index < tokens.len()) {
            return None;
        }
        rows.push(GroupRow {
            id: first.id,
            kind: first.kind,
            value: first.value,
            negate,
            alternatives: if alternatives.is_empty() {
                None
            } else {
                Some(alternatives)
            },
        });
        if index < tokens.len() {
            if raw_at(index) != "&&" {
                return None;
            }
            index += 1;
            if index == tokens.len() {
                return None;
            }
        }
    }
    if rows.is_empty() { None } else { Some(rows) }
}

pub fn parse_group_conditions(source: String) -> Result<Option<Vec<Value>>, String> {
    Ok(parse_group_conditions_rows(&source)
        .map(|rows| rows.into_iter().map(|r| r.to_value()).collect()))
}

pub fn serialize_group_conditions(rows: Value) -> Result<Option<String>, String> {
    if rows.is_null() || js::is_undefined(&rows) {
        return Err("TypeError: Cannot read properties of null (reading 'length')".to_string());
    }
    let Some(arr) = rows.as_array() else {
        if let Some(s) = rows.as_str() {
            if s.is_empty() {
                return Ok(None);
            }
            return Err("TypeError: rows is not iterable".to_string());
        }
        return Ok(None);
    };
    if arr.is_empty() {
        return Ok(None);
    }
    let mut terms: Vec<String> = Vec::new();
    for row in arr {
        let Some(o) = row.as_object() else {
            return Err("TypeError: Cannot read properties of null (reading 'kind')".to_string());
        };
        let row_kind = obj_defined(o, "kind").and_then(Value::as_str).unwrap_or("");
        let (call, _) = lookup_group_call(row_kind)?;
        let negate = obj_defined(o, "negate").is_some_and(is_truthy);
        let alts = match obj_defined(o, "alternatives") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(a)) => a.clone(),
            Some(_) => return Err("TypeError: row.alternatives is not iterable".to_string()),
        };
        let has_alts = !alts.is_empty();
        let mut arguments: Vec<String> = Vec::new();
        for term in std::iter::once(row).chain(alts.iter()) {
            let Some(to) = term.as_object() else {
                return Err(
                    "TypeError: Cannot read properties of null (reading 'value')".to_string(),
                );
            };
            let tval = obj_defined(to, "value")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    "TypeError: Cannot read properties of undefined (reading 'trim')".to_string()
                })?;
            let tkind = obj_defined(to, "kind")
                .and_then(Value::as_str)
                .unwrap_or("");
            let values = parse_condition_values_vec(tval);
            let Some(values) = values else {
                return Ok(None);
            };
            let (tfam, prefix) = lookup_group_call(tkind)?;
            if tfam != call {
                return Ok(None);
            }
            if tkind == "group"
                && (arr.len() != 1
                    || negate
                    || has_alts
                    || values.iter().any(|v| v.contains('|') || v.contains(',')))
            {
                return Ok(None);
            }
            for v in values {
                let q = quote_str(&v)?;
                if prefix.is_empty() {
                    arguments.push(q);
                } else {
                    arguments.push(format!("{prefix} {q}"));
                }
            }
        }
        terms.push(format!(
            "{}{call}({})",
            if negate { "!" } else { "" },
            arguments.join(", ")
        ));
    }
    Ok(Some(terms.join(" && ")))
}

fn row_state_from_value(rows: Option<&Value>) -> Result<String, String> {
    let Some(v) = rows.filter(|v| !v.is_null() && !js::is_undefined(v)) else {
        return Ok("null".to_string());
    };
    let Some(arr) = v.as_array() else {
        return Err("TypeError: rows.map is not a function".to_string());
    };
    let mut mapped: Vec<Value> = Vec::with_capacity(arr.len());
    for row in arr {
        let Some(o) = row.as_object() else {
            return Err("TypeError: Cannot destructure property 'kind' of 'undefined'".to_string());
        };
        let mut m = Map::new();
        if let Some(k) = obj_defined(o, "kind") {
            m.insert("kind".to_string(), k.clone());
        }
        if let Some(val) = obj_defined(o, "value") {
            m.insert("value".to_string(), val.clone());
        }
        if let Some(neg) = obj_defined(o, "negate") {
            m.insert("negate".to_string(), neg.clone());
        }
        if let Some(alts_v) = obj_defined(o, "alternatives")
            && let Some(alts) = alts_v.as_array()
            && !alts.is_empty()
        {
            let mut am = Vec::with_capacity(alts.len());
            for a in alts {
                if let Some(ao) = a.as_object() {
                    let mut entry = Map::new();
                    if let Some(k) = obj_defined(ao, "kind") {
                        entry.insert("kind".to_string(), k.clone());
                    }
                    if let Some(val) = obj_defined(ao, "value") {
                        entry.insert("value".to_string(), val.clone());
                    }
                    am.push(Value::Object(entry));
                }
            }
            m.insert("alternatives".to_string(), Value::Array(am));
        }
        mapped.push(Value::Object(m));
    }
    serde_json::to_string(&mapped).map_err(|e| e.to_string())
}

pub fn group_filter_draft(source: String, advanced: Option<bool>) -> Result<Value, String> {
    let rows_val = match parse_group_conditions_rows(&source) {
        Some(rows) => Value::Array(rows.into_iter().map(|r| r.to_value()).collect()),
        None => Value::Null,
    };
    let initial = row_state_from_value(Some(&rows_val))?;
    Ok(json!({
        "id": alloc_id(),
        "source": source,
        "rows": rows_val,
        "initial": initial,
        "advanced": advanced.unwrap_or(false),
    }))
}

pub fn group_filter_text(filter: Value) -> Result<Value, String> {
    let fo = crate::port::js_str::ensure_not_nullish(&filter, "rows")?;
    let rows = fo.and_then(|o| obj_defined(o, "rows"));
    let source = fo
        .and_then(|o| o.get("source"))
        .cloned()
        .unwrap_or_else(js::undefined);
    if rows.is_none_or(|r| !is_truthy(r)) {
        return Ok(source);
    }
    let state = row_state_from_value(rows)?;
    let initial = fo
        .and_then(|o| obj_defined(o, "initial"))
        .and_then(Value::as_str);
    if initial == Some(state.as_str()) {
        return Ok(source);
    }
    Ok(
        match serialize_group_conditions(rows.cloned().unwrap_or(Value::Null))? {
            Some(s) => Value::String(s),
            None => Value::Null,
        },
    )
}

pub fn group_filter_texts(filters: Vec<Value>) -> Result<Value, String> {
    let mut out = Vec::with_capacity(filters.len());
    for f in filters {
        let fallback = f
            .as_object()
            .and_then(|o| o.get("source"))
            .cloned()
            .unwrap_or_else(js::undefined);
        let res = group_filter_text(f)?;
        if res.is_null() || js::is_undefined(&res) {
            out.push(fallback);
        } else {
            out.push(res);
        }
    }
    Ok(Value::Array(out))
}

pub fn reconcile_group_filters(
    previous: Vec<Value>,
    texts: Vec<String>,
) -> Result<Vec<Value>, String> {
    let mut remaining = previous;
    let mut out = Vec::with_capacity(texts.len());
    for text in texts {
        let mut found_idx = None;
        for (i, filter) in remaining.iter().enumerate() {
            let res = group_filter_text(filter.clone())?;
            let eff = if res.is_null() || js::is_undefined(&res) {
                filter
                    .as_object()
                    .and_then(|o| obj_defined(o, "source"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                res.as_str().map(str::to_string)
            };
            if eff.as_deref() == Some(text.as_str()) {
                found_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = found_idx {
            out.push(remaining.remove(idx));
        } else {
            out.push(group_filter_draft(text, Some(false))?);
        }
    }
    Ok(out)
}

pub fn new_group_filter() -> Result<Value, String> {
    let mut draft = group_filter_draft(String::new(), Some(true))?;
    let cond = new_group_condition()?;
    if let Some(o) = draft.as_object_mut() {
        o.insert("rows".to_string(), Value::Array(vec![cond]));
    }
    Ok(draft)
}

// --- rustRegex.ts (validator) + groupFilters.ts ---

struct UnicodeTables {
    binary: HashSet<String>,
    categories: HashSet<String>,
    scripts: HashSet<String>,
}

fn loose_name(name: &str) -> String {
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b' ' | b'-'))
    {
        return String::new();
    }
    name.bytes()
        .filter(|&b| !matches!(b, b' ' | b'_' | b'-'))
        .map(|b| (b as char).to_ascii_lowercase())
        .collect()
}

fn unicode_tables() -> &'static UnicodeTables {
    static TABLES: OnceLock<UnicodeTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let make = |s: &str| -> HashSet<String> {
            s.split_whitespace().map(loose_name).collect()
        };
        UnicodeTables {
            binary: make("ASCII ASCII_Hex_Digit Alphabetic Any Assigned Bidi_Control Bidi_Mirrored Case_Ignorable Cased Changes_When_Casefolded Changes_When_Casemapped Changes_When_Lowercased Changes_When_Titlecased Changes_When_Uppercased Dash Default_Ignorable_Code_Point Deprecated Diacritic Emoji Emoji_Component Emoji_Modifier Emoji_Modifier_Base Emoji_Presentation Extended_Pictographic Extender Grapheme_Base Grapheme_Extend Hex_Digit IDS_Binary_Operator IDS_Trinary_Operator ID_Continue ID_Start Ideographic Join_Control Logical_Order_Exception Lowercase Math Noncharacter_Code_Point Pattern_Syntax Pattern_White_Space Quotation_Mark Radical Regional_Indicator Sentence_Terminal Soft_Dotted Terminal_Punctuation Unified_Ideograph Uppercase Variation_Selector White_Space XID_Continue XID_Start"),
            categories: make("C Other Cc Control Cf Format Cn Unassigned Co Private_Use L Letter LC Cased_Letter Ll Lowercase_Letter Lm Modifier_Letter Lo Other_Letter Lt Titlecase_Letter Lu Uppercase_Letter M Mark Mc Spacing_Mark Me Enclosing_Mark Mn Nonspacing_Mark N Number Nd Decimal_Number Nl Letter_Number No Other_Number P Punctuation Pc Connector_Punctuation Pd Dash_Punctuation Pe Close_Punctuation Pf Final_Punctuation Pi Initial_Punctuation Po Other_Punctuation Ps Open_Punctuation S Symbol Sc Currency_Symbol Sk Modifier_Symbol Sm Math_Symbol So Other_Symbol Z Separator Zl Line_Separator Zp Paragraph_Separator Zs Space_Separator"),
            scripts: make("Adlam Ahom Anatolian_Hieroglyphs Arabic Armenian Avestan Balinese Bamum Bassa_Vah Batak Bengali Bhaiksuki Bopomofo Brahmi Braille Buginese Buhid Canadian_Aboriginal Carian Caucasian_Albanian Chakma Cham Cherokee Chorasmian Common Coptic Cuneiform Cypriot Cypro_Minoan Cyrillic Deseret Devanagari Dives_Akuru Dogra Duployan Egyptian_Hieroglyphs Elbasan Elymaic Ethiopic Garay Georgian Glagolitic Gothic Grantha Greek Gujarati Gunjala_Gondi Gurmukhi Gurung_Khema Han Hangul Hanifi_Rohingya Hanunoo Hatran Hebrew Hiragana Imperial_Aramaic Inherited Inscriptional_Pahlavi Inscriptional_Parthian Javanese Kaithi Kannada Katakana Kawi Kayah_Li Kharoshthi Khitan_Small_Script Khmer Khojki Khudawadi Kirat_Rai Lao Latin Lepcha Limbu Linear_A Linear_B Lisu Lycian Lydian Mahajani Makasar Malayalam Mandaic Manichaean Marchen Masaram_Gondi Medefaidrin Meetei_Mayek Mende_Kikakui Meroitic_Cursive Meroitic_Hieroglyphs Miao Modi Mongolian Mro Multani Myanmar Nabataean Nag_Mundari Nandinagari New_Tai_Lue Newa Nko Nushu Nyiakeng_Puachue_Hmong Ogham Ol_Chiki Ol_Onal Old_Hungarian Old_Italic Old_North_Arabian Old_Permic Old_Persian Old_Sogdian Old_South_Arabian Old_Turkic Old_Uyghur Oriya Osage Osmanya Pahawh_Hmong Palmyrene Pau_Cin_Hau Phags_Pa Phoenician Psalter_Pahlavi Rejang Runic Samaritan Saurashtra Sharada Shavian Siddham SignWriting Sinhala Sogdian Sora_Sompeng Soyombo Sundanese Sunuwar Syloti_Nagri Syriac Tagalog Tagbanwa Tai_Le Tai_Tham Tai_Viet Takri Tamil Tangsa Tangut Telugu Thaana Thai Tibetan Tifinagh Tirhuta Todhri Toto Tulu_Tigalari Ugaritic Vai Vithkuqi Wancho Warang_Citi Yezidi Yi Zanabazar_Square"),
        }
    })
}

fn valid_unicode_property(body: &str) -> bool {
    let t = unicode_tables();
    let by_val = body
        .find("!=")
        .map(|p| (&body[..p], &body[p + 2..]))
        .or_else(|| body.find(['=', ':']).map(|p| (&body[..p], &body[p + 1..])));
    if let Some((k, v)) = by_val {
        let lk = loose_name(k);
        let lv = loose_name(v);
        match lk.as_str() {
            "script" | "sc" | "scriptextensions" | "scx" => t.scripts.contains(&lv),
            "generalcategory" | "gc" => t.categories.contains(&lv),
            _ => false,
        }
    } else {
        let lv = loose_name(body);
        t.binary.contains(&lv) || t.categories.contains(&lv) || t.scripts.contains(&lv)
    }
}

/// Validates that `source` is accepted by `rustRegex.ts`'s `rustRegexToJs`, then compiles with `regex::Regex`.
fn compile_rust_regex(source: &str) -> Option<regex::Regex> {
    let mut rest = source;
    while let Some(after) = rest.strip_prefix("(?") {
        let close = after.find(')')?;
        let flags = &after[..close];
        if flags.is_empty() || !flags.bytes().all(|b| matches!(b, b'i' | b'm' | b's')) {
            break;
        }
        let uniq: HashSet<u8> = flags.bytes().collect();
        if uniq.len() != flags.len() {
            return None;
        }
        rest = &after[close + 1..];
    }

    struct Parser {
        chars: Vec<char>,
        i: usize,
        repeats: usize,
    }
    impl Parser {
        fn peek(&self, off: usize) -> Option<char> {
            self.chars.get(self.i + off).copied()
        }
        fn take(&mut self) -> Option<char> {
            let c = self.chars.get(self.i).copied()?;
            self.i += 1;
            Some(c)
        }
        fn ahead(&self, s: &str) -> bool {
            for (idx, ch) in (self.i..).zip(s.chars()) {
                if self.chars.get(idx) != Some(&ch) {
                    return false;
                }
            }
            true
        }
        fn hex(&mut self, digits: usize) -> Option<u32> {
            let mut text = String::new();
            if self.peek(0) == Some('{') {
                self.i += 1;
                while self.peek(0) != Some('}') {
                    text.push(self.take()?);
                }
                self.i += 1;
            } else {
                for _ in 0..digits {
                    text.push(self.take()?);
                }
            }
            if text.is_empty() || text.len() > 8 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            let cp = u32::from_str_radix(&text, 16).ok()?;
            if cp > 0x10ffff || (0xd800..=0xdfff).contains(&cp) {
                return None;
            }
            Some(cp)
        }
        fn escape(&mut self, in_class: bool) -> Option<Option<u32>> {
            let c = self.take()?;
            if matches!(c, 'd' | 'D' | 's' | 'S' | 'w' | 'W') {
                return Some(None);
            }
            if c == 'p' || c == 'P' {
                let body = if self.peek(0) == Some('{') {
                    self.i += 1;
                    let mut b = String::new();
                    while self.peek(0) != Some('}') {
                        b.push(self.take()?);
                    }
                    self.i += 1;
                    b
                } else {
                    self.take()?.to_string()
                };
                return valid_unicode_property(&body).then_some(None);
            }
            let ctrl = match c {
                'a' => Some(7),
                'f' => Some(12),
                't' => Some(9),
                'n' => Some(10),
                'r' => Some(13),
                'v' => Some(11),
                _ => None,
            };
            if let Some(cp) = ctrl {
                return Some(Some(cp));
            }
            if c == 'x' {
                return Some(Some(self.hex(2)?));
            }
            if c == 'u' {
                return Some(Some(self.hex(4)?));
            }
            if c == 'U' {
                return Some(Some(self.hex(8)?));
            }
            if !in_class {
                if matches!(c, 'A' | 'z') {
                    return Some(None);
                }
                if c == 'b' && self.peek(0) == Some('{') {
                    self.i += 1;
                    let mut name = String::new();
                    while self.peek(0) != Some('}') {
                        name.push(self.take()?);
                    }
                    self.i += 1;
                    return matches!(name.as_str(), "start" | "end" | "start-half" | "end-half")
                        .then_some(None);
                }
                if matches!(c, 'b' | 'B' | '<' | '>') {
                    return Some(None);
                }
            }
            if (c as u32) <= 0x7f && !c.is_ascii_alphanumeric() && !matches!(c, '<' | '>') {
                Some(Some(c as u32))
            } else {
                None
            }
        }
        fn bracket(&mut self) -> Option<()> {
            if self.peek(0) == Some('^') {
                self.i += 1;
            }
            let mut operand_len = 0usize;
            let mut first = true;
            loop {
                let ch = self.peek(0)?;
                if ch == ']' && !first {
                    self.i += 1;
                    break;
                }
                if self.ahead("&&") || self.ahead("--") || self.ahead("~~") {
                    if first || self.ahead("~~") || operand_len == 0 {
                        return None;
                    }
                    self.i += 2;
                    operand_len = 0;
                    continue;
                }
                first = false;
                let piece_cp: Option<u32> = if ch == '[' {
                    self.i += 1;
                    let tail: String = self.chars[self.i..(self.i + 12).min(self.chars.len())]
                        .iter()
                        .collect();
                    let mut posix_len = None;
                    if let Some(after_colon) = tail.strip_prefix(':') {
                        let after_neg = after_colon.strip_prefix('^').unwrap_or(after_colon);
                        if let Some(end) = after_neg.find(":]") {
                            let name = &after_neg[..end];
                            if matches!(
                                name,
                                "alnum"
                                    | "alpha"
                                    | "ascii"
                                    | "blank"
                                    | "cntrl"
                                    | "digit"
                                    | "graph"
                                    | "lower"
                                    | "print"
                                    | "punct"
                                    | "space"
                                    | "upper"
                                    | "word"
                                    | "xdigit"
                            ) {
                                let consumed = (tail.len() - after_neg.len()) + end + 2;
                                posix_len = Some(consumed);
                            } else {
                                return None;
                            }
                        }
                    }
                    if let Some(n) = posix_len {
                        self.i += n;
                    } else {
                        self.bracket()?;
                    }
                    None
                } else if ch == '\\' {
                    self.i += 1;
                    self.escape(true)?
                } else {
                    Some(self.take()? as u32)
                };
                let ranged = self.peek(0) == Some('-')
                    && self.peek(1) != Some(']')
                    && self.peek(1) != Some('-');
                if piece_cp.is_none() {
                    if ranged {
                        return None;
                    }
                    operand_len += 1;
                    continue;
                }
                if !ranged {
                    operand_len += 1;
                    continue;
                }
                self.i += 1;
                if self.peek(0) == Some('[') {
                    return None;
                }
                let end_cp = if self.take()? == '\\' {
                    self.escape(true)??
                } else {
                    self.chars[self.i - 1] as u32
                };
                if end_cp < piece_cp? {
                    return None;
                }
                operand_len += 1;
            }
            if operand_len == 0 {
                return None;
            }
            Some(())
        }
        fn alternation(&mut self, depth: usize) -> Option<bool> {
            let mut branches = 0usize;
            let mut atoms: Vec<(bool, bool)> = Vec::new(); // (quantified, varies)
            let mut varies = false;
            loop {
                let c = self.peek(0);
                if matches!(c, None | Some(')') | Some('|')) {
                    branches += 1;
                    varies = varies || branches > 1;
                    atoms.clear();
                    match c {
                        None => {
                            if depth != 0 {
                                return None;
                            }
                            break;
                        }
                        Some(')') => {
                            self.i += 1;
                            if depth == 0 {
                                return None;
                            }
                            break;
                        }
                        Some('|') => {
                            self.i += 1;
                            continue;
                        }
                        _ => {}
                    }
                }
                let ch = self.take()?;
                if matches!(ch, '*' | '+' | '?') {
                    let (min, max) = match ch {
                        '?' => (0u64, Some(1u64)),
                        '+' => (1u64, None),
                        _ => (0u64, None),
                    };
                    let (q, v) = atoms.pop()?;
                    if q || (max.is_none_or(|m| m > 1) && v) {
                        return None;
                    }
                    if max != Some(min) {
                        self.repeats += 1;
                        if self.repeats > 3 {
                            return None;
                        }
                    }
                    if self.peek(0) == Some('?') {
                        self.i += 1;
                    }
                    atoms.push((true, false));
                    varies = true;
                } else if ch == '{' {
                    let mut body = String::new();
                    while self.peek(0) != Some('}') {
                        body.push(self.take()?);
                    }
                    self.i += 1;
                    let b = js_trim(&body);
                    let (min_s, comma, max_s) = match b.find(',') {
                        Some(pos) => (js_trim(&b[..pos]), true, js_trim(&b[pos + 1..])),
                        None => (b, false, ""),
                    };
                    if min_s.is_empty()
                        || !min_s.bytes().all(|x| x.is_ascii_digit())
                        || (!max_s.is_empty() && !max_s.bytes().all(|x| x.is_ascii_digit()))
                    {
                        return None;
                    }
                    let min: u64 = min_s.parse().ok()?;
                    let max: Option<u64> = if comma && max_s.is_empty() {
                        None
                    } else if !max_s.is_empty() {
                        Some(max_s.parse().ok()?)
                    } else {
                        Some(min)
                    };
                    if min > 1000 || max.is_some_and(|m| m > 1000 || m < min) {
                        return None;
                    }
                    let (q, v) = atoms.pop()?;
                    if q || (max.is_none_or(|m| m > 1) && v) {
                        return None;
                    }
                    if max != Some(min) {
                        self.repeats += 1;
                        if self.repeats > 3 {
                            return None;
                        }
                    }
                    if self.peek(0) == Some('?') {
                        self.i += 1;
                    }
                    atoms.push((true, false));
                    varies = true;
                } else if ch == '(' {
                    if self.peek(0) == Some('?') {
                        self.i += 1;
                        if self.take()? != ':' {
                            return None;
                        }
                    }
                    let gv = self.alternation(depth + 1)?;
                    varies = varies || gv;
                    atoms.push((false, gv));
                } else if ch == '[' {
                    self.bracket()?;
                    atoms.push((false, false));
                } else if ch == '\\' {
                    self.escape(false)?;
                    atoms.push((false, false));
                } else {
                    atoms.push((false, false));
                }
            }
            Some(varies)
        }
    }

    let mut parser = Parser {
        chars: rest.chars().collect(),
        i: 0,
        repeats: 0,
    };
    parser.alternation(0)?;
    regex::Regex::new(source).ok()
}

enum FilterTest {
    Exact(String),
    Keyword(String),
    Regex(regex::Regex),
}

impl FilterTest {
    fn matches(&self, candidate: &str) -> bool {
        match self {
            Self::Exact(v) => candidate == v,
            Self::Keyword(v) => candidate.contains(v.as_str()),
            Self::Regex(re) => re.is_match(candidate),
        }
    }
}

struct FilterTerm {
    is_name: bool,
    negated: bool,
    exact: Vec<String>,
    tests: Vec<FilterTest>,
}

fn parse_filter_term(raw: &str) -> Option<FilterTerm> {
    let (negated, body) = match raw.strip_prefix('!') {
        Some(rest) => (true, js_trim(rest)),
        None => (false, raw),
    };
    let (is_name, inner_start) = if body.starts_with("name(") {
        (true, 5)
    } else if body.starts_with("subtag(") {
        (false, 7)
    } else {
        return None;
    };
    if !body.ends_with(')') {
        return None;
    }
    let mut exact = Vec::new();
    let mut tests = Vec::new();
    for arg in split_top_vec(&body[inner_start..body.len() - 1], ",") {
        let (kind, rest) = if let Some(r) = arg.strip_prefix("keyword:") {
            (1u8, js_trim_start(r))
        } else if let Some(r) = arg.strip_prefix("regex:") {
            (2u8, js_trim_start(r))
        } else {
            (0u8, arg.as_str())
        };
        let value = unquote_str(js_trim(rest));
        if value.is_empty() {
            continue;
        }
        match kind {
            1 => tests.push(FilterTest::Keyword(value)),
            2 => {
                let re = compile_rust_regex(&value)?;
                tests.push(FilterTest::Regex(re));
            }
            _ => {
                exact.push(value.clone());
                tests.push(FilterTest::Exact(value));
            }
        }
    }
    if tests.is_empty() {
        None
    } else {
        Some(FilterTerm {
            is_name,
            negated,
            exact,
            tests,
        })
    }
}

fn parse_filter_line(filter: &str) -> Option<Vec<FilterTerm>> {
    let parts = split_top_vec(filter, "&&");
    if parts.is_empty() {
        return None;
    }
    let mut terms = Vec::with_capacity(parts.len());
    for part in parts {
        terms.push(parse_filter_term(&part)?);
    }
    Some(terms)
}

pub fn group_admits(filters: Vec<String>, node: Value) -> Result<Value, String> {
    let Some(o) = node.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'name')".to_string());
    };
    let name = obj_defined(o, "name").and_then(Value::as_str);
    let subtag = obj_defined(o, "subscription_tag").and_then(Value::as_str);

    let lines: Vec<Vec<FilterTerm>> = filters
        .iter()
        .filter_map(|f| parse_filter_line(f))
        .collect();
    if lines.is_empty() {
        return Ok(Value::Bool(
            filters.is_empty() && !is_builtin_outbound_str(name),
        ));
    }
    let name_str = name.unwrap_or("");
    let builtin = is_builtin_outbound_str(name);
    let admitted = lines.iter().any(|line| {
        let builtin_ok = !builtin
            || line
                .iter()
                .any(|t| t.is_name && !t.negated && t.exact.iter().any(|e| e == name_str));
        if !builtin_ok {
            return false;
        }
        line.iter().all(|term| {
            let matched = if !term.is_name && subtag.is_none() {
                false
            } else {
                let target = if term.is_name {
                    name_str
                } else {
                    subtag.unwrap_or("")
                };
                term.tests.iter().any(|t| t.matches(target))
            };
            matched != term.negated
        })
    });
    Ok(Value::Bool(admitted))
}
