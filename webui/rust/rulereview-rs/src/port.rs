//! The Rust port of `apps/lite/src/ruleReview.ts`.

#![forbid(unsafe_code)]

use crate::js;
use dae_core::port::{conditions, js_str, text};
use dae_core::utf16;
use regex::Regex;
use serde_json::{Value, json};
use std::sync::OnceLock;

const ROUTING_ORD: &[&str] = &[
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
const DNS_REQ_ORD: &[&str] = &[
    "qnameSuffix",
    "qnameFull",
    "qnameKeyword",
    "qnameGeosite",
    "qtype",
    "sip",
];
const DNS_RESP_ORD: &[&str] = &[
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

fn extra_spec(kind: &str) -> Option<(&'static str, &'static str)> {
    Some(match kind {
        "domainRegex" => ("domain", "regex"),
        "ipversion" => ("ipversion", ""),
        "dscp" => ("dscp", ""),
        "qnameRegex" => ("qname", "regex"),
        _ => return None,
    })
}

fn find_blocks<'a>(blocks: &'a [text::TextBlock], path: &[&str]) -> Vec<&'a text::TextBlock> {
    let mut cur: Vec<&text::TextBlock> = blocks.iter().collect();
    for (idx, &name) in path.iter().enumerate() {
        let found: Vec<&text::TextBlock> = cur.into_iter().filter(|b| b.name == name).collect();
        if idx + 1 == path.len() {
            return found;
        }
        cur = found.into_iter().flat_map(|b| b.children.iter()).collect();
    }
    Vec::new()
}

fn split_ws_and(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap_or('\0');
        if js_str::is_js_ws(ch) {
            let mut j = i;
            while j < s.len() && s[j..].chars().next().is_some_and(js_str::is_js_ws) {
                j += s[j..].chars().next().map_or(1, char::len_utf8);
            }
            if s[j..].starts_with("&&") {
                let after_amp = j + 2;
                let mut k = after_amp;
                while k < s.len() && s[k..].chars().next().is_some_and(js_str::is_js_ws) {
                    k += s[k..].chars().next().map_or(1, char::len_utf8);
                }
                if k > after_amp {
                    parts.push(&s[start..i]);
                    start = k;
                    i = k;
                    continue;
                }
            }
            i = j;
        } else {
            i += ch.len_utf8();
        }
    }
    parts.push(&s[start..]);
    parts
}

fn serialize_builder(rows: &[Value]) -> bool {
    !rows.is_empty()
        && rows.iter().all(|row| {
            let kind = row.get("kind").and_then(Value::as_str).unwrap_or("");
            let Some((name, prefix)) = extra_spec(kind) else {
                return conditions::serialize_conditions(Value::Array(vec![row.clone()]))
                    .ok()
                    .flatten()
                    .is_some();
            };
            let val = row.get("value").and_then(Value::as_str).unwrap_or("");
            let Some(vals) = conditions::parse_condition_values_vec(val) else {
                return false;
            };
            if vals.is_empty() || vals.iter().any(|v| !text::is_quotable_bool(v)) {
                return false;
            }
            let neg = if row.get("negate").and_then(Value::as_bool).unwrap_or(false) {
                "!"
            } else {
                ""
            };
            let args: Vec<String> = vals
                .iter()
                .filter_map(|v| {
                    let q = text::quote_str(v).ok()?;
                    Some(if prefix.is_empty() {
                        q
                    } else {
                        format!("{prefix}: {q}")
                    })
                })
                .collect();
            args.len() == vals.len()
                && text::is_fragment_bool(&format!("{neg}{name}({})", args.join(", ")))
        })
}

fn parse_builder(expr: &str, kind: &str) -> Result<Option<Vec<Value>>, String> {
    let (ord, extras): (&[&str], &[&str]) = match kind {
        "routing" => (ROUTING_ORD, &["domainRegex", "ipversion", "dscp"]),
        "request" => (DNS_REQ_ORD, &["qnameRegex"]),
        "response" => (DNS_RESP_ORD, &["qnameRegex"]),
        _ => return Err("TypeError: dnsConditionKinds[kind] is not iterable".into()),
    };
    let ord_vec: Vec<String> = ord.iter().map(|s| (*s).to_string()).collect();
    if let Some(parsed) = conditions::parse_conditions(expr.to_string(), ord_vec.clone())? {
        return Ok(Some(parsed));
    }
    let mut rows = Vec::new();
    for part in split_ws_and(expr) {
        if let Some(p) = conditions::parse_conditions(part.to_string(), ord_vec.clone())? {
            rows.extend(p);
            continue;
        }
        let trimmed = js_str::js_trim(part);
        let (neg, rest) = match trimmed.strip_prefix('!') {
            Some(r) => (true, js_str::js_trim_start(r)),
            None => (false, trimmed),
        };
        let fn_len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        let fn_name = &rest[..fn_len];
        let Some(inner) = rest[fn_len..]
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
        else {
            return Ok(None);
        };
        if fn_name.is_empty()
            || inner
                .chars()
                .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
        {
            return Ok(None);
        }
        let tokens = text::scan_config_raw(inner).tokens;
        let units = utf16::units(inner);
        let raw = |n: usize| utf16::slice(&units, tokens[n].from, tokens[n].to);
        let mut vals = Vec::new();
        let mut prefix = String::new();
        let mut i = 0;
        while i < tokens.len() {
            let mut p = String::new();
            if i + 1 < tokens.len() && raw(i + 1) == ":" {
                p = raw(i);
                i += 2;
            }
            if i >= tokens.len()
                || tokens[i].kind != text::TokenKind::Quoted
                || (!vals.is_empty() && p != prefix)
            {
                return Ok(None);
            }
            prefix = p;
            vals.push(text::unquote_str(&raw(i)));
            i += 1;
            if i < tokens.len() {
                if raw(i) != "," {
                    return Ok(None);
                }
                i += 1;
            }
        }
        let key = extras
            .iter()
            .copied()
            .find(|&k| extra_spec(k) == Some((fn_name, prefix.as_str())));
        let Some(k) = key.filter(|_| !vals.is_empty()) else {
            return Ok(None);
        };
        rows.push(
            json!({ "kind": k, "value": conditions::condition_values_str(&vals), "negate": neg }),
        );
    }
    Ok(serialize_builder(&rows).then_some(rows))
}

struct RuleRow {
    expression: String,
    target: String,
    fallback: bool,
    conditions: Option<Vec<Value>>,
}

fn parse_rule_rows(source: &str, kind: &str) -> Result<Vec<RuleRow>, String> {
    let scan = text::scan_config_raw(source);
    let path: Vec<&str> = if kind == "routing" {
        vec!["routing"]
    } else {
        vec!["dns", "routing", kind]
    };
    let blocks = find_blocks(&scan.blocks, &path);
    let [block] = blocks.as_slice() else {
        return Ok(Vec::new());
    };
    let tokens: Vec<&text::TextToken> = scan
        .tokens
        .iter()
        .filter(|t| {
            t.from > block.open && t.to <= block.close && t.kind != text::TokenKind::Comment
        })
        .collect();
    let units = utf16::units(source);
    let mut rows = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let first = tokens[i];
        i += 1;
        let mut last = first;
        while i < tokens.len() && (tokens[i].line == last.line || tokens[i].parens > 0) {
            last = tokens[i];
            i += 1;
        }
        let actual: Vec<&text::TextToken> = tokens
            .iter()
            .copied()
            .filter(|t| t.from >= first.from && t.to <= last.to)
            .collect();
        let tok = |n: usize| {
            actual
                .get(n)
                .map(|t| utf16::slice(&units, t.from, t.to))
                .unwrap_or_default()
        };
        let fallback = matches!(tok(0).as_str(), "fallback" | "default") && tok(1) == ":";
        let arrow_idx = if fallback {
            (actual.len() > 1).then_some(1)
        } else {
            actual
                .iter()
                .position(|t| t.parens == 0 && utf16::slice(&units, t.from, t.to) == "->")
        };
        let before = arrow_idx.and_then(|idx| idx.checked_sub(1).and_then(|b| actual.get(b)));
        let expression = match (arrow_idx, before) {
            (Some(_), Some(b)) => utf16::slice(&units, first.from, b.to),
            _ => utf16::slice(&units, first.from, last.to),
        };
        let target = arrow_idx
            .map(|idx| js_str::js_trim(&utf16::slice(&units, actual[idx].to, last.to)).to_string())
            .unwrap_or_default();
        let conds = if !fallback && arrow_idx.is_some() {
            parse_builder(&expression, kind)?
        } else {
            None
        };
        rows.push(RuleRow {
            expression,
            target,
            fallback,
            conditions: conds,
        });
    }
    Ok(rows)
}

// ---- Minimal inline phrase renderer for rules & targets ----------------------------------------

fn is_han(s: &str) -> bool {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\p{Han}").ok())
        .as_ref()
        .is_some_and(|r| r.is_match(s))
}

fn is_latin(s: &str) -> bool {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[\p{Latin}\p{Nd}]").ok())
        .as_ref()
        .is_some_and(|r| r.is_match(s))
}

fn needs_gap(a: Option<u16>, b: Option<u16>) -> bool {
    let (Some(ca), Some(cb)) = (
        a.and_then(|u| char::from_u32(u32::from(u))),
        b.and_then(|u| char::from_u32(u32::from(u))),
    ) else {
        return false;
    };
    let (mut ba, mut bb) = ([0u8; 4], [0u8; 4]);
    let (sa, sb) = (ca.encode_utf8(&mut ba), cb.encode_utf8(&mut bb));
    (is_han(sa) && is_latin(sb)) || (is_latin(sa) && is_han(sb))
}

fn check_t(t: &Value, key: &str) -> Result<Value, String> {
    if t.is_null() || js::is_undefined(t) {
        let kind = if t.is_null() { "null" } else { "undefined" };
        return Err(format!(
            "TypeError: Cannot read properties of {kind} (reading '{key}')"
        ));
    }
    Ok(t.as_object()
        .and_then(|m| m.get(key).cloned())
        .unwrap_or_else(js::undefined))
}

fn js_replace(hay: &str, needle: &str, rep: &str) -> String {
    let Some(pos) = hay.find(needle) else {
        return hay.to_string();
    };
    let (before, after) = (&hay[..pos], &hay[pos + needle.len()..]);
    let mut exp = String::with_capacity(rep.len());
    let b = rep.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'$' && i + 1 < b.len() {
            match b[i + 1] {
                b'$' => {
                    exp.push('$');
                    i += 2;
                    continue;
                }
                b'&' => {
                    exp.push_str(needle);
                    i += 2;
                    continue;
                }
                b'`' => {
                    exp.push_str(before);
                    i += 2;
                    continue;
                }
                b'\'' => {
                    exp.push_str(after);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        if let Some(ch) = rep[i..].chars().next() {
            exp.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    format!("{before}{exp}{after}")
}

fn render_tpl(t: &Value, key: &str, vals: &[(&str, String)]) -> Result<String, String> {
    let tv = check_t(t, key)?;
    let Some(tpl) = tv.as_str() else {
        return Err("TypeError: Cannot read properties of undefined (reading 'matchAll')".into());
    };
    let b = tpl.as_bytes();
    let (mut out, mut last, mut i) = (String::with_capacity(tpl.len()), 0, 0);
    while i < b.len() {
        if b[i] == b'{' {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == b'}' {
                let name = &tpl[i + 1..j];
                let filled = vals
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map_or(&tpl[i..=j], |(_, v)| v.as_str());
                out.push_str(&tpl[last..i]);
                if needs_gap(out.encode_utf16().last(), filled.encode_utf16().next()) {
                    out.push(' ');
                }
                out.push_str(filled);
                if needs_gap(
                    filled.encode_utf16().last(),
                    tpl[j + 1..].encode_utf16().next(),
                ) {
                    out.push(' ');
                }
                last = j + 1;
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&tpl[last..]);
    Ok(out)
}

fn render_target(target: &str, t: &Value) -> Result<String, String> {
    let trimmed = js_str::js_trim(target);
    let must = trimmed.strip_suffix("(must)").filter(|p| {
        !p.chars()
            .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    });
    let name = js_str::js_trim(must.unwrap_or(target));
    let base = match name {
        "direct" => render_tpl(t, "out_direct", &[])?,
        "block" | "reject" => render_tpl(t, "out_block", &[])?,
        "accept" => render_tpl(t, "out_accept", &[])?,
        "asis" => render_tpl(t, "out_asis", &[])?,
        _ => name.to_string(),
    };
    if must.is_some() {
        render_tpl(t, "mustTarget", &[("target", base)])
    } else {
        Ok(base)
    }
}

fn geo_key(kind: &str, val: &str) -> Option<&'static str> {
    match (kind, val) {
        ("geosite" | "qnameGeosite", "cn") => Some("geo_site_cn"),
        ("geosite" | "qnameGeosite", "gfw") => Some("geo_site_gfw"),
        ("geosite" | "qnameGeosite", "category-ads-all" | "category-ads") => Some("geo_site_ads"),
        ("geosite" | "qnameGeosite", "google-cn") => Some("geo_site_google_cn"),
        ("geosite" | "qnameGeosite", "category-games") => Some("geo_site_games"),
        ("geosite" | "qnameGeosite", "category-games@cn") => Some("geo_site_games_cn"),
        ("geosite" | "qnameGeosite", "apple") => Some("geo_site_apple"),
        ("geosite" | "qnameGeosite", "microsoft") => Some("geo_site_microsoft"),
        ("geosite" | "qnameGeosite", "telegram") => Some("geo_site_telegram"),
        ("geosite" | "qnameGeosite", "openai") => Some("geo_site_openai"),
        ("geosite" | "qnameGeosite", "youtube") => Some("geo_site_youtube"),
        ("geosite" | "qnameGeosite", "netflix") => Some("geo_site_netflix"),
        ("geosite" | "qnameGeosite", "disney") => Some("geo_site_disney"),
        ("geosite" | "qnameGeosite", "spotify") => Some("geo_site_spotify"),
        ("geosite" | "qnameGeosite", "bilibili") => Some("geo_site_bilibili"),
        ("geosite" | "qnameGeosite", "netease") => Some("geo_site_netease"),
        ("geosite" | "qnameGeosite", "bahamut") => Some("geo_site_bahamut"),
        ("geoip" | "answerGeoip", "private") => Some("geo_ip_private"),
        ("geoip" | "answerGeoip", "cn") => Some("geo_ip_cn"),
        _ => None,
    }
}

fn render_cond(c: &Value, t: &Value) -> Result<String, String> {
    let kind = c.get("kind").and_then(Value::as_str).unwrap_or("");
    let val = c.get("value").and_then(Value::as_str).unwrap_or("");
    let values = conditions::parse_condition_values_vec(val)
        .unwrap_or_else(|| vec![text::unquote_str(js_str::js_trim(val))]);
    let known =
        ROUTING_ORD.contains(&kind) || DNS_RESP_ORD.contains(&kind) || extra_spec(kind).is_some();
    let kind_s = if known {
        render_tpl(t, &format!("kind_{kind}"), &[])?
    } else {
        render_tpl(t, "customRule", &[])?
    };
    let mut parts = Vec::with_capacity(values.len());
    for v in values {
        parts.push(if let Some(gk) = geo_key(kind, &v) {
            render_tpl(t, gk, &[])?
        } else if kind == "l4proto" {
            v.to_uppercase()
        } else {
            v
        });
    }
    let sep_v = check_t(t, "listSep")?;
    let sep = if js::is_undefined(&sep_v) {
        ",".to_string()
    } else {
        js_str::js_to_string(&sep_v)
    };
    let base = render_tpl(
        t,
        "condValue",
        &[("kind", kind_s), ("values", parts.join(&sep))],
    )?;
    if c.get("negate").and_then(Value::as_bool).unwrap_or(false) {
        render_tpl(t, "condNegated", &[("cond", base)])
    } else {
        Ok(base)
    }
}

fn render_rule(conds: Option<&[Value]>, target: &str, t: &Value) -> Result<String, String> {
    let mut parts = Vec::new();
    match conds {
        Some(list) if !list.is_empty() => {
            for c in list {
                parts.push(render_cond(c, t)?);
            }
        }
        _ => parts.push(render_tpl(t, "customRule", &[])?),
    }
    let sep_v = check_t(t, "condAnd")?;
    let sep = if js::is_undefined(&sep_v) {
        ",".to_string()
    } else {
        js_str::js_to_string(&sep_v)
    };
    let tgt = render_target(target, t)?;
    render_tpl(
        t,
        "ruleArrow",
        &[("conditions", parts.join(&sep)), ("target", tgt)],
    )
}

fn req_str(t: &Value, key: &str) -> Result<String, String> {
    let v = check_t(t, key)?;
    v.as_str()
        .map(str::to_string)
        .ok_or_else(|| "TypeError: Cannot read properties of undefined (reading 'replace')".into())
}

// ---- Exported ports ----------------------------------------------------------------------------

/// Port of `ruleReview` (`apps/lite/src/ruleReview.ts:7`).
pub fn rule_review(
    before: String,
    after: String,
    kind: String,
    t: Value,
) -> Result<Vec<Value>, String> {
    let previous = parse_rule_rows(&before, &kind)?;
    let next = parse_rule_rows(&after, &kind)?;
    let mut removed: Vec<&RuleRow> = previous.iter().filter(|r| !r.fallback).collect();
    let mut added: Vec<&RuleRow> = next.iter().filter(|r| !r.fallback).collect();
    for i in (0..removed.len()).rev() {
        if let Some(m) = added
            .iter()
            .position(|r| r.expression == removed[i].expression && r.target == removed[i].target)
        {
            removed.remove(i);
            added.remove(m);
        }
    }
    let mut summary: Vec<Value> = Vec::new();
    for row in removed {
        let tpl = req_str(&t, "reviewRuleRemoved")?;
        let desc = render_rule(row.conditions.as_deref(), &row.target, &t)?;
        summary.push(Value::String(js_replace(&tpl, "{rule}", &desc)));
    }
    for row in added {
        let tpl = req_str(&t, "reviewRuleAdded")?;
        let desc = render_rule(row.conditions.as_deref(), &row.target, &t)?;
        summary.push(Value::String(js_replace(&tpl, "{rule}", &desc)));
    }
    let old_fb = previous
        .iter()
        .find(|r| r.fallback)
        .map(|r| r.target.as_str());
    let new_fb = next.iter().find(|r| r.fallback).map(|r| r.target.as_str());
    if old_fb != new_fb {
        let tpl = req_str(&t, "reviewRuleFallback")?;
        let label_key = if kind == "routing" {
            "fallback"
        } else if kind == "request" {
            "dnsFallback"
        } else {
            "dnsResponseFallback"
        };
        let label = js_str::js_to_string(&check_t(&t, label_key)?);
        let target_s = match new_fb {
            Some(fb) if !fb.is_empty() => render_target(fb, &t)?,
            _ => js_str::js_to_string(&check_t(&t, "defaultValue")?),
        };
        let line = js_replace(&js_replace(&tpl, "{label}", &label), "{target}", &target_s);
        summary.push(Value::String(line));
    }
    if summary.is_empty() && before != after {
        summary.push(check_t(&t, "reviewRuleOrder")?);
    }
    Ok(summary)
}

/// Port of `dnsReview` (`apps/lite/src/ruleReview.ts:35`).
pub fn dns_review(before: String, after: String, t: Value) -> Result<Vec<Value>, String> {
    let servers = |src: &str| -> Vec<text::TextField> {
        let scan = text::scan_config_raw(src);
        find_blocks(&scan.blocks, &["dns", "upstream"])
            .into_iter()
            .flat_map(|b| text::block_fields_raw(src, b, &scan.tokens))
            .collect()
    };
    let previous = servers(&before);
    let next = servers(&after);
    let mut summary: Vec<Value> = Vec::new();
    for field in &previous {
        if !next.iter().any(|item| item.name == field.name) {
            let tpl = req_str(&t, "reviewServerRemoved")?;
            summary.push(Value::String(js_replace(&tpl, "{name}", &field.name)));
        }
    }
    for field in &next {
        let old = previous.iter().find(|item| item.name == field.name);
        if old.is_none_or(|o| o.value != field.value) {
            let key = if old.is_some() {
                "reviewServerChanged"
            } else {
                "reviewServerAdded"
            };
            let tpl = req_str(&t, key)?;
            let url = text::unquote_str(&field.value);
            let line = js_replace(&js_replace(&tpl, "{name}", &field.name), "{url}", &url);
            summary.push(Value::String(line));
        }
    }
    let section = |src: &str, kind: &str| -> String {
        let scan = text::scan_config_raw(src);
        let blocks = find_blocks(&scan.blocks, &["dns", "routing", kind]);
        match blocks.first() {
            Some(b) => utf16::slice(&utf16::units(src), b.from, b.to),
            None => String::new(),
        }
    };
    for kind in ["request", "response"] {
        if section(&before, kind) != section(&after, kind) {
            summary.extend(rule_review(
                before.clone(),
                after.clone(),
                kind.to_string(),
                t.clone(),
            )?);
        }
    }
    if summary.is_empty() {
        Ok(vec![check_t(&t, "reviewRuleOrder")?])
    } else {
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages() -> Value {
        json!({
            "listSep": ", ",
            "condAnd": " and ",
            "condValue": "{kind}: {values}",
            "condNegated": "{cond} (match when not met)",
            "ruleArrow": "{conditions} → {target}",
            "mustTarget": "{target} (forced)",
            "customRule": "Custom rule",
            "kind_domainSuffix": "Domain and subdomains",
            "kind_domainRegex": "Domain pattern",
            "out_direct": "Direct",
            "reviewRuleAdded": "Add: {rule}",
            "reviewRuleRemoved": "Remove: {rule}",
            "reviewRuleOrder": "Change rule order or rule notes.",
            "reviewRuleFallback": "{label}: {target}",
            "fallback": "Other traffic goes to",
            "dnsFallback": "Other domains use",
            "dnsResponseFallback": "Other results",
            "defaultValue": "Backend default",
            "reviewServerAdded": "Add DNS server {name}: {url}",
            "reviewServerRemoved": "Remove DNS server {name}.",
            "reviewServerChanged": "Change DNS server {name} to {url}."
        })
    }

    #[test]
    fn routing_rule_added_removed_and_fallback() {
        let before = "routing {\n  domain(suffix: old.example) -> proxy\n  fallback: direct\n}\n";
        let after = "routing {\n  domain(suffix: new.example) -> direct\n  fallback: proxy\n}\n";
        let out = rule_review(before.into(), after.into(), "routing".into(), messages()).unwrap();
        assert_eq!(
            out,
            vec![
                json!("Remove: Domain and subdomains: old.example → proxy"),
                json!("Add: Domain and subdomains: new.example → Direct"),
                json!("Other traffic goes to: proxy"),
            ]
        );
    }

    #[test]
    fn order_only_and_missing_t_keys() {
        let out = rule_review("a".into(), "b".into(), "routing".into(), json!({})).unwrap();
        assert_eq!(out, vec![js::undefined()]);
        assert!(rule_review("a".into(), "b".into(), "routing".into(), Value::Null).is_err());
        assert_eq!(
            rule_review("same".into(), "same".into(), "routing".into(), Value::Null).unwrap(),
            Vec::<Value>::new()
        );
    }

    #[test]
    fn dns_server_changes_and_order_fallback() {
        let before =
            "dns {\n  upstream {\n    test: 'tls://1.1.1.1'\n    old: 'udp://8.8.8.8'\n  }\n}\n";
        let after = "dns {\n  upstream {\n    test: 'tls://9.9.9.9'\n    added: 'https://dns.google/dns-query'\n  }\n}\n";
        let out = dns_review(before.into(), after.into(), messages()).unwrap();
        assert_eq!(
            out,
            vec![
                json!("Remove DNS server old."),
                json!("Change DNS server test to tls://9.9.9.9."),
                json!("Add DNS server added: https://dns.google/dns-query"),
            ]
        );
        assert_eq!(
            dns_review("x".into(), "x".into(), messages()).unwrap(),
            vec![json!("Change rule order or rule notes.")]
        );
        assert!(dns_review("x".into(), "x".into(), Value::Null).is_err());
    }
}
