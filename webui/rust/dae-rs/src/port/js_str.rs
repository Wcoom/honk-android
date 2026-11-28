//! Shared JS-compatible string, UTF-16, URL, base64, and Value helpers.

#![forbid(unsafe_code)]

use crate::js;
use crate::utf16;
use serde_json::Value;

/// Whether a UTF-16 code unit matches ECMAScript `/\s/` (and `String.prototype.trim`).
/// Notably includes U+FEFF (BOM) and excludes U+0085 (NEL).
pub fn is_js_ws_u16(u: u16) -> bool {
    matches!(
        u,
        0x0009..=0x000D
            | 0x0020
            | 0x00A0
            | 0x1680
            | 0x2000..=0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
            | 0xFEFF
    )
}

pub fn is_js_ws(c: char) -> bool {
    u16::try_from(c as u32).is_ok_and(is_js_ws_u16)
}

pub fn js_trim(s: &str) -> &str {
    s.trim_matches(is_js_ws)
}

pub fn js_trim_start(s: &str) -> &str {
    s.trim_start_matches(is_js_ws)
}

pub fn js_trim_end(s: &str) -> &str {
    s.trim_end_matches(is_js_ws)
}

pub fn js_slice(units: &[u16], start: usize, end: usize) -> String {
    utf16::slice(units, start, end)
}

pub fn js_slice_from(units: &[u16], start: usize) -> String {
    utf16::slice(units, start, units.len())
}

/// JS `str.lastIndexOf(singleChar, from)` where `from` may be negative (clamped to 0).
pub fn last_index_of_u16(units: &[u16], needle: u16, from: isize) -> Option<usize> {
    if units.is_empty() {
        return None;
    }
    let pos = if from < 0 {
        0
    } else {
        (from as usize).min(units.len() - 1)
    };
    (0..=pos).rev().find(|&i| units[i] == needle)
}

/// JS `str.lastIndexOf('\n', at - 1) + 1`.
pub fn line_start_u16(units: &[u16], at: usize) -> usize {
    match last_index_of_u16(units, b'\n' as u16, at as isize - 1) {
        Some(pos) => pos + 1,
        None => 0,
    }
}

/// JS `str.indexOf('\n', from)`.
pub fn index_of_u16(units: &[u16], needle: u16, from: usize) -> Option<usize> {
    if from >= units.len() {
        return None;
    }
    units[from..]
        .iter()
        .position(|&u| u == needle)
        .map(|i| i + from)
}

pub fn is_space_or_tab(s: &str) -> bool {
    s.bytes().all(|b| b == b' ' || b == b'\t')
}

pub fn is_space_tab_cr(s: &str) -> bool {
    s.bytes().all(|b| b == b' ' || b == b'\t' || b == b'\r')
}

/// Matches JS `/\n([ \t]+)\S/` (where `\S` is `!is_js_ws`).
pub fn first_indented_line(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j > start
                && let Some(ch) = s[j..].chars().next()
                && !is_js_ws(ch)
            {
                return Some(s[start..j].to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

/// Matches JS `/^[ \t]*/` at `s`.
pub fn leading_space_tabs(s: &str) -> &str {
    let n = s.bytes().take_while(|&b| b == b' ' || b == b'\t').count();
    &s[..n]
}

/// Matches JS `/^\s*/` at `s` returning UTF-16 length.
pub fn leading_js_ws_u16_len(s: &str) -> usize {
    s.chars()
        .take_while(|&c| is_js_ws(c))
        .map(char::len_utf16)
        .sum()
}

/// JS truthiness of a `serde_json::Value`.
pub fn is_truthy(v: &Value) -> bool {
    if js::is_undefined(v) {
        return false;
    }
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) => true,
        Value::Object(o) => {
            if let Some(x) = js::as_number(v) {
                return x != 0.0 && !x.is_nan();
            }
            if o.len() == 1 && o.contains_key("$utf16") {
                return o["$utf16"].as_array().is_some_and(|a| !a.is_empty());
            }
            true
        }
    }
}

/// JS `String(v)` for primitives/values used in configs.
pub fn js_to_string(v: &Value) -> String {
    if js::is_undefined(v) {
        return "undefined".to_string();
    }
    if let Some(n) = js::as_number(v) {
        return js::number_to_string(n);
    }
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => js::number_to_string(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .map(|x| {
                if x.is_null() || js::is_undefined(x) {
                    String::new()
                } else {
                    js_to_string(x)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// JS `Number(s)` for string inputs.
pub fn js_str_to_number(s: &str) -> f64 {
    let t = js_trim(s);
    if t.is_empty() {
        return 0.0;
    }
    if t == "Infinity" || t == "+Infinity" {
        return f64::INFINITY;
    }
    if t == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return f64::NAN;
        }
        return u128::from_str_radix(rest, 16)
            .map(|v| v as f64)
            .unwrap_or(f64::INFINITY);
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        if rest.is_empty() || !rest.bytes().all(|b| b == b'0' || b == b'1') {
            return f64::NAN;
        }
        return u128::from_str_radix(rest, 2)
            .map(|v| v as f64)
            .unwrap_or(f64::INFINITY);
    }
    if let Some(rest) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        if rest.is_empty() || !rest.bytes().all(|b| matches!(b, b'0'..=b'7')) {
            return f64::NAN;
        }
        return u128::from_str_radix(rest, 8)
            .map(|v| v as f64)
            .unwrap_or(f64::INFINITY);
    }
    // Reject Rust-only float spellings like "inf", "nan", "NaN".
    if t.bytes().any(|b| matches!(b, b'i' | b'I' | b'n' | b'N')) {
        return f64::NAN;
    }
    t.parse::<f64>().unwrap_or(f64::NAN)
}

/// JS `Number(value)` for any JSON value.
pub fn js_val_to_number(v: &Value) -> f64 {
    if js::is_undefined(v) {
        return f64::NAN;
    }
    if let Some(x) = js::as_number(v) {
        return x;
    }
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => js_str_to_number(s),
        Value::Array(a) if a.is_empty() => 0.0,
        Value::Array(a) if a.len() == 1 => js_val_to_number(&a[0]),
        _ => f64::NAN,
    }
}

/// JS `Number.parseInt(s, 10)` returning `f64` (may be NaN).
pub fn js_parse_int_10(s: &str) -> f64 {
    let t = js_trim_start(s);
    let (neg, rest) = if let Some(r) = t.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = t.strip_prefix('+') {
        (false, r)
    } else {
        (false, t)
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return f64::NAN;
    }
    let val = digits.parse::<f64>().unwrap_or(f64::INFINITY);
    if neg { -val } else { val }
}

/// JS `encodeURIComponent(s)`.
pub fn encode_uri_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            );
        if keep {
            out.push(b as char);
        } else {
            push_pct(&mut out, b);
        }
    }
    out
}

fn push_pct(out: &mut String, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(HEX[(b >> 4) as usize] as char);
    out.push(HEX[(b & 0x0f) as usize] as char);
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// JS `decodeURIComponent(s)`.
pub fn decode_uri_component(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err("URIError: URI malformed".to_string());
            }
            let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) else {
                return Err("URIError: URI malformed".to_string());
            };
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "URIError: URI malformed".to_string())
}

/// WHATWG `application/x-www-form-urlencoded` decoding (for `URLSearchParams`).
fn form_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// WHATWG `application/x-www-form-urlencoded` encoding (for `URLSearchParams.toString`).
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b == b' ' {
            out.push('+');
        } else if b.is_ascii_alphanumeric() || matches!(b, b'*' | b'-' | b'.' | b'_') {
            out.push(b as char);
        } else {
            push_pct(&mut out, b);
        }
    }
    out
}

/// WHATWG `URLSearchParams` implementation.
#[derive(Clone, Debug, Default)]
pub struct UrlSearchParams {
    pub pairs: Vec<(String, String)>,
}

impl UrlSearchParams {
    pub fn parse(query: &str) -> Self {
        let q = query.strip_prefix('?').unwrap_or(query);
        let mut pairs = Vec::new();
        for part in q.split('&') {
            if part.is_empty() {
                continue;
            }
            let (k, v) = match part.find('=') {
                Some(eq) => (&part[..eq], &part[eq + 1..]),
                None => (part, ""),
            };
            pairs.push((form_decode(k), form_decode(v)));
        }
        Self { pairs }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn has(&self, key: &str) -> bool {
        self.pairs.iter().any(|(k, _)| k == key)
    }

    pub fn delete(&mut self, key: &str) {
        self.pairs.retain(|(k, _)| k != key);
    }

    pub fn set(&mut self, key: &str, value: String) {
        let mut found = false;
        self.pairs.retain_mut(|(k, v)| {
            if k == key {
                if !found {
                    *v = value.clone();
                    found = true;
                    true
                } else {
                    false
                }
            } else {
                true
            }
        });
        if !found {
            self.pairs.push((key.to_string(), value));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn serialize(&self) -> String {
        self.pairs
            .iter()
            .map(|(k, v)| format!("{}={}", form_encode(k), form_encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// JS `decode64` from `nodeForm.ts`: URL-safe base64 + `atob` + fatal UTF-8 decode.
pub fn decode_base64_utf8(value: &str) -> Result<String, String> {
    // Strip ASCII whitespace (\t \n \x0c \r space), as forgiving base64 / atob does.
    let mut clean: Vec<u8> = Vec::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'\t' | b'\n' | 0x0C | b'\r' | b' ' => {}
            b'-' => clean.push(b'+'),
            b'_' => clean.push(b'/'),
            _ => clean.push(b),
        }
    }
    if clean.len().is_multiple_of(4) {
        while clean.last() == Some(&b'=') {
            clean.pop();
        }
    }
    if clean.len() % 4 == 1 {
        return Err("InvalidCharacterError: Invalid base64".to_string());
    }
    let mut bytes = Vec::with_capacity(clean.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for &b in &clean {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return Err("InvalidCharacterError: Invalid base64".to_string()),
        };
        acc = (acc << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    String::from_utf8(bytes)
        .map_err(|_| "TypeError: The encoded data was not valid for encoding utf-8".to_string())
}

/// JS `encode64` from `nodeForm.ts`: UTF-8 bytes encoded with standard base64 + `=` padding.
pub fn encode_base64_utf8(value: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Apply UTF-16 text edits (each with UTF-16 `from`, `to`, and replacement `text`),
/// sorted by `b.from.cmp(&a.from)` (stable, matching JS).
pub fn apply_utf16_edits(
    text: &str,
    mut edits: Vec<(usize, usize, String)>,
    tie_by_to: bool,
) -> String {
    if tie_by_to {
        edits.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    } else {
        edits.sort_by_key(|a| std::cmp::Reverse(a.0));
    }
    let mut units = utf16::units(text);
    for (from, to, repl) in edits {
        let end = to.min(units.len());
        let start = from.min(end);
        let repl_u = utf16::units(&repl);
        units.splice(start..end, repl_u);
    }
    String::from_utf16_lossy(&units)
}

/// Convert a JS object field that might be `None` or `$undefined` to `Option<&Value>` (excluding null and undefined).
pub fn obj_defined<'a>(obj: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a Value> {
    obj.get(key).filter(|v| !js::is_undefined(v))
}

/// Read an integer index from a JSON `Value` field (`u64` / `f64`).
pub fn val_as_usize(v: &Value) -> Option<usize> {
    v.as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .or_else(|| {
            js::as_number(v)
                .filter(|x| x.is_finite() && *x >= 0.0)
                .map(|x| x as usize)
        })
}

/// Rejects JS `null` and `undefined` with a TypeError; returns `Ok(Some(map))` for objects,
/// and `Ok(None)` for non-nullish primitives/arrays (whose named property reads yield `undefined`).
pub fn ensure_not_nullish<'a>(
    v: &'a Value,
    prop: &str,
) -> Result<Option<&'a serde_json::Map<String, Value>>, String> {
    if v.is_null() {
        return Err(format!(
            "TypeError: Cannot read properties of null (reading '{prop}')"
        ));
    }
    if js::is_undefined(v) {
        return Err(format!(
            "TypeError: Cannot read properties of undefined (reading '{prop}')"
        ));
    }
    Ok(v.as_object())
}

/// JS `String.prototype.slice(start, end)` with `f64` bounds (handles NaN, Infinity, negatives).
pub fn js_slice_f64(units: &[u16], start: f64, end: f64) -> String {
    let len = units.len();
    let clamp = |x: f64| -> usize {
        if x.is_nan() {
            0
        } else if x.is_infinite() {
            if x > 0.0 { len } else { 0 }
        } else {
            let t = x.trunc();
            if t < 0.0 {
                (len as f64 + t).max(0.0) as usize
            } else {
                t.min(len as f64) as usize
            }
        }
    };
    let s = clamp(start);
    let e = clamp(end);
    if s >= e {
        String::new()
    } else {
        utf16::slice(units, s, e)
    }
}

/// JS `str.lastIndexOf(singleChar, position)` with `f64` position.
pub fn last_index_of_u16_f64(units: &[u16], needle: u16, pos: f64) -> Option<usize> {
    if units.is_empty() {
        return None;
    }
    let idx = if pos.is_nan() || (pos.is_infinite() && pos > 0.0) {
        units.len() - 1
    } else if pos <= 0.0 {
        0
    } else {
        (pos.trunc() as usize).min(units.len() - 1)
    };
    (0..=idx).rev().find(|&i| units[i] == needle)
}
