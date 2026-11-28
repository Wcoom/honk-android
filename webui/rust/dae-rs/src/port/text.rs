//! Port of `src/dae/text.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    ensure_not_nullish, index_of_u16, is_js_ws, is_js_ws_u16, is_space_or_tab, js_slice,
    js_slice_f64, js_trim, js_val_to_number, last_index_of_u16, last_index_of_u16_f64,
    val_as_usize,
};
use crate::utf16;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Text,
    Quoted,
    Comment,
    Symbol,
}

impl TokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Quoted => "quoted",
            Self::Comment => "comment",
            Self::Symbol => "symbol",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "quoted" => Some(Self::Quoted),
            "comment" => Some(Self::Comment),
            "symbol" => Some(Self::Symbol),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextToken {
    pub from: usize,
    pub to: usize,
    pub line: usize,
    pub kind: TokenKind,
    pub depth: usize,
    pub parens: usize,
}

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub name: String,
    pub from: usize,
    pub open: usize,
    pub close: usize,
    pub to: usize,
    pub line: usize,
    pub end_line: usize,
    pub depth: usize,
    pub children: Vec<TextBlock>,
}

#[derive(Clone, Debug)]
pub struct TextField {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub value_from: usize,
    pub value_to: usize,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub blocks: Vec<TextBlock>,
    pub tokens: Vec<TextToken>,
}

#[derive(Clone, Debug)]
pub struct BlockEntry {
    pub from: usize,
    pub to: usize,
    pub block: Option<TextBlock>,
}

impl TextToken {
    pub fn to_value(&self) -> Value {
        json!({
            "from": self.from,
            "to": self.to,
            "line": self.line,
            "kind": self.kind.as_str(),
            "depth": self.depth,
            "parens": self.parens,
        })
    }

    pub fn from_value(v: &Value) -> Option<Self> {
        let o = v.as_object()?;
        Some(Self {
            from: o.get("from").and_then(val_as_usize).unwrap_or(0),
            to: o.get("to").and_then(val_as_usize).unwrap_or(0),
            line: o.get("line").and_then(val_as_usize).unwrap_or(0),
            kind: o
                .get("kind")
                .and_then(Value::as_str)
                .and_then(TokenKind::from_str)
                .unwrap_or(TokenKind::Text),
            depth: o.get("depth").and_then(val_as_usize).unwrap_or(0),
            parens: o.get("parens").and_then(val_as_usize).unwrap_or(0),
        })
    }
}

impl TextBlock {
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "from": self.from,
            "open": self.open,
            "close": self.close,
            "to": self.to,
            "line": self.line,
            "endLine": self.end_line,
            "depth": self.depth,
            "children": self.children.iter().map(Self::to_value).collect::<Vec<_>>(),
        })
    }

    pub fn from_value(v: &Value) -> Result<Self, String> {
        let Some(o) = v.as_object() else {
            return Err("TypeError: Cannot read properties of null (reading 'open')".to_string());
        };
        let children = match o.get("children").and_then(Value::as_array) {
            Some(arr) => arr
                .iter()
                .map(Self::from_value)
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        Ok(Self {
            name: o
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            from: o.get("from").and_then(val_as_usize).unwrap_or(0),
            open: o.get("open").and_then(val_as_usize).unwrap_or(0),
            close: o.get("close").and_then(val_as_usize).unwrap_or(0),
            to: o.get("to").and_then(val_as_usize).unwrap_or(0),
            line: o.get("line").and_then(val_as_usize).unwrap_or(0),
            end_line: o.get("endLine").and_then(val_as_usize).unwrap_or(0),
            depth: o.get("depth").and_then(val_as_usize).unwrap_or(0),
            children,
        })
    }
}

impl TextField {
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "from": self.from,
            "to": self.to,
            "valueFrom": self.value_from,
            "valueTo": self.value_to,
            "value": self.value,
        })
    }
}

impl ScanResult {
    pub fn to_value(&self) -> Value {
        json!({
            "blocks": self.blocks.iter().map(TextBlock::to_value).collect::<Vec<_>>(),
            "tokens": self.tokens.iter().map(TextToken::to_value).collect::<Vec<_>>(),
        })
    }

    pub fn from_value(v: &Value) -> Result<Self, String> {
        let Some(o) = v.as_object() else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'filter')".to_string(),
            );
        };
        let Some(blocks_arr) = o.get("blocks").and_then(Value::as_array) else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'filter')".to_string(),
            );
        };
        let blocks = blocks_arr
            .iter()
            .map(TextBlock::from_value)
            .collect::<Result<Vec<_>, _>>()?;
        let tokens = match o.get("tokens").and_then(Value::as_array) {
            Some(arr) => arr.iter().filter_map(TextToken::from_value).collect(),
            None => Vec::new(),
        };
        Ok(Self { blocks, tokens })
    }
}

pub fn is_bare_name_bool(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

pub fn is_bare_name(value: String) -> Result<Value, String> {
    Ok(Value::Bool(is_bare_name_bool(&value)))
}

pub fn quote_str(value: &str) -> Result<String, String> {
    let trailing_bs = value.bytes().rev().take_while(|&b| b == b'\\').count();
    if value.contains('\'') || value.contains('\r') || value.contains('\n') || trailing_bs % 2 == 1
    {
        return Err("config.unquotable".to_string());
    }
    Ok(format!("'{value}'"))
}

pub fn quote(value: String) -> Result<String, String> {
    quote_str(&value)
}

pub fn is_quotable_bool(value: &str) -> bool {
    quote_str(value).is_ok()
}

pub fn is_quotable(value: String) -> Result<bool, String> {
    Ok(is_quotable_bool(&value))
}

pub fn unquote_str(text: &str) -> String {
    if text.starts_with('\'') || text.starts_with('"') {
        let units = utf16::units(text);
        if units.len() <= 2 {
            String::new()
        } else {
            js_slice(&units, 1, units.len() - 1)
        }
    } else {
        text.to_string()
    }
}

pub fn unquote(text: String) -> Result<Value, String> {
    Ok(Value::String(unquote_str(&text)))
}

fn is_symbol_u16(u: u16) -> bool {
    matches!(u, 0x7B | 0x7D | 0x3A | 0x28 | 0x29 | 0x2C) // { } : ( ) ,
}

fn is_word_stop_u16(u: u16) -> bool {
    is_js_ws_u16(u) || is_symbol_u16(u) || u == b'\'' as u16 || u == b'"' as u16
}

pub fn scan_config_raw(text: &str) -> ScanResult {
    let units = utf16::units(text);
    let len = units.len();
    let mut tokens: Vec<TextToken> = Vec::new();
    let mut blocks: Vec<TextBlock> = Vec::new();
    let mut stack: Vec<TextBlock> = Vec::new();
    let mut line: usize = 0;
    let mut parens: usize = 0;
    let mut i: usize = 0;

    while i < len {
        let c = units[i];
        if is_js_ws_u16(c) {
            if c == b'\n' as u16 {
                line += 1;
            }
            i += 1;
            continue;
        }
        let from = i;
        let start_line = line;
        let kind;
        if c == b'#' as u16 {
            kind = TokenKind::Comment;
            while i < len && units[i] != b'\n' as u16 {
                i += 1;
            }
        } else if c == b'\'' as u16 || c == b'"' as u16 {
            kind = TokenKind::Quoted;
            i += 1;
            while i < len {
                let next = units[i];
                i += 1;
                if next == b'\n' as u16 {
                    line += 1;
                }
                if next == b'\\' as u16 && i < len {
                    if units[i] == b'\n' as u16 {
                        line += 1;
                    }
                    i += 1;
                } else if next == c {
                    break;
                }
            }
        } else if is_symbol_u16(c) {
            kind = TokenKind::Symbol;
            i += 1;
        } else if c == b'-' as u16 && i + 1 < len && units[i + 1] == b'>' as u16 {
            kind = TokenKind::Text;
            i += 2;
        } else {
            kind = TokenKind::Text;
            while i < len
                && !is_word_stop_u16(units[i])
                && !(units[i] == b'-' as u16 && i + 1 < len && units[i + 1] == b'>' as u16)
            {
                i += 1;
            }
        }
        let token = TextToken {
            from,
            to: i,
            line: start_line,
            kind,
            depth: stack.len(),
            parens,
        };
        if kind == TokenKind::Symbol && c == b'{' as u16 {
            let mut head_index = tokens.len() as isize - 1;
            while head_index >= 0 {
                let t = &tokens[head_index as usize];
                if t.kind == TokenKind::Comment
                    || (t.to - t.from == 1 && units[t.from] == b':' as u16)
                {
                    head_index -= 1;
                } else {
                    break;
                }
            }
            let head = if head_index >= 0 {
                Some(&tokens[head_index as usize])
            } else {
                None
            };
            let block = TextBlock {
                name: head
                    .map(|h| unquote_str(&js_slice(&units, h.from, h.to)))
                    .unwrap_or_default(),
                from: head.map_or(from, |h| h.from),
                open: from,
                close: len,
                to: len,
                line: head.map_or(line, |h| h.line),
                end_line: line,
                depth: stack.len(),
                children: Vec::new(),
            };
            stack.push(block);
        } else if kind == TokenKind::Symbol && c == b'}' as u16 {
            if let Some(mut block) = stack.pop() {
                block.close = from;
                block.to = i;
                block.end_line = line;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(block);
                } else {
                    blocks.push(block);
                }
            }
        } else if kind == TokenKind::Symbol && c == b'(' as u16 {
            parens += 1;
        } else if kind == TokenKind::Symbol && c == b')' as u16 {
            parens = parens.saturating_sub(1);
        }
        tokens.push(token);
    }

    while let Some(block) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(block);
        } else {
            blocks.push(block);
        }
    }

    ScanResult { blocks, tokens }
}

pub fn scan_config(text: String) -> Result<Value, String> {
    Ok(scan_config_raw(&text).to_value())
}

pub fn is_fragment_bool(text: &str) -> bool {
    if text.contains('\r') || text.contains('\n') {
        return false;
    }
    let units = utf16::units(text);
    let mut depth: isize = 0;
    for token in scan_config_raw(text).tokens {
        if token.kind == TokenKind::Comment {
            return false;
        }
        if token.kind == TokenKind::Quoted {
            continue;
        }
        let raw = js_slice(&units, token.from, token.to);
        if raw.contains('{') || raw.contains('}') {
            return false;
        }
        if raw == "(" {
            depth += 1;
        }
        if raw == ")" {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
    }
    depth == 0
}

pub fn is_fragment(text: String) -> Result<bool, String> {
    Ok(is_fragment_bool(&text))
}

pub fn uncomment_str(text: &str) -> String {
    let mut units = utf16::units(text);
    let comments: Vec<TextToken> = scan_config_raw(text)
        .tokens
        .into_iter()
        .filter(|t| t.kind == TokenKind::Comment)
        .collect();
    for token in comments.into_iter().rev() {
        let end = token.to.min(units.len());
        let start = token.from.min(end);
        units.drain(start..end);
    }
    String::from_utf16_lossy(&units)
}

pub fn uncomment(text: String) -> Result<String, String> {
    Ok(uncomment_str(&text))
}

pub fn block_fields_raw(text: &str, block: &TextBlock, tokens: &[TextToken]) -> Vec<TextField> {
    let units = utf16::units(text);
    let mut fields: Vec<TextField> = Vec::new();
    let mut has_current = false;
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if token.from <= block.open || token.from >= block.close {
            i += 1;
            continue;
        }
        let raw = js_slice(&units, token.from, token.to);
        if token.parens == 0
            && (token.kind == TokenKind::Comment
                || (has_current
                    && fields
                        .last()
                        .is_some_and(|f| js_slice(&units, f.value_to, token.from).contains('\n'))))
        {
            has_current = false;
        }
        if token.depth != block.depth + 1 || token.kind == TokenKind::Comment {
            i += 1;
            continue;
        }
        let next = tokens.get(i + 1);
        let starts_field = !has_current
            || fields.last().is_some_and(|f| {
                f.value_to > f.value_from
                    && token.from > f.value_to
                    && i > 0
                    && js_slice(&units, tokens[i - 1].from, tokens[i - 1].to) != ","
            });
        if starts_field
            && token.parens == 0
            && (token.kind == TokenKind::Text || token.kind == TokenKind::Quoted)
            && next.is_some_and(|n| js_slice(&units, n.from, n.to) == ":")
        {
            let next_to = next.map_or(token.to, |n| n.to);
            fields.push(TextField {
                name: unquote_str(&raw),
                from: token.from,
                to: next_to,
                value_from: next_to,
                value_to: next_to,
                value: String::new(),
            });
            has_current = true;
            i += 2;
        } else {
            if has_current && let Some(f) = fields.last_mut() {
                f.to = token.to;
                f.value_to = token.to;
            }
            i += 1;
        }
    }
    for f in &mut fields {
        let slice = js_slice(&units, f.value_from, f.value_to);
        f.value = js_trim(&slice).to_string();
    }
    fields
}

pub fn block_fields(text: String, block: Value, tokens: Vec<Value>) -> Result<Vec<Value>, String> {
    let bo = ensure_not_nullish(&block, "open")?;
    let prop_f64 = |k: &str| -> f64 {
        bo.and_then(|m| m.get(k))
            .map(js_val_to_number)
            .unwrap_or(f64::NAN)
    };
    let b_open = prop_f64("open");
    let b_close = prop_f64("close");
    let b_depth = prop_f64("depth");
    let toks: Vec<TextToken> = tokens.iter().filter_map(TextToken::from_value).collect();
    let units = utf16::units(&text);
    let mut fields: Vec<TextField> = Vec::new();
    let mut has_current = false;
    let mut i = 0;
    while i < toks.len() {
        let token = &toks[i];
        let tf = token.from as f64;
        if tf <= b_open || tf >= b_close {
            i += 1;
            continue;
        }
        let raw = js_slice(&units, token.from, token.to);
        if token.parens == 0
            && (token.kind == TokenKind::Comment
                || (has_current
                    && fields
                        .last()
                        .is_some_and(|f| js_slice(&units, f.value_to, token.from).contains('\n'))))
        {
            has_current = false;
        }
        if (token.depth as f64) != b_depth + 1.0 || token.kind == TokenKind::Comment {
            i += 1;
            continue;
        }
        let next = toks.get(i + 1);
        let starts_field = !has_current
            || fields.last().is_some_and(|f| {
                f.value_to > f.value_from
                    && token.from > f.value_to
                    && i > 0
                    && js_slice(&units, toks[i - 1].from, toks[i - 1].to) != ","
            });
        if starts_field
            && token.parens == 0
            && (token.kind == TokenKind::Text || token.kind == TokenKind::Quoted)
            && next.is_some_and(|n| js_slice(&units, n.from, n.to) == ":")
        {
            let next_to = next.map_or(token.to, |n| n.to);
            fields.push(TextField {
                name: unquote_str(&raw),
                from: token.from,
                to: next_to,
                value_from: next_to,
                value_to: next_to,
                value: String::new(),
            });
            has_current = true;
            i += 2;
        } else {
            if has_current && let Some(f) = fields.last_mut() {
                f.to = token.to;
                f.value_to = token.to;
            }
            i += 1;
        }
    }
    for f in &mut fields {
        let slice = js_slice(&units, f.value_from, f.value_to);
        f.value = js_trim(&slice).to_string();
    }
    Ok(fields.into_iter().map(|f| f.to_value()).collect())
}

pub fn block_body_raw(text: &str, block: &TextBlock) -> Vec<String> {
    let units = utf16::units(text);
    let body = js_slice(&units, block.open + 1, block.close);
    if block.line == block.end_line {
        return vec![body];
    }
    strip_block_body_lines(&body)
}

fn strip_block_body_lines(body: &str) -> Vec<String> {
    let mut s = body;
    let lead_end = s.char_indices().find(|&(_, c)| !is_js_ws(c) || c == '\n');
    if let Some((idx, '\n')) = lead_end {
        s = &s[idx + 1..];
    }
    if let Some(nl) = s.rfind('\n')
        && s[nl + 1..].chars().all(|c| is_js_ws(c) && c != '\n')
    {
        s = &s[..nl];
    }
    s.split('\n').map(str::to_string).collect()
}

pub fn block_body(text: String, block: Value) -> Result<Vec<String>, String> {
    let bo = ensure_not_nullish(&block, "open")?;
    let open = bo
        .and_then(|m| m.get("open"))
        .map(js_val_to_number)
        .unwrap_or(f64::NAN);
    let close_val = bo.and_then(|m| m.get("close"));
    let units = utf16::units(&text);
    let close = match close_val {
        None => units.len() as f64,
        Some(v) if js::is_undefined(v) => units.len() as f64,
        Some(v) => js_val_to_number(v),
    };
    let line_v = bo
        .and_then(|m| m.get("line"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let end_line_v = bo
        .and_then(|m| m.get("endLine"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let same_line = match (js::as_number(&line_v), js::as_number(&end_line_v)) {
        (Some(a), Some(b)) => a == b,
        _ => line_v == end_line_v,
    };
    let body = js_slice_f64(&units, open + 1.0, close);
    if same_line {
        return Ok(vec![body]);
    }
    Ok(strip_block_body_lines(&body))
}

pub fn block_entries_raw(text: &str, block: &TextBlock) -> Vec<BlockEntry> {
    let units = utf16::units(text);
    let mut entries = Vec::new();
    let mut from = block.open + 1;
    let head = js_slice(&units, from, block.close);
    let has_lead_nl = head.chars().find(|&c| !is_js_ws(c) || c == '\n') == Some('\n');
    if has_lead_nl && let Some(nl) = index_of_u16(&units, b'\n' as u16, from) {
        from = nl + 1;
    }
    let last_line = match last_index_of_u16(&units, b'\n' as u16, block.close as isize - 1) {
        Some(pos) => pos + 1,
        None => 0,
    };
    let end = if last_line > from && is_space_or_tab(&js_slice(&units, last_line, block.close)) {
        last_line - 1
    } else {
        block.close
    };
    while from <= end {
        let newline = index_of_u16(&units, b'\n' as u16, from);
        let mut to = match newline {
            None => end,
            Some(nl) => nl.min(end),
        };
        let child = block
            .children
            .iter()
            .find(|child| child.from >= from && child.from <= to)
            .cloned();
        if let Some(ref c) = child {
            let after = index_of_u16(&units, b'\n' as u16, c.to);
            to = match after {
                None => end,
                Some(nl) => nl.min(end),
            };
        }
        entries.push(BlockEntry {
            from,
            to,
            block: child,
        });
        from = to + 1;
    }
    entries
}

pub fn block_entries(text: String, block: Value) -> Result<Vec<Value>, String> {
    let bo = ensure_not_nullish(&block, "open")?;
    let prop_f64 = |k: &str| -> f64 {
        bo.and_then(|m| m.get(k))
            .map(js_val_to_number)
            .unwrap_or(f64::NAN)
    };
    let open = prop_f64("open");
    let close = prop_f64("close");
    let units = utf16::units(&text);
    let mut from: f64 = open + 1.0;
    let head = js_slice_f64(&units, from, close);
    let has_lead_nl = head.chars().find(|&c| !is_js_ws(c) || c == '\n') == Some('\n');
    if has_lead_nl {
        let from_idx = if from.is_nan() || from < 0.0 {
            0
        } else if from.is_infinite() {
            units.len()
        } else {
            (from.trunc() as usize).min(units.len())
        };
        from = match index_of_u16(&units, b'\n' as u16, from_idx) {
            Some(nl) => (nl + 1) as f64,
            None => 0.0,
        };
    }
    let last_line = match last_index_of_u16_f64(&units, b'\n' as u16, close - 1.0) {
        Some(pos) => (pos + 1) as f64,
        None => 0.0,
    };
    let end = if last_line > from && is_space_or_tab(&js_slice_f64(&units, last_line, close)) {
        last_line - 1.0
    } else {
        close
    };
    let children_val = bo.and_then(|m| m.get("children"));
    let mut entries = Vec::new();
    while from <= end {
        let from_idx = if from.is_nan() || from < 0.0 {
            0
        } else if from.is_infinite() {
            units.len()
        } else {
            (from.trunc() as usize).min(units.len())
        };
        let newline = index_of_u16(&units, b'\n' as u16, from_idx);
        let mut to: f64 = match newline {
            None => end,
            Some(nl) => (nl as f64).min(end),
        };
        let Some(children) = children_val.and_then(Value::as_array) else {
            return Err(
                "TypeError: Cannot read properties of undefined (reading 'find')".to_string(),
            );
        };
        let mut child_found: Option<&Value> = None;
        for c in children {
            let co = ensure_not_nullish(c, "from")?;
            let cf = co
                .and_then(|m| m.get("from"))
                .map(js_val_to_number)
                .unwrap_or(f64::NAN);
            if cf >= from && cf <= to {
                child_found = Some(c);
                break;
            }
        }
        if let Some(c) = child_found {
            let ct = c
                .as_object()
                .and_then(|m| m.get("to"))
                .map(js_val_to_number)
                .unwrap_or(f64::NAN);
            let ct_idx = if ct.is_nan() || ct < 0.0 {
                0
            } else if ct.is_infinite() {
                units.len()
            } else {
                (ct.trunc() as usize).min(units.len())
            };
            let after = index_of_u16(&units, b'\n' as u16, ct_idx);
            to = match after {
                None => end,
                Some(nl) => (nl as f64).min(end),
            };
        }
        entries.push(json!({
            "from": js::number(from),
            "to": js::number(to),
            "block": match child_found {
                Some(c) => c.clone(),
                None => js::undefined(),
            },
        }));
        from = to + 1.0;
        if entries.len() > units.len() + 16 {
            break;
        }
    }
    Ok(entries)
}
