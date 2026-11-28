//! Port of `src/dae/templates.ts`, `src/dae/setup.ts`, `src/dae/sources.ts`,
//! and `src/dae/startingRouting.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::groups::read_group_entries_raw;
use crate::port::js_str::{is_truthy, js_slice, js_trim, obj_defined};
use crate::port::rule_text::dns_upstreams_raw;
use crate::port::text::{TokenKind, block_body_raw, scan_config_raw, uncomment_str, unquote_str};
use crate::port::vocab::is_builtin_outbound_str;
use crate::utf16;
use indexmap::IndexMap;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::OnceLock;

// --- startingRouting.ts ---

const DEMO_ROUTING: &str = "routing {\n  pname(NetworkManager, systemd-resolved) && l4proto(udp) && dport(53) -> direct(must)\n  dip(geoip: private) -> direct(must)\n  domain(suffix: doubleclick.net) -> block\n  domain(geosite: cn) -> direct\n  domain(geosite: telegram) -> proxy\n  include rules.dae\n  fallback: auto\n}";

const DEMO_ROUTING_INCLUDE: &str = "# Household exceptions, kept apart from config.dae.\n# The TV never leaves through a node.\nmac(aa:bb:cc:dd:ee:ff) && ipversion(4) -> direct\n\n# AI\ndomain(geosite: openai) -> proxy\nsip(10.0.0.0/24) && dport(25) -> block\n";

// --- templates.ts ---

const NETWORK_MANAGER: &[&str] = &["# NetworkManager", "pname(NetworkManager) -> direct"];
const LOCAL: &[&str] = &[
    "# dae presets: the LAN and multicast stay off the proxy",
    "dip(224.0.0.0/3, 'ff00::/8') -> direct",
    "dip(geoip:private) -> direct",
];
const QUIC: &[&str] = &["# Block UDP/443", "l4proto(udp) && dport(443) -> block"];
const ADS: &[&str] = &["# Ads", "domain(geosite:category-ads-all) -> block"];
const CHINA_VENDORS: &[&str] = &[
    "# Mainland China games",
    "domain(geosite:category-games@cn) -> direct",
];
const FORMER_CHINA_VENDORS: &str = "domain(geosite:google-cn, geosite:category-games@cn) -> direct";
const CHINA: &[&str] = &[
    "# Mainland China",
    "domain(geosite:cn) -> direct",
    "dip(geoip:cn) -> direct",
];
const MEDIA_CODES: &[&str] = &[
    "youtube",
    "netflix",
    "disney",
    "hbo",
    "primevideo",
    "twitch",
    "spotify",
];

const ALL_TEMPLATES: &[&str] = &[
    "global",
    "bypass",
    "gfw",
    "homebound",
    "single",
    "services",
    "regions",
];

fn telegram_rules(target: &str) -> Vec<String> {
    vec![
        "# Telegram".to_string(),
        format!("domain(geosite:telegram) -> {target}"),
        format!("dip(geoip:telegram) -> {target}"),
    ]
}

fn media_rules(target: &str) -> Vec<String> {
    let args = MEDIA_CODES
        .iter()
        .map(|c| format!("geosite:{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    vec![
        "# Overseas media".to_string(),
        format!("domain({args}) -> {target}"),
    ]
}

fn gfw_rules(target: &str) -> Vec<String> {
    vec![
        "# GFW list".to_string(),
        format!("domain(geosite:gfw) -> {target}"),
    ]
}

fn template_tail_and_fallback(template: &str) -> Option<(Vec<String>, &'static str)> {
    let mut rules: Vec<String> = Vec::new();
    let fallback = match template {
        "global" => "{group}",
        "bypass" => {
            rules.extend(CHINA_VENDORS.iter().map(|s| (*s).to_string()));
            rules.extend(CHINA.iter().map(|s| (*s).to_string()));
            "{group}"
        }
        "gfw" => {
            rules.extend(telegram_rules("{group}"));
            rules.extend(gfw_rules("{group}"));
            "direct"
        }
        "homebound" => {
            rules.push("# Mainland China".to_string());
            rules.push("domain(geosite:cn) -> cn".to_string());
            rules.push("dip(geoip:cn) -> cn".to_string());
            "direct"
        }
        "single" => {
            rules.extend(CHINA_VENDORS.iter().map(|s| (*s).to_string()));
            rules.extend(telegram_rules("proxy"));
            rules.extend(media_rules("proxy"));
            rules.extend(gfw_rules("proxy"));
            rules.extend(CHINA.iter().map(|s| (*s).to_string()));
            "proxy"
        }
        "services" => {
            rules.extend(CHINA_VENDORS.iter().map(|s| (*s).to_string()));
            rules.push("# Microsoft".to_string());
            rules.push("domain(geosite:microsoft) -> direct".to_string());
            rules.push("# Apple".to_string());
            rules.push("domain(geosite:apple) -> apple".to_string());
            rules.extend(telegram_rules("telegram"));
            rules.extend(media_rules("media"));
            rules.extend(gfw_rules("proxy"));
            rules.extend(CHINA.iter().map(|s| (*s).to_string()));
            "proxy"
        }
        "regions" => {
            rules.extend(CHINA_VENDORS.iter().map(|s| (*s).to_string()));
            rules.push("# Microsoft".to_string());
            rules.push("domain(geosite:microsoft) -> direct".to_string());
            rules.push("# Apple".to_string());
            rules.push("domain(geosite:apple) -> direct".to_string());
            rules.extend(telegram_rules("telegram"));
            rules.push("# OpenAI".to_string());
            rules.push("domain(geosite:openai) -> ai".to_string());
            rules.push("# NetEase Music".to_string());
            rules.push("domain(geosite:netease) -> direct".to_string());
            rules.push("# Games".to_string());
            rules.push("domain(geosite:category-games) -> direct".to_string());
            rules.push("# YouTube".to_string());
            rules.push("domain(geosite:youtube) -> youtube".to_string());
            rules.push("# Netflix".to_string());
            rules.push("domain(geosite:netflix) -> netflix".to_string());
            rules.push("# Bahamut".to_string());
            rules.push("domain(geosite:bahamut) -> bahamut".to_string());
            rules.push("# Bilibili".to_string());
            rules.push("domain(geosite:bilibili) -> direct".to_string());
            rules.extend(media_rules("media"));
            rules.extend(gfw_rules("proxy"));
            rules.extend(CHINA.iter().map(|s| (*s).to_string()));
            "proxy"
        }
        _ => return None,
    };
    Some((rules, fallback))
}

fn template_group_names(template: &str) -> Option<&'static [&'static str]> {
    match template {
        "global" | "bypass" | "gfw" => Some(&[]),
        "homebound" => Some(&["cn"]),
        "single" => Some(&["proxy", "auto"]),
        "services" => Some(&["proxy", "auto", "telegram", "media", "apple"]),
        "regions" => Some(&[
            "proxy", "auto", "hk", "jp", "us", "tw", "sg", "kr", "telegram", "ai", "youtube",
            "netflix", "bahamut", "media",
        ]),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct TemplateOpts {
    block_ads: bool,
    block_quic: bool,
    network_manager_direct: bool,
}

fn template_rules_raw(template: &str, opts: TemplateOpts) -> Result<Vec<String>, String> {
    let Some((tail, _)) = template_tail_and_fallback(template) else {
        return Err("TypeError: Cannot read properties of undefined (reading 'rules')".to_string());
    };
    let mut out = Vec::new();
    if opts.network_manager_direct {
        out.extend(NETWORK_MANAGER.iter().map(|s| (*s).to_string()));
    }
    out.extend(LOCAL.iter().map(|s| (*s).to_string()));
    if opts.block_quic {
        out.extend(QUIC.iter().map(|s| (*s).to_string()));
    }
    if opts.block_ads {
        out.extend(ADS.iter().map(|s| (*s).to_string()));
    }
    out.extend(tail);
    Ok(out)
}

pub fn template_rules(template: String, options: Option<Value>) -> Result<Vec<String>, String> {
    let mut opts = TemplateOpts {
        block_ads: false,
        block_quic: true,
        network_manager_direct: true,
    };
    if let Some(Value::Object(o)) = options.as_ref().filter(|v| !js::is_undefined(v)) {
        if let Some(v) = obj_defined(o, "blockAds") {
            opts.block_ads = is_truthy(v);
        }
        if let Some(v) = obj_defined(o, "blockQuic") {
            opts.block_quic = is_truthy(v);
        }
        if let Some(v) = obj_defined(o, "networkManagerDirect") {
            opts.network_manager_direct = is_truthy(v);
        }
    }
    template_rules_raw(&template, opts)
}

fn split_logic_ops(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = raw.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"&&") || bytes[i..].starts_with(b"||") {
            if i > start {
                out.push(raw[start..i].to_string());
            }
            out.push(raw[i..i + 2].to_string());
            i += 2;
            start = i;
        } else if bytes[i] == b'!' {
            if i > start {
                out.push(raw[start..i].to_string());
            }
            out.push("!".to_string());
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }
    if start < bytes.len() {
        out.push(raw[start..].to_string());
    }
    out
}

fn routing_entries(text: &str) -> Option<Vec<Vec<String>>> {
    let scan = scan_config_raw(text);
    let units = utf16::units(text);
    let mut entries: Vec<Vec<String>> = Vec::new();
    for block in scan.blocks.iter().filter(|b| b.name == "routing") {
        if !block.children.is_empty() {
            return None;
        }
        let mut line: isize = -1;
        let mut parens: usize = 0;
        for token in &scan.tokens {
            if token.from <= block.open
                || token.from >= block.close
                || token.kind == TokenKind::Comment
            {
                continue;
            }
            let raw = js_slice(&units, token.from, token.to);
            if token.line as isize != line && parens == 0 {
                entries.push(Vec::new());
            }
            line = token.line as isize;
            if let Some(cur) = entries.last_mut() {
                if token.kind == TokenKind::Quoted {
                    cur.push(if parens > 0 {
                        unquote_str(&raw)
                    } else {
                        raw.clone()
                    });
                } else {
                    cur.extend(split_logic_ops(&raw));
                }
            }
            if token.kind == TokenKind::Symbol && raw == "(" {
                parens += 1;
            } else if token.kind == TokenKind::Symbol && raw == ")" {
                parens = parens.saturating_sub(1);
            }
        }
    }
    Some(entries)
}

const GROUP_SLOT: &str = "\0group";

fn template_entries(template: &str, opts: TemplateOpts) -> Vec<Vec<String>> {
    let Ok(rules) = template_rules_raw(template, opts) else {
        return Vec::new();
    };
    let fallback = template_tail_and_fallback(template)
        .map(|(_, fb)| fb)
        .unwrap_or("");
    let mut lines = Vec::with_capacity(rules.len() + 3);
    lines.push("routing {".to_string());
    lines.extend(rules);
    lines.push(format!("fallback: {fallback}"));
    lines.push("}".to_string());
    let src = lines.join("\n").replace("{group}", GROUP_SLOT);
    routing_entries(&src).unwrap_or_default()
}

pub fn detect_template(text: String) -> Result<Option<Value>, String> {
    let former_src = format!("routing {{\n{FORMER_CHINA_VENDORS}\n}}");
    let former = routing_entries(&former_src)
        .and_then(|e| e.into_iter().next())
        .map(|e| e.join(""))
        .unwrap_or_default();
    let current_src = format!("routing {{\n{}\n}}", CHINA_VENDORS.join("\n"));
    let current = routing_entries(&current_src)
        .and_then(|e| e.into_iter().next())
        .unwrap_or_default();
    let Some(raw_entries) = routing_entries(&text) else {
        return Ok(None);
    };
    let entries: Vec<Vec<String>> = raw_entries
        .into_iter()
        .map(|e| {
            if e.join("") == former {
                current.clone()
            } else {
                e
            }
        })
        .collect();
    if entries.is_empty() {
        return Ok(None);
    }
    for &template in ALL_TEMPLATES {
        for block_ads in [false, true] {
            for block_quic in [true, false] {
                for network_manager_direct in [true, false] {
                    let opts = TemplateOpts {
                        block_ads,
                        block_quic,
                        network_manager_direct,
                    };
                    let expected = template_entries(template, opts);
                    if expected.len() != entries.len() {
                        continue;
                    }
                    let mut group: Option<&str> = None;
                    let mut same = true;
                    for (want, got) in expected.iter().zip(entries.iter()) {
                        if want.len() != got.len() {
                            same = false;
                            break;
                        }
                        for (wt, gt) in want.iter().zip(got.iter()) {
                            if wt != GROUP_SLOT {
                                if wt != gt {
                                    same = false;
                                    break;
                                }
                            } else {
                                if is_builtin_outbound_str(Some(gt.as_str()))
                                    || group.is_some_and(|g| g != gt.as_str())
                                {
                                    same = false;
                                    break;
                                }
                                group = Some(gt.as_str());
                            }
                        }
                        if !same {
                            break;
                        }
                    }
                    if same {
                        return Ok(Some(json!({
                            "template": template,
                            "group": group,
                            "blockAds": block_ads,
                            "blockQuic": block_quic,
                            "networkManagerDirect": network_manager_direct,
                        })));
                    }
                }
            }
        }
    }
    Ok(None)
}

pub fn routing_options(text: String) -> Result<Option<Value>, String> {
    let Some(entries) = routing_entries(&text) else {
        return Ok(None);
    };
    let written: Vec<String> = entries
        .into_iter()
        .filter(|e| {
            !matches!(
                e.first().map(String::as_str),
                Some("fallback" | "default" | "include")
            )
        })
        .map(|e| e.join(""))
        .collect();
    if written.is_empty() {
        return Ok(None);
    }
    let holds = |lines: &[&str]| -> bool {
        let src = format!("routing {{\n{}\n}}", lines.join("\n"));
        routing_entries(&src)
            .unwrap_or_default()
            .into_iter()
            .all(|want| written.contains(&want.join("")))
    };
    Ok(Some(json!({
        "blockAds": holds(ADS),
        "blockQuic": holds(QUIC),
        "networkManagerDirect": holds(NETWORK_MANAGER),
    })))
}

fn combined_routing_entries(text: &str) -> Option<Vec<Vec<String>>> {
    let entries = routing_entries(text)?;
    let blocks = scan_config_raw(text).blocks;
    let mut units = utf16::units(text);
    for b in blocks.into_iter().rev() {
        let end = b.to.min(units.len());
        let start = b.from.min(end);
        units.splice(start..end, [b'\n' as u16]);
    }
    let fragments = String::from_utf16_lossy(&units);
    let bare = routing_entries(&format!("routing {{\n{fragments}\n}}"))?;
    let combined: Vec<Vec<String>> = entries
        .into_iter()
        .chain(bare)
        .filter(|e| e.first().map(String::as_str) != Some("include"))
        .collect();
    let (fallbacks, others): (Vec<_>, Vec<_>) = combined
        .into_iter()
        .partition(|e| e.first().map(String::as_str) == Some("fallback"));
    Some(others.into_iter().chain(fallbacks).collect())
}

fn starting_entries() -> &'static Vec<Vec<Vec<String>>> {
    static START: OnceLock<Vec<Vec<Vec<String>>>> = OnceLock::new();
    START.get_or_init(|| {
        let direct_fallback = vec![
            "fallback".to_string(),
            ":".to_string(),
            "direct".to_string(),
        ];
        let mut global_direct = template_entries(
            "global",
            TemplateOpts {
                block_ads: false,
                block_quic: true,
                network_manager_direct: true,
            },
        );
        global_direct.pop();
        global_direct.push(direct_fallback.clone());
        let demo = combined_routing_entries(&format!("{DEMO_ROUTING}\n{DEMO_ROUTING_INCLUDE}"))
            .unwrap_or_default();
        vec![global_direct, vec![direct_fallback], demo]
    })
}

pub fn is_preset_routing(text: String) -> Result<bool, String> {
    let Some(entries) = combined_routing_entries(&text) else {
        return Ok(false);
    };
    if entries.is_empty() {
        return Ok(true);
    }
    Ok(starting_entries().iter().any(|exp| exp == &entries))
}

// --- setup.ts ---

pub fn holds_dns(texts: Vec<String>) -> Result<bool, String> {
    Ok(texts
        .iter()
        .any(|t| scan_config_raw(t).blocks.iter().any(|b| b.name == "dns")))
}

pub fn holds_routing(texts: Vec<String>) -> Result<bool, String> {
    Ok(texts.iter().any(|t| {
        scan_config_raw(t).blocks.iter().any(|b| {
            b.name == "routing"
                && block_body_raw(t, b)
                    .iter()
                    .any(|line| !js_trim(&uncomment_str(line)).is_empty())
        })
    }))
}

pub fn template_groups(rules: String, defined: Vec<String>) -> Result<Vec<String>, String> {
    let Some(groups) = template_group_names(&rules) else {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'groups')".to_string(),
        );
    };
    let have: HashSet<&str> = defined.iter().map(String::as_str).collect();
    let wanted: Vec<String> = groups
        .iter()
        .copied()
        .filter(|g| !have.contains(g))
        .map(str::to_string)
        .collect();
    if !wanted.is_empty() || !defined.is_empty() {
        Ok(wanted)
    } else {
        Ok(vec!["proxy".to_string()])
    }
}

// --- sources.ts ---

fn collect_named(
    sources: &[Value],
    draft: Option<&Value>,
    read: impl Fn(&str) -> Vec<(String, String)>,
) -> Result<Value, String> {
    let draft_defined = draft.filter(|v| !v.is_null() && !js::is_undefined(v));
    let draft_obj = draft_defined.and_then(Value::as_object);
    let draft_id = draft_obj
        .and_then(|o| o.get("id"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let mut map: IndexMap<String, (String, String)> = IndexMap::new();
    for source in sources {
        let so = crate::port::js_str::ensure_not_nullish(source, "id")?;
        let src_id = so
            .and_then(|o| o.get("id"))
            .cloned()
            .unwrap_or_else(js::undefined);
        let text_val = if src_id == draft_id {
            if draft_defined.is_none() {
                return Err(
                    "TypeError: Cannot read properties of undefined (reading 'content')"
                        .to_string(),
                );
            }
            draft_obj.and_then(|o| obj_defined(o, "content"))
        } else {
            so.and_then(|o| obj_defined(o, "content"))
        };
        if let Some(tv) = text_val {
            let s = tv.as_str().ok_or_else(|| {
                "TypeError: Cannot read properties of undefined (reading 'content')".to_string()
            })?;
            for (name, written) in read(s) {
                map.insert(written.clone(), (name, written));
            }
        }
    }
    Ok(Value::Array(
        map.into_values()
            .map(|(name, written)| json!({"name": name, "written": written}))
            .collect(),
    ))
}

pub fn all_group_names(sources: Vec<Value>, draft: Option<Value>) -> Result<Value, String> {
    collect_named(&sources, draft.as_ref(), |text| {
        read_group_entries_raw(text)
            .into_iter()
            .map(|g| (g.name, g.written))
            .collect()
    })
}

pub fn all_upstream_names(sources: Vec<Value>, draft: Option<Value>) -> Result<Value, String> {
    collect_named(&sources, draft.as_ref(), |text| {
        dns_upstreams_raw(text, None)
            .into_iter()
            .map(|(written, _)| (unquote_str(&written), written))
            .collect()
    })
}

fn is_redacted_path(path: Option<&str>) -> bool {
    matches!(path, Some("<redacted>" | ""))
}

pub fn redacted(source: Value) -> Result<Value, String> {
    let so = crate::port::js_str::ensure_not_nullish(&source, "path")?;
    let path = so
        .and_then(|o| obj_defined(o, "path"))
        .and_then(Value::as_str);
    Ok(Value::Bool(is_redacted_path(path)))
}

pub fn file_name(source: Value) -> Result<Value, String> {
    let Some(o) = source.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'path')".to_string());
    };
    let path = obj_defined(o, "path").and_then(Value::as_str);
    if is_redacted_path(path) {
        let kind = obj_defined(o, "kind").and_then(Value::as_str).unwrap_or("");
        if kind == "main" {
            return Ok(Value::String("config.dae".to_string()));
        }
        let id = obj_defined(o, "id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                "TypeError: Cannot read properties of undefined (reading 'slice')".to_string()
            })?;
        let id_u = utf16::units(id);
        let short = js_slice(&id_u, 0, 8);
        return Ok(Value::String(format!("{kind}-{short}.dae")));
    }
    let Some(p) = path else {
        return Err("TypeError: Cannot read properties of undefined (reading 'split')".to_string());
    };
    let last = p.rsplit('/').next().unwrap_or("");
    Ok(Value::String(if last.is_empty() {
        "config.dae".to_string()
    } else {
        last.to_string()
    }))
}

pub fn validation_sources(
    sources: Vec<Value>,
    replacement: Option<Value>,
) -> Result<Option<Value>, String> {
    let main = sources
        .iter()
        .find(|s| s.get("kind").and_then(Value::as_str) == Some("main"));
    let Some(main) = main else {
        return Ok(None);
    };
    let mut authored: Vec<&Value> = vec![main];
    for s in &sources {
        let kind = s.get("kind").and_then(Value::as_str);
        let path = s.get("path").and_then(Value::as_str);
        if kind == Some("include") && !is_redacted_path(path) {
            authored.push(s);
        }
    }
    let repl = replacement.filter(is_truthy);
    if let Some(r) = &repl {
        let rid = r
            .as_object()
            .and_then(|o| o.get("id"))
            .cloned()
            .unwrap_or_else(js::undefined);
        if !authored.iter().any(|s| {
            s.as_object()
                .and_then(|o| o.get("id"))
                .cloned()
                .unwrap_or_else(js::undefined)
                == rid
        }) {
            return Ok(None);
        }
    }
    let out: Vec<Value> = authored
        .into_iter()
        .map(|s| {
            let mut m = Map::new();
            let id = s
                .as_object()
                .and_then(|o| o.get("id"))
                .cloned()
                .unwrap_or_else(js::undefined);
            let path = s.get("path").and_then(Value::as_str);
            m.insert("id".to_string(), id.clone());
            if !is_redacted_path(path)
                && let Some(pv) = s.get("path")
            {
                m.insert("path".to_string(), pv.clone());
            }
            let content = if let Some(r) = &repl
                && r.as_object()
                    .and_then(|o| o.get("id"))
                    .cloned()
                    .unwrap_or_else(js::undefined)
                    == id
            {
                r.as_object()
                    .and_then(|o| o.get("content"))
                    .cloned()
                    .unwrap_or_else(js::undefined)
            } else {
                s.as_object()
                    .and_then(|o| o.get("content"))
                    .cloned()
                    .unwrap_or_else(js::undefined)
            };
            m.insert("content".to_string(), content);
            Value::Object(m)
        })
        .collect();
    Ok(Some(Value::Array(out)))
}

fn dotted_key_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // INFALLIBLE: constant valid regex pattern
        regex::Regex::new(r"(?-u)\b[a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+\b").expect("valid regex")
    })
}

pub fn restart_settings(diagnostics: Vec<Value>) -> Result<Vec<Value>, String> {
    let re = dotted_key_regex();
    let mut out = Vec::new();
    for item in diagnostics {
        let io = crate::port::js_str::ensure_not_nullish(&item, "level")?;
        if io
            .and_then(|o| obj_defined(o, "level"))
            .and_then(Value::as_str)
            != Some("error")
            || io
                .and_then(|o| obj_defined(o, "code"))
                .and_then(Value::as_str)
                != Some("restart-required")
        {
            continue;
        }
        let msg = io
            .and_then(|o| obj_defined(o, "message"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let key: Value = if let Some(m) = re.find(msg) {
            Value::String(m.as_str().to_string())
        } else if !msg.is_empty() {
            Value::String(msg.to_string())
        } else {
            Value::Null
        };
        let source_id = io
            .and_then(|o| o.get("source_id"))
            .cloned()
            .unwrap_or_else(js::undefined);
        let line = io
            .and_then(|o| o.get("line"))
            .cloned()
            .unwrap_or_else(js::undefined);
        out.push(json!({
            "key": key,
            "sourceId": source_id,
            "line": line,
        }));
    }
    Ok(out)
}

pub fn restart_required(diagnostics: Vec<Value>) -> Result<Value, String> {
    Ok(json!(restart_settings(diagnostics)?.len()))
}

pub fn unwritten_restart_settings(
    settings: Vec<Value>,
    sources: Vec<Value>,
) -> Result<Value, String> {
    let mut out = Vec::new();
    for setting in settings {
        let Some(o) = setting.as_object() else {
            return Err("TypeError: Cannot read properties of null (reading 'key')".to_string());
        };
        let leaf = obj_defined(o, "key")
            .and_then(Value::as_str)
            .and_then(|k| k.rsplit('.').next());
        let Some(leaf) = leaf.filter(|s| !s.is_empty()) else {
            continue;
        };
        let mentioned = sources.iter().any(|s| {
            s.as_object()
                .and_then(|so| obj_defined(so, "content"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .contains(leaf)
        });
        if !mentioned {
            out.push(setting);
        }
    }
    Ok(Value::Array(out))
}
