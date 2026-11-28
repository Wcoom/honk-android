//! Port of `src/dae/outboundMode.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    first_indented_line, index_of_u16, is_space_or_tab, js_slice, js_slice_from, js_trim,
    js_trim_end, line_start_u16, obj_defined,
};
use crate::port::text::{TextBlock, TextToken, TokenKind, scan_config_raw, unquote_str};
use crate::utf16;
use serde_json::{Value, json};

const MODE_MARK: &str = "# doona: outbound mode";

struct FallbackLine {
    value_from: usize,
    value_to: usize,
    target: String,
}

fn routing_fallback_line(
    text: &str,
    block: &TextBlock,
    tokens: &[TextToken],
) -> Option<FallbackLine> {
    let units = utf16::units(text);
    let inside: Vec<&TextToken> = tokens
        .iter()
        .filter(|t| {
            t.from > block.open
                && t.to < block.close
                && t.depth == 1
                && t.kind != TokenKind::Comment
        })
        .collect();
    for (i, token) in inside.iter().enumerate() {
        if i > 0 && inside[i - 1].line == token.line {
            continue;
        }
        let line_tokens: Vec<&TextToken> = inside
            .iter()
            .copied()
            .filter(|item| item.line == token.line)
            .collect();
        let key = js_slice(&units, token.from, token.to);
        if (key != "fallback" && key != "default")
            || line_tokens.len() < 3
            || js_slice(&units, line_tokens[1].from, line_tokens[1].to) != ":"
        {
            continue;
        }
        let value_from = line_tokens[2].from;
        let value_to = line_tokens.last().map_or(value_from, |t| t.to);
        let raw_slice = js_slice(&units, value_from, value_to);
        let stripped = js_trim_end(&raw_slice);
        let without_must = stripped.strip_suffix("(must)").unwrap_or(&raw_slice);
        let raw = js_trim(without_must);
        return Some(FallbackLine {
            value_from,
            value_to,
            target: if raw.is_empty() {
                String::new()
            } else {
                unquote_str(raw)
            },
        });
    }
    None
}

pub fn read_rule_fallback(text: String) -> Result<String, String> {
    let scan = scan_config_raw(&text);
    let Some(block) = scan.blocks.iter().rfind(|b| b.name == "routing") else {
        return Ok(String::new());
    };
    let Some(line) = routing_fallback_line(&text, block, &scan.tokens) else {
        return Ok(String::new());
    };
    if line.target.is_empty() || line.target == "direct" || line.target == "block" {
        return Ok(String::new());
    }
    Ok(line.target)
}

pub fn write_rule_fallback(text: String, target: String) -> Result<String, String> {
    let scan = scan_config_raw(&text);
    let units = utf16::units(&text);
    let Some(block) = scan.blocks.iter().rfind(|b| b.name == "routing") else {
        return Err("act.modeNoRouting".to_string());
    };
    if let Some(line) = routing_fallback_line(&text, block, &scan.tokens) {
        return Ok(format!(
            "{}{target}{}",
            js_slice(&units, 0, line.value_from),
            js_slice_from(&units, line.value_to),
        ));
    }
    let body = js_slice(&units, block.open + 1, block.close);
    let indent = first_indented_line(&body).unwrap_or_else(|| "    ".to_string());
    let before_close = line_start_u16(&units, block.close);
    let at = if is_space_or_tab(&js_slice(&units, before_close, block.close)) {
        before_close
    } else {
        block.close
    };
    let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
        ""
    } else {
        "\n"
    };
    Ok(format!(
        "{}{sep}{indent}fallback: {target}\n{}",
        js_slice(&units, 0, at),
        js_slice_from(&units, at),
    ))
}

struct ModeRange {
    from: usize,
    to: usize,
    target: String,
}

fn mode_ranges(text: &str, blocks: &[TextBlock], tokens: &[TextToken]) -> Vec<ModeRange> {
    let units = utf16::units(text);
    let mut out = Vec::new();
    for token in tokens {
        if token.kind != TokenKind::Comment {
            continue;
        }
        if js_trim(&js_slice(&units, token.from, token.to)) != MODE_MARK {
            continue;
        }
        let in_routing = blocks.iter().any(|b| {
            b.name == "routing" && token.from > b.open && token.to < b.close && token.depth == 1
        });
        if !in_routing {
            continue;
        }
        let from = line_start_u16(&units, token.from);
        let line: Vec<&TextToken> = tokens
            .iter()
            .filter(|item| item.from >= from && item.to <= token.from)
            .collect();
        if line.iter().any(|item| item.depth != 1) {
            continue;
        }
        let arrow = line
            .iter()
            .position(|item| item.parens == 0 && js_slice(&units, item.from, item.to) == "->");
        let target = match arrow {
            Some(idx) if idx + 1 < line.len() => {
                js_trim(&js_slice(&units, line[idx + 1].from, token.from)).to_string()
            }
            _ => String::new(),
        };
        let extra = usize::from(units.get(token.to) == Some(&(b'\n' as u16)));
        out.push(ModeRange {
            from,
            to: token.to + extra,
            target,
        });
    }
    out
}

pub fn read_mode(text: String) -> Result<Value, String> {
    let scan = scan_config_raw(&text);
    let found = mode_ranges(&text, &scan.blocks, &scan.tokens)
        .into_iter()
        .find(|r| !r.target.is_empty());
    Ok(match found {
        None => json!({"mode": "rule"}),
        Some(r) if r.target == "direct" => json!({"mode": "direct"}),
        Some(r) => json!({"mode": "global", "target": r.target}),
    })
}

pub fn write_mode(text: String, next: Value) -> Result<String, String> {
    let scanned = scan_config_raw(&text);
    let mut units = utf16::units(&text);
    for range in mode_ranges(&text, &scanned.blocks, &scanned.tokens)
        .into_iter()
        .rev()
    {
        let end = range.to.min(units.len());
        let start = range.from.min(end);
        units.drain(start..end);
    }
    let text = String::from_utf16_lossy(&units);
    let Some(o) = next.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'mode')".to_string());
    };
    let mode = obj_defined(o, "mode").and_then(Value::as_str);
    if mode == Some("rule") {
        return Ok(text);
    }
    let scan = scan_config_raw(&text);
    let units = utf16::units(&text);
    let Some(block) = scan.blocks.iter().find(|b| b.name == "routing") else {
        return Err("act.modeNoRouting".to_string());
    };
    let body = js_slice(&units, block.open + 1, block.close);
    let indent = first_indented_line(&body).unwrap_or_else(|| "    ".to_string());
    let mut at = block.open + 1;
    if units.get(at) == Some(&(b'\n' as u16)) {
        at += 1;
    }
    let mut ordinary = false;
    let tokens = &scan.tokens;
    for (i, arrow) in tokens.iter().enumerate() {
        if arrow.from <= block.open
            || arrow.to > block.close
            || arrow.depth != 1
            || arrow.parens != 0
            || js_slice(&units, arrow.from, arrow.to) != "->"
        {
            continue;
        }
        let mandatory = tokens.get(i + 1).is_some()
            && tokens
                .get(i + 2)
                .is_some_and(|t| js_slice(&units, t.from, t.to) == "(")
            && tokens
                .get(i + 3)
                .is_some_and(|t| js_slice(&units, t.from, t.to) == "must")
            && tokens
                .get(i + 4)
                .is_some_and(|t| js_slice(&units, t.from, t.to) == ")");
        if !mandatory {
            ordinary = true;
            continue;
        }
        if ordinary {
            return Err("act.modeInterleaved".to_string());
        }
        let close_to = tokens[i + 4].to;
        let end = index_of_u16(&units, b'\n' as u16, close_to);
        at = match end {
            Some(nl) if nl < block.close => nl + 1,
            _ => close_to,
        };
        if let Some(following) = tokens.get(i + 5)
            && following.from < at
            && following.kind != TokenKind::Comment
        {
            return Err("act.modeInterleaved".to_string());
        }
    }
    let outbound = if mode == Some("direct") {
        "direct".to_string()
    } else {
        obj_defined(o, "target")
            .map(crate::port::js_str::js_to_string)
            .unwrap_or_else(|| "undefined".to_string())
    };
    let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
        ""
    } else {
        "\n"
    };
    Ok(format!(
        "{}{sep}{indent}l4proto(tcp, udp) -> {outbound} {MODE_MARK}\n{}",
        js_slice(&units, 0, at),
        js_slice_from(&units, at),
    ))
}

pub fn same_mode(a: Value, b: Value) -> Result<Value, String> {
    if a.is_null() || js::is_undefined(&a) {
        return Err("TypeError: Cannot read properties of null (reading 'mode')".to_string());
    }
    if b.is_null() || js::is_undefined(&b) {
        return Err("TypeError: Cannot read properties of null (reading 'mode')".to_string());
    }
    let am = a
        .as_object()
        .and_then(|o| o.get("mode"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let bm = b
        .as_object()
        .and_then(|o| o.get("mode"))
        .cloned()
        .unwrap_or_else(js::undefined);
    if am != bm {
        return Ok(Value::Bool(false));
    }
    if am.as_str() != Some("global") {
        return Ok(Value::Bool(true));
    }
    let at = a
        .as_object()
        .and_then(|o| o.get("target"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let bt = b
        .as_object()
        .and_then(|o| o.get("target"))
        .cloned()
        .unwrap_or_else(js::undefined);
    Ok(Value::Bool(at == bt))
}

pub fn mode_rule_lines(text: String) -> Result<Value, String> {
    let scan = scan_config_raw(&text);
    let units = utf16::units(&text);
    Ok(Value::Array(
        mode_ranges(&text, &scan.blocks, &scan.tokens)
            .into_iter()
            .map(|r| {
                let count = units[..r.from.min(units.len())]
                    .iter()
                    .filter(|&&u| u == b'\n' as u16)
                    .count()
                    + 1;
                json!(count)
            })
            .collect(),
    ))
}
