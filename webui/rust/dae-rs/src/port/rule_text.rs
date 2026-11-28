//! Port of `src/dae/ruleText.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    first_indented_line, index_of_u16, is_space_or_tab, is_truthy, js_slice, js_slice_from,
    js_to_string, line_start_u16, obj_defined, val_as_usize,
};
use crate::port::text::{
    ScanResult, TextBlock, TextToken, TokenKind, block_fields_raw, scan_config_raw, unquote_str,
};
use crate::utf16;
use serde_json::{Map, Value, json};

pub fn source_for(list: Value, source: Value) -> Result<Value, String> {
    if !is_truthy(&source) {
        return Ok(js::undefined());
    }
    if list.is_null() || js::is_undefined(&list) {
        return Err("TypeError: Cannot read properties of null (reading 'find')".to_string());
    }
    let Some(arr) = list.as_array() else {
        return Err("TypeError: list.find is not a function".to_string());
    };
    let source_id = source
        .as_object()
        .and_then(|o| o.get("source_id"))
        .cloned()
        .unwrap_or_else(js::undefined);
    for item in arr {
        if item.is_null() || js::is_undefined(item) {
            return Err("TypeError: Cannot read properties of null (reading 'id')".to_string());
        }
        let item_id = item
            .as_object()
            .and_then(|o| o.get("id"))
            .cloned()
            .unwrap_or_else(js::undefined);
        if item_id == source_id {
            return Ok(item.clone());
        }
    }
    Ok(js::undefined())
}

fn compact(value: &str) -> String {
    let scan = scan_config_raw(value);
    let units = utf16::units(value);
    scan.tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .map(|t| js_slice(&units, t.from, t.to))
        .collect::<Vec<_>>()
        .join("")
}

type BlockFinder<'a> = Box<dyn Fn(&[TextBlock]) -> Vec<&TextBlock> + 'a>;

struct Placement<'a> {
    within: BlockFinder<'a>,
    fallbacks: &'static [&'static str],
    target: String,
    fold_case: bool,
    bare_include: bool,
}

fn anchor_at(
    source: &Value,
    rule: &Value,
    place: Placement<'_>,
    scan: Option<Value>,
) -> Result<Option<Value>, String> {
    let ro = crate::port::js_str::ensure_not_nullish(rule, "source")?;
    let Some(rsrc) = ro
        .and_then(|o| obj_defined(o, "source"))
        .filter(|v| is_truthy(v))
    else {
        return Ok(None);
    };
    let so = crate::port::js_str::ensure_not_nullish(source, "id")?;
    let rsrc_id = rsrc
        .as_object()
        .and_then(|o| o.get("source_id"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let src_id = so
        .and_then(|o| o.get("id"))
        .cloned()
        .unwrap_or_else(js::undefined);
    if rsrc_id != src_id {
        return Ok(None);
    }
    let text = so
        .and_then(|o| obj_defined(o, "content"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let rule_line_num = rsrc
        .as_object()
        .and_then(|o| o.get("line"))
        .and_then(js::as_number)
        .unwrap_or(f64::NAN);
    let target_line_f = rule_line_num - 1.0;
    let scanned = match scan.filter(|v| !v.is_null() && !js::is_undefined(v)) {
        Some(sv) => ScanResult::from_value(&sv)?,
        None => scan_config_raw(text),
    };
    let ScanResult { blocks, tokens } = scanned;
    let units = utf16::units(text);
    let start = tokens
        .iter()
        .position(|t| (t.line as f64) == target_line_f && t.kind != TokenKind::Comment);
    let Some(start) = start else {
        return Ok(None);
    };
    if tokens[start].parens != 0 {
        return Ok(None);
    }
    let mut line = tokens[start].line;
    let mut actual: Vec<&TextToken> = Vec::new();
    for tok in &tokens[start..] {
        if tok.line != line && tok.parens == 0 {
            break;
        }
        if tok.kind != TokenKind::Comment {
            actual.push(tok);
        }
        if tok.line != line {
            line = tok.line;
        }
    }
    let Some(&first) = actual.first() else {
        return Ok(None);
    };
    let Some(&last) = actual.last() else {
        return Ok(None);
    };
    let within = (place.within)(&blocks)
        .into_iter()
        .any(|b| first.from > b.open && last.to <= b.close && first.depth == b.depth + 1);
    let src_kind = so
        .and_then(|o| obj_defined(o, "kind"))
        .and_then(Value::as_str);
    if !within
        && !(place.bare_include
            && src_kind == Some("include")
            && blocks.is_empty()
            && first.depth == 0)
    {
        return Ok(None);
    }
    let first_raw = js_slice(&units, first.from, first.to);
    let fallback = actual.len() >= 3
        && place.fallbacks.contains(&first_raw.as_str())
        && js_slice(&units, actual[1].from, actual[1].to) == ":";
    let rule_kind = ro
        .and_then(|o| obj_defined(o, "kind"))
        .and_then(Value::as_str);
    if (rule_kind == Some("fallback")) != fallback {
        return Ok(None);
    }
    let fold = |v: String| -> String { if place.fold_case { v.to_lowercase() } else { v } };
    let bare = |from: usize, to: usize| -> String { compact(&js_slice(&units, from, to)) };
    let arrow_pos = if fallback {
        Some(1)
    } else {
        actual
            .iter()
            .position(|t| t.parens == 0 && js_slice(&units, t.from, t.to) == "->")
    };
    let Some(arrow_idx) = arrow_pos else {
        return Ok(None);
    };
    let arrow = actual[arrow_idx];
    let target = compact(&place.target);
    if fold(bare(arrow.to, last.to)) != fold(target.clone()) {
        return Ok(None);
    }
    let expr = ro
        .and_then(|o| obj_defined(o, "expression"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let shown = compact(expr);
    let arrow_target = format!("->{target}");
    let condition = if fold(shown.clone()).ends_with(&fold(arrow_target)) {
        let su = utf16::units(&shown);
        let tu_len = target.encode_utf16().count() + 2;
        js_slice(&su, 0, su.len().saturating_sub(tu_len))
    } else {
        shown
    };
    if !fallback && !expr.contains("<redacted>") && bare(first.from, arrow.from) != condition {
        return Ok(None);
    }
    let from = line_start_u16(&units, first.from);
    let newline = index_of_u16(&units, b'\n' as u16, last.to);
    let to = match newline {
        None => units.len(),
        Some(nl) => nl + 1,
    };
    let condition_end = if arrow_idx > 0 {
        Some(actual[arrow_idx - 1])
    } else {
        None
    };
    let mut target_obj = Map::new();
    target_obj.insert("from".to_string(), json!(arrow.to));
    target_obj.insert("to".to_string(), json!(last.to));
    if place.fold_case {
        target_obj.insert("foldCase".to_string(), Value::Bool(true));
    }
    let mut out = Map::new();
    out.insert("from".to_string(), json!(from));
    out.insert("to".to_string(), json!(to));
    out.insert(
        "indent".to_string(),
        Value::String(js_slice(&units, from, first.from)),
    );
    out.insert(
        "text".to_string(),
        Value::String(js_slice(&units, from, to)),
    );
    out.insert("target".to_string(), Value::Object(target_obj));
    if !fallback && let Some(ce) = condition_end {
        out.insert(
            "condition".to_string(),
            json!({"from": first.from, "to": ce.to}),
        );
    }
    Ok(Some(Value::Object(out)))
}

pub fn rule_anchor(
    source: Value,
    rule: Value,
    scan: Option<Value>,
) -> Result<Option<Value>, String> {
    let target = if let Some(ro) = rule.as_object() {
        let outbound = obj_defined(ro, "outbound")
            .filter(|v| !v.is_null())
            .map(js_to_string)
            .unwrap_or_default();
        let must = obj_defined(ro, "must").is_some_and(is_truthy);
        format!("{outbound}{}", if must { "(must)" } else { "" })
    } else {
        String::new()
    };
    anchor_at(
        &source,
        &rule,
        Placement {
            within: Box::new(|blocks| blocks.iter().filter(|b| b.name == "routing").collect()),
            fallbacks: &["fallback"],
            target,
            fold_case: false,
            bare_include: true,
        },
        scan,
    )
}

pub fn dns_rule_target_val(rule: &Value) -> Result<Value, String> {
    let ro = crate::port::js_str::ensure_not_nullish(rule, "action")?;
    let action = ro
        .and_then(|o| o.get("action"))
        .cloned()
        .unwrap_or_else(js::undefined);
    if matches!(action.as_str(), Some("upstream" | "requery")) {
        match ro.and_then(|o| obj_defined(o, "upstream")) {
            None | Some(Value::Null) => Ok(Value::String(String::new())),
            Some(v) => Ok(v.clone()),
        }
    } else {
        Ok(action)
    }
}

pub fn dns_rule_target(rule: Value) -> Result<Value, String> {
    dns_rule_target_val(&rule)
}

fn dns_list_blocks<'a>(blocks: &'a [TextBlock], list: &str) -> Vec<&'a TextBlock> {
    blocks
        .iter()
        .filter(|b| b.name == "dns")
        .flat_map(|dns| dns.children.iter().filter(|c| c.name == "routing"))
        .flat_map(|routing| routing.children.iter().filter(|c| c.name == list))
        .collect()
}

pub fn dns_rule_anchor(
    source: Value,
    rule: Value,
    list: String,
    scan: Option<Value>,
) -> Result<Option<Value>, String> {
    let target_v = dns_rule_target_val(&rule)?;
    let target = js_to_string(&target_v);
    anchor_at(
        &source,
        &rule,
        Placement {
            within: Box::new(move |blocks| dns_list_blocks(blocks, &list)),
            fallbacks: &["fallback", "default"],
            target,
            fold_case: true,
            bare_include: false,
        },
        scan,
    )
}

fn block_end(source: &Value, block: &TextBlock) -> Option<(Value, Map<String, Value>, String)> {
    let o = source.as_object()?;
    if !obj_defined(o, "writable").is_some_and(is_truthy) {
        return None;
    }
    let text = obj_defined(o, "content")
        .and_then(Value::as_str)
        .unwrap_or("");
    let units = utf16::units(text);
    let from = line_start_u16(&units, block.close);
    let closing = js_slice(&units, from, block.close);
    if from <= block.open || !is_space_or_tab(&closing) {
        return None;
    }
    let newline = index_of_u16(&units, b'\n' as u16, block.close);
    let to = match newline {
        None => units.len(),
        Some(nl) => nl + 1,
    };
    let inner =
        first_indented_line(&js_slice(&units, block.open + 1, block.close)).unwrap_or_else(|| {
            let cu = utf16::units(&closing);
            let step = match cu.len().checked_div(block.depth) {
                Some(q) => js_slice(&cu, 0, q),
                None => {
                    if cu.is_empty() {
                        String::new()
                    } else {
                        closing.clone()
                    }
                }
            };
            format!("{closing}{}", if step.is_empty() { "    " } else { &step })
        });
    let mut anchor = Map::new();
    anchor.insert("from".to_string(), json!(from));
    anchor.insert("to".to_string(), json!(to));
    anchor.insert("indent".to_string(), Value::String(inner));
    anchor.insert(
        "text".to_string(),
        Value::String(js_slice(&units, from, to)),
    );
    Some((source.clone(), anchor, closing))
}

pub fn dns_list_end(sources: Vec<Value>, list: String) -> Result<Option<Value>, String> {
    let mut scanned_sources: Vec<(Value, Vec<TextBlock>)> = Vec::with_capacity(sources.len());
    for s in sources {
        let Some(o) = s.as_object() else {
            return Err(
                "TypeError: Cannot read properties of null (reading 'content')".to_string(),
            );
        };
        let Some(content) = obj_defined(o, "content").and_then(Value::as_str) else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
            );
        };
        let blocks = scan_config_raw(content).blocks;
        scanned_sources.push((s, blocks));
    }
    let mut lists: Vec<(&Value, &TextBlock)> = Vec::new();
    for (s, blocks) in &scanned_sources {
        for b in dns_list_blocks(blocks, &list) {
            lists.push((s, b));
        }
    }
    if !lists.is_empty() {
        if lists.len() != 1 {
            return Ok(None);
        }
        return Ok(
            block_end(lists[0].0, lists[0].1).map(|(source, anchor, _)| {
                json!({
                    "source": source,
                    "anchor": Value::Object(anchor),
                })
            }),
        );
    }
    let mut routings: Vec<(&Value, &TextBlock)> = Vec::new();
    for (s, blocks) in &scanned_sources {
        for dns in blocks.iter().filter(|b| b.name == "dns") {
            for r in dns.children.iter().filter(|c| c.name == "routing") {
                routings.push((s, r));
            }
        }
    }
    if routings.len() != 1 {
        return Ok(None);
    }
    let Some((source, mut anchor, closing)) = block_end(routings[0].0, routings[0].1) else {
        return Ok(None);
    };
    let indent = anchor
        .get("indent")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let step = if indent.starts_with(&closing) && indent.len() > closing.len() {
        &indent[closing.len()..]
    } else {
        "    "
    };
    anchor.insert(
        "indent".to_string(),
        Value::String(format!("{indent}{step}")),
    );
    anchor.insert(
        "open".to_string(),
        Value::String(format!("{indent}{list} {{\n")),
    );
    anchor.insert("close".to_string(), Value::String(format!("{indent}}}\n")));
    Ok(Some(json!({
        "source": source,
        "anchor": Value::Object(anchor),
    })))
}

pub fn dns_upstreams_raw(text: &str, scan: Option<&ScanResult>) -> Vec<(String, String)> {
    let owned;
    let sc = match scan {
        Some(s) => s,
        None => {
            owned = scan_config_raw(text);
            &owned
        }
    };
    let units = utf16::units(text);
    let mut out = Vec::new();
    for dns in sc.blocks.iter().filter(|b| b.name == "dns") {
        for upstream in dns.children.iter().filter(|c| c.name == "upstream") {
            for field in block_fields_raw(text, upstream, &sc.tokens) {
                let name_to = sc
                    .tokens
                    .iter()
                    .find(|t| t.from == field.from)
                    .map_or(field.from, |t| t.to);
                out.push((
                    js_slice(&units, field.from, name_to),
                    unquote_str(&field.value),
                ));
            }
        }
    }
    out
}

pub fn dns_upstreams(text: String, scan: Option<Value>) -> Result<Vec<Value>, String> {
    let parsed_scan = match scan.filter(|v| !js::is_undefined(v)) {
        Some(v) => Some(ScanResult::from_value(&v)?),
        None => None,
    };
    Ok(dns_upstreams_raw(&text, parsed_scan.as_ref())
        .into_iter()
        .map(|(name, address)| json!({"name": name, "address": address}))
        .collect())
}

pub fn dns_upstream_names(text: String, scan: Option<Value>) -> Result<Vec<String>, String> {
    let parsed_scan = match scan.filter(|v| !js::is_undefined(v)) {
        Some(v) => Some(ScanResult::from_value(&v)?),
        None => None,
    };
    Ok(dns_upstreams_raw(&text, parsed_scan.as_ref())
        .into_iter()
        .map(|(name, _)| name)
        .collect())
}

pub fn rule_line_str(condition: &str, outbound: &str, must: bool) -> String {
    format!(
        "{condition} -> {outbound}{}",
        if must { "(must)" } else { "" }
    )
}

pub fn rule_line(condition: String, outbound: String, must: Option<bool>) -> Result<Value, String> {
    Ok(Value::String(rule_line_str(
        &condition,
        &outbound,
        must.unwrap_or(false),
    )))
}

struct ParsedAnchor {
    from: usize,
    to: usize,
    indent: String,
    text: String,
    target: Option<(usize, usize, bool)>,
    condition: Option<(usize, usize)>,
    open: String,
    close: String,
}

fn parse_anchor(anchor: &Value) -> Result<ParsedAnchor, String> {
    let ao = crate::port::js_str::ensure_not_nullish(anchor, "from")?;
    let from = ao
        .and_then(|o| o.get("from"))
        .and_then(val_as_usize)
        .unwrap_or(0);
    let to = ao
        .and_then(|o| o.get("to"))
        .and_then(val_as_usize)
        .unwrap_or(0);
    let indent = ao
        .and_then(|o| obj_defined(o, "indent"))
        .map(js_to_string)
        .unwrap_or_else(|| "undefined".to_string());
    let text = ao
        .and_then(|o| obj_defined(o, "text"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let target = ao
        .and_then(|o| obj_defined(o, "target"))
        .and_then(Value::as_object)
        .map(|t| {
            (
                t.get("from").and_then(val_as_usize).unwrap_or(0),
                t.get("to").and_then(val_as_usize).unwrap_or(0),
                obj_defined(t, "foldCase").is_some_and(is_truthy),
            )
        });
    let condition = ao
        .and_then(|o| obj_defined(o, "condition"))
        .and_then(Value::as_object)
        .map(|c| {
            (
                c.get("from").and_then(val_as_usize).unwrap_or(0),
                c.get("to").and_then(val_as_usize).unwrap_or(0),
            )
        });
    let open = ao
        .and_then(|o| obj_defined(o, "open"))
        .filter(|v| !v.is_null())
        .map(js_to_string)
        .unwrap_or_default();
    let close = ao
        .and_then(|o| obj_defined(o, "close"))
        .filter(|v| !v.is_null())
        .map(js_to_string)
        .unwrap_or_default();
    Ok(ParsedAnchor {
        from,
        to,
        indent,
        text,
        target,
        condition,
        open,
        close,
    })
}

pub fn add_rule(
    text: String,
    anchor: Value,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let a = parse_anchor(&anchor)?;
    let units = utf16::units(&text);
    if anchor
        .as_object()
        .and_then(|o| o.get("text"))
        .and_then(Value::as_str)
        != Some(js_slice(&units, a.from, a.to).as_str())
    {
        return Ok(None);
    }
    Ok(Some(format!(
        "{}{}{}{}\n{}{}",
        js_slice(&units, 0, a.from),
        a.open,
        a.indent,
        rule_line_str(&condition, &outbound, must),
        a.close,
        js_slice_from(&units, a.from),
    )))
}

pub fn replace_rule_target(
    text: String,
    anchor: Value,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let a = parse_anchor(&anchor)?;
    let Some((t_from, t_to, fold_case)) = a.target else {
        return Ok(None);
    };
    let units = utf16::units(&text);
    if anchor
        .as_object()
        .and_then(|o| o.get("text"))
        .and_then(Value::as_str)
        != Some(js_slice(&units, a.from, a.to).as_str())
    {
        return Ok(None);
    }
    let target = format!("{outbound}{}", if must { "(must)" } else { "" });
    let fold = |s: String| if fold_case { s.to_lowercase() } else { s };
    if fold(compact(&js_slice(&units, t_from, t_to))) == fold(compact(&target)) {
        return Ok(Some(text));
    }
    Ok(Some(format!(
        "{} {target}{}",
        js_slice(&units, 0, t_from),
        js_slice_from(&units, t_to),
    )))
}

pub fn remove_rule(text: String, anchor: Value) -> Result<Option<String>, String> {
    let a = parse_anchor(&anchor)?;
    let units = utf16::units(&text);
    if anchor
        .as_object()
        .and_then(|o| o.get("text"))
        .and_then(Value::as_str)
        != Some(js_slice(&units, a.from, a.to).as_str())
    {
        return Ok(None);
    }
    Ok(Some(format!(
        "{}{}",
        js_slice(&units, 0, a.from),
        js_slice_from(&units, a.to),
    )))
}

pub fn replace_rule(
    text: String,
    anchor: Value,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let targeted = replace_rule_target(text.clone(), anchor.clone(), outbound, must)?;
    let Some(targeted) = targeted else {
        return Ok(None);
    };
    let a = parse_anchor(&anchor)?;
    let Some((c_from, c_to)) = a.condition else {
        return Ok(Some(targeted));
    };
    let units = utf16::units(&text);
    let old_cond = js_slice(&units, c_from, c_to);
    if old_cond == condition {
        return Ok(Some(targeted));
    }
    let newline = if a.text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let comments: String = scan_config_raw(&old_cond)
        .tokens
        .into_iter()
        .filter(|t| t.kind == TokenKind::Comment)
        .map(|t| {
            let c = js_slice(&units, c_from + t.from, c_from + t.to);
            format!(
                "{}{newline}{}",
                c.strip_suffix('\r').unwrap_or(&c),
                a.indent
            )
        })
        .collect();
    let tu = utf16::units(&targeted);
    Ok(Some(format!(
        "{}{comments}{condition}{}",
        js_slice(&tu, 0, c_from),
        js_slice_from(&tu, c_to),
    )))
}

pub fn add_fallback(text: String, anchor: Value, target: String) -> Result<Option<String>, String> {
    let a = parse_anchor(&anchor)?;
    let units = utf16::units(&text);
    if anchor
        .as_object()
        .and_then(|o| o.get("text"))
        .and_then(Value::as_str)
        != Some(js_slice(&units, a.from, a.to).as_str())
    {
        return Ok(None);
    }
    Ok(Some(format!(
        "{}{}{}fallback: {target}\n{}{}",
        js_slice(&units, 0, a.from),
        a.open,
        a.indent,
        a.close,
        js_slice_from(&units, a.from),
    )))
}
