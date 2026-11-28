//! Port of `src/dae/settings.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    apply_utf16_edits, first_indented_line, is_truthy, js_slice, js_str_to_number, js_trim,
    leading_js_ws_u16_len, obj_defined,
};
use crate::port::text::{
    block_fields_raw, is_quotable_bool, quote_str, scan_config_raw, unquote_str,
};
use crate::utf16;
use num_bigint::BigInt;
use serde_json::Value;

fn parse_js_bigint(s: &str) -> Result<BigInt, String> {
    let t = js_trim(s);
    if t.is_empty() {
        return Ok(BigInt::from(0));
    }
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return BigInt::parse_bytes(rest.as_bytes(), 16)
            .ok_or_else(|| format!("SyntaxError: Cannot convert {s} to a BigInt"));
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        return BigInt::parse_bytes(rest.as_bytes(), 2)
            .ok_or_else(|| format!("SyntaxError: Cannot convert {s} to a BigInt"));
    }
    if let Some(rest) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
        return BigInt::parse_bytes(rest.as_bytes(), 8)
            .ok_or_else(|| format!("SyntaxError: Cannot convert {s} to a BigInt"));
    }
    let (neg, digits) = if let Some(r) = t.strip_prefix('-') {
        (true, r)
    } else if let Some(r) = t.strip_prefix('+') {
        (false, r)
    } else {
        (false, t)
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("SyntaxError: Cannot convert {s} to a BigInt"));
    }
    let abs = BigInt::parse_bytes(digits.as_bytes(), 10)
        .ok_or_else(|| format!("SyntaxError: Cannot convert {s} to a BigInt"))?;
    Ok(if neg { -abs } else { abs })
}

pub fn number_setting(field: Value) -> Result<bool, String> {
    let Some(o) = field.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'type')".to_string());
    };
    if obj_defined(o, "type").and_then(Value::as_str) != Some("integer") {
        return Ok(false);
    }
    if obj_defined(o, "choices").is_some_and(is_truthy) {
        return Ok(false);
    }
    if obj_defined(o, "hexMax").is_some_and(is_truthy) {
        return Ok(false);
    }
    let Some(max_v) = obj_defined(o, "max") else {
        return Ok(false);
    };
    let max_s = crate::port::js_str::js_to_string(max_v);
    let max_bi = parse_js_bigint(&max_s)?;
    Ok(max_bi <= BigInt::from(9_007_199_254_740_991i64))
}

fn setting_items(value: &str) -> Vec<String> {
    let scan = scan_config_raw(value);
    let units = utf16::units(value);
    let commas: Vec<usize> = scan
        .tokens
        .iter()
        .filter(|t| js_slice(&units, t.from, t.to) == ",")
        .map(|t| t.from)
        .collect();
    let mut bounds: Vec<isize> = Vec::with_capacity(commas.len() + 2);
    bounds.push(-1);
    for c in commas {
        bounds.push(c as isize);
    }
    bounds.push(units.len() as isize);
    bounds
        .windows(2)
        .map(|w| {
            let start = (w[0] + 1) as usize;
            let end = w[1] as usize;
            let slice = js_slice(&units, start, end);
            unquote_str(js_trim(&slice))
        })
        .collect()
}

pub fn setting_value(field: Value, value: String) -> Result<String, String> {
    let fo = crate::port::js_str::ensure_not_nullish(&field, "type")?;
    let text = unquote_str(&value);
    let ftype = fo
        .and_then(|o| obj_defined(o, "type"))
        .and_then(Value::as_str);
    if ftype == Some("boolean") {
        let lower = text.to_ascii_lowercase();
        if matches!(lower.as_str(), "true" | "yes" | "1" | "on") {
            return Ok("true".to_string());
        }
        if matches!(lower.as_str(), "false" | "no" | "0" | "off") {
            return Ok("false".to_string());
        }
    }
    if let Some(hex_max_v) = fo
        .and_then(|o| obj_defined(o, "hexMax"))
        .filter(|v| is_truthy(v))
    {
        let digits = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(&text);
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            let hex_max_s = crate::port::js_str::js_to_string(hex_max_v);
            let hex_max = parse_js_bigint(&hex_max_s)?;
            let hex = parse_js_bigint(&format!("0x{digits}"))?;
            if hex <= hex_max {
                return Ok(hex.to_string());
            }
            if digits.bytes().all(|b| b.is_ascii_digit()) {
                let dec = parse_js_bigint(digits)?;
                if dec <= hex_max {
                    return Ok(dec.to_string());
                }
            }
        }
    }
    if ftype == Some("list") {
        let mut out = Vec::new();
        for item in setting_items(&value) {
            if item.contains(',') {
                out.push(quote_str(&item)?);
            } else {
                out.push(item);
            }
        }
        return Ok(out.join(", "));
    }
    Ok(text)
}

fn parse_duration_parts(value: &str) -> Option<(&str, &str)> {
    let (num, unit) = if let Some(n) = value.strip_suffix("ms") {
        (n, "ms")
    } else if let Some(n) = value.strip_suffix('s') {
        (n, "s")
    } else if let Some(n) = value.strip_suffix('m') {
        (n, "m")
    } else if let Some(n) = value.strip_suffix('h') {
        (n, "h")
    } else {
        (value, "")
    };
    if num.is_empty() {
        return None;
    }
    match num.find('.') {
        Some(dot) => {
            let a = &num[..dot];
            let b = &num[dot + 1..];
            if a.is_empty()
                || b.is_empty()
                || !a.bytes().all(|c| c.is_ascii_digit())
                || !b.bytes().all(|c| c.is_ascii_digit())
            {
                return None;
            }
        }
        None => {
            if !num.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
        }
    }
    Some((num, unit))
}

fn serialize_setting_ref(field: &Value, value: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        return Ok(Some(String::new()));
    }
    if field.is_null() || js::is_undefined(field) {
        return Err("TypeError: Cannot read properties of null (reading 'choices')".to_string());
    }
    let o = field.as_object();
    let choices = o
        .and_then(|m| obj_defined(m, "choices"))
        .filter(|v| !v.is_null());
    if let Some(ch) = choices {
        match ch {
            Value::Array(arr) => {
                if arr.iter().any(|c| c.as_str() == Some(value)) {
                    return Ok(Some(value.to_string()));
                }
            }
            Value::String(s) => {
                if s.contains(value) {
                    return Ok(Some(value.to_string()));
                }
            }
            _ => return Err("TypeError: field.choices.includes is not a function".to_string()),
        }
    }
    let ftype = o
        .and_then(|m| obj_defined(m, "type"))
        .and_then(Value::as_str);
    if ftype == Some("boolean") {
        return Ok((value == "true" || value == "false").then(|| value.to_string()));
    }
    if ftype == Some("integer") {
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(None);
        }
        let max_v = o
            .and_then(|m| m.get("max"))
            .cloned()
            .unwrap_or_else(js::undefined);
        let max_bi = parse_js_bigint(&crate::port::js_str::js_to_string(&max_v))?;
        let val_bi = parse_js_bigint(value)?;
        if val_bi > max_bi {
            return Ok(None);
        }
        let is_hex = o
            .and_then(|m| obj_defined(m, "hexMax"))
            .is_some_and(is_truthy);
        return Ok(Some(if is_hex {
            format!("0x{:x}", val_bi)
        } else {
            value.to_string()
        }));
    }
    if choices.is_some_and(is_truthy) {
        return Ok(None);
    }
    if ftype == Some("duration") {
        let Some((num, unit)) = parse_duration_parts(value) else {
            return Ok(None);
        };
        let Some(units_arr) = o
            .and_then(|m| obj_defined(m, "units"))
            .and_then(Value::as_array)
        else {
            return Ok(None);
        };
        if !units_arr.iter().any(|u| u.as_str() == Some(unit)) {
            return Ok(None);
        }
        let seconds = units_arr.iter().any(|u| u.as_str() == Some("h"));
        let integer = if seconds { unit != "ms" } else { unit == "ms" };
        if integer {
            if num.contains('.') {
                return Ok(None);
            }
            let scale: u64 = match unit {
                "h" => 3600,
                "m" => 60,
                _ => 1,
            };
            let max_u64 = BigInt::from(u64::MAX);
            let n = parse_js_bigint(num)?;
            return Ok((n * BigInt::from(scale) <= max_u64).then(|| value.to_string()));
        }
        let n_f = js_str_to_number(num);
        let duration = if seconds {
            (n_f / 1000.0).ceil()
        } else {
            n_f * if unit == "s" { 1000.0 } else { 1.0 }
        };
        return Ok((duration < 18_446_744_073_709_551_616.0).then(|| value.to_string()));
    }
    let values: Vec<String> = if ftype == Some("list") {
        setting_items(value)
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![value.to_string()]
    };
    if ftype == Some("list") && values.len() == 1 && values[0].contains(',') {
        return Ok(None);
    }
    let bare_items = o
        .and_then(|m| obj_defined(m, "bareItems"))
        .is_some_and(is_truthy);
    if bare_items {
        let ok = !values.is_empty()
            && values.iter().all(|v| {
                !v.is_empty()
                    && v.bytes().all(|b| {
                        b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'*')
                    })
            });
        return Ok(ok.then(|| values.join(", ")));
    }
    if !values.is_empty() && values.iter().all(|v| is_quotable_bool(v)) {
        let quoted = values
            .iter()
            .map(|v| quote_str(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(quoted.join(", ")))
    } else {
        Ok(None)
    }
}

pub fn serialize_setting(field: Value, value: String) -> Result<Option<String>, String> {
    serialize_setting_ref(&field, &value)
}

pub fn write_settings(
    text: String,
    section: Value,
    index: f64,
    patch: Value,
) -> Result<String, String> {
    let Some(so) = section.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'name')".to_string());
    };
    let sec_name = obj_defined(so, "name").and_then(Value::as_str);
    let scan = scan_config_raw(&text);
    let units = utf16::units(&text);
    let matching_blocks: Vec<_> = scan
        .blocks
        .iter()
        .filter(|b| Some(b.name.as_str()) == sec_name)
        .collect();
    let idx_opt = if index.is_finite() && index >= 0.0 && index.fract() == 0.0 {
        Some(index as usize)
    } else {
        None
    };
    let block = idx_opt.and_then(|i| matching_blocks.get(i).copied());
    let fields = match block {
        Some(b) => block_fields_raw(&text, b, &scan.tokens),
        None => Vec::new(),
    };
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    let indent = block
        .and_then(|b| first_indented_line(&js_slice(&units, b.open + 1, b.close)))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "  ".to_string());
    let Some(po) = patch.as_object() else {
        if patch.is_null() || js::is_undefined(&patch) {
            return Err("TypeError: Cannot convert undefined or null to object".to_string());
        }
        return Ok(text);
    };
    let sec_fields = obj_defined(so, "fields").and_then(Value::as_array);
    for (key, val_v) in po {
        let Some(sf) = sec_fields else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'find')".to_string(),
            );
        };
        let value = crate::port::js_str::js_to_string(val_v);
        let definition = sf
            .iter()
            .find(|f| f.get("key").and_then(Value::as_str) == Some(key.as_str()));
        let serialized = match definition {
            Some(def) => serialize_setting_ref(def, &value)?,
            None => None,
        };
        let Some(serialized) = serialized else {
            return Err("config.globalInvalid".to_string());
        };
        let matches: Vec<_> = fields.iter().filter(|f| f.name == *key).collect();
        if matches.len() > 1 {
            return Err("config.globalDuplicate".to_string());
        }
        if value.is_empty() {
            if let Some(m) = matches.first() {
                edits.push((m.from, m.to, String::new()));
            }
        } else if let Some(field) = matches.first() {
            let lead = leading_js_ws_u16_len(&js_slice(&units, field.value_from, field.value_to));
            edits.push((field.value_from + lead, field.value_to, serialized));
        } else {
            added.push(format!("{indent}{key}: {serialized}"));
        }
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    if !added.is_empty() {
        let body = format!("{}{newline}", added.join(newline));
        if let Some(b) = block {
            edits.push((b.close, b.close, format!("{newline}{body}")));
        } else {
            let prefix = if !text.is_empty() && !text.ends_with('\n') {
                newline
            } else {
                ""
            };
            let sname = sec_name.unwrap_or("undefined");
            edits.push((
                units.len(),
                units.len(),
                format!("{prefix}{sname} {{{newline}{body}}}{newline}"),
            ));
        }
    }
    Ok(apply_utf16_edits(&text, edits, false))
}
