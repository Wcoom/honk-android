//! Port of `src/dae/subscriptions.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    apply_utf16_edits, first_indented_line, index_of_u16, is_js_ws, is_space_or_tab, js_slice,
    js_str_to_number, js_to_string, js_trim, leading_space_tabs, line_start_u16, obj_defined,
};
use crate::port::text::{
    TextBlock, TextToken, TokenKind, is_bare_name_bool, is_quotable_bool, quote_str,
    scan_config_raw, unquote_str,
};
use crate::utf16;
use serde_json::{Value, json};
use url::Url;

fn match_scheme_not_port(text: &str) -> Option<&str> {
    let colon = text.find(':')?;
    let scheme = &text[..colon];
    if scheme.is_empty() {
        return None;
    }
    let mut b = scheme.bytes();
    if !b.next()?.is_ascii_alphabetic() {
        return None;
    }
    if !b.all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'.' | b'-')) {
        return None;
    }
    let after = &text[colon + 1..];
    let digit_len = after.bytes().take_while(u8::is_ascii_digit).count();
    if digit_len > 0 {
        let next = after.as_bytes().get(digit_len).copied();
        if matches!(next, None | Some(b'/' | b'?' | b'#')) {
            return None;
        }
    }
    Some(scheme)
}

pub fn complete_subscription_url_str(value: &str) -> String {
    let text = js_trim(value);
    if let Some(scheme) = match_scheme_not_port(text) {
        if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
            return format!("{}{}", scheme.to_ascii_lowercase(), &text[scheme.len()..]);
        }
        return text.to_string();
    }
    if text.is_empty() {
        return String::new();
    }
    let completed = format!("https://{text}");
    let Ok(parsed) = Url::parse(&completed) else {
        return text.to_string();
    };
    let hostname = parsed.host_str().unwrap_or("");
    if hostname.contains('.') || hostname == "localhost" || hostname.starts_with('[') {
        completed
    } else {
        text.to_string()
    }
}

pub fn complete_subscription_url(value: String) -> Result<Value, String> {
    Ok(Value::String(complete_subscription_url_str(&value)))
}

pub fn is_subscription_url(value: String) -> Result<Value, String> {
    let completed = complete_subscription_url_str(&value);
    let rest = completed
        .strip_prefix("http://")
        .or_else(|| completed.strip_prefix("https://"));
    let ok = rest.is_some_and(|r| !r.is_empty() && !r.chars().any(is_js_ws));
    Ok(Value::Bool(ok))
}

pub fn parse_interval_num(value: &str) -> Option<f64> {
    let text = js_trim(value);
    if let Some(body) = text.strip_suffix("ms") {
        let num_str = body.strip_prefix('+').unwrap_or(body);
        if is_unsigned_js_float(num_str) {
            return Some((js_str_to_number(num_str) / 1000.0).ceil());
        }
    }
    let rest = text.strip_prefix('+').unwrap_or(text);
    let (digits, scale) = if let Some(d) = rest.strip_suffix('s') {
        (d, 1.0)
    } else if let Some(d) = rest.strip_suffix('m') {
        (d, 60.0)
    } else if let Some(d) = rest.strip_suffix('h') {
        (d, 3600.0)
    } else {
        (rest, 1.0)
    };
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        return Some(js_str_to_number(digits) * scale);
    }
    None
}

fn is_unsigned_js_float(s: &str) -> bool {
    let (mantissa, exp) = match s.find(['e', 'E']) {
        Some(pos) => (&s[..pos], Some(&s[pos + 1..])),
        None => (s, None),
    };
    if let Some(e) = exp {
        let e = e
            .strip_prefix('+')
            .or_else(|| e.strip_prefix('-'))
            .unwrap_or(e);
        if e.is_empty() || !e.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    if mantissa.is_empty() {
        return false;
    }
    match mantissa.find('.') {
        Some(dot) => {
            let a = &mantissa[..dot];
            let b = &mantissa[dot + 1..];
            (!a.is_empty() || !b.is_empty())
                && a.bytes().all(|c| c.is_ascii_digit())
                && b.bytes().all(|c| c.is_ascii_digit())
        }
        None => mantissa.bytes().all(|c| c.is_ascii_digit()),
    }
}

pub fn parse_interval(value: String) -> Result<Value, String> {
    Ok(match parse_interval_num(&value) {
        Some(n) => js::number(n),
        None => Value::Null,
    })
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}

pub fn url_host_str(url: Option<&str>) -> Option<String> {
    let u = url.filter(|s| !s.is_empty())?;
    let parsed = Url::parse(u).ok()?;
    parsed
        .host_str()
        .filter(|h| !h.is_empty())
        .map(str::to_string)
}

pub fn url_host(url: Option<String>) -> Result<Value, String> {
    Ok(match url_host_str(url.as_deref()) {
        Some(h) => Value::String(h),
        None => Value::Null,
    })
}

pub fn agent_problem(agent: String, written: bool) -> Result<Option<Value>, String> {
    let value = js_trim(&agent);
    let len = value.encode_utf16().count();
    let printable = len <= 256 && value.bytes().all(|b| (0x20..=0x7E).contains(&b));
    if !printable {
        return Ok(Some(Value::String("nodes.agentInvalid".to_string())));
    }
    if written && !is_quotable_bool(value) {
        Ok(Some(Value::String("config.unquotable".to_string())))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug)]
struct Range {
    from: usize,
    to: usize,
}

impl Range {
    fn to_value(self) -> Value {
        json!({"from": self.from, "to": self.to})
    }
}

#[derive(Clone, Debug)]
struct SubField {
    name: String,
    value: String,
    key: Range,
    at: Range,
}

#[derive(Clone, Copy, Debug)]
struct AgentAt {
    open: usize,
    close: usize,
    quoted: bool,
}

#[derive(Clone, Debug)]
struct SubEntry {
    tag: String,
    url: String,
    form: &'static str,
    naming: &'static str,
    ua: Option<String>,
    interval: Option<f64>,
    cache: Option<bool>,
    route: Option<String>,
    options: Vec<(String, String)>,
    from: usize,
    to: usize,
    line: usize,
    tag_at: Option<Range>,
    url_at: Range,
    block: Option<TextBlock>,
    container: Option<TextBlock>,
    fields: Vec<SubField>,
    agent_at: Option<AgentAt>,
    comment: Option<Range>,
}

struct Segment {
    parts: Vec<TextToken>,
    block: Option<TextBlock>,
    comment: Option<TextToken>,
}

fn raw_tok(units: &[u16], tok: Option<&TextToken>) -> String {
    tok.map(|t| js_slice(units, t.from, t.to))
        .unwrap_or_default()
}

fn value_of(units: &[u16], parts: &[&TextToken]) -> String {
    if parts.is_empty() {
        String::new()
    } else if parts.len() == 1 && parts[0].kind == TokenKind::Quoted {
        unquote_str(&raw_tok(units, Some(parts[0])))
    } else {
        js_slice(
            units,
            parts[0].from,
            parts.last().map_or(parts[0].to, |t| t.to),
        )
    }
}

fn segments(units: &[u16], container: &TextBlock, tokens: &[TextToken]) -> Vec<Segment> {
    let inside: Vec<&TextToken> = tokens
        .iter()
        .filter(|t| t.from > container.open && t.from < container.close)
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < inside.len() {
        let start = inside[i];
        if start.kind == TokenKind::Comment {
            i += 1;
            continue;
        }
        let mut parts = vec![start.clone()];
        let mut block: Option<TextBlock> = None;
        let mut j = i + 1;
        while j < inside.len() {
            let token = inside[j];
            if token.kind == TokenKind::Comment {
                break;
            }
            if raw_tok(units, Some(token)) == "{" && token.depth == container.depth + 1 {
                block = container
                    .children
                    .iter()
                    .find(|c| c.open == token.from)
                    .cloned();
                let limit = block.as_ref().map_or(container.close, |b| b.to);
                while j < inside.len() && inside[j].from < limit {
                    j += 1;
                }
                break;
            }
            if token.line != start.line {
                break;
            }
            parts.push(token.clone());
            j += 1;
        }
        let end = block
            .as_ref()
            .map(|b| b.to)
            .unwrap_or_else(|| parts.last().map_or(start.to, |t| t.to));
        let next = inside.get(j).copied();
        let comment = next
            .filter(|n| {
                n.kind == TokenKind::Comment && !js_slice(units, end, n.from).contains('\n')
            })
            .cloned();
        out.push(Segment {
            parts,
            block,
            comment: comment.clone(),
        });
        i = if comment.is_some() { j + 1 } else { j };
    }
    out
}

fn read_sub_fields(
    units: &[u16],
    block: &TextBlock,
    tokens: &[TextToken],
) -> Option<Vec<SubField>> {
    let mut fields = Vec::new();
    for seg in segments(units, block, tokens) {
        if seg.block.is_some() {
            return None;
        }
        if seg.parts.len() < 2
            || raw_tok(units, seg.parts.get(1)) != ":"
            || seg.parts[0].kind == TokenKind::Symbol
        {
            continue;
        }
        let value = &seg.parts[2..];
        let at = if let (Some(first), Some(last)) = (value.first(), value.last()) {
            Range {
                from: first.from,
                to: last.to,
            }
        } else {
            Range {
                from: seg.parts[1].to,
                to: seg.parts[1].to,
            }
        };
        fields.push(SubField {
            name: unquote_str(&raw_tok(units, Some(&seg.parts[0]))),
            value: js_slice(units, at.from, at.to),
            key: Range {
                from: seg.parts[0].from,
                to: at.to,
            },
            at,
        });
    }
    Some(fields)
}

fn split_tag(units: &[u16], parts: &[TextToken]) -> (String, Option<Range>, Vec<TextToken>) {
    let first = &parts[0];
    if first.kind == TokenKind::Quoted {
        if raw_tok(units, parts.get(1)) != ":" {
            return (String::new(), None, parts.to_vec());
        }
        return (
            unquote_str(&raw_tok(units, Some(first))),
            Some(Range {
                from: first.from,
                to: first.to,
            }),
            parts[2..].to_vec(),
        );
    }
    let colon = parts.iter().position(|t| raw_tok(units, Some(t)) == ":");
    let Some(colon) = colon else {
        return (String::new(), None, parts.to_vec());
    };
    let after_colon = parts[colon].to;
    let starts_slashes = units.get(after_colon) == Some(&(b'/' as u16))
        && units.get(after_colon + 1) == Some(&(b'/' as u16));
    if colon == 0 || starts_slashes {
        return (String::new(), None, parts.to_vec());
    }
    let tag_at = Range {
        from: first.from,
        to: parts[colon - 1].to,
    };
    let tag = js_trim(&js_slice(units, tag_at.from, tag_at.to)).to_string();
    (tag, Some(tag_at), parts[colon + 1..].to_vec())
}

enum ReadResult {
    None,
    Wrapper,
    Entry(Box<SubEntry>),
}

fn read_sub_entry(units: &[u16], segment: &Segment, tokens: &[TextToken]) -> ReadResult {
    let first = &segment.parts[0];
    let to = segment
        .block
        .as_ref()
        .map(|b| b.to)
        .unwrap_or_else(|| segment.parts.last().map_or(first.to, |t| t.to));
    let comment = segment.comment.as_ref().map(|c| Range {
        from: c.from,
        to: c.to,
    });
    let last_of = |fields: &[SubField], name: &str| -> Option<SubField> {
        fields.iter().rev().find(|f| f.name == name).cloned()
    };
    let read_f = |field: Option<&SubField>| -> Option<String> {
        let f = field?;
        let toks: Vec<&TextToken> = tokens
            .iter()
            .filter(|t| t.from >= f.at.from && t.to <= f.at.to)
            .collect();
        Some(value_of(units, &toks))
    };

    let (mut tag, tag_at, value) = split_tag(units, &segment.parts);
    if let Some(block) = &segment.block {
        if tag_at.is_none()
            || value.len() > 1
            || (value.len() == 1 && value[0].kind != TokenKind::Quoted)
        {
            return ReadResult::Wrapper;
        }
        let Some(fields) = read_sub_fields(units, block, tokens) else {
            return ReadResult::None;
        };
        let f_ua = last_of(&fields, "ua");
        let f_interval = last_of(&fields, "interval");
        let f_cache = last_of(&fields, "cache");
        let f_route = last_of(&fields, "route");
        let f_detour = last_of(&fields, "download_detour");
        let ua = read_f(f_ua.as_ref());
        let interval = read_f(f_interval.as_ref()).and_then(|s| parse_interval_num(&s));
        let cache = read_f(f_cache.as_ref()).and_then(|s| parse_bool(&s));
        let route = read_f(f_route.as_ref());
        let listed: Vec<(String, String)> = fields
            .iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect();
        if value.is_empty() {
            let Some(url_f) = last_of(&fields, "url") else {
                return ReadResult::None;
            };
            let url = read_f(Some(&url_f)).unwrap_or_default();
            return ReadResult::Entry(Box::new(SubEntry {
                tag,
                url,
                form: "block",
                naming: "tag",
                ua,
                interval,
                cache,
                route: route.or_else(|| read_f(f_detour.as_ref())),
                options: listed.into_iter().filter(|(n, _)| n != "url").collect(),
                from: first.from,
                to,
                line: first.line + 1,
                tag_at,
                url_at: url_f.at,
                block: Some(block.clone()),
                container: None,
                fields,
                agent_at: None,
                comment,
            }));
        }
        let url_at = Range {
            from: value[0].from,
            to: value[0].to,
        };
        return ReadResult::Entry(Box::new(SubEntry {
            tag,
            url: unquote_str(&raw_tok(units, Some(&value[0]))),
            form: "options",
            naming: "tag",
            ua,
            interval,
            cache,
            route,
            options: listed,
            from: first.from,
            to,
            line: first.line + 1,
            tag_at,
            url_at,
            block: Some(block.clone()),
            container: None,
            fields,
            agent_at: None,
            comment,
        }));
    }

    if value.is_empty() {
        return ReadResult::None;
    }
    let head = &value[0];
    let mut url: String;
    let mut form = "short";
    let mut agent_at: Option<AgentAt> = None;
    let mut ua: Option<String> = None;
    if head.kind != TokenKind::Quoted {
        form = "bare";
        url = js_slice(units, head.from, value.last().map_or(head.to, |t| t.to));
    } else {
        url = unquote_str(&raw_tok(units, Some(head)));
        if value.len() > 1 {
            let open = &value[1];
            let close = &value[value.len() - 1];
            if raw_tok(units, Some(open)) != "("
                || raw_tok(units, Some(close)) != ")"
                || close.parens != open.parens + 1
                || value[2..value.len() - 1]
                    .iter()
                    .any(|t| t.parens <= open.parens)
            {
                return ReadResult::None;
            }
            let inner = js_slice(units, open.to, close.from);
            let trimmed = js_trim(&inner);
            let quoted = value.len() == 4 && value[2].kind == TokenKind::Quoted;
            ua = Some(if quoted { unquote_str(trimmed) } else { inner });
            form = "agent";
            agent_at = Some(AgentAt {
                open: open.from,
                close: close.to,
                quoted,
            });
        }
    }
    let mut naming = "tag";
    if tag_at.is_none() {
        if let Some(colon) = url.find(':')
            && head.kind == TokenKind::Quoted
            && !url[colon..].starts_with("://")
            && url[colon + 1..].contains("://")
        {
            naming = "embedded";
            tag = js_trim(&url[..colon]).to_string();
            url = js_trim(&url[colon + 1..]).to_string();
        } else {
            naming = "host";
            tag = if url.contains("://") {
                url_host_str(Some(&url)).unwrap_or_default()
            } else {
                String::new()
            };
        }
    }
    if tag.is_empty() || url.is_empty() {
        return ReadResult::None;
    }
    let url_at = Range {
        from: head.from,
        to: if form == "bare" {
            value.last().map_or(head.to, |t| t.to)
        } else {
            head.to
        },
    };
    let options = match &ua {
        Some(u) => vec![("ua".to_string(), u.clone())],
        None => Vec::new(),
    };
    ReadResult::Entry(Box::new(SubEntry {
        tag,
        url,
        form,
        naming,
        ua,
        interval: None,
        cache: None,
        route: None,
        options,
        from: first.from,
        to,
        line: first.line + 1,
        tag_at,
        url_at,
        block: None,
        container: None,
        fields: Vec::new(),
        agent_at,
        comment,
    }))
}

fn read_entries(text: &str) -> Vec<SubEntry> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut entries = Vec::new();
    fn visit(
        units: &[u16],
        container: &TextBlock,
        tokens: &[TextToken],
        entries: &mut Vec<SubEntry>,
    ) {
        for seg in segments(units, container, tokens) {
            match read_sub_entry(units, &seg, tokens) {
                ReadResult::Wrapper => {
                    if let Some(b) = &seg.block {
                        visit(units, b, tokens, entries);
                    }
                }
                ReadResult::Entry(mut e) => {
                    e.container = Some(container.clone());
                    entries.push(*e);
                }
                ReadResult::None => {}
            }
        }
    }
    for section in scan.blocks.iter().filter(|b| b.name == "subscription") {
        visit(&units, section, &scan.tokens, &mut entries);
    }
    entries
}

pub fn read_subscription_entries(text: String) -> Result<Vec<Value>, String> {
    Ok(read_entries(&text)
        .into_iter()
        .map(|e| {
            let opts: Vec<Value> = e
                .options
                .into_iter()
                .map(|(n, v)| json!({"name": n, "value": v}))
                .collect();
            if e.form == "block" || e.form == "options" {
                json!({
                    "from": e.from,
                    "to": e.to,
                    "line": e.line,
                    "ua": e.ua,
                    "interval": e.interval.map(js::number),
                    "cache": e.cache,
                    "route": e.route,
                    "tag": e.tag,
                    "naming": e.naming,
                    "form": e.form,
                    "url": e.url,
                    "options": opts,
                    "tagAt": e.tag_at.map(Range::to_value),
                    "urlAt": e.url_at.to_value(),
                })
            } else {
                json!({
                    "from": e.from,
                    "to": e.to,
                    "line": e.line,
                    "tag": e.tag,
                    "url": e.url,
                    "form": e.form,
                    "naming": e.naming,
                    "ua": e.ua,
                    "interval": e.interval.map(js::number),
                    "cache": e.cache,
                    "route": e.route,
                    "options": opts,
                    "tagAt": e.tag_at.map(Range::to_value),
                    "urlAt": e.url_at.to_value(),
                })
            }
        })
        .collect())
}

fn bare_agent(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let bad = |c: char| is_js_ws(c) || matches!(c, '(' | ')' | '#' | '\'' | '"' | '{' | '}');
    let first = value.chars().next();
    let last = value.chars().next_back();
    if first.is_none_or(bad) || last.is_none_or(bad) {
        return false;
    }
    !value
        .chars()
        .any(|c| matches!(c, '(' | ')' | '#' | '\'' | '"' | '{' | '}'))
}

fn write_name(name: &str, written: Option<&str>) -> Result<String, String> {
    let was_quoted = written.is_some_and(|w| w.starts_with('\'') || w.starts_with('"'));
    if was_quoted || !is_bare_name_bool(name) {
        quote_str(name)
    } else {
        Ok(name.to_string())
    }
}

fn indent_at(units: &[u16], at: usize) -> String {
    let ls = line_start_u16(units, at);
    leading_space_tabs(&js_slice(units, ls, at)).to_string()
}

fn deeper(outer: &str, inner: &str) -> Option<String> {
    if inner.len() > outer.len() && inner.starts_with(outer) {
        Some(inner[outer.len()..].to_string())
    } else {
        None
    }
}

fn indent_step(text: &str, units: &[u16], entries: &[SubEntry], entry: &SubEntry) -> String {
    for other in std::iter::once(entry).chain(entries.iter()) {
        if let Some(b) = &other.block
            && b.line != b.end_line
            && let Some(field) = other.fields.first()
            && let Some(step) = deeper(
                &indent_at(units, other.from),
                &indent_at(units, field.key.from),
            )
        {
            return step;
        }
    }
    if let Some(c) = &entry.container
        && let Some(step) = deeper(&indent_at(units, c.open), &indent_at(units, entry.from))
    {
        return step;
    }
    first_indented_line(text).unwrap_or_else(|| "    ".to_string())
}

fn write_interval(seconds: f64) -> String {
    if seconds != 0.0 && seconds % 3600.0 == 0.0 {
        format!("{}h", js::number_to_string(seconds / 3600.0))
    } else if seconds != 0.0 && seconds % 60.0 == 0.0 {
        format!("{}m", js::number_to_string(seconds / 60.0))
    } else {
        format!("{}s", js::number_to_string(seconds))
    }
}

fn write_sub_inner(text: &str, tag: &str, change: &Value) -> Result<String, String> {
    let entries = read_entries(text);
    let Some(entry) = entries.iter().find(|e| e.tag == tag) else {
        return Err("nodes.editMissing".to_string());
    };
    let Some(co) = change.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'tag')".to_string());
    };
    let units = utf16::units(text);

    let next_tag = match obj_defined(co, "tag").filter(|v| !v.is_null()) {
        Some(v) => js_to_string(v),
        None => entry.tag.clone(),
    };
    let next_url = match obj_defined(co, "url").filter(|v| !v.is_null()) {
        Some(v) => js_to_string(v),
        None => entry.url.clone(),
    };
    let ch_ua = obj_defined(co, "ua");
    let next_ua = match ch_ua {
        None => entry.ua.clone(),
        Some(Value::Null) => None,
        Some(v) => Some(js_to_string(v)),
    };
    let ch_interval = obj_defined(co, "interval");
    let next_interval = match ch_interval {
        None => entry.interval,
        Some(Value::Null) => None,
        Some(v) => js::as_number(v),
    };
    let ch_cache = obj_defined(co, "cache");
    let next_cache = match ch_cache {
        None => entry.cache,
        Some(Value::Null) => None,
        Some(v) => v.as_bool(),
    };
    let ch_route = obj_defined(co, "route");
    let next_route = match ch_route {
        None => entry.route.clone(),
        Some(Value::Null) => None,
        Some(v) => Some(js_to_string(v)),
    };

    if next_tag != tag && entries.iter().any(|o| o.tag == next_tag) {
        return Err("nodes.tagTaken".to_string());
    }
    let changed_tag = next_tag != entry.tag;
    let changed_url = next_url != entry.url;
    let changed_ua = next_ua != entry.ua;
    let changed_interval = ch_interval.is_some() && next_interval != entry.interval;
    let changed_cache = ch_cache.is_some() && next_cache != entry.cache;
    let changed_route = ch_route.is_some() && next_route != entry.route;
    if !(changed_tag
        || changed_url
        || changed_ua
        || changed_interval
        || changed_cache
        || changed_route)
    {
        return Ok(text.to_string());
    }

    let written_tag = entry.tag_at.map(|r| js_slice(&units, r.from, r.to));
    let Some(block) = &entry.block else {
        let named = if entry.naming == "tag" && !changed_tag {
            written_tag.clone().unwrap_or_default()
        } else {
            write_name(&next_tag, None)?
        };
        let header = format!("{named}: {}", quote_str(&next_url)?);
        if (changed_interval && next_interval.is_some())
            || (changed_cache && next_cache.is_some())
            || (changed_route && next_route.is_some())
        {
            let indent = indent_at(&units, entry.from);
            let inner = format!("{indent}{}", indent_step(text, &units, &entries, entry));
            let cmt = entry
                .comment
                .map(|c| format!(" {}", js_slice(&units, c.from, c.to)))
                .unwrap_or_default();
            let mut lines = vec![
                format!("{named}: {{{cmt}"),
                format!("{inner}url: {}", quote_str(&next_url)?),
            ];
            if let Some(u) = &next_ua {
                lines.push(format!("{inner}ua: {}", quote_str(u)?));
            }
            if let Some(iv) = next_interval {
                lines.push(format!("{inner}interval: {}", write_interval(iv)));
            }
            if let Some(c) = next_cache {
                lines.push(format!("{inner}cache: {c}"));
            }
            if let Some(r) = &next_route {
                lines.push(format!("{inner}route: {}", write_name(r, None)?));
            }
            lines.push(format!("{indent}}}"));
            let to = entry.comment.map_or(entry.to, |c| c.to);
            return Ok(apply_utf16_edits(
                text,
                vec![(entry.from, to, lines.join("\n"))],
                false,
            ));
        }
        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        let agent = match &next_ua {
            None => String::new(),
            Some(u) => {
                let need_quote = entry.agent_at.is_some_and(|a| a.quoted) || !bare_agent(u);
                let body = if need_quote { quote_str(u)? } else { u.clone() };
                format!("({body})")
            }
        };
        if entry.naming != "tag" && (changed_tag || changed_url) {
            edits.push((entry.from, entry.url_at.to, header));
        } else {
            if changed_tag && let Some(ta) = entry.tag_at {
                edits.push((
                    ta.from,
                    ta.to,
                    write_name(&next_tag, written_tag.as_deref())?,
                ));
            }
            if changed_url || (entry.form == "bare" && changed_ua) {
                edits.push((entry.url_at.from, entry.url_at.to, quote_str(&next_url)?));
            }
        }
        if changed_ua {
            if let Some(a) = entry.agent_at {
                edits.push((a.open, a.close, agent));
            } else {
                edits.push((entry.to, entry.to, agent));
            }
        }
        return Ok(apply_utf16_edits(text, edits, false));
    };

    if block.line == block.end_line
        && !entry.fields.is_empty()
        && (changed_ua || changed_interval || changed_cache || changed_route)
    {
        let indent = indent_at(&units, entry.from);
        let inner = format!("{indent}{}", indent_step(text, &units, &entries, entry));
        let body = js_trim(&js_slice(&units, block.open + 1, block.close)).to_string();
        let expanded = apply_utf16_edits(
            text,
            vec![(
                block.open,
                block.to,
                format!("{{\n{inner}{body}\n{indent}}}"),
            )],
            false,
        );
        return write_sub_inner(&expanded, tag, change);
    }

    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    if changed_tag && let Some(ta) = entry.tag_at {
        edits.push((
            ta.from,
            ta.to,
            write_name(&next_tag, written_tag.as_deref())?,
        ));
    }
    if changed_url {
        edits.push((entry.url_at.from, entry.url_at.to, quote_str(&next_url)?));
    }
    let inner = match entry.fields.first() {
        Some(ff) => indent_at(&units, ff.key.from),
        None => format!(
            "{}{}",
            indent_at(&units, entry.from),
            indent_step(text, &units, &entries, entry)
        ),
    };
    let close_line = line_start_u16(&units, block.close);
    let close_own_line = is_space_or_tab(&js_slice(&units, close_line, block.close));
    let mut removed: Vec<SubField> = Vec::new();
    let mut kept = false;

    let mut set_field_val = |name: &str, val: Option<String>| -> Result<(), String> {
        let fields: Vec<SubField> = entry
            .fields
            .iter()
            .filter(|f| f.name == name)
            .cloned()
            .collect();
        let detours: Vec<SubField> = if name == "route" && entry.form == "block" {
            entry
                .fields
                .iter()
                .filter(|f| f.name == "download_detour")
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        for f in &detours {
            removed.push(f.clone());
            let from = line_start_u16(&units, f.key.from);
            let to = match index_of_u16(&units, b'\n' as u16, f.at.to) {
                Some(nl) => nl + 1,
                None => f.at.to,
            };
            edits.push((from, to, String::new()));
        }
        let last = fields.last();
        let Some(plain) = val else {
            for f in &fields {
                removed.push(f.clone());
                let from = line_start_u16(&units, f.key.from);
                let to = match index_of_u16(&units, b'\n' as u16, f.at.to) {
                    Some(nl) => nl + 1,
                    None => f.at.to,
                };
                edits.push((from, to, String::new()));
            }
            return Ok(());
        };
        kept = true;
        let quoted = last.is_some_and(|l| l.value.starts_with('\'') || l.value.starts_with('"'));
        let written = match name {
            "ua" => quote_str(&plain)?,
            "route" => write_name(&plain, None)?,
            _ if quoted => format!("'{plain}'"),
            _ => plain,
        };
        if let Some(l) = last {
            edits.push((l.at.from, l.at.to, written));
        } else if close_own_line {
            edits.push((
                close_line,
                close_line,
                format!("{inner}{name}: {written}\n"),
            ));
        } else {
            edits.push((
                block.close,
                block.close,
                format!(
                    "\n{inner}{name}: {written}\n{}",
                    indent_at(&units, entry.from)
                ),
            ));
        }
        Ok(())
    };

    if changed_ua {
        set_field_val("ua", next_ua.clone())?;
    }
    if changed_interval {
        set_field_val("interval", next_interval.map(write_interval))?;
    }
    if changed_cache {
        set_field_val("cache", next_cache.map(|b| b.to_string()))?;
    }
    if changed_route {
        set_field_val("route", next_route.clone())?;
    }

    if entry.form == "options" && !kept && removed.len() == entry.fields.len() {
        let mut rest = js_slice(&units, block.open + 1, block.close);
        for f in &removed {
            let from = line_start_u16(&units, f.key.from);
            let to = match index_of_u16(&units, b'\n' as u16, f.at.to) {
                Some(nl) => nl + 1,
                None => 0,
            };
            let needle = js_slice(&units, from, to);
            rest = rest.replacen(&needle, "", 1);
        }
        if js_trim(&rest).is_empty() {
            let mut shrunk: Vec<(usize, usize, String)> = edits
                .into_iter()
                .filter(|(from, _, _)| *from < block.open)
                .collect();
            shrunk.push((entry.url_at.to, block.to, String::new()));
            return Ok(apply_utf16_edits(text, shrunk, false));
        }
    }

    Ok(apply_utf16_edits(text, edits, false))
}

pub fn write_subscription_entry(
    text: String,
    tag: String,
    change: Value,
) -> Result<String, String> {
    write_sub_inner(&text, &tag, &change)
}
