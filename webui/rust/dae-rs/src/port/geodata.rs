//! Port of `src/dae/geodata.ts` and `src/dae/probes.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{is_truthy, obj_defined};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use url::Url;

struct PresetDef {
    id: &'static str,
    repo: &'static str,
    geosite_file: &'static str,
    geoip_file: &'static str,
    geosite_size: u64,
    geoip_size: u64,
    lite: bool,
}

const PRESETS: &[PresetDef] = &[
    PresetDef {
        id: "metacubex",
        repo: "MetaCubeX/meta-rules-dat",
        geosite_file: "geosite.dat",
        geoip_file: "geoip.dat",
        geosite_size: 4_200_000,
        geoip_size: 16_600_000,
        lite: false,
    },
    PresetDef {
        id: "metacubex-lite",
        repo: "MetaCubeX/meta-rules-dat",
        geosite_file: "geosite-lite.dat",
        geoip_file: "geoip-lite.dat",
        geosite_size: 176_000,
        geoip_size: 207_000,
        lite: true,
    },
    PresetDef {
        id: "loyalsoldier",
        repo: "Loyalsoldier/v2ray-rules-dat",
        geosite_file: "geosite.dat",
        geoip_file: "geoip.dat",
        geosite_size: 11_100_000,
        geoip_size: 16_900_000,
        lite: false,
    },
];

const LITE_GEOSITE: &[&str] = &[
    "abema",
    "apple",
    "applemusic",
    "bahamut",
    "bilibili",
    "biliintl",
    "cloudflare",
    "cn",
    "ehentai",
    "github",
    "google",
    "microsoft",
    "netflix",
    "onedrive",
    "openai",
    "pixiv",
    "private",
    "proxy",
    "proxymedia",
    "spotify",
    "telegram",
    "tiktok",
    "twitter",
    "youtube",
];

const LITE_GEOIP: &[&str] = &[
    "apple",
    "bilibili",
    "cloudflare",
    "cloudfront",
    "cn",
    "facebook",
    "google",
    "jp",
    "netflix",
    "private",
    "telegram",
    "twitter",
];

fn preset_urls(repo: &str, file: &str) -> [String; 2] {
    [
        format!("https://raw.githubusercontent.com/{repo}/release/{file}"),
        format!("https://fastly.jsdelivr.net/gh/{repo}@release/{file}"),
    ]
}

fn preset_mirrors(repo: &str, file: &str) -> [String; 2] {
    [
        format!("https://cdn.jsdelivr.net/gh/{repo}@release/{file}"),
        format!("https://testingcf.jsdelivr.net/gh/{repo}@release/{file}"),
    ]
}

fn preset_value(p: &PresetDef) -> Value {
    let categories = if p.lite {
        json!({
            "geosite": LITE_GEOSITE,
            "geoip": LITE_GEOIP,
        })
    } else {
        Value::Null
    };
    json!({
        "id": p.id,
        "urls": {
            "geosite": preset_urls(p.repo, p.geosite_file),
            "geoip": preset_urls(p.repo, p.geoip_file),
        },
        "mirrors": {
            "geosite": preset_mirrors(p.repo, p.geosite_file),
            "geoip": preset_mirrors(p.repo, p.geoip_file),
        },
        "sizes": {
            "geosite": p.geosite_size,
            "geoip": p.geoip_size,
        },
        "categories": categories,
    })
}

pub fn match_preset(urls: Value) -> Result<Option<Value>, String> {
    if urls.is_null() || js::is_undefined(&urls) {
        return Err("TypeError: Cannot read properties of null (reading 'geosite')".to_string());
    }
    for p in PRESETS {
        let mut all_ok = true;
        for (kind, file) in [("geosite", p.geosite_file), ("geoip", p.geoip_file)] {
            let entry = urls.as_object().and_then(|o| obj_defined(o, kind));
            let Some(val) = entry else {
                return Err(
                    "TypeError: Cannot read properties of undefined (reading 'length')".to_string(),
                );
            };
            if val.is_null() {
                return Err(
                    "TypeError: Cannot read properties of null (reading 'length')".to_string(),
                );
            }
            let Some(arr) = val.as_array() else {
                // JS: if val is e.g. string with length > 0, val.every throws; if non-empty not array, error; if length === 0 (like ""), short-circuit false.
                if let Some(s) = val.as_str() {
                    if s.is_empty() {
                        all_ok = false;
                        break;
                    }
                    return Err("TypeError: urls[kind].every is not a function".to_string());
                }
                all_ok = false;
                break;
            };
            if arr.is_empty() {
                all_ok = false;
                break;
            }
            let allowed_urls = preset_urls(p.repo, file);
            let allowed_mirrors = preset_mirrors(p.repo, file);
            if !arr.iter().all(|u| {
                u.as_str().is_some_and(|s| {
                    allowed_urls.iter().any(|a| a == s) || allowed_mirrors.iter().any(|a| a == s)
                })
            }) {
                all_ok = false;
                break;
            }
        }
        if all_ok {
            return Ok(Some(preset_value(p)));
        }
    }
    Ok(None)
}

pub fn missing_categories(
    preset: Option<Value>,
    required: Option<Value>,
) -> Result<Option<Value>, String> {
    let Some(preset_val) = preset else {
        return Ok(None);
    };
    let Some(req_val) = required else {
        return Ok(None);
    };
    if !is_truthy(&preset_val) || !is_truthy(&req_val) {
        return Ok(None);
    }
    let Some(categories) = preset_val
        .as_object()
        .and_then(|o| obj_defined(o, "categories"))
        .filter(|c| is_truthy(c))
    else {
        return Ok(None);
    };
    let mut missing = Map::new();
    for kind in ["geosite", "geoip"] {
        let cat_val = categories.as_object().and_then(|o| obj_defined(o, kind));
        let mut known = HashSet::new();
        if let Some(v) = cat_val {
            match v {
                Value::Array(arr) => {
                    for item in arr {
                        known.insert(item.clone());
                    }
                }
                Value::String(s) => {
                    for ch in s.chars() {
                        known.insert(Value::String(ch.to_string()));
                    }
                }
                Value::Null => {}
                _ => {
                    return Err("TypeError: preset.categories[kind] is not iterable".to_string());
                }
            }
        }
        let req_kind = req_val.as_object().and_then(|o| obj_defined(o, kind));
        let req_list: &[Value] = match req_kind {
            None | Some(Value::Null) => &[],
            Some(Value::Array(a)) => a.as_slice(),
            Some(_) => {
                return Err(
                    "TypeError: (required[kind] ?? []).filter is not a function".to_string()
                );
            }
        };
        let lacking: Vec<Value> = req_list
            .iter()
            .filter(|code| !known.contains(*code))
            .cloned()
            .collect();
        if !lacking.is_empty() {
            missing.insert(kind.to_string(), Value::Array(lacking));
        }
    }
    if missing.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Value::Object(missing)))
    }
}

pub fn geodata_preset(id: String) -> Result<Value, String> {
    match PRESETS.iter().find(|p| p.id == id) {
        Some(p) => Ok(preset_value(p)),
        None => Ok(js::undefined()),
    }
}

pub fn valid_geodata_url(url: String) -> Result<bool, String> {
    let rest = if let Some(r) = url.strip_prefix("http://") {
        r
    } else if let Some(r) = url.strip_prefix("https://") {
        r
    } else {
        return Ok(false);
    };
    if rest.contains('#') {
        return Ok(false);
    }
    let auth_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if rest[..auth_end].contains('@') {
        return Ok(false);
    }
    if url.chars().count() > 4096 {
        return Ok(false);
    }
    Ok(Url::parse(&url).is_ok())
}

// --- probes.ts ---

#[derive(Clone, Copy)]
struct ProbeSupport {
    connect: bool,
    stream: bool,
    udp: Option<bool>,
}

fn probe_support_for(protocol: Option<&str>) -> ProbeSupport {
    let key = match protocol {
        Some("shadowsocks") => Some("ss"),
        other => other,
    };
    match key {
        Some("ss" | "socks5") => ProbeSupport {
            connect: true,
            stream: true,
            udp: Some(true),
        },
        Some("vmess") => ProbeSupport {
            connect: true,
            stream: true,
            udp: Some(false),
        },
        Some("vless" | "trojan" | "anytls") => ProbeSupport {
            connect: true,
            stream: true,
            udp: None,
        },
        Some("hysteria2" | "tuic" | "juicity") => ProbeSupport {
            connect: false,
            stream: true,
            udp: Some(true),
        },
        Some("chain") => ProbeSupport {
            connect: false,
            stream: true,
            udp: None,
        },
        Some("direct") => ProbeSupport {
            connect: false,
            stream: true,
            udp: Some(true),
        },
        _ => ProbeSupport {
            connect: false,
            stream: false,
            udp: Some(false),
        },
    }
}

pub fn node_probe_support(protocol: Option<String>) -> Result<Value, String> {
    let s = probe_support_for(protocol.as_deref());
    Ok(json!({
        "connect": s.connect,
        "stream": s.stream,
        "udp": s.udp,
    }))
}

pub fn supports_node_probe(
    protocol: Option<String>,
    kind: Value,
    transports: Value,
) -> Result<bool, String> {
    let support = probe_support_for(protocol.as_deref());
    let is_tcp_connect = kind.as_str() == Some("tcp_connect");
    if is_tcp_connect {
        if !support.connect {
            return Ok(false);
        }
        let Some(arr) = transports.as_array() else {
            return Err("TypeError: transports.every is not a function".to_string());
        };
        Ok(arr.iter().all(|t| t.as_str() == Some("tcp")))
    } else {
        if !support.stream {
            return Ok(false);
        }
        let Some(arr) = transports.as_array() else {
            return Err("TypeError: transports.every is not a function".to_string());
        };
        let is_dns = kind.as_str() == Some("dns");
        Ok(arr
            .iter()
            .all(|t| t.as_str() == Some("tcp") || (is_dns && support.udp != Some(false))))
    }
}
