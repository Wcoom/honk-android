//! Outbound mode (`rule` / `direct` / `global`) & rule-fallback delegating directly to
//! `doona_logic::dae` (the `rust-anything`-verified `dae-rs` port of `src/dae/outboundMode.ts`),
//! plus a pure-Rust `.dae` syntax highlighter for Doona's Config editor.

use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutboundMode {
    Rule,
    Direct,
    Global(String),
}

impl OutboundMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboundMode::Rule => "rule",
            OutboundMode::Direct => "direct",
            OutboundMode::Global(_) => "global",
        }
    }

    pub fn global_target(&self) -> Option<&str> {
        match self {
            OutboundMode::Global(t) => Some(t.as_str()),
            _ => None,
        }
    }
}

fn strip_dae_quotes(raw: &str) -> String {
    let t = raw.trim();
    if t.len() >= 2
        && ((t.starts_with('\'') && t.ends_with('\'')) || (t.starts_with('"') && t.ends_with('"')))
    {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

fn format_dae_outbound_ident(target: &str) -> String {
    let clean = strip_dae_quotes(target);
    let needs_quote = clean.chars().any(|c| {
        c.is_whitespace() || matches!(c, '/' | ':' | '(' | ')' | '{' | '}' | ',' | '#' | '\'' | '"')
    });
    if needs_quote {
        format!("'{}'", clean.replace('\'', "\\'"))
    } else {
        clean
    }
}

pub fn read_mode(content: &str) -> OutboundMode {
    match doona_logic::dae::read_mode(content.to_string()) {
        Ok(val) => match val.get("mode").and_then(Value::as_str) {
            Some("direct") => OutboundMode::Direct,
            Some("global") => {
                let target = val
                    .get("target")
                    .and_then(Value::as_str)
                    .map(strip_dae_quotes)
                    .unwrap_or_default();
                OutboundMode::Global(target)
            }
            _ => OutboundMode::Rule,
        },
        Err(_) => OutboundMode::Rule,
    }
}

pub fn read_rule_fallback(content: &str) -> String {
    doona_logic::dae::read_rule_fallback(content.to_string())
        .map(|s| strip_dae_quotes(&s))
        .unwrap_or_default()
}

pub fn write_mode(content: &str, mode: &OutboundMode) -> Result<String, String> {
    let next = match mode {
        OutboundMode::Rule => json!({ "mode": "rule" }),
        OutboundMode::Direct => json!({ "mode": "direct" }),
        OutboundMode::Global(target) => {
            json!({ "mode": "global", "target": format_dae_outbound_ident(target) })
        }
    };
    doona_logic::dae::write_mode(content.to_string(), next).map_err(|err| {
        if err.contains("act.modeInterleaved") {
            "act.modeInterleaved".to_string()
        } else if err.contains("act.modeNoRouting") {
            "act.modeNoRouting".to_string()
        } else {
            err
        }
    })
}

pub fn write_rule_fallback(content: &str, target: &str) -> Result<String, String> {
    doona_logic::dae::write_rule_fallback(content.to_string(), format_dae_outbound_ident(target))
}

/// Pure-Rust `.dae` syntax tokenizer returning `(css_class, text)` spans matching Doona's `.rp-dae-*` tokens.
pub fn highlight_dae(source: &str) -> Vec<(&'static str, String)> {
    const KEYWORDS: &[&str] = &[
        "global",
        "subscription",
        "node",
        "dns",
        "group",
        "routing",
        "upstream",
        "request",
        "response",
        "fixed_domain_ttl",
        "fallback",
        "filter",
        "policy",
        "must_direct",
        "must_rules",
        "direct",
        "block",
        "proxy",
        "true",
        "false",
        "include",
    ];
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'#' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            out.push(("rp-dae-comment", source[start..i].to_string()));
        } else if b == b'\'' || b == b'"' {
            let quote = b;
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push(("rp-dae-string", source[start..i].to_string()));
        } else if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            out.push(("rp-dae-operator", "->".to_string()));
            i += 2;
        } else if (b == b'&' && i + 1 < bytes.len() && bytes[i + 1] == b'&')
            || (b == b'|' && i + 1 < bytes.len() && bytes[i + 1] == b'|')
        {
            out.push(("rp-dae-operator", source[i..i + 2].to_string()));
            i += 2;
        } else if matches!(b, b'{' | b'}' | b'(' | b')' | b':' | b',' | b'!') {
            out.push(("rp-dae-punctuation", source[i..i + 1].to_string()));
            i += 1;
        } else if b.is_ascii_digit() {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'.' | b'/'))
            {
                i += 1;
            }
            out.push(("rp-dae-number", source[start..i].to_string()));
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'-'))
            {
                i += 1;
            }
            let word = &source[start..i];
            let cls = if KEYWORDS.contains(&word) {
                "rp-dae-keyword"
            } else if i < bytes.len() && bytes[i] == b':' {
                "rp-dae-propertyName"
            } else {
                "rp-dae-variableName"
            };
            out.push((cls, word.to_string()));
        } else {
            let ch_len = source[i..].chars().next().map_or(1, char::len_utf8);
            out.push(("", source[i..i + ch_len].to_string()));
            i += ch_len;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_outbound_mode_and_fallback_via_dae_core() {
        let cfg = "routing {\n    pname(NetworkManager) -> must_direct\n    dip(geoip:cn) -> direct\n    fallback: 总模式\n}\n";
        assert_eq!(read_mode(cfg), OutboundMode::Rule);
        assert_eq!(read_rule_fallback(cfg), "总模式");

        let global = write_mode(cfg, &OutboundMode::Global("自建CDN优选".into())).unwrap();
        assert_eq!(
            read_mode(&global),
            OutboundMode::Global("自建CDN优选".into())
        );

        let direct = write_mode(&global, &OutboundMode::Direct).unwrap();
        assert_eq!(read_mode(&direct), OutboundMode::Direct);

        let back = write_mode(&direct, &OutboundMode::Rule).unwrap();
        assert_eq!(read_mode(&back), OutboundMode::Rule);

        let fb = write_rule_fallback(&back, "智能策略").unwrap();
        assert_eq!(read_rule_fallback(&fb), "智能策略");
    }
}
