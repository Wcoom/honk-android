//! The Rust port of `apps/lite/src/ruleDraft.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::utf16;
use dae_core::port::conditions::{condition_values_str, parse_condition_values_vec};
use dae_core::port::js_str::{
    ensure_not_nullish, index_of_u16, is_js_ws, is_space_or_tab, is_truthy, js_slice, js_slice_f64,
    js_slice_from, js_to_string, js_trim, js_trim_end, js_val_to_number, line_start_u16,
    obj_defined,
};
use dae_core::port::rule_text::rule_line_str;
use dae_core::port::text::{
    TextBlock, TokenKind, is_fragment_bool, is_quotable_bool, quote_str, scan_config_raw,
    unquote_str,
};
use serde_json::{Map, Value, json};

const CONDITION_KINDS: &[&str] = &[
    "domainSuffix",
    "domain",
    "domainKeyword",
    "geosite",
    "dip",
    "geoip",
    "sip",
    "mac",
    "dport",
    "sport",
    "pname",
    "l4proto",
];

const DNS_REQUEST_KINDS: &[&str] = &[
    "qnameSuffix",
    "qnameFull",
    "qnameKeyword",
    "qnameGeosite",
    "qtype",
    "sip",
];

const DNS_RESPONSE_KINDS: &[&str] = &[
    "qnameSuffix",
    "qnameFull",
    "qnameKeyword",
    "qnameGeosite",
    "qtype",
    "sip",
    "upstream",
    "answerIp",
    "answerGeoip",
];

const EXTRA: &[(&str, &str, &str)] = &[
    ("domainRegex", "domain", "regex"),
    ("ipversion", "ipversion", ""),
    ("dscp", "dscp", ""),
    ("qnameRegex", "qname", "regex"),
];

fn extra_lookup(kind: &str) -> Option<(&'static str, &'static str)> {
    EXTRA
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map(|(_, name, prefix)| (*name, *prefix))
}

fn list_path(kind: &str) -> Vec<String> {
    if kind == "routing" {
        vec!["routing".to_string()]
    } else {
        vec!["dns".to_string(), "routing".to_string(), kind.to_string()]
    }
}

fn find_blocks_raw(text: &str, path: &[String]) -> Vec<TextBlock> {
    let mut blocks = scan_config_raw(text).blocks;
    for (index, name) in path.iter().enumerate() {
        let found: Vec<TextBlock> = blocks.into_iter().filter(|b| b.name == *name).collect();
        if index == path.len() - 1 {
            return found;
        }
        blocks = found.into_iter().flat_map(|b| b.children).collect();
    }
    Vec::new()
}

fn split_ws_and_ws(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < text.len() {
        if let Some(ch) = text[i..].chars().next()
            && is_js_ws(ch)
        {
            let ws_start = i;
            let mut j = i + ch.len_utf8();
            while let Some(c2) = text[j..].chars().next() {
                if is_js_ws(c2) {
                    j += c2.len_utf8();
                } else {
                    break;
                }
            }
            if text[j..].starts_with("&&") {
                let after_and = j + 2;
                let mut k = after_and;
                while let Some(c3) = text[k..].chars().next() {
                    if is_js_ws(c3) {
                        k += c3.len_utf8();
                    } else {
                        break;
                    }
                }
                if k > after_and {
                    parts.push(&text[start..ws_start]);
                    start = k;
                    i = k;
                    continue;
                }
            }
            i = j;
        } else if let Some(ch) = text[i..].chars().next() {
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    parts.push(&text[start..]);
    parts
}

/// Match JS `/^(!?)\s*(\w+)\((.*)\)$/` (`.` excludes `\n`, `\r`, `\u{2028}`, `\u{2029}`).
fn match_extra_call(part: &str) -> Option<(bool, &str, &str)> {
    let trimmed = js_trim(part);
    let (negate, rest) = match trimmed.strip_prefix('!') {
        Some(r) => (true, r),
        None => (false, trimmed),
    };
    let rest = rest.trim_start_matches(is_js_ws);
    let name_len = rest
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
        .count();
    if name_len == 0 {
        return None;
    }
    let fn_name = &rest[..name_len];
    let after_name = &rest[name_len..];
    let inner = after_name.strip_prefix('(')?.strip_suffix(')')?;
    if inner
        .chars()
        .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    {
        return None;
    }
    Some((negate, fn_name, inner))
}

fn parse_builder(text: &str, kind: &str) -> Result<Option<Vec<Value>>, String> {
    let all_kinds = kinds_for(kind.to_string())?;
    let ordinary_kinds: Vec<String> = all_kinds
        .iter()
        .filter(|k| extra_lookup(k).is_none())
        .cloned()
        .collect();
    if let Some(parsed) =
        dae_core::port::parse_conditions(text.to_string(), ordinary_kinds.clone())?
    {
        return Ok(Some(parsed));
    }
    let mut rows: Vec<Value> = Vec::new();
    for part in split_ws_and_ws(text) {
        if let Some(ordinary) =
            dae_core::port::parse_conditions(part.to_string(), ordinary_kinds.clone())?
        {
            rows.extend(ordinary);
            continue;
        }
        let Some((negate, fn_name, inner)) = match_extra_call(part) else {
            return Ok(None);
        };
        let tokens = scan_config_raw(inner).tokens;
        let inner_u = utf16::units(inner);
        let raw = |n: usize| -> String { js_slice(&inner_u, tokens[n].from, tokens[n].to) };
        let mut values: Vec<String> = Vec::new();
        let mut prefix = String::new();
        let mut i = 0usize;
        while i < tokens.len() {
            let mut p = String::new();
            if i + 1 < tokens.len() && raw(i + 1) == ":" {
                p = raw(i);
                i += 2;
            }
            let Some(tok) = tokens.get(i) else {
                return Ok(None);
            };
            if tok.kind != TokenKind::Quoted || (!values.is_empty() && p != prefix) {
                return Ok(None);
            }
            prefix = p;
            values.push(unquote_str(&raw(i)));
            i += 1;
            if i < tokens.len() {
                let sep = raw(i);
                i += 1;
                if sep != "," {
                    return Ok(None);
                }
            }
        }
        let key = EXTRA
            .iter()
            .find(|(k, name, pref)| {
                all_kinds.iter().any(|ak| ak == k) && *name == fn_name && *pref == prefix
            })
            .map(|(k, _, _)| *k);
        let Some(key) = key else {
            return Ok(None);
        };
        if values.is_empty() {
            return Ok(None);
        }
        rows.push(json!({
            "kind": key,
            "value": condition_values_str(&values),
            "negate": negate,
        }));
    }
    if serialize_builder(Value::Array(rows.clone()))?.is_some() {
        Ok(Some(rows))
    } else {
        Ok(None)
    }
}

struct ParsedRow {
    from: usize,
    to: usize,
    raw: String,
    note: String,
    expression: String,
    target: String,
    fallback: bool,
    conditions: Option<Vec<Value>>,
    anchor: Value,
    anchor_indent: String,
}

impl ParsedRow {
    fn to_value(&self) -> Value {
        json!({
            "from": self.from,
            "to": self.to,
            "raw": self.raw,
            "note": self.note,
            "expression": self.expression,
            "target": self.target,
            "fallback": self.fallback,
            "conditions": match &self.conditions {
                Some(rows) => Value::Array(rows.clone()),
                None => Value::Null,
            },
            "anchor": self.anchor,
        })
    }
}

struct ParsedDoc {
    text: String,
    kind: String,
    block: Option<TextBlock>,
    rows: Vec<ParsedRow>,
    ambiguous: bool,
}

impl ParsedDoc {
    fn to_value(&self) -> Value {
        json!({
            "text": self.text,
            "kind": self.kind,
            "block": match &self.block {
                Some(b) => b.to_value(),
                None => Value::Null,
            },
            "rows": self.rows.iter().map(ParsedRow::to_value).collect::<Vec<_>>(),
            "ambiguous": self.ambiguous,
        })
    }
}

fn leading_blank_newline(units: &[u16], cursor: usize) -> Option<usize> {
    let mut i = cursor;
    while i < units.len() && (units[i] == b' ' as u16 || units[i] == b'\t' as u16) {
        i += 1;
    }
    if i < units.len() && units[i] == b'\r' as u16 {
        i += 1;
    }
    if i < units.len() && units[i] == b'\n' as u16 {
        Some(i + 1)
    } else {
        None
    }
}

fn extract_note(prefix: &str) -> String {
    let mut out = Vec::new();
    for raw_line in prefix.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim_start_matches(is_js_ws);
        if let Some(after_hash) = trimmed.strip_prefix('#') {
            let cleaned = match after_hash.chars().next() {
                Some(ch) if is_js_ws(ch) => &after_hash[ch.len_utf8()..],
                _ => after_hash,
            };
            out.push(cleaned);
        }
    }
    out.join("\n")
}

fn parse_rule_list_raw(text: &str, kind: &str) -> Result<ParsedDoc, String> {
    let path = list_path(kind);
    let blocks = find_blocks_raw(text, &path);
    let block = if blocks.len() == 1 {
        blocks.into_iter().next()
    } else {
        None
    };
    let mut ambiguous = (0..path.len()).any(|i| find_blocks_raw(text, &path[..=i]).len() > 1);
    let mut doc = ParsedDoc {
        text: text.to_string(),
        kind: kind.to_string(),
        block: block.clone(),
        rows: Vec::new(),
        ambiguous,
    };
    let Some(block) = block else {
        return Ok(doc);
    };
    let units = utf16::units(text);
    let tokens: Vec<_> = scan_config_raw(text)
        .tokens
        .into_iter()
        .filter(|t| t.from > block.open && t.to <= block.close && t.kind != TokenKind::Comment)
        .collect();
    let mut i = 0usize;
    let mut cursor = block.open + 1;
    if let Some(next_cursor) = leading_blank_newline(&units, cursor) {
        cursor = next_cursor;
    }
    while i < tokens.len() {
        let first = &tokens[i];
        let mut last = first;
        i += 1;
        while i < tokens.len() && (tokens[i].line == last.line || tokens[i].parens > 0) {
            last = &tokens[i];
            i += 1;
        }
        let to = match index_of_u16(&units, b'\n' as u16, last.to) {
            Some(nl) if nl < block.close => nl + 1,
            _ => last.to,
        };
        let actual: Vec<_> = tokens
            .iter()
            .filter(|t| t.from >= first.from && t.to <= last.to)
            .collect();
        let raw_token = |n: usize| -> String {
            actual
                .get(n)
                .map(|t| js_slice(&units, t.from, t.to))
                .unwrap_or_default()
        };
        let fallback =
            matches!(raw_token(0).as_str(), "fallback" | "default") && raw_token(1) == ":";
        let arrow_idx = if fallback {
            Some(1)
        } else {
            actual
                .iter()
                .position(|t| t.parens == 0 && js_slice(&units, t.from, t.to) == "->")
        };
        let arrow = arrow_idx.and_then(|idx| actual.get(idx).copied());
        let before = arrow_idx
            .and_then(|idx| idx.checked_sub(1))
            .and_then(|idx| actual.get(idx).copied());
        let expression = match (arrow, before) {
            (Some(_), Some(b)) => js_slice(&units, first.from, b.to),
            _ => js_slice(&units, first.from, last.to),
        };
        let target = arrow
            .map(|a| js_trim(&js_slice(&units, a.to, last.to)).to_string())
            .unwrap_or_default();
        let line_from = line_start_u16(&units, first.from);
        let indent = js_slice(&units, line_from.max(block.open + 1), first.from);
        let raw = js_slice(&units, cursor, to);
        let mut anchor_map = Map::new();
        anchor_map.insert("from".to_string(), json!(cursor));
        anchor_map.insert("to".to_string(), json!(to));
        anchor_map.insert("text".to_string(), Value::String(raw.clone()));
        anchor_map.insert("indent".to_string(), Value::String(indent.clone()));
        if let Some(a) = arrow {
            anchor_map.insert("target".to_string(), json!({"from": a.to, "to": last.to}));
        }
        if !fallback && let Some(b) = before {
            anchor_map.insert(
                "condition".to_string(),
                json!({"from": first.from, "to": b.to}),
            );
        }
        let note = extract_note(&js_slice(&units, cursor, first.from));
        let conditions = if !fallback && arrow.is_some() {
            parse_builder(&expression, kind)?
        } else {
            None
        };
        doc.rows.push(ParsedRow {
            from: cursor,
            to,
            raw,
            note,
            expression,
            target,
            fallback,
            conditions,
            anchor: Value::Object(anchor_map),
            anchor_indent: indent,
        });
        cursor = to;
    }
    ambiguous = ambiguous || doc.rows.iter().filter(|r| r.fallback).count() > 1;
    doc.ambiguous = ambiguous;
    Ok(doc)
}

fn js_length_val(v: &Value) -> Result<Value, String> {
    if v.is_null() {
        return Err("TypeError: Cannot read properties of null (reading 'length')".to_string());
    }
    if js::is_undefined(v) {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
        );
    }
    Ok(match v {
        Value::Array(a) => js::number(a.len() as f64),
        Value::String(s) => js::number(s.encode_utf16().count() as f64),
        Value::Object(o) => o.get("length").cloned().unwrap_or_else(js::undefined),
        _ => js::undefined(),
    })
}

fn js_strict_eq(a: &Value, b: &Value) -> bool {
    if let (Some(na), Some(nb)) = (js::as_number(a), js::as_number(b)) {
        return na == nb;
    }
    a == b
}

fn valid_array_index(x: f64, len: usize) -> Option<usize> {
    if x.is_finite() && x >= 0.0 && x.fract() == 0.0 && x < len as f64 {
        Some(x as usize)
    } else {
        None
    }
}

fn ensure_block_inner(text: &str, path: &[String]) -> Result<String, String> {
    if path.is_empty() {
        return Err("RangeError: Maximum call stack size exceeded".to_string());
    }
    if !find_blocks_raw(text, path).is_empty() {
        return Ok(text.to_string());
    }
    if path.len() == 1 {
        let sep = if text.ends_with('\n') { "" } else { "\n" };
        return Ok(format!("{text}{sep}{} {{\n}}\n", path[0]));
    }
    let parent_path = &path[..path.len() - 1];
    let text = ensure_block_inner(text, parent_path)?;
    let parent = find_blocks_raw(&text, parent_path);
    if parent.len() != 1 {
        return Err("Ambiguous block".to_string());
    }
    let block = &parent[0];
    let units = utf16::units(&text);
    let at = line_start_u16(&units, block.close);
    let indent = "  ".repeat(path.len() - 1);
    let pos = if is_space_or_tab(&js_slice(&units, at, block.close)) {
        at
    } else {
        block.close
    };
    let last = &path[path.len() - 1];
    Ok(format!(
        "{}{indent}{last} {{\n{indent}}}\n{}",
        js_slice(&units, 0, pos),
        js_slice_from(&units, pos)
    ))
}

fn strip_undefined_for_json(v: &Value) -> Option<Value> {
    if js::is_undefined(v) {
        return None;
    }
    match v {
        Value::Array(arr) => Some(Value::Array(
            arr.iter()
                .map(|item| strip_undefined_for_json(item).unwrap_or(Value::Null))
                .collect(),
        )),
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, val) in map {
                if let Some(clean) = strip_undefined_for_json(val) {
                    out.insert(k.clone(), clean);
                }
            }
            Some(Value::Object(out))
        }
        other => Some(other.clone()),
    }
}

fn js_json_stringify(v: &Value) -> Option<String> {
    let clean = strip_undefined_for_json(v)?;
    serde_json::to_string(&clean).ok()
}

fn ends_with_must(target: &str) -> bool {
    let s = js_trim_end(target);
    let Some(s) = s.strip_suffix(')') else {
        return false;
    };
    let s = js_trim_end(s);
    let Some(s) = s.strip_suffix("must") else {
        return false;
    };
    js_trim_end(s).ends_with('(')
}

/// Port of `kindsFor` (`apps/lite/src/ruleDraft.ts:17`).
pub fn kinds_for(kind: String) -> Result<Vec<String>, String> {
    let list: &[&str] = match kind.as_str() {
        "routing" => {
            let mut out: Vec<String> = CONDITION_KINDS.iter().map(|s| (*s).to_string()).collect();
            out.extend(
                ["domainRegex", "ipversion", "dscp"]
                    .into_iter()
                    .map(str::to_string),
            );
            return Ok(out);
        }
        "request" => DNS_REQUEST_KINDS,
        "response" => DNS_RESPONSE_KINDS,
        _ => {
            return Err("TypeError: dnsConditionKinds[kind] is not iterable".to_string());
        }
    };
    let mut out: Vec<String> = list.iter().map(|s| (*s).to_string()).collect();
    out.push("qnameRegex".to_string());
    Ok(out)
}

/// Port of `serializeBuilder` (`apps/lite/src/ruleDraft.ts:20`).
pub fn serialize_builder(rows: Value) -> Result<Option<String>, String> {
    if rows.is_null() {
        return Err("TypeError: Cannot read properties of null (reading 'length')".to_string());
    }
    if js::is_undefined(&rows) {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
        );
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
    let mut results: Vec<Option<String>> = Vec::with_capacity(arr.len());
    for row in arr {
        let ro = ensure_not_nullish(row, "kind")?;
        let kind_s = ro
            .and_then(|o| obj_defined(o, "kind"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some((name, prefix)) = extra_lookup(kind_s) else {
            let ordinary = dae_core::port::serialize_conditions(Value::Array(vec![row.clone()]))?;
            results.push(ordinary);
            continue;
        };
        let Some(val_v) = ro.and_then(|o| obj_defined(o, "value")) else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'trim')".to_string(),
            );
        };
        let Some(val_s) = val_v.as_str() else {
            return Err("TypeError: text.trim is not a function".to_string());
        };
        let Some(values) = parse_condition_values_vec(val_s) else {
            results.push(None);
            continue;
        };
        if values.is_empty() || values.iter().any(|v| !is_quotable_bool(v)) {
            results.push(None);
            continue;
        }
        let negate = ro
            .and_then(|o| obj_defined(o, "negate"))
            .is_some_and(is_truthy);
        let mut args = Vec::with_capacity(values.len());
        for v in &values {
            let q = quote_str(v)?;
            if prefix.is_empty() {
                args.push(q);
            } else {
                args.push(format!("{prefix}: {q}"));
            }
        }
        let call = format!(
            "{}{name}({})",
            if negate { "!" } else { "" },
            args.join(", ")
        );
        results.push(is_fragment_bool(&call).then_some(call));
    }
    if results.iter().any(Option::is_none) {
        Ok(None)
    } else {
        Ok(Some(
            results
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" && "),
        ))
    }
}

/// Port of `findBlocks` (`apps/lite/src/ruleDraft.ts:81`).
pub fn find_blocks(text: String, path: Vec<String>) -> Result<Vec<Value>, String> {
    Ok(find_blocks_raw(&text, &path)
        .into_iter()
        .map(|b| b.to_value())
        .collect())
}

/// Port of `parseRuleList` (`apps/lite/src/ruleDraft.ts:90`).
pub fn parse_rule_list(text: String, kind: String) -> Result<Value, String> {
    Ok(parse_rule_list_raw(&text, &kind)?.to_value())
}

/// Port of `writeRuleList` (`apps/lite/src/ruleDraft.ts:151`).
pub fn write_rule_list(doc: Value, rows: Value) -> Result<String, String> {
    let rows_val = if js::is_undefined(&rows) {
        ensure_not_nullish(&doc, "rows")?
            .and_then(|o| o.get("rows"))
            .cloned()
            .unwrap_or_else(js::undefined)
    } else {
        rows
    };
    let rows_len = js_length_val(&rows_val)?;
    let doc_rows = ensure_not_nullish(&doc, "rows")?
        .and_then(|o| o.get("rows"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let doc_rows_len = js_length_val(&doc_rows)?;
    if !js_strict_eq(&rows_len, &doc_rows_len) {
        return Err("Rule slot count changed".to_string());
    }
    let doc_text = doc
        .as_object()
        .and_then(|o| obj_defined(o, "text"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "TypeError: Cannot read properties of undefined (reading 'slice')".to_string()
        })?;
    let units = utf16::units(doc_text);
    let count = js::as_number(&doc_rows_len)
        .filter(|x| x.is_finite() && *x > 0.0)
        .map(|x| x.ceil() as usize)
        .unwrap_or(0);
    let doc_arr = doc_rows.as_array();
    let rows_arr = rows_val.as_array();
    let mut result = String::new();
    let mut cursor: f64 = 0.0;
    for i in 0..count {
        let slot = doc_arr.and_then(|a| a.get(i)).ok_or_else(|| {
            "TypeError: Cannot read properties of undefined (reading 'from')".to_string()
        })?;
        let slot_o = ensure_not_nullish(slot, "from")?;
        let slot_from = match slot_o.and_then(|o| obj_defined(o, "from")) {
            Some(v) => js_val_to_number(v),
            None => units.len() as f64,
        };
        let row_i = rows_arr.and_then(|a| a.get(i)).ok_or_else(|| {
            "TypeError: Cannot read properties of undefined (reading 'raw')".to_string()
        })?;
        let row_o = ensure_not_nullish(row_i, "raw")?;
        let raw_v = row_o
            .and_then(|o| o.get("raw"))
            .cloned()
            .unwrap_or_else(js::undefined);
        result.push_str(&js_slice_f64(&units, cursor, slot_from));
        result.push_str(&js_to_string(&raw_v));
        cursor = slot_o
            .and_then(|o| o.get("to"))
            .map(js_val_to_number)
            .unwrap_or(f64::NAN);
    }
    result.push_str(&js_slice_f64(&units, cursor, units.len() as f64));
    Ok(result)
}

/// Port of `moveRule` (`apps/lite/src/ruleDraft.ts:162`).
pub fn move_rule(text: String, kind: String, from: f64, to: f64) -> Result<String, String> {
    let doc = parse_rule_list_raw(&text, &kind)?;
    let slots: Vec<usize> = doc
        .rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| (!r.fallback).then_some(i))
        .collect();
    if from == to {
        return Ok(text);
    }
    let (Some(fi), Some(ti)) = (
        valid_array_index(from, slots.len()),
        valid_array_index(to, slots.len()),
    ) else {
        return Ok(text);
    };
    let mut ordered: Vec<usize> = slots.clone();
    let moved = ordered.remove(fi);
    ordered.insert(ti, moved);
    let mut row_order: Vec<usize> = (0..doc.rows.len()).collect();
    for (&slot, &src) in slots.iter().zip(ordered.iter()) {
        row_order[slot] = src;
    }
    let units = utf16::units(&doc.text);
    let mut result = String::new();
    let mut cursor = 0usize;
    for (i, slot) in doc.rows.iter().enumerate() {
        result.push_str(&js_slice(&units, cursor, slot.from));
        result.push_str(&doc.rows[row_order[i]].raw);
        cursor = slot.to;
    }
    result.push_str(&js_slice_from(&units, cursor));
    Ok(result)
}

/// Port of `ensureBlock` (`apps/lite/src/ruleDraft.ts:173`).
pub fn ensure_block(text: String, path: Vec<String>) -> Result<String, String> {
    ensure_block_inner(&text, &path)
}

/// Port of `addDraftRule` (`apps/lite/src/ruleDraft.ts:185`).
pub fn add_draft_rule(
    text: String,
    kind: String,
    condition: String,
    target: String,
    position: Option<String>,
) -> Result<String, String> {
    let path = list_path(&kind);
    let text = ensure_block_inner(&text, &path)?;
    let doc = parse_rule_list_raw(&text, &kind)?;
    let Some(block) = doc.block else {
        return Err("Ambiguous block".to_string());
    };
    if doc.ambiguous {
        return Err("Ambiguous block".to_string());
    }
    let units = utf16::units(&text);
    let fallback = doc.rows.iter().find(|r| r.fallback);
    let line = line_start_u16(&units, block.close);
    let default_at = if is_space_or_tab(&js_slice(&units, line, block.close)) {
        line
    } else {
        block.close
    };
    let at = if position.as_deref() == Some("first") {
        doc.rows.first().map(|r| r.from)
    } else {
        fallback.map(|r| r.from)
    }
    .unwrap_or(default_at);
    let fallback_indent = "  ".repeat(path.len());
    let indent = doc
        .rows
        .first()
        .map(|r| r.anchor_indent.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&fallback_indent);
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
        ""
    } else {
        eol
    };
    let rendered = rule_line_str(&condition, &target, false);
    Ok(format!(
        "{}{sep}{indent}{rendered}{eol}{}",
        js_slice(&units, 0, at),
        js_slice_from(&units, at)
    ))
}

/// Port of `editDraftRule` (`apps/lite/src/ruleDraft.ts:196`).
pub fn edit_draft_rule(
    text: String,
    rule: Value,
    conditions: Value,
    target: String,
) -> Result<String, String> {
    let Some(condition) = serialize_builder(conditions.clone())? else {
        return Err("Invalid condition".to_string());
    };
    let ro = ensure_not_nullish(&rule, "conditions")?;
    let rule_conds = ro
        .and_then(|o| o.get("conditions"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let unchanged = js_json_stringify(&conditions) == js_json_stringify(&rule_conds);
    let chosen_cond = if unchanged {
        ro.and_then(|o| o.get("expression"))
            .map(js_to_string)
            .unwrap_or_else(|| "undefined".to_string())
    } else {
        condition
    };
    let anchor = ro
        .and_then(|o| o.get("anchor"))
        .cloned()
        .unwrap_or_else(js::undefined);
    Ok(
        dae_core::port::replace_rule(text.clone(), anchor, chosen_cond, target, false)?
            .unwrap_or(text),
    )
}

/// Port of `setFallback` (`apps/lite/src/ruleDraft.ts:203`).
pub fn set_fallback(text: String, kind: String, target: String) -> Result<String, String> {
    let path = list_path(&kind);
    let text = ensure_block_inner(&text, &path)?;
    let doc = parse_rule_list_raw(&text, &kind)?;
    if let Some(row) = doc.rows.iter().find(|r| r.fallback) {
        return Ok(dae_core::port::replace_rule(
            text.clone(),
            row.anchor.clone(),
            String::new(),
            target,
            false,
        )?
        .unwrap_or(text));
    }
    let Some(block) = doc.block else {
        return Err("Ambiguous block".to_string());
    };
    if doc.ambiguous {
        return Err("Ambiguous block".to_string());
    }
    let units = utf16::units(&text);
    let line = line_start_u16(&units, block.close);
    let at = if is_space_or_tab(&js_slice(&units, line, block.close)) {
        line
    } else {
        block.close
    };
    let fallback_indent = "  ".repeat(path.len());
    let indent = doc
        .rows
        .first()
        .map(|r| r.anchor_indent.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&fallback_indent);
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
        ""
    } else {
        eol
    };
    Ok(format!(
        "{}{sep}{indent}fallback: {target}{eol}{}",
        js_slice(&units, 0, at),
        js_slice_from(&units, at)
    ))
}

/// Port of `changeCount` (`apps/lite/src/ruleDraft.ts:216`).
pub fn change_count(before: String, after: String) -> Result<f64, String> {
    if before == after {
        return Ok(0.0);
    }
    let a_all: Vec<&str> = before.split('\n').collect();
    let b_all: Vec<&str> = after.split('\n').collect();
    let mut start = 0usize;
    while start < a_all.len() && start < b_all.len() && a_all[start] == b_all[start] {
        start += 1;
    }
    let mut end = 0usize;
    while end < a_all.len() - start
        && end < b_all.len() - start
        && a_all[a_all.len() - 1 - end] == b_all[b_all.len() - 1 - end]
    {
        end += 1;
    }
    let a = &a_all[start..a_all.len() - end];
    let b = &b_all[start..b_all.len() - end];
    if a.len().saturating_mul(b.len()) > 1_000_000 {
        return Ok(a.len().max(b.len()) as f64);
    }
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut prev = vec![0usize; short.len() + 1];
    let mut curr = vec![0usize; short.len() + 1];
    for &lb in long.iter().rev() {
        for (j, &sa) in short.iter().enumerate().rev() {
            curr[j] = if sa == lb {
                prev[j + 1] + 1
            } else {
                prev[j].max(curr[j + 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    let lcs_len = prev[0];
    Ok((a.len() - lcs_len).max(b.len() - lcs_len) as f64)
}

/// Port of `stageAuthoredRules` (`apps/lite/src/ruleDraft.ts:222`).
pub fn stage_authored_rules(before: String, after: String) -> Result<String, String> {
    if dae_core::port::write_mode(before.clone(), json!({"mode": "rule"}))? == after {
        return Ok(before);
    }
    let mode = dae_core::port::read_mode(before.clone())?;
    if mode.get("mode").and_then(Value::as_str) == Some("rule") {
        return Ok(after);
    }
    let lines: Vec<&str> = before.split_inclusive('\n').collect();
    let mode_lines = dae_core::port::mode_rule_lines(before.clone())?;
    let mut owned = String::new();
    if let Some(arr) = mode_lines.as_array() {
        for item in arr {
            if let Some(n) = item.as_u64().and_then(|v| usize::try_from(v).ok())
                && n > 0
                && let Some(line) = lines.get(n - 1)
            {
                owned.push_str(line);
            }
        }
    }
    let doc = parse_rule_list_raw(&after, "routing")?;
    let after_u = utf16::units(&after);
    let first = doc
        .rows
        .iter()
        .find(|row| row.fallback || !ends_with_must(&row.target));
    let at = match first {
        Some(row) => row.from,
        None => {
            let Some(block) = doc.block else {
                return Err(
                    "TypeError: Cannot read properties of null (reading 'close')".to_string(),
                );
            };
            line_start_u16(&after_u, block.close)
        }
    };
    Ok(format!(
        "{}{owned}{}",
        js_slice(&after_u, 0, at),
        js_slice_from(&after_u, at)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: &str = "routing {\n  # first\n  domain(suffix: example.net) -> direct\n  domain(regex: '^foo') -> proxy\n  fallback: proxy\n}\n";

    #[test]
    fn kinds_and_builder_roundtrip() {
        assert!(
            kinds_for("routing".into())
                .unwrap()
                .contains(&"domainRegex".to_string())
        );
        assert!(
            kinds_for("request".into())
                .unwrap()
                .contains(&"qnameRegex".to_string())
        );
        assert!(
            kinds_for("response".into())
                .unwrap()
                .contains(&"answerIp".to_string())
        );
        assert!(kinds_for("invalid".into()).is_err());

        let rows = json!([
            {"kind": "domainRegex", "value": "^a\\.b$", "negate": true},
            {"kind": "dport", "value": "443", "negate": false}
        ]);
        let serialized = serialize_builder(rows).unwrap().unwrap();
        assert_eq!(serialized, "!domain(regex: '^a\\.b$') && dport(443)");
        assert_eq!(serialize_builder(json!([])).unwrap(), None);
        assert_eq!(serialize_builder(json!(0)).unwrap(), None);
    }

    #[test]
    fn parse_move_add_edit_and_fallback() {
        let doc = parse_rule_list(CFG.to_string(), "routing".into()).unwrap();
        assert_eq!(doc["ambiguous"], false);
        assert_eq!(doc["rows"].as_array().unwrap().len(), 3);
        assert_eq!(doc["rows"][0]["note"], "first");
        assert_eq!(doc["rows"][1]["conditions"][0]["kind"], "domainRegex");

        let moved = move_rule(CFG.to_string(), "routing".into(), 0.0, 1.0).unwrap();
        assert!(moved.find("regex").unwrap() < moved.find("example.net").unwrap());
        assert_eq!(
            move_rule(CFG.to_string(), "routing".into(), f64::NAN, 0.0).unwrap(),
            CFG
        );

        let added = add_draft_rule(
            CFG.to_string(),
            "routing".into(),
            "dport(80)".into(),
            "direct".into(),
            Some("first".into()),
        )
        .unwrap();
        assert!(added.find("dport(80)").unwrap() < added.find("# first").unwrap());

        let edited = edit_draft_rule(
            CFG.to_string(),
            doc["rows"][0].clone(),
            doc["rows"][0]["conditions"].clone(),
            "block".into(),
        )
        .unwrap();
        assert!(edited.contains("domain(suffix: example.net) -> block"));

        let fb = set_fallback(CFG.to_string(), "routing".into(), "direct".into()).unwrap();
        assert!(fb.contains("fallback: direct"));
        assert!(ensure_block("".into(), vec![]).is_err());
    }

    #[test]
    fn change_count_and_stage_authored_rules() {
        assert_eq!(change_count("a\nb\n".into(), "a\nb\n".into()).unwrap(), 0.0);
        assert_eq!(
            change_count("a\nb\nc".into(), "a\nx\ny\nc".into()).unwrap(),
            2.0
        );

        let before = "routing {\n  pname(daemon) -> direct(must)\n  l4proto(tcp, udp) -> proxy # doona: outbound mode\n  fallback: proxy\n}\n";
        let after = "routing {\n  domain(suffix: example.com) -> direct\n  pname(daemon) -> direct(must)\n  fallback: proxy\n}\n";
        let staged = stage_authored_rules(before.into(), after.into()).unwrap();
        assert!(staged.contains("# doona: outbound mode"));
        assert!(
            staged.find("# doona: outbound mode").unwrap() < staged.find("example.com").unwrap()
        );
    }
}
