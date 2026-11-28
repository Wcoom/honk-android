//! Port of `src/dae/groups.ts`.

#![forbid(unsafe_code)]

use crate::port::js_str::{
    apply_utf16_edits, first_indented_line, index_of_u16, is_js_ws, is_space_or_tab,
    is_space_tab_cr, js_slice, js_slice_from, js_trim, leading_js_ws_u16_len, line_start_u16,
    obj_defined,
};
use crate::port::text::{
    TextBlock, TextField, TextToken, TokenKind, block_fields_raw, is_bare_name_bool,
    is_fragment_bool, is_quotable_bool, quote_str, scan_config_raw, unquote_str,
};
use crate::utf16;
use indexmap::IndexMap;
use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct GroupEntry {
    pub name: String,
    pub written: String,
    pub filters: Vec<String>,
    pub policy: Option<String>,
    pub default: Option<String>,
    pub final_: Option<String>,
    pub exit: Option<String>,
    pub interrupt: Option<String>,
    pub from: usize,
    pub to: usize,
}

impl GroupEntry {
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "written": self.written,
            "filters": self.filters,
            "policy": self.policy,
            "default": self.default,
            "final": self.final_,
            "exit": self.exit,
            "interrupt": self.interrupt,
            "from": self.from,
            "to": self.to,
        })
    }
}

pub fn read_group_entries_raw(text: &str) -> Vec<GroupEntry> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut out = Vec::new();
    for block in scan.blocks.iter().filter(|b| b.name == "group") {
        for entry in &block.children {
            let fields = block_fields_raw(text, entry, &scan.tokens);
            let head = scan.tokens.iter().find(|t| t.from == entry.from);
            let last = |name: &str| -> Option<String> {
                fields
                    .iter()
                    .rev()
                    .find(|f| f.name == name)
                    .map(|f| f.value.clone())
            };
            out.push(GroupEntry {
                name: entry.name.clone(),
                written: head
                    .map(|h| js_slice(&units, h.from, h.to))
                    .unwrap_or_else(|| entry.name.clone()),
                filters: fields
                    .iter()
                    .filter(|f| f.name == "filter")
                    .map(|f| f.value.clone())
                    .collect(),
                policy: last("policy"),
                default: last("default"),
                final_: last("final"),
                exit: last("exit"),
                interrupt: last("interrupt_connections"),
                from: entry.line,
                to: entry.end_line,
            });
        }
    }
    out
}

pub fn read_group_entries(text: String) -> Result<Vec<Value>, String> {
    Ok(read_group_entries_raw(&text)
        .into_iter()
        .map(|g| g.to_value())
        .collect())
}

pub fn quote_name_str(value: &str) -> Result<String, String> {
    if is_bare_name_bool(value) {
        Ok(value.to_string())
    } else {
        quote_str(value)
    }
}

pub fn quote_name(value: String) -> Result<Value, String> {
    Ok(Value::String(quote_name_str(&value)?))
}

pub fn is_writable_name(value: String) -> Result<Value, String> {
    Ok(Value::Bool(
        is_bare_name_bool(&value) || is_quotable_bool(&value),
    ))
}

pub fn group_name_problem(name: String, taken: Value) -> Result<Value, String> {
    if !is_bare_name_bool(&name) {
        return Ok(Value::String("invalid".to_string()));
    }
    let set_arr = taken
        .as_object()
        .filter(|o| o.len() == 1)
        .and_then(|o| o.get("$set"))
        .and_then(Value::as_array);
    let Some(items) = set_arr else {
        return Err("TypeError: taken.has is not a function".to_string());
    };
    if items.iter().any(|v| v.as_str() == Some(name.as_str())) {
        Ok(Value::String("taken".to_string()))
    } else {
        Ok(Value::Null)
    }
}

struct GroupUpdate {
    filters: Vec<String>,
    policy: Option<String>,
    default: Option<Option<String>>,
    final_: Option<Option<String>>,
    exit: Option<Option<String>>,
    interrupt: Option<Option<String>>,
}

fn parse_opt_str_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<Option<String>> {
    match obj_defined(obj, key) {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(s)) => Some(Some(s.clone())),
        Some(v) => Some(Some(crate::port::js_str::js_to_string(v))),
    }
}

fn parse_group_update(next: &Value) -> Result<GroupUpdate, String> {
    let Some(o) = next.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'filters')".to_string());
    };
    let Some(filters_val) = obj_defined(o, "filters") else {
        return Err("TypeError: Cannot read properties of undefined (reading 'map')".to_string());
    };
    let Some(filters_arr) = filters_val.as_array() else {
        return Err("TypeError: next.filters.map is not a function".to_string());
    };
    let filters = filters_arr
        .iter()
        .map(crate::port::js_str::js_to_string)
        .collect();
    let policy = match obj_defined(o, "policy") {
        None | Some(Value::Null) => None,
        Some(v) => Some(crate::port::js_str::js_to_string(v)),
    };
    Ok(GroupUpdate {
        filters,
        policy,
        default: parse_opt_str_field(o, "default"),
        final_: parse_opt_str_field(o, "final"),
        exit: parse_opt_str_field(o, "exit"),
        interrupt: parse_opt_str_field(o, "interrupt"),
    })
}

const SINGLE_KEYS: [(&str, &str); 5] = [
    ("policy", "policy"),
    ("default", "default"),
    ("final", "final"),
    ("exit", "exit"),
    ("interrupt", "interrupt_connections"),
];

fn update_single_key<'a>(u: &'a GroupUpdate, key: &str) -> Option<Option<&'a str>> {
    match key {
        "policy" => Some(u.policy.as_deref()),
        "default" => u.default.as_ref().map(|o| o.as_deref()),
        "final" => u.final_.as_ref().map(|o| o.as_deref()),
        "exit" => u.exit.as_ref().map(|o| o.as_deref()),
        "interrupt" => u.interrupt.as_ref().map(|o| o.as_deref()),
        _ => None,
    }
}

fn rewrite_fields(
    text: &str,
    entry: &TextBlock,
    fields: &[TextField],
    inner: &str,
    next: &GroupUpdate,
) -> String {
    let units = utf16::units(text);
    let filters: Vec<&TextField> = fields.iter().filter(|f| f.name == "filter").collect();
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let line_end = |at: usize| -> usize {
        match index_of_u16(&units, b'\n' as u16, at) {
            Some(pos) => pos + 1,
            None => 0,
        }
    };
    let remove = |field: &TextField, edits: &mut Vec<(usize, usize, String)>| {
        let start = line_start_u16(&units, field.from);
        let end = line_end(field.to);
        let whole = is_space_tab_cr(&js_slice(&units, start, field.from))
            && end > 0
            && is_space_tab_cr(&js_slice(&units, field.to, end - 1));
        if whole {
            edits.push((start, end, String::new()));
        } else {
            edits.push((field.from, field.to, String::new()));
        }
    };
    let replace = |field: &TextField, value: &str, edits: &mut Vec<(usize, usize, String)>| {
        if field.value == value {
            return;
        }
        let raw_val = js_slice(&units, field.value_from, field.value_to);
        let lead_len = leading_js_ws_u16_len(&raw_val);
        let prefix = if lead_len > 0 { "" } else { " " };
        edits.push((
            field.value_from + lead_len,
            field.value_to,
            format!("{prefix}{value}"),
        ));
    };
    let before_field = |field: &TextField| -> usize {
        let start = line_start_u16(&units, field.from);
        if is_space_or_tab(&js_slice(&units, start, field.from)) {
            start
        } else {
            field.from
        }
    };
    let mut added: IndexMap<usize, String> = IndexMap::new();
    let mut add = |at: usize, line: String| {
        added.entry(at).or_default().push_str(&line);
    };
    let policies: Vec<&TextField> = fields.iter().filter(|f| f.name == "policy").collect();
    let after_field = |at: usize| -> usize {
        let end = line_end(at);
        if end > 0 && end <= entry.close {
            end
        } else {
            at
        }
    };
    let filter_at = if let Some(last) = filters.last() {
        after_field(last.to)
    } else if let Some(first) = policies.first() {
        before_field(first)
    } else {
        after_field(entry.open + 1)
    };

    let mut change_filters = |from: usize, to: usize, values: &[String], at: usize| {
        for (i, field) in filters[from..to].iter().enumerate() {
            if let Some(val) = values.get(i) {
                replace(field, val, &mut edits);
            } else {
                remove(field, &mut edits);
            }
        }
        let skip = to - from;
        if skip < values.len() {
            for val in &values[skip..] {
                add(at, format!("{inner}filter: {val}\n"));
            }
        }
    };

    let mut old_from = 0usize;
    let mut new_from = 0usize;
    for (i, value) in next.filters.iter().enumerate() {
        let found = filters
            .iter()
            .enumerate()
            .skip(old_from)
            .find(|(_, field)| field.value == *value)
            .map(|(idx, _)| idx);
        let Some(at) = found else {
            continue;
        };
        change_filters(
            old_from,
            at,
            &next.filters[new_from..i],
            before_field(filters[at]),
        );
        old_from = at + 1;
        new_from = i + 1;
    }
    change_filters(
        old_from,
        filters.len(),
        &next.filters[new_from..],
        filter_at,
    );

    for (i, &(key, fname)) in SINGLE_KEYS.iter().enumerate() {
        let Some(value_opt) = update_single_key(next, key) else {
            continue;
        };
        let own: Vec<&TextField> = fields.iter().filter(|f| f.name == fname).collect();
        if own.len() > 1 {
            for f in &own[..own.len() - 1] {
                remove(f, &mut edits);
            }
        }
        if let Some(last_own) = own.last() {
            if let Some(v) = value_opt.filter(|s| !s.is_empty()) {
                replace(last_own, v, &mut edits);
            } else {
                remove(last_own, &mut edits);
            }
            continue;
        }
        let Some(v) = value_opt.filter(|s| !s.is_empty()) else {
            continue;
        };
        if key == "policy" {
            add(filter_at, format!("{inner}policy: {v}\n"));
            continue;
        }
        let before = fields.iter().rev().find(|f| {
            f.name == "filter"
                || SINGLE_KEYS[..i]
                    .iter()
                    .any(|&(_, prev_fn)| f.name == prev_fn)
        });
        let pos = match before {
            Some(b) => after_field(b.to),
            None => after_field(entry.open + 1),
        };
        add(pos, format!("{inner}{fname}: {v}\n"));
    }

    for (at, lines) in added {
        let prev_char_is_nl = at > 0 && units.get(at - 1) == Some(&(b'\n' as u16));
        let ls = line_start_u16(&units, at);
        let inline =
            at > entry.open && !prev_char_is_nl && !is_space_or_tab(&js_slice(&units, ls, at));
        let txt = if inline {
            format!("\n{lines}{inner}")
        } else {
            lines
        };
        edits.push((at, at, txt));
    }

    apply_utf16_edits(text, edits, true)
}

fn write_group_entry_inner(text: &str, name: &str, next: &GroupUpdate) -> Result<String, String> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let sections: Vec<&TextBlock> = scan.blocks.iter().filter(|b| b.name == "group").collect();
    let entry = sections
        .iter()
        .flat_map(|b| b.children.iter())
        .rfind(|e| e.name == name);
    if let Some(entry) = entry {
        let all_groups = read_group_entries_raw(text);
        if let Some(current) = all_groups.iter().rev().find(|g| g.name == name) {
            let same = current.filters == next.filters
                && current.policy == next.policy
                && next.default.as_ref().is_none_or(|v| *v == current.default)
                && next.final_.as_ref().is_none_or(|v| *v == current.final_)
                && next.exit.as_ref().is_none_or(|v| *v == current.exit)
                && next
                    .interrupt
                    .as_ref()
                    .is_none_or(|v| *v == current.interrupt);
            if same {
                return Ok(text.to_string());
            }
        }
        let ls = line_start_u16(&units, entry.from);
        let prefix = js_slice(&units, ls, entry.from);
        let indent = if prefix.chars().all(is_js_ws) {
            prefix
        } else {
            "  ".to_string()
        };
        let fields = block_fields_raw(text, entry, &scan.tokens);
        let body_start = entry.open + 1;
        let old = js_slice(&units, body_start, entry.close);
        let inner = first_indented_line(&old).unwrap_or_else(|| {
            if indent.is_empty() {
                "    ".to_string()
            } else {
                format!("{indent}{indent}")
            }
        });
        if entry.line != entry.end_line {
            return Ok(rewrite_fields(text, entry, &fields, &inner, next));
        }
        let mut starts = Vec::with_capacity(fields.len() + 2);
        starts.push(body_start);
        for f in &fields {
            starts.push(f.from);
        }
        starts.push(entry.close);
        let mut lines = Vec::new();
        for w in starts.windows(2) {
            let piece = js_slice(&units, w[0], w[1]);
            let trimmed = js_trim(&piece);
            if !trimmed.is_empty() {
                lines.push(format!("{inner}{trimmed}\n"));
            }
        }
        let expanded = format!(
            "{}\n{}{}{}",
            js_slice(&units, 0, body_start),
            lines.concat(),
            indent,
            js_slice_from(&units, entry.close),
        );
        return write_group_entry_inner(&expanded, name, next);
    }

    let block = sections.last().copied();
    let indent = block
        .and_then(|b| first_indented_line(&js_slice(&units, b.open + 1, b.close)))
        .unwrap_or_else(|| "    ".to_string());
    let mut body_lines = Vec::new();
    body_lines.push(format!("{indent}{} {{", quote_name_str(name)?));
    for f in &next.filters {
        body_lines.push(format!("{indent}{indent}filter: {f}"));
    }
    if let Some(v) = next.policy.as_deref().filter(|s| !s.is_empty()) {
        body_lines.push(format!("{indent}{indent}policy: {v}"));
    }
    if let Some(Some(v)) = next.default.as_ref().map(|o| o.as_deref())
        && !v.is_empty()
    {
        body_lines.push(format!("{indent}{indent}default: {v}"));
    }
    if let Some(Some(v)) = next.final_.as_ref().map(|o| o.as_deref())
        && !v.is_empty()
    {
        body_lines.push(format!("{indent}{indent}final: {v}"));
    }
    if let Some(Some(v)) = next.exit.as_ref().map(|o| o.as_deref())
        && !v.is_empty()
    {
        body_lines.push(format!("{indent}{indent}exit: {v}"));
    }
    if let Some(Some(v)) = next.interrupt.as_ref().map(|o| o.as_deref())
        && !v.is_empty()
    {
        body_lines.push(format!("{indent}{indent}interrupt_connections: {v}"));
    }
    body_lines.push(format!("{indent}}}"));
    let body = body_lines.join("\n");

    if let Some(block) = block {
        let close_line = line_start_u16(&units, block.close);
        let at = if is_space_or_tab(&js_slice(&units, close_line, block.close)) {
            close_line
        } else {
            block.close
        };
        let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
            ""
        } else {
            "\n"
        };
        return Ok(format!(
            "{}{sep}{body}\n{}",
            js_slice(&units, 0, at),
            js_slice_from(&units, at),
        ));
    }
    let trimmed_end = text.trim_end_matches('\n');
    Ok(format!("{trimmed_end}\n\ngroup {{\n{body}\n}}\n"))
}

pub fn write_group_entry(text: String, name: String, next: Value) -> Result<String, String> {
    let upd = parse_group_update(&next)?;
    write_group_entry_inner(&text, &name, &upd)
}

pub fn name_text(value: Value, written: Value) -> Result<Value, String> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    if !written.is_null()
        && !crate::js::is_undefined(&written)
        && let Some(ws) = written.as_str()
        && value.as_str().is_some_and(|vs| unquote_str(ws) == vs)
    {
        return Ok(written);
    }
    if crate::js::is_undefined(&value) {
        return Ok(crate::js::undefined());
    }
    let vs = crate::port::js_str::js_to_string(&value);
    Ok(Value::String(quote_name_str(&vs)?))
}

pub fn split_top_vec(text: &str, separator: &str) -> Vec<String> {
    let units = utf16::units(text);
    let sep = utf16::units(separator);
    let mut parts: Vec<String> = Vec::new();
    let mut depth: isize = 0;
    let mut quote_ch: Option<u16> = None;
    let mut start: usize = 0;
    let mut i: usize = 0;
    while i < units.len() {
        let c = units[i];
        if let Some(q) = quote_ch {
            if c == b'\\' as u16 {
                i += 1;
            } else if c == q {
                quote_ch = None;
            }
        } else if c == b'\'' as u16 || c == b'"' as u16 {
            quote_ch = Some(c);
        } else if c == b'(' as u16 {
            depth += 1;
        } else if c == b')' as u16 {
            depth -= 1;
        } else if depth == 0 && !sep.is_empty() && units[i..].starts_with(&sep) {
            parts.push(js_slice(&units, start, i));
            start = i + sep.len();
            i += sep.len();
            continue;
        }
        i += 1;
    }
    parts.push(js_slice_from(&units, start));
    parts.into_iter().map(|p| js_trim(&p).to_string()).collect()
}

pub fn split_top(text: String, separator: String) -> Result<Vec<String>, String> {
    Ok(split_top_vec(&text, &separator))
}

fn extract_entry_filters(entry: &Value) -> Result<Vec<String>, String> {
    let Some(o) = entry.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'filters')".to_string());
    };
    let Some(f) = obj_defined(o, "filters") else {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'flatMap')".to_string(),
        );
    };
    let Some(arr) = f.as_array() else {
        return Err("TypeError: entry.filters.flatMap is not a function".to_string());
    };
    Ok(arr.iter().map(crate::port::js_str::js_to_string).collect())
}

fn nested_in_filters(filters: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for filter in filters {
        for term in split_top_vec(filter, "&&") {
            if term.starts_with("group(") && term.ends_with(')') {
                let inner = &term[6..term.len() - 1];
                for arg in split_top_vec(inner, ",") {
                    let unq = unquote_str(&arg);
                    for tag in unq.split(['|', ',']) {
                        let t = js_trim(tag);
                        if !t.is_empty() {
                            out.push(t.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

pub fn nested_in(entry: Value) -> Result<Vec<String>, String> {
    let filters = extract_entry_filters(&entry)?;
    Ok(nested_in_filters(&filters))
}

fn exact_spans(filter: &str, call: &str) -> Option<Vec<TextToken>> {
    let scan = scan_config_raw(filter);
    let tokens = scan.tokens;
    let units = utf16::units(filter);
    let raw: Vec<String> = tokens
        .iter()
        .map(|t| js_slice(&units, t.from, t.to))
        .collect();
    if tokens.first().map(|t| t.kind) != Some(TokenKind::Text)
        || raw.first().map(String::as_str) != Some(call)
        || raw.get(1).map(String::as_str) != Some("(")
        || raw.last().map(String::as_str) != Some(")")
        || tokens.last().map(|t| t.parens) != Some(1)
    {
        return None;
    }
    let mut values = Vec::new();
    let mut expect_value = true;
    for i in 2..tokens.len().saturating_sub(1) {
        let token = &tokens[i];
        if token.parens != 1 {
            return None;
        }
        if expect_value {
            if token.kind != TokenKind::Text && token.kind != TokenKind::Quoted {
                return None;
            }
            if token.kind == TokenKind::Text
                && (raw[i].contains(':') || raw[i].chars().any(is_js_ws))
            {
                return None;
            }
            values.push(token.clone());
        } else if raw[i] != "," {
            return None;
        }
        expect_value = !expect_value;
    }
    if expect_value && !values.is_empty() {
        None
    } else {
        Some(values)
    }
}

pub fn exact_tokens_vec(filter: &str, call: &str) -> Option<Vec<String>> {
    let spans = exact_spans(filter, call)?;
    let units = utf16::units(filter);
    Some(
        spans
            .into_iter()
            .map(|t| js_slice(&units, t.from, t.to))
            .collect(),
    )
}

pub fn exact_tokens(filter: String, call: Value) -> Result<Value, String> {
    let Some(call_str) = call.as_str() else {
        return Ok(Value::Null);
    };
    match exact_tokens_vec(&filter, call_str) {
        Some(list) => Ok(Value::Array(list.into_iter().map(Value::String).collect())),
        None => Ok(Value::Null),
    }
}

fn splice_exact(
    filter: &str,
    spans: &[TextToken],
    drop: &HashSet<usize>,
    append: &[String],
) -> Option<String> {
    let kept: Vec<usize> = (0..spans.len()).filter(|i| !drop.contains(i)).collect();
    if kept.is_empty() && append.is_empty() {
        return None;
    }
    let units = utf16::units(filter);
    let mut body = String::new();
    for (n, &index) in kept.iter().enumerate() {
        body.push_str(&js_slice(&units, spans[index].from, spans[index].to));
        if n + 1 < kept.len() {
            body.push_str(&js_slice(&units, spans[index].to, spans[index + 1].from));
        }
    }
    if !append.is_empty() {
        if !body.is_empty() {
            body.push_str(", ");
        }
        body.push_str(&append.join(", "));
    }
    Some(format!(
        "{}{body}{}",
        js_slice(&units, 0, spans[0].from),
        js_slice_from(&units, spans.last().map_or(0, |s| s.to)),
    ))
}

fn exact_in(filters: &[String], call: &str) -> Vec<String> {
    filters
        .iter()
        .flat_map(|f| {
            exact_tokens_vec(f, call)
                .unwrap_or_default()
                .into_iter()
                .map(|t| unquote_str(&t))
        })
        .collect()
}

pub fn named_in(entry: Value) -> Result<Vec<String>, String> {
    let filters = extract_entry_filters(&entry)?;
    Ok(exact_in(&filters, "name"))
}

fn classify_filters_raw(filters: &[String]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let names = exact_in(filters, "name");
    let subtags = exact_in(filters, "subtag");
    let rules = filters
        .iter()
        .filter(|f| {
            exact_tokens_vec(f, "name").is_none() && exact_tokens_vec(f, "subtag").is_none()
        })
        .cloned()
        .collect();
    (names, subtags, rules)
}

pub fn classify_filters(entry: Value) -> Result<Value, String> {
    let filters = extract_entry_filters(&entry)?;
    let (names, subtags, rules) = classify_filters_raw(&filters);
    Ok(json!({
        "names": names,
        "subtags": subtags,
        "rules": rules,
    }))
}

pub fn describe_filters(filters: Vec<String>) -> Result<Value, String> {
    let mut every_node = false;
    let mut groups: Vec<String> = Vec::new();
    let mut rules: Vec<String> = Vec::new();
    for filter in filters {
        let trimmed = js_trim(&filter);
        let excluded: Option<Vec<String>> = trimmed
            .strip_prefix('!')
            .and_then(|rest| exact_tokens_vec(js_trim(rest), "name"))
            .map(|toks| toks.into_iter().map(|t| unquote_str(&t)).collect());
        let is_every = excluded.as_ref().is_some_and(|ex| {
            let uniq: HashSet<&str> = ex.iter().map(String::as_str).collect();
            ex.len() == 2 && uniq.len() == 2 && uniq.contains("direct") && uniq.contains("block")
        });
        if is_every {
            every_node = true;
        } else if exact_tokens_vec(trimmed, "group").is_some() {
            groups.extend(nested_in_filters(&[trimmed.to_string()]));
        } else {
            rules.push(filter);
        }
    }
    let mut seen = HashSet::new();
    let unique_groups: Vec<String> = groups
        .into_iter()
        .filter(|g| seen.insert(g.clone()))
        .collect();
    Ok(json!({
        "everyNode": every_node,
        "groups": unique_groups,
        "rules": rules,
    }))
}

fn expression_names(filters: &[String], tag: &str) -> bool {
    let (_, _, rules) = classify_filters_raw(filters);
    rules.iter().any(|filter| {
        let tokens = scan_config_raw(filter).tokens;
        let units = utf16::units(filter);
        let raw: Vec<String> = tokens
            .iter()
            .map(|t| js_slice(&units, t.from, t.to))
            .collect();
        let mut call = String::new();
        for (i, token) in tokens.iter().enumerate() {
            if token.parens == 0 && raw.get(i + 1).map(String::as_str) == Some("(") {
                call = raw[i].strip_prefix('!').unwrap_or(&raw[i]).to_string();
            }
            let prev_colon = i > 0 && raw[i - 1] == ":";
            let next_colon = raw.get(i + 1).map(String::as_str) == Some(":");
            let is_value = token.parens == 1
                && (token.kind == TokenKind::Text || token.kind == TokenKind::Quoted)
                && !prev_colon
                && !next_colon;
            if call == "subtag" && is_value && unquote_str(&raw[i]) == tag {
                return true;
            }
        }
        false
    })
}

fn cites_exactly(filters: &[String], tag: &str) -> bool {
    classify_filters_raw(filters).1.iter().any(|s| s == tag)
}

pub fn citing_groups(text: String, tag: String) -> Result<Value, String> {
    let out: Vec<Value> = read_group_entries_raw(&text)
        .into_iter()
        .filter(|g| cites_exactly(&g.filters, &tag))
        .map(|g| Value::String(g.name))
        .collect();
    Ok(Value::Array(out))
}

pub fn named_in_expression(text: String, tag: String) -> Result<Value, String> {
    let found = read_group_entries_raw(&text)
        .iter()
        .any(|g| expression_names(&g.filters, &tag));
    Ok(Value::Bool(found))
}

pub fn groups_naming_tag(text: String, tag: String) -> Result<Value, String> {
    let out: Vec<Value> = read_group_entries_raw(&text)
        .into_iter()
        .filter(|g| cites_exactly(&g.filters, &tag) || expression_names(&g.filters, &tag))
        .map(|g| Value::String(g.name))
        .collect();
    Ok(Value::Array(out))
}

fn edit_exact(
    text: &str,
    group: &str,
    call: &str,
    add: &[String],
    remove: &[String],
) -> Result<String, String> {
    let entries = read_group_entries_raw(text);
    let entry = entries.iter().rev().find(|e| e.name == group);
    if entry.is_none() && add.is_empty() {
        return Ok(text.to_string());
    }
    let gone: HashSet<&str> = remove.iter().map(String::as_str).collect();
    let empty_filters: Vec<String> = Vec::new();
    let entry_filters = entry.map_or(&empty_filters, |e| &e.filters);
    let present: HashSet<String> = exact_in(entry_filters, call)
        .into_iter()
        .filter(|v| !gone.contains(v.as_str()))
        .collect();
    let mut seen_add = HashSet::new();
    let added: Vec<String> = add
        .iter()
        .filter(|v| seen_add.insert((*v).clone()))
        .filter(|v| !present.contains(*v) && !gone.contains(v.as_str()))
        .cloned()
        .collect();

    let write_added = |list: &[String]| -> Result<Vec<String>, String> {
        let all_single_quoted = !list.is_empty() && list.iter().all(|v| v.starts_with('\''));
        added
            .iter()
            .map(|v| {
                if all_single_quoted {
                    quote_str(v)
                } else {
                    quote_name_str(v)
                }
            })
            .collect()
    };

    let mut placed = added.is_empty();
    let mut filters: Vec<String> = Vec::new();
    for filter in entry_filters {
        let Some(spans) = exact_spans(filter, call) else {
            filters.push(filter.clone());
            continue;
        };
        if spans.is_empty() {
            filters.push(filter.clone());
            continue;
        }
        let units = utf16::units(filter);
        let raw: Vec<String> = spans
            .iter()
            .map(|s| js_slice(&units, s.from, s.to))
            .collect();
        let drop: HashSet<usize> = raw
            .iter()
            .enumerate()
            .filter_map(|(i, v)| gone.contains(unquote_str(v).as_str()).then_some(i))
            .collect();
        let kept: Vec<String> = raw
            .iter()
            .enumerate()
            .filter(|(i, _)| !drop.contains(i))
            .map(|(_, v)| v.clone())
            .collect();
        let append = if placed {
            Vec::new()
        } else {
            write_added(&kept)?
        };
        let spliced = splice_exact(filter, &spans, &drop, &append);
        placed = true;
        if let Some(s) = spliced {
            filters.push(s);
        }
    }
    if !placed {
        filters.push(format!("{call}({})", write_added(&[])?.join(", ")));
    }
    if let Some(e) = entry {
        if filters == e.filters {
            return Ok(text.to_string());
        }
        if !e.filters.is_empty() && filters.is_empty() {
            return Ok(text.to_string());
        }
    }
    let upd = GroupUpdate {
        filters,
        policy: entry.and_then(|e| e.policy.clone()),
        default: None,
        final_: None,
        exit: None,
        interrupt: None,
    };
    write_group_entry_inner(text, group, &upd)
}

pub fn add_subtags_to_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<Value, String> {
    Ok(Value::String(edit_exact(
        &text,
        &group,
        "subtag",
        &tags,
        &[],
    )?))
}

pub fn add_nodes_to_group(
    text: String,
    group: String,
    names: Vec<String>,
) -> Result<Value, String> {
    Ok(Value::String(edit_exact(
        &text,
        &group,
        "name",
        &names,
        &[],
    )?))
}

pub fn remove_subtags_from_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<Value, String> {
    Ok(Value::String(edit_exact(
        &text,
        &group,
        "subtag",
        &[],
        &tags,
    )?))
}

fn has_rule_syntax(v: &str) -> bool {
    v.contains("&&")
        || v.contains("->")
        || v.bytes()
            .any(|b| matches!(b, b'(' | b')' | b'\'' | b'"' | b'#' | b'{' | b'}'))
}

pub fn rule_condition_str(kind: &str, value: &str) -> Result<Option<String>, String> {
    let values: Vec<String> = value
        .split(|c: char| c == ',' || is_js_ws(c))
        .map(|v| js_trim(v).to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if values
        .iter()
        .any(|v| has_rule_syntax(v) || (v.contains(':') && !is_quotable_bool(v)))
    {
        return Ok(None);
    }
    let written: Vec<String> = values
        .iter()
        .map(|v| {
            if v.contains(':') {
                quote_str(v)
            } else {
                Ok(v.clone())
            }
        })
        .collect::<Result<_, _>>()?;
    let list = written.join(", ");
    let qualified = |prefix: &str| -> String {
        written
            .iter()
            .map(|v| format!("{prefix}: {v}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let condition = match kind {
        "domain" => format!("domain({})", qualified("full")),
        "domainSuffix" => format!("domain({})", qualified("suffix")),
        "domainKeyword" => format!("domain({})", qualified("keyword")),
        "geosite" => format!("domain({})", qualified("geosite")),
        "dip" => format!("dip({list})"),
        "geoip" => format!("dip({})", qualified("geoip")),
        "sip" => format!("sip({list})"),
        "mac" => format!("mac({list})"),
        "dport" => format!("dport({list})"),
        "sport" => format!("sport({list})"),
        "pname" => format!("pname({list})"),
        "l4proto" => format!("l4proto({list})"),
        "qnameSuffix" => format!("qname({})", qualified("suffix")),
        "qnameFull" => format!("qname({})", qualified("full")),
        "qnameKeyword" => format!("qname({})", qualified("keyword")),
        "qnameGeosite" => format!("qname({})", qualified("geosite")),
        "qtype" => format!("qtype({list})"),
        "upstream" => format!("upstream({list})"),
        "answerIp" => format!("ip({list})"),
        "answerGeoip" => format!("ip({})", qualified("geoip")),
        _ => {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
            );
        }
    };
    if is_fragment_bool(&condition) && !condition.contains("->") {
        Ok(Some(condition))
    } else {
        Ok(None)
    }
}

pub fn rule_condition(kind: Value, value: String) -> Result<Option<String>, String> {
    let kind_str = kind.as_str().unwrap_or("");
    rule_condition_str(kind_str, &value)
}
