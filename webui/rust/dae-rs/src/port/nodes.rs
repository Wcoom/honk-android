//! Port of `src/dae/nodes.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::groups::quote_name_str;
use crate::port::js_str::{
    apply_utf16_edits, first_indented_line, index_of_u16, is_js_ws, is_space_or_tab, js_slice,
    js_slice_from, js_to_string, js_trim, js_trim_end, js_trim_start, line_start_u16, obj_defined,
};
use crate::port::text::{
    TextToken, TokenKind, block_entries_raw, block_fields_raw, is_quotable_bool, quote_str,
    scan_config_raw, unquote_str,
};
use crate::utf16;
use indexmap::IndexMap;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct NodeEntry {
    pub name: String,
    pub link: String,
    pub from: usize,
    pub to: usize,
    pub line: usize,
    pub name_to: usize,
    pub link_from: usize,
}

impl NodeEntry {
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "link": self.link,
            "from": self.from,
            "to": self.to,
            "line": self.line,
            "nameTo": self.name_to,
            "linkFrom": self.link_from,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChainHop {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

#[derive(Clone, Debug)]
pub struct ChainEntry {
    pub name: String,
    pub hops: Vec<ChainHop>,
    pub from: usize,
    pub to: usize,
    pub line: usize,
    pub name_to: usize,
    pub hops_from: usize,
}

impl ChainEntry {
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "hops": self.hops.iter().map(|h| json!({"name": h.name, "from": h.from, "to": h.to})).collect::<Vec<_>>(),
            "from": self.from,
            "to": self.to,
            "line": self.line,
            "nameTo": self.name_to,
            "hopsFrom": self.hops_from,
        })
    }
}

pub fn is_node_link_bool(value: &str) -> bool {
    let t = js_trim(value);
    let Some(pos) = t.find("://") else {
        return false;
    };
    let scheme = &t[..pos];
    let rest = &t[pos + 3..];
    if scheme.is_empty() || rest.is_empty() {
        return false;
    }
    let mut bytes = scheme.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    if !bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'.' | b'-')) {
        return false;
    }
    !rest.chars().any(is_js_ws)
}

pub fn is_node_link(value: String) -> Result<Value, String> {
    Ok(Value::Bool(is_node_link_bool(&value)))
}

pub fn read_node_entries_raw(text: &str) -> Vec<NodeEntry> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut out = Vec::new();
    for block in scan.blocks.iter().filter(|b| b.name == "node") {
        for entry in block_entries_raw(text, block) {
            if entry.block.is_some() {
                continue;
            }
            let parts: Vec<&TextToken> = scan
                .tokens
                .iter()
                .filter(|t| {
                    t.from >= entry.from && t.to <= entry.to && t.kind != TokenKind::Comment
                })
                .collect();
            if parts.len() < 3 || js_slice(&units, parts[1].from, parts[1].to) != ":" {
                continue;
            }
            let head = js_slice(&units, parts[0].from, parts[0].to);
            let last_to = parts.last().map_or(parts[2].from, |t| t.to);
            let link = unquote_str(&js_slice(&units, parts[2].from, last_to));
            if !is_node_link_bool(&link) || !is_quotable_bool(&link) {
                continue;
            }
            out.push(NodeEntry {
                name: unquote_str(&head),
                link,
                from: parts[0].from,
                to: last_to,
                line: parts[0].line + 1,
                name_to: parts[0].to,
                link_from: parts[2].from,
            });
        }
    }
    out
}

pub fn read_node_entries(text: String) -> Result<Vec<Value>, String> {
    Ok(read_node_entries_raw(&text)
        .into_iter()
        .map(|e| e.to_value())
        .collect())
}

pub fn read_chain_entries_raw(text: &str) -> Vec<ChainEntry> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut out = Vec::new();
    for block in scan.blocks.iter().filter(|b| b.name == "node") {
        for entry in block_entries_raw(text, block) {
            if entry.block.is_some() {
                continue;
            }
            let parts: Vec<&TextToken> = scan
                .tokens
                .iter()
                .filter(|t| {
                    t.from >= entry.from && t.to <= entry.to && t.kind != TokenKind::Comment
                })
                .collect();
            if parts.len() < 3 || js_slice(&units, parts[1].from, parts[1].to) != ":" {
                continue;
            }
            let value = &parts[2..];
            let raw = |i: usize| js_slice(&units, value[i].from, value[i].to);
            let has_scheme = value.iter().enumerate().any(|(i, token)| {
                raw(i) == ":"
                    && value
                        .get(i + 1)
                        .is_some_and(|n| n.from == token.to && raw(i + 1).starts_with("//"))
            });
            if has_scheme {
                continue;
            }
            let arrows: Vec<&TextToken> = value
                .iter()
                .enumerate()
                .filter(|(i, token)| token.kind == TokenKind::Text && raw(*i) == "->")
                .map(|(_, t)| *t)
                .collect();
            if arrows.is_empty() {
                continue;
            }
            let last_to = value.last().map_or(parts[1].to, |t| t.to);
            let mut bounds = Vec::with_capacity(arrows.len() * 2 + 2);
            bounds.push(parts[1].to);
            for a in arrows {
                bounds.push(a.from);
                bounds.push(a.to);
            }
            bounds.push(last_to);
            let mut hops = Vec::new();
            for pair in bounds.as_chunks::<2>().0 {
                let piece = js_slice(&units, pair[0], pair[1]);
                let piece_u16_len = piece.encode_utf16().count();
                let trim_start_u16_len = js_trim_start(&piece).encode_utf16().count();
                let trim_end_u16_len = js_trim_end(&piece).encode_utf16().count();
                let from = pair[0] + (piece_u16_len - trim_start_u16_len);
                let to = from.max(pair[1] - (piece_u16_len - trim_end_u16_len));
                hops.push(ChainHop {
                    name: unquote_str(&js_slice(&units, from, to)),
                    from,
                    to,
                });
            }
            let head = js_slice(&units, parts[0].from, parts[0].to);
            out.push(ChainEntry {
                name: unquote_str(&head),
                hops,
                from: parts[0].from,
                to: last_to,
                line: parts[0].line + 1,
                name_to: parts[0].to,
                hops_from: value[0].from,
            });
        }
    }
    out
}

pub fn read_chain_entries(text: String) -> Result<Vec<Value>, String> {
    Ok(read_chain_entries_raw(&text)
        .into_iter()
        .map(|e| e.to_value())
        .collect())
}

struct Reference {
    from: usize,
    to: usize,
    kind: &'static str,
    name: String,
}

fn is_name_call_head(s: &str) -> bool {
    let s = s.strip_prefix("&&").unwrap_or(s);
    let s = s.strip_prefix('!').unwrap_or(s);
    s == "name"
}

fn name_references(text: &str, name: &str) -> Vec<Reference> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut refs = Vec::new();

    for block in scan.blocks.iter().filter(|b| b.name == "group") {
        for group in &block.children {
            for field in block_fields_raw(text, group, &scan.tokens) {
                if matches!(field.name.as_str(), "default" | "final" | "exit") {
                    if unquote_str(&field.value) == name {
                        let val_u16_len = field.value.encode_utf16().count();
                        refs.push(Reference {
                            from: field.value_to.saturating_sub(val_u16_len),
                            to: field.value_to,
                            kind: "group",
                            name: group.name.clone(),
                        });
                    }
                    continue;
                }
                if field.name != "filter" {
                    continue;
                }
                let parts: Vec<&TextToken> = scan
                    .tokens
                    .iter()
                    .filter(|t| t.from >= field.value_from && t.to <= field.value_to)
                    .collect();
                let raw = |idx: usize| -> String {
                    parts
                        .get(idx)
                        .map(|t| js_slice(&units, t.from, t.to))
                        .unwrap_or_default()
                };
                if parts.len() >= 3 {
                    for i in 0..parts.len() - 2 {
                        if !is_name_call_head(&raw(i)) || raw(i + 1) != "(" {
                            continue;
                        }
                        let mut j = i + 2;
                        while j < parts.len() && raw(j) != ")" {
                            let prev = raw(j - 1);
                            let next = raw(j + 1);
                            if (prev == "(" || prev == ",")
                                && (next == "," || next == ")")
                                && unquote_str(&raw(j)) == name
                            {
                                refs.push(Reference {
                                    from: parts[j].from,
                                    to: parts[j].to,
                                    kind: "group",
                                    name: group.name.clone(),
                                });
                            }
                            j += 1;
                        }
                    }
                }
            }
        }
    }

    for block in scan.blocks.iter().filter(|b| b.name == "dns") {
        for upstream in block.children.iter().filter(|c| c.name == "upstream") {
            for field in block_fields_raw(text, upstream, &scan.tokens) {
                let parts: Vec<&TextToken> = scan
                    .tokens
                    .iter()
                    .filter(|t| {
                        t.from >= field.value_from
                            && t.to <= field.value_to
                            && t.kind != TokenKind::Comment
                    })
                    .collect();
                if parts.len() >= 2 {
                    let target = parts[parts.len() - 1];
                    let arrow = parts[parts.len() - 2];
                    if js_slice(&units, arrow.from, arrow.to) == "->"
                        && unquote_str(&js_slice(&units, target.from, target.to)) == name
                    {
                        refs.push(Reference {
                            from: target.from,
                            to: target.to,
                            kind: "dns",
                            name: field.name,
                        });
                    }
                }
            }
        }
    }

    for chain in read_chain_entries_raw(text) {
        for hop in chain.hops {
            if hop.name == name {
                refs.push(Reference {
                    from: hop.from,
                    to: hop.to,
                    kind: "chain",
                    name: chain.name.clone(),
                });
            }
        }
    }

    refs
}

pub fn node_referenced(text: String, name: String) -> Result<Value, String> {
    Ok(Value::Bool(!name_references(&text, &name).is_empty()))
}

pub fn node_referrers(text: String, name: String) -> Result<Vec<Value>, String> {
    let mut unique: IndexMap<String, (&'static str, String)> = IndexMap::new();
    for r in name_references(&text, &name) {
        unique.insert(format!("{}:{}", r.kind, r.name), (r.kind, r.name));
    }
    Ok(unique
        .into_values()
        .map(|(kind, name)| json!({"kind": kind, "name": name}))
        .collect())
}

fn current_chain(text: &str, original: &Value) -> Result<ChainEntry, String> {
    let Some(o) = original.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'name')".to_string());
    };
    let orig_name = obj_defined(o, "name").and_then(Value::as_str);
    let orig_hops: Option<Vec<String>> =
        obj_defined(o, "hops").and_then(Value::as_array).map(|arr| {
            arr.iter()
                .map(|h| {
                    h.as_object()
                        .and_then(|ho| obj_defined(ho, "name"))
                        .map(js_to_string)
                        .unwrap_or_default()
                })
                .collect()
        });
    let entries: Vec<ChainEntry> = read_chain_entries_raw(text)
        .into_iter()
        .filter(|e| Some(e.name.as_str()) == orig_name)
        .collect();
    if entries.len() != 1 {
        return Err("nodes.editNodeMissing".to_string());
    }
    let found_hops: Vec<String> = entries[0].hops.iter().map(|h| h.name.clone()).collect();
    if orig_hops.as_deref() != Some(found_hops.as_slice()) {
        return Err("nodes.editNodeMissing".to_string());
    }
    Ok(entries.into_iter().next().unwrap_or_else(|| unreachable!()))
}

pub fn remove_chain_entry(text: String, original: Value) -> Result<String, String> {
    let entry = current_chain(&text, &original)?;
    let units = utf16::units(&text);
    let start = line_start_u16(&units, entry.from);
    let newline = index_of_u16(&units, b'\n' as u16, entry.to);
    let end = newline.unwrap_or(units.len());
    let before = js_slice(&units, start, entry.from);
    let after = js_slice(&units, entry.to, end);
    let after_trimmed = js_trim_start(&after);
    let only_ws_or_comment = after_trimmed.is_empty() || after_trimmed.starts_with('#');
    if before.chars().any(|c| !is_js_ws(c)) || !only_ws_or_comment {
        return Ok(format!(
            "{}{}",
            js_slice(&units, 0, entry.from),
            js_slice_from(&units, entry.to),
        ));
    }
    let cut_to = if newline.is_none() { end } else { end + 1 };
    Ok(format!(
        "{}{}",
        js_slice(&units, 0, start),
        js_slice_from(&units, cut_to),
    ))
}

fn chain_hop_candidate_bool(node: &Value) -> Result<bool, String> {
    if node.is_null() || js::is_undefined(node) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    let proto = node
        .as_object()
        .and_then(|o| obj_defined(o, "protocol"))
        .and_then(Value::as_str)
        .unwrap_or("");
    Ok(!matches!(proto, "chain" | "direct" | "block"))
}

pub fn chain_hop_candidate(node: Value) -> Result<Value, String> {
    Ok(Value::Bool(chain_hop_candidate_bool(&node)?))
}

fn tunnel_dialable_bool(node: &Value) -> Result<bool, String> {
    if node.is_null() || js::is_undefined(node) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    let proto = node
        .as_object()
        .and_then(|o| obj_defined(o, "protocol"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let transport = node
        .as_object()
        .and_then(|o| obj_defined(o, "stream_transport"))
        .and_then(Value::as_str);
    Ok(matches!(
        proto,
        "socks5" | "ss" | "shadowsocks" | "trojan" | "vmess" | "vless"
    ) && transport != Some("xhttp"))
}

pub fn tunnel_dialable(node: Value) -> Result<Value, String> {
    Ok(Value::Bool(tunnel_dialable_bool(&node)?))
}

pub fn chain_hop_problem(
    nodes: Vec<Value>,
    name: String,
    index: f64,
) -> Result<Option<String>, String> {
    let matches: Vec<&Value> = nodes
        .iter()
        .filter(|n| {
            n.as_object()
                .and_then(|o| obj_defined(o, "name"))
                .and_then(Value::as_str)
                == Some(name.as_str())
        })
        .collect();
    if matches.len() != 1 {
        return Ok(Some(if !matches.is_empty() {
            "ambiguous-chain-hop-reference".to_string()
        } else {
            "invalid-chain-hop-reference".to_string()
        }));
    }
    let m = matches[0];
    if !chain_hop_candidate_bool(m)? || (index > 0.0 && !tunnel_dialable_bool(m)?) {
        Ok(Some("unsupported-chain-hop".to_string()))
    } else {
        Ok(None)
    }
}

pub fn chain_udp_problem(hops: Vec<Value>) -> Result<Value, String> {
    for node in &hops {
        if !tunnel_dialable_bool(node)? {
            let name = node
                .as_object()
                .and_then(|o| o.get("name"))
                .cloned()
                .unwrap_or_else(js::undefined);
            return Ok(json!({"kind": "hop", "name": name}));
        }
    }
    if let Some(exit) = hops.last() {
        let proto = exit
            .as_object()
            .and_then(|o| obj_defined(o, "protocol"))
            .and_then(Value::as_str);
        if !matches!(proto, Some("trojan" | "vless")) {
            let name = exit
                .as_object()
                .and_then(|o| o.get("name"))
                .cloned()
                .unwrap_or_else(js::undefined);
            return Ok(json!({"kind": "exit", "name": name}));
        }
    }
    Ok(Value::Null)
}

fn parse_chain_update(next: &Value) -> Result<(String, Vec<String>), String> {
    let Some(o) = next.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'hops')".to_string());
    };
    let Some(hops_v) = obj_defined(o, "hops") else {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
        );
    };
    let Some(hops_arr) = hops_v.as_array() else {
        return Err("TypeError: next.hops.some is not a function".to_string());
    };
    let hops = hops_arr
        .iter()
        .map(|h| h.as_str().map(str::to_string).unwrap_or_default())
        .collect();
    let name = obj_defined(o, "name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok((name, hops))
}

fn chain_text(
    text: &str,
    name: &str,
    hops: &[String],
    renamed_from: Option<&str>,
) -> Result<(String, String), String> {
    if hops.len() < 2 || hops.len() > 4 {
        return Err("nodes.proxyChain.hopCount".to_string());
    }
    if name.is_empty()
        || !is_quotable_bool(name)
        || hops.iter().any(|h| h.is_empty() || !is_quotable_bool(h))
    {
        return Err("config.unquotable".to_string());
    }
    if renamed_from != Some(name) {
        let taken = read_node_entries_raw(text).iter().any(|e| e.name == name)
            || read_chain_entries_raw(text).iter().any(|e| e.name == name);
        if taken {
            return Err("nodes.nameTaken".to_string());
        }
    }
    let quoted_name = quote_str(name)?;
    let quoted_hops = hops
        .iter()
        .map(|h| quote_name_str(h))
        .collect::<Result<Vec<_>, _>>()?
        .join(" -> ");
    Ok((quoted_name, quoted_hops))
}

fn rewrite_entry(
    text: &str,
    from_name: &str,
    to_name: &str,
    name_edit: (usize, usize, String),
    value_edit: Option<(usize, usize, String)>,
) -> Result<String, String> {
    let mut changes: Vec<(usize, usize, String)> = Vec::new();
    if let Some(ve) = value_edit {
        changes.push(ve);
    }
    if to_name != from_name {
        changes.push(name_edit);
        let qn = quote_name_str(to_name)?;
        for r in name_references(text, from_name) {
            changes.push((r.from, r.to, qn.clone()));
        }
    }
    Ok(apply_utf16_edits(text, changes, false))
}

pub fn write_chain_entry(text: String, original: Value, next: Value) -> Result<String, String> {
    let entry = current_chain(&text, &original)?;
    let (next_name, next_hops) = parse_chain_update(&next)?;
    let (quoted_name, value) = chain_text(&text, &next_name, &next_hops, Some(&entry.name))?;
    let old_hops_joined = entry
        .hops
        .iter()
        .map(|h| h.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let hops_edit = if next_hops.join("\n") != old_hops_joined {
        Some((entry.hops_from, entry.to, value))
    } else {
        None
    };
    rewrite_entry(
        &text,
        &entry.name,
        &next_name,
        (entry.from, entry.name_to, quoted_name),
        hops_edit,
    )
}

pub fn append_chain_entry(text: String, next: Value) -> Result<String, String> {
    let (next_name, next_hops) = parse_chain_update(&next)?;
    let (name, value) = chain_text(&text, &next_name, &next_hops, None)?;
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let scan = scan_config_raw(&text);
    let Some(block) = scan.blocks.iter().rfind(|b| b.name == "node") else {
        let sep = if js_trim(&text).is_empty() {
            String::new()
        } else {
            format!("{eol}{eol}")
        };
        return Ok(format!(
            "{}{sep}node {{{eol}  {name}: {value}{eol}}}{eol}",
            js_trim_end(&text)
        ));
    };
    let units = utf16::units(&text);
    let indent = first_indented_line(&js_slice(&units, block.open + 1, block.close))
        .unwrap_or_else(|| "  ".to_string());
    let close_line = line_start_u16(&units, block.close);
    let (head, tail) = if is_space_or_tab(&js_slice(&units, close_line, block.close)) {
        (
            js_slice(&units, 0, close_line),
            js_slice_from(&units, close_line),
        )
    } else {
        (
            format!("{}{eol}", js_trim_end(&js_slice(&units, 0, block.close))),
            js_slice_from(&units, block.close),
        )
    };
    Ok(format!("{head}{indent}{name}: {value}{eol}{tail}"))
}

pub fn write_node_entry(text: String, original: Value, next: Value) -> Result<String, String> {
    let Some(oo) = original.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'name')".to_string());
    };
    let orig_name = obj_defined(oo, "name").and_then(Value::as_str);
    let orig_link = obj_defined(oo, "link").and_then(Value::as_str);
    let all = read_node_entries_raw(&text);
    let entries: Vec<&NodeEntry> = all
        .iter()
        .filter(|e| Some(e.name.as_str()) == orig_name)
        .collect();
    if entries.len() != 1 || Some(entries[0].link.as_str()) != orig_link {
        return Err("nodes.editNodeMissing".to_string());
    }
    let Some(no) = next.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'name')".to_string());
    };
    let next_name = obj_defined(no, "name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let next_link = obj_defined(no, "link")
        .and_then(Value::as_str)
        .unwrap_or("");
    if next_name.is_empty()
        || !is_quotable_bool(next_name)
        || !is_node_link_bool(next_link)
        || !is_quotable_bool(next_link)
    {
        return Err("config.unquotable".to_string());
    }
    let entry = entries[0];
    if next_name != entry.name && all.iter().any(|e| e.name == next_name) {
        return Err("nodes.nameTaken".to_string());
    }
    let link_edit = if next_link != entry.link {
        Some((entry.link_from, entry.to, quote_str(next_link)?))
    } else {
        None
    };
    rewrite_entry(
        &text,
        &entry.name,
        next_name,
        (entry.from, entry.name_to, quote_str(next_name)?),
        link_edit,
    )
}
