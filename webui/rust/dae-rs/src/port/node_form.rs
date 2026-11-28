//! Port of `src/dae/nodeForm.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    UrlSearchParams, decode_base64_utf8, decode_uri_component, encode_base64_utf8,
    encode_uri_component, is_js_ws, js_parse_int_10, js_to_string, js_trim, js_trim_start,
    js_val_to_number, obj_defined,
};
use serde_json::{Map, Value, json};
use std::collections::HashSet;

const PROTOCOLS: &[&str] = &[
    "vmess",
    "vless",
    "hysteria2",
    "ss",
    "trojan",
    "tuic",
    "anytls",
    "socks5",
    "juicity",
];

const SHADOWSOCKS_METHODS: &[&str] = &[
    "aes-128-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "2022-blake3-aes-128-gcm",
    "2022-blake3-aes-256-gcm",
    "2022-blake3-chacha20-poly1305",
];
const VMESS_METHODS: &[&str] = &["auto", "aes-128-gcm"];
const CONGESTION_CONTROLS: &[&str] = &["cubic", "bbr", "new_reno"];
const OBFS_TYPES: &[&str] = &["", "salamander"];
const TRANSPORTS: &[&str] = &["tcp", "ws", "grpc", "xhttp"];
const SS_PLUGINS: &[&str] = &["", "v2ray-plugin", "obfs-local"];
const PACKET_ENCODINGS: &[&str] = &["auto", "none", "xudp", "uot-v2"];
const MUX_MODES: &[&str] = &["off", "h2mux", "xray"];
const UDP443_POLICIES: &[&str] = &["reject", "skip", "allow"];
const XHTTP_MODES: &[&str] = &["auto", "packet-up", "stream-up", "stream-one"];

const ALIASES: &[(&str, &[&str])] = &[
    ("sni", &["sni", "peer"]),
    ("plugin-opts", &["plugin-opts", "plugin_opts"]),
    ("insecure", &["allowInsecure", "allow_insecure", "insecure"]),
    ("type", &["type", "network", "net"]),
    ("serviceName", &["serviceName", "service_name"]),
    ("transportHost", &["host", "obfsParam"]),
    ("security", &["security", "tls"]),
    ("flow", &["flow", "xtls"]),
    ("pinSHA256", &["pinSHA256", "pin_sha256"]),
    ("obfs-password", &["obfs-password", "obfs_password"]),
    ("ech_config", &["ech_config", "echconfig"]),
];

const JSON_KEYS: &[(&str, &str)] = &[
    ("host", "add"),
    ("port", "port"),
    ("username", "id"),
    ("name", "ps"),
    ("type", "net"),
    ("path", "path"),
    ("transportHost", "host"),
    ("sni", "sni"),
    ("method", "scy"),
    ("security", "tls"),
    ("alpn", "alpn"),
    ("mode", "mode"),
    ("extra", "extra"),
];

pub fn is_protocol(value: String) -> Result<bool, String> {
    Ok(PROTOCOLS.contains(&value.as_str()))
}

pub fn protocol_name(protocol: Value) -> Result<Value, String> {
    if protocol.is_null() || js::is_undefined(&protocol) {
        return Err(
            "TypeError: Cannot read properties of null (reading 'toUpperCase')".to_string(),
        );
    }
    let Some(s) = protocol.as_str() else {
        return Err("TypeError: protocol.toUpperCase is not a function".to_string());
    };
    let name = match s {
        "ss" => "Shadowsocks".to_string(),
        "hysteria2" => "Hysteria2".to_string(),
        "anytls" => "AnyTLS".to_string(),
        "juicity" => "Juicity".to_string(),
        _ => s.to_uppercase(),
    };
    Ok(Value::String(name))
}

fn empty_node_map(protocol_val: Value) -> Map<String, Value> {
    let p = protocol_val.as_str();
    let security = if matches!(p, Some("vless" | "trojan" | "anytls")) {
        "tls"
    } else {
        "none"
    };
    let method = if p == Some("vmess") {
        "auto"
    } else {
        "aes-128-gcm"
    };
    let mut m = Map::new();
    m.insert("protocol".to_string(), protocol_val);
    m.insert("host".to_string(), Value::String(String::new()));
    m.insert("port".to_string(), Value::String("443".to_string()));
    m.insert("username".to_string(), Value::String(String::new()));
    m.insert("password".to_string(), Value::String(String::new()));
    m.insert("name".to_string(), Value::String(String::new()));
    m.insert("security".to_string(), Value::String(security.to_string()));
    m.insert("type".to_string(), Value::String("tcp".to_string()));
    m.insert("method".to_string(), Value::String(method.to_string()));
    m
}

pub fn empty_node(protocol: Value) -> Result<Value, String> {
    let effective = if js::is_undefined(&protocol) {
        Value::String("vless".to_string())
    } else {
        protocol
    };
    Ok(Value::Object(empty_node_map(effective)))
}

fn split_chain_arrow(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'>' {
            let left = js_trim_end_idx(s, start, i);
            parts.push(&s[start..left]);
            let mut after = i + 2;
            while after < s.len() {
                if let Some(ch) = s[after..].chars().next()
                    && is_js_ws(ch)
                {
                    after += ch.len_utf8();
                } else {
                    break;
                }
            }
            start = after;
            i = after;
        } else {
            i += 1;
        }
    }
    parts.push(&s[start..]);
    parts
}

fn js_trim_end_idx(s: &str, start: usize, end: usize) -> usize {
    let slice = &s[start..end];
    let trimmed = crate::port::js_str::js_trim_end(slice);
    start + trimmed.len()
}

fn field_str(fields: &Map<String, Value>, key: &str) -> String {
    fields
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn set_field(fields: &mut Map<String, Value>, key: &str, val: String) {
    fields.insert(key.to_string(), Value::String(val));
}

pub fn parse_node_link(original: String) -> Result<Value, String> {
    let trimmed = js_trim(&original);
    let chain_parts = split_chain_arrow(trimmed);
    let first = chain_parts[0];
    let chain = &chain_parts[1..];
    let scheme = first
        .find("://")
        .map(|pos| &first[..pos])
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric()))
        .map(str::to_ascii_lowercase);
    let protocol = match scheme.as_deref() {
        Some("hy2" | "hysteria") => "hysteria2",
        Some(s) if PROTOCOLS.contains(&s) => s,
        _ => return Err("Unsupported protocol".to_string()),
    };
    let mut fields = empty_node_map(Value::String(protocol.to_string()));
    let after_scheme = first.find("://").map_or("", |p| &first[p + 3..]);
    let (mut body, name_val) = match after_scheme.rfind('#') {
        Some(hash) => {
            let frag = after_scheme[hash + 1..].replace('+', "%2B");
            let parsed = UrlSearchParams::parse(&format!("name={frag}"));
            (
                &after_scheme[..hash],
                parsed.get("name").unwrap_or("").to_string(),
            )
        }
        None => (after_scheme, String::new()),
    };
    set_field(&mut fields, "name", name_val);
    let (mut authority, query) = match body.find('?') {
        Some(q) => (body[..q].to_string(), body[q + 1..].to_string()),
        None => (body.to_string(), String::new()),
    };
    body = "";
    let _ = body;

    let mut json_doc: Option<Value> = None;
    let mut raw_credentials = false;
    let mut encoded_authority = false;

    if protocol == "vmess" {
        let decoded = decode_base64_utf8(&authority)?;
        if js_trim_start(&decoded).starts_with('{') {
            let parsed: Value =
                serde_json::from_str(&decoded).map_err(|e| format!("SyntaxError: {e}"))?;
            let obj = parsed.as_object().ok_or("SyntaxError: expected object")?;
            for &(key, source) in JSON_KEYS {
                if let Some(v) = obj_defined(obj, source) {
                    let s = match v {
                        Value::Null | Value::Array(_) | Value::Object(_) => {
                            serde_json::to_string(v).unwrap_or_default()
                        }
                        _ => js_to_string(v),
                    };
                    set_field(&mut fields, key, s);
                }
            }
            let tls_is_tls = obj_defined(obj, "tls").and_then(Value::as_str) == Some("tls");
            set_field(
                &mut fields,
                "security",
                if tls_is_tls { "tls" } else { "none" }.to_string(),
            );
            let scy = obj_defined(obj, "scy").filter(|v| crate::port::js_str::is_truthy(v));
            let sec = obj_defined(obj, "security").filter(|v| crate::port::js_str::is_truthy(v));
            let method_raw = scy
                .or(sec)
                .map(js_to_string)
                .unwrap_or_else(|| "auto".to_string());
            set_field(&mut fields, "method", js_trim(&method_raw).to_lowercase());
            if field_str(&fields, "type") == "grpc" {
                let path = field_str(&fields, "path");
                set_field(&mut fields, "serviceName", path);
            }
            if field_str(&fields, "type") == "splithttp" {
                set_field(&mut fields, "type", "xhttp".to_string());
            }
            if field_str(&fields, "type").is_empty() {
                set_field(&mut fields, "type", "tcp".to_string());
            }
            json_doc = Some(parsed);
        } else {
            authority = decoded;
            encoded_authority = true;
        }
    } else if matches!(protocol, "ss" | "vless")
        && !authority.contains('@')
        && !authority.contains(':')
    {
        let stripped = authority.strip_suffix('/').unwrap_or(&authority);
        authority = match decode_base64_utf8(stripped) {
            Ok(d) => d,
            Err(_) => {
                let first_seg = authority.split('/').next().unwrap_or("");
                decode_base64_utf8(first_seg)?
            }
        };
        raw_credentials = protocol == "ss";
        encoded_authority = true;
    }

    if json_doc.is_none() {
        let (mut credentials, after_at) = match authority.rfind('@') {
            Some(at) => (authority[..at].to_string(), &authority[at + 1..]),
            None => (String::new(), authority.as_str()),
        };
        let endpoint = after_at.split('/').next().unwrap_or("");
        if protocol == "ss" && !credentials.is_empty() && !credentials.contains(':') {
            credentials = decode_base64_utf8(&credentials)?;
            raw_credentials = true;
        }
        let decode = |v: &str| -> Result<String, String> {
            if raw_credentials {
                Ok(v.to_string())
            } else {
                decode_uri_component(v)
            }
        };
        let colon = credentials.find(':');
        if matches!(protocol, "trojan" | "anytls" | "vless") {
            let raw_single = match colon {
                Some(c) => &credentials[c + 1..],
                None => &credentials,
            };
            let single = decode(raw_single)?;
            if protocol == "vless" {
                set_field(&mut fields, "username", single);
            } else {
                set_field(&mut fields, "password", single);
            }
        } else {
            let (u_raw, p_raw) = match colon {
                Some(c) => (&credentials[..c], &credentials[c + 1..]),
                None => (credentials.as_str(), ""),
            };
            set_field(&mut fields, "username", decode(u_raw)?);
            set_field(&mut fields, "password", decode(p_raw)?);
        }
        if protocol == "ss" {
            let mut m = field_str(&fields, "username");
            if !m.contains('-')
                && let Ok(dec) = decode_base64_utf8(&m)
            {
                m = dec;
            }
            set_field(&mut fields, "method", m);
            set_field(&mut fields, "username", String::new());
        }
        if protocol == "hysteria2" {
            set_field(&mut fields, "password", decode_uri_component(&credentials)?);
            set_field(&mut fields, "username", String::new());
        }
        // Parse endpoint: `/^(\[[^\]]+\]|[^:]+)(?::([^/]+))?$/`
        let (host_raw, port_opt) = parse_endpoint(endpoint).ok_or("Invalid address")?;
        let host = host_raw.strip_prefix('[').unwrap_or(host_raw);
        let host = host.strip_suffix(']').unwrap_or(host);
        set_field(&mut fields, "host", host.to_string());
        let port = port_opt.unwrap_or("443").to_string();
        if protocol == "hysteria2" && (port.contains(',') || port.contains('-')) {
            let first_port = port.split([',', '-']).next().unwrap_or("").to_string();
            set_field(&mut fields, "port", first_port);
            set_field(&mut fields, "mport", port);
        } else {
            set_field(&mut fields, "port", port);
        }

        let params = UrlSearchParams::parse(&query);
        for (key, value) in &params.pairs {
            if !matches!(
                key.as_str(),
                "protocol" | "port" | "username" | "password" | "name" | "method"
            ) {
                let target_key = if key == "host" { "transportHost" } else { key };
                set_field(&mut fields, target_key, value.clone());
            }
        }
        for &(key, names) in ALIASES {
            if let Some(found) = names
                .iter()
                .find_map(|n| params.get(n).filter(|v| !v.is_empty()))
            {
                set_field(&mut fields, key, found.to_string());
            }
        }
        if encoded_authority && matches!(protocol, "vmess" | "vless") {
            if !params.has("security") {
                set_field(
                    &mut fields,
                    "security",
                    if params.get("tls") == Some("1") {
                        "tls"
                    } else {
                        "none"
                    }
                    .to_string(),
                );
            }
            if let Some(obfs) = params.get("obfs").filter(|v| !v.is_empty()) {
                set_field(
                    &mut fields,
                    "type",
                    if obfs == "websocket" { "ws" } else { obfs }.to_string(),
                );
            }
            if field_str(&fields, "type") == "grpc" && field_str(&fields, "serviceName").is_empty()
            {
                let path = field_str(&fields, "path");
                set_field(&mut fields, "serviceName", path);
            }
        }
        if matches!(protocol, "vless" | "trojan") {
            let security = params.get("security");
            let reality =
                security == Some("reality") || ["pbk", "sid", "spx"].iter().any(|k| params.has(k));
            let resolved = if reality {
                "reality"
            } else if security == Some("none") || params.get("tls") == Some("0") {
                "none"
            } else if security.is_some_and(|s| !s.is_empty())
                || params.get("tls") == Some("1")
                || !encoded_authority
            {
                "tls"
            } else {
                "none"
            };
            set_field(&mut fields, "security", resolved.to_string());
        }
        if !["type", "network", "net"]
            .iter()
            .any(|k| params.get(k).is_some_and(|v| !v.is_empty()))
            && let Some(obfs) = params.get("obfs").filter(|v| !v.is_empty())
            && matches!(protocol, "vless" | "vmess")
        {
            set_field(
                &mut fields,
                "type",
                if obfs == "websocket" { "ws" } else { obfs }.to_string(),
            );
        }
        if field_str(&fields, "type") == "grpc"
            && field_str(&fields, "serviceName").is_empty()
            && params.get("obfs") == Some("grpc")
        {
            let path = field_str(&fields, "path");
            set_field(&mut fields, "serviceName", path);
        }
        if !params.get("flow").is_some_and(|v| !v.is_empty()) && params.has("xtls") {
            set_field(
                &mut fields,
                "flow",
                if params.get("xtls") == Some("2") {
                    "xtls-rprx-vision"
                } else {
                    ""
                }
                .to_string(),
            );
        }
        if let Some(ins) = fields
            .get("insecure")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            let norm = if matches!(
                js_trim(&ins).to_lowercase().as_str(),
                "true" | "yes" | "1" | "on"
            ) {
                "1"
            } else {
                "0"
            };
            set_field(&mut fields, "insecure", norm.to_string());
        }
        if field_str(&fields, "flow") == "xtls-rprx-vision-udp443" {
            set_field(&mut fields, "flow", "xtls-rprx-vision".to_string());
        }
        if field_str(&fields, "type") == "splithttp" {
            set_field(&mut fields, "type", "xhttp".to_string());
        }
        if protocol == "ss"
            && let Some(pl) = fields
                .get("plugin")
                .and_then(Value::as_str)
                .map(str::to_string)
            && let Some(semi) = pl.find(';')
        {
            set_field(&mut fields, "plugin", pl[..semi].to_string());
            if field_str(&fields, "plugin-opts").is_empty() {
                set_field(&mut fields, "plugin-opts", pl[semi + 1..].to_string());
            }
        }
        if field_str(&fields, "name").is_empty() {
            set_field(
                &mut fields,
                "name",
                params.get("remark").unwrap_or("").to_string(),
            );
        }
        if protocol == "vmess" {
            if !field_str(&fields, "password").is_empty() {
                let u = field_str(&fields, "username");
                let p = field_str(&fields, "password");
                set_field(&mut fields, "method", u);
                set_field(&mut fields, "username", p);
                set_field(&mut fields, "password", String::new());
            }
            let cur_method = field_str(&fields, "method");
            let new_method = params
                .get("scy")
                .or_else(|| params.get("encryption"))
                .map(str::to_string)
                .unwrap_or_else(|| match params.get("security") {
                    Some("auto") => "auto".to_string(),
                    Some("aes-128-gcm") => "aes-128-gcm".to_string(),
                    _ => cur_method,
                });
            set_field(&mut fields, "method", new_method);
            let is_tls = params.get("tls") == Some("1") || params.get("security") == Some("tls");
            set_field(
                &mut fields,
                "security",
                if is_tls { "tls" } else { "none" }.to_string(),
            );
        }
    }

    if protocol == "vmess" {
        let m = js_trim(&field_str(&fields, "method")).to_lowercase();
        set_field(
            &mut fields,
            "method",
            if m.is_empty() { "auto".to_string() } else { m },
        );
    }

    let tail = if chain.is_empty() {
        String::new()
    } else {
        format!(" -> {}", chain.join(" -> "))
    };
    let fields_val = Value::Object(fields.clone());
    let mut doc = Map::new();
    doc.insert("original".to_string(), Value::String(original));
    doc.insert("initial".to_string(), Value::Object(fields));
    doc.insert("query".to_string(), Value::String(query));
    doc.insert("json".to_string(), json_doc.unwrap_or_else(js::undefined));
    doc.insert("tail".to_string(), Value::String(tail));

    Ok(json!({
        "document": Value::Object(doc),
        "fields": fields_val,
    }))
}

fn parse_endpoint(endpoint: &str) -> Option<(&str, Option<&str>)> {
    if endpoint.is_empty() {
        return None;
    }
    if endpoint.starts_with('[') {
        let close = endpoint.find(']')?;
        if close == 1 {
            return None;
        }
        let host = &endpoint[..=close];
        let rest = &endpoint[close + 1..];
        if rest.is_empty() {
            return Some((host, None));
        }
        let port = rest.strip_prefix(':')?;
        if port.is_empty() || port.contains('/') {
            return None;
        }
        return Some((host, Some(port)));
    }
    match endpoint.find(':') {
        Some(0) => None,
        Some(colon) => {
            let host = &endpoint[..colon];
            let port = &endpoint[colon + 1..];
            if port.is_empty() || port.contains('/') {
                return None;
            }
            Some((host, Some(port)))
        }
        None => Some((endpoint, None)),
    }
}

fn strip_undefined_for_stringify(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, val) in m {
                if !js::is_undefined(val) {
                    out.insert(k.clone(), strip_undefined_for_stringify(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(
            a.iter()
                .map(|x| {
                    if js::is_undefined(x) {
                        Value::Null
                    } else {
                        strip_undefined_for_stringify(x)
                    }
                })
                .collect(),
        ),
        _ => v.clone(),
    }
}

fn js_json_eq(a: &Value, b: &Value) -> bool {
    let sa = serde_json::to_string(&strip_undefined_for_stringify(a)).ok();
    let sb = serde_json::to_string(&strip_undefined_for_stringify(b)).ok();
    sa.is_some() && sa == sb
}

pub fn node_link(fields: Value, document: Option<Value>) -> Result<String, String> {
    let doc_obj = document
        .as_ref()
        .filter(|v| crate::port::js_str::is_truthy(v))
        .and_then(Value::as_object);
    if let Some(doc) = doc_obj
        && let Some(initial) = doc.get("initial")
        && js_json_eq(&fields, initial)
    {
        return Ok(doc
            .get("original")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string());
    }
    let fields_obj = fields.as_object();
    let proto_val = fields_obj
        .and_then(|o| o.get("protocol"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let proto = proto_val.as_str().unwrap_or("");
    let init_obj = doc_obj
        .and_then(|d| d.get("initial"))
        .and_then(Value::as_object);
    let same = init_obj
        .and_then(|i| i.get("protocol"))
        .is_some_and(|ip| ip == &proto_val);
    let tail = if same {
        doc_obj
            .and_then(|d| obj_defined(d, "tail"))
            .and_then(Value::as_str)
            .unwrap_or("")
    } else {
        ""
    };

    if proto == "vmess"
        && same
        && let Some(doc) = doc_obj
        && let Some(doc_json) = obj_defined(doc, "json").and_then(Value::as_object)
    {
        let mut json = doc_json.clone();
        for &(key, target) in JSON_KEYS {
            let init_v = init_obj
                .and_then(|i| i.get(key))
                .cloned()
                .unwrap_or_else(js::undefined);
            let cur_v = fields_obj
                .and_then(|o| o.get(key))
                .cloned()
                .unwrap_or_else(js::undefined);
            if init_v == cur_v {
                continue;
            }
            let new_val = if key == "security" {
                Value::String(if cur_v.as_str() == Some("tls") {
                    "tls".to_string()
                } else {
                    String::new()
                })
            } else if key == "extra"
                && let Some(s) = cur_v.as_str()
                && !s.is_empty()
            {
                serde_json::from_str(s).map_err(|e| format!("SyntaxError: {e}"))?
            } else if cur_v.is_null() || js::is_undefined(&cur_v) {
                Value::String(String::new())
            } else {
                cur_v
            };
            json.insert(target.to_string(), new_val);
        }
        let init_m = init_obj.and_then(|i| i.get("method"));
        let cur_m = fields_obj.and_then(|o| o.get("method"));
        if init_m != cur_m {
            json.remove("security");
        }
        let init_sn = init_obj.and_then(|i| i.get("serviceName"));
        let cur_sn = fields_obj.and_then(|o| o.get("serviceName"));
        if init_sn != cur_sn
            && fields_obj
                .and_then(|o| o.get("type"))
                .and_then(Value::as_str)
                == Some("grpc")
        {
            if let Some(sn) = cur_sn.filter(|v| !js::is_undefined(v)) {
                json.insert("path".to_string(), sn.clone());
            } else {
                json.remove("path");
            }
        }
        let encoded = encode_base64_utf8(
            &serde_json::to_string(&Value::Object(json)).map_err(|e| e.to_string())?,
        );
        return Ok(format!("vmess://{encoded}{tail}"));
    }

    let query = if same {
        doc_obj
            .and_then(|d| obj_defined(d, "query"))
            .and_then(Value::as_str)
            .unwrap_or("")
    } else {
        ""
    };
    let mut params = UrlSearchParams::parse(query);
    if let Some(fo) = fields_obj {
        for (key, value) in fo {
            if matches!(
                key.as_str(),
                "protocol" | "host" | "port" | "username" | "password" | "name" | "method"
            ) {
                continue;
            }
            if same
                && init_obj
                    .and_then(|i| i.get(key))
                    .cloned()
                    .unwrap_or_else(js::undefined)
                    == *value
            {
                continue;
            }
            let alias_list = ALIASES.iter().find(|(k, _)| k == key).map(|(_, a)| *a);
            if let Some(alist) = alias_list {
                for &alias in alist {
                    params.delete(if alias == "transportHost" {
                        "host"
                    } else {
                        alias
                    });
                }
            } else {
                params.delete(if key == "transportHost" { "host" } else { key });
            }
            if key == "type" && matches!(proto, "vmess" | "vless") {
                params.delete("obfs");
            }
            if crate::port::js_str::is_truthy(value) {
                let pkey = if key == "transportHost" { "host" } else { key };
                params.set(pkey, js_to_string(value));
            }
        }
    }

    if proto == "ss" {
        let po_changed = !same
            || fields_obj
                .and_then(|o| o.get("plugin-opts"))
                .cloned()
                .unwrap_or_else(js::undefined)
                != init_obj
                    .and_then(|i| i.get("plugin-opts"))
                    .cloned()
                    .unwrap_or_else(js::undefined);
        if po_changed {
            if let Some(pl) = fields_obj
                .and_then(|o| obj_defined(o, "plugin"))
                .filter(|v| crate::port::js_str::is_truthy(v))
            {
                params.set("plugin", js_to_string(pl));
            } else {
                params.delete("plugin");
            }
        }
    }
    if !matches!(proto, "vmess" | "vless" | "trojan") {
        params.delete("security");
        params.delete("type");
    }
    if proto == "vless" && !params.has("encryption") {
        params.set("encryption", "none".to_string());
    }
    if proto == "vmess" {
        let m_changed = !same
            || fields_obj
                .and_then(|o| o.get("method"))
                .cloned()
                .unwrap_or_else(js::undefined)
                != init_obj
                    .and_then(|i| i.get("method"))
                    .cloned()
                    .unwrap_or_else(js::undefined);
        if m_changed {
            params.delete("encryption");
            if let Some(sec) = params.get("security")
                && !matches!(sec, "none" | "tls")
            {
                params.delete("security");
            }
            let m = fields_obj
                .and_then(|o| o.get("method"))
                .map(js_to_string)
                .unwrap_or_else(|| "undefined".to_string());
            params.set("scy", m);
        }
        if params.get("security") == Some("none") {
            params.delete("security");
        }
    }
    if let Some(mp) = fields_obj
        .and_then(|o| obj_defined(o, "mport"))
        .filter(|v| crate::port::js_str::is_truthy(v))
    {
        let mp_changed = !same
            || init_obj
                .and_then(|i| i.get("mport"))
                .cloned()
                .unwrap_or_else(js::undefined)
                != *mp;
        let orig_head = doc_obj
            .and_then(|d| obj_defined(d, "original"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("");
        if mp_changed || orig_head.contains(':') || orig_head.contains(',') {
            params.set("mport", js_to_string(mp));
        }
    }

    let get_s = |k: &str| -> String {
        fields_obj
            .and_then(|o| o.get(k))
            .map(js_to_string)
            .unwrap_or_else(|| "undefined".to_string())
    };
    let host_v = fields_obj.and_then(|o| obj_defined(o, "host"));
    let Some(host_str) = host_v.and_then(Value::as_str) else {
        return Err(
            "TypeError: Cannot read properties of undefined (reading 'includes')".to_string(),
        );
    };
    let method_s = get_s("method");
    let password_v = fields_obj.and_then(|o| obj_defined(o, "password"));
    let password_s = get_s("password");
    let username_s = get_s("username");
    let port_s = get_s("port");

    let auth = if proto == "ss" {
        encode_base64_utf8(&format!("{method_s}:{password_s}"))
    } else if matches!(proto, "trojan" | "anytls" | "hysteria2") {
        encode_uri_component(&password_s)
    } else {
        let has_pw = password_v.is_some_and(crate::port::js_str::is_truthy);
        if has_pw {
            format!(
                "{}:{}",
                encode_uri_component(&username_s),
                encode_uri_component(&password_s)
            )
        } else {
            encode_uri_component(&username_s)
        }
    };

    let host = if host_str.contains(':') {
        let stripped = host_str.strip_prefix('[').unwrap_or(host_str);
        let stripped = stripped.strip_suffix(']').unwrap_or(stripped);
        format!("[{stripped}]")
    } else {
        host_str.to_string()
    };

    let authority = if proto == "vmess" {
        let prefix = if method_s == "auto" { "auto:" } else { "" };
        encode_base64_utf8(&format!("{prefix}{username_s}@{host}:{port_s}"))
    } else if !auth.is_empty() {
        format!("{auth}@{host}:{port_s}")
    } else {
        format!("{host}:{port_s}")
    };

    let query_str = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.serialize())
    };
    let name_part = match fields_obj
        .and_then(|o| obj_defined(o, "name"))
        .filter(|v| crate::port::js_str::is_truthy(v))
    {
        Some(nv) => format!("#{}", encode_uri_component(&js_to_string(nv))),
        None => String::new(),
    };
    let proto_out = js_to_string(&proto_val);
    Ok(format!(
        "{proto_out}://{authority}{query_str}{name_part}{tail}"
    ))
}

fn security_options_slice(protocol: Option<&str>) -> Option<&'static [&'static str]> {
    match protocol {
        Some("trojan") => Some(&["tls", "reality"]),
        Some("vless") => Some(&["none", "tls", "reality"]),
        Some("vmess") => Some(&["none", "tls"]),
        _ => None,
    }
}

pub fn security_options(protocol: Value) -> Result<Value, String> {
    match security_options_slice(protocol.as_str()) {
        Some(opts) => Ok(json!(opts)),
        None => Ok(js::undefined()),
    }
}

fn uses_tls_bool(f: &Value) -> Result<bool, String> {
    if f.is_null() || js::is_undefined(f) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    let proto = f
        .as_object()
        .and_then(|o| obj_defined(o, "protocol"))
        .and_then(Value::as_str);
    let sec = f
        .as_object()
        .and_then(|o| obj_defined(o, "security"))
        .and_then(Value::as_str);
    Ok(!matches!(proto, Some("ss" | "socks5"))
        && (sec != Some("none")
            || matches!(proto, Some("tuic" | "juicity" | "hysteria2" | "anytls"))))
}

pub fn uses_tls(f: Value) -> Result<Value, String> {
    Ok(Value::Bool(uses_tls_bool(&f)?))
}

fn flow_offered(f: &Value) -> bool {
    let o = f.as_object();
    let p = o
        .and_then(|m| obj_defined(m, "protocol"))
        .and_then(Value::as_str);
    let t = o
        .and_then(|m| obj_defined(m, "type"))
        .and_then(Value::as_str);
    let s = o
        .and_then(|m| obj_defined(m, "security"))
        .and_then(Value::as_str);
    p == Some("vless") && t == Some("tcp") && s != Some("none")
}

struct ManualFieldSpec {
    key: &'static str,
    options: Option<&'static [&'static str]>,
    flag: Option<&'static str>,
    required: Option<bool>,
    range: Option<(f64, f64)>,
}

impl ManualFieldSpec {
    fn plain(key: &'static str) -> Self {
        Self {
            key,
            options: None,
            flag: None,
            required: None,
            range: None,
        }
    }
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("key".to_string(), Value::String(self.key.to_string()));
        if let Some(opts) = self.options {
            m.insert("options".to_string(), json!(opts));
        }
        if let Some(flag) = self.flag {
            m.insert("flag".to_string(), Value::String(flag.to_string()));
        }
        if let Some((lo, hi)) = self.range {
            m.insert("range".to_string(), json!([js::number(lo), js::number(hi)]));
        }
        if let Some(req) = self.required {
            m.insert("required".to_string(), Value::Bool(req));
        }
        Value::Object(m)
    }
}

fn manual_fields_list(f: &Value) -> Result<Vec<ManualFieldSpec>, String> {
    if f.is_null() || js::is_undefined(f) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    let o = f.as_object();
    let p = o
        .and_then(|m| obj_defined(m, "protocol"))
        .and_then(Value::as_str);
    let sec = o
        .and_then(|m| obj_defined(m, "security"))
        .and_then(Value::as_str);
    let ty = o
        .and_then(|m| obj_defined(m, "type"))
        .and_then(Value::as_str);
    let obfs = o
        .and_then(|m| obj_defined(m, "obfs"))
        .is_some_and(crate::port::js_str::is_truthy);
    let mut fields = Vec::new();
    if matches!(p, Some("vmess" | "vless" | "tuic" | "juicity" | "socks5")) {
        let mut s = ManualFieldSpec::plain("username");
        s.required = Some(p != Some("socks5"));
        fields.push(s);
    }
    if !matches!(p, Some("vmess" | "vless")) {
        let mut s = ManualFieldSpec::plain("password");
        s.required = Some(p != Some("socks5"));
        fields.push(s);
    }
    if p == Some("ss") {
        let mut s = ManualFieldSpec::plain("method");
        s.options = Some(SHADOWSOCKS_METHODS);
        fields.push(s);
    }
    if p == Some("vmess") {
        let mut s = ManualFieldSpec::plain("method");
        s.options = Some(VMESS_METHODS);
        fields.push(s);
    }
    if let Some(opts) = security_options_slice(p) {
        let mut s = ManualFieldSpec::plain("security");
        s.options = Some(opts);
        fields.push(s);
    }
    if p == Some("ss") {
        let mut s = ManualFieldSpec::plain("plugin");
        s.options = Some(SS_PLUGINS);
        fields.push(s);
    }
    if flow_offered(f) {
        let mut s = ManualFieldSpec::plain("flow");
        s.options = Some(&["", "xtls-rprx-vision"]);
        fields.push(s);
    }
    if uses_tls_bool(f)? {
        fields.push(ManualFieldSpec::plain("sni"));
        let mut ins = ManualFieldSpec::plain("insecure");
        ins.flag = Some("1");
        fields.push(ins);
    }
    if sec == Some("reality") {
        let mut pbk = ManualFieldSpec::plain("pbk");
        pbk.required = Some(true);
        fields.push(pbk);
        fields.push(ManualFieldSpec::plain("sid"));
        fields.push(ManualFieldSpec::plain("spx"));
    }
    if matches!(p, Some("vmess" | "vless" | "trojan")) {
        let mut t = ManualFieldSpec::plain("type");
        t.options = Some(TRANSPORTS);
        fields.push(t);
        if matches!(ty, Some("ws" | "xhttp")) {
            fields.push(ManualFieldSpec::plain("transportHost"));
            fields.push(ManualFieldSpec::plain("path"));
        }
        if ty == Some("grpc") {
            fields.push(ManualFieldSpec::plain("serviceName"));
        }
        if ty == Some("xhttp") {
            let mut m = ManualFieldSpec::plain("mode");
            m.options = Some(XHTTP_MODES);
            fields.push(m);
        }
    }
    if p == Some("hysteria2") {
        let mut ob = ManualFieldSpec::plain("obfs");
        ob.options = Some(OBFS_TYPES);
        fields.push(ob);
        if obfs {
            fields.push(ManualFieldSpec::plain("obfs-password"));
        }
    }
    if p == Some("tuic") {
        let mut cc = ManualFieldSpec::plain("congestion_control");
        cc.options = Some(CONGESTION_CONTROLS);
        fields.push(cc);
    }
    Ok(fields)
}

pub fn manual_fields(f: Value) -> Result<Vec<Value>, String> {
    Ok(manual_fields_list(&f)?
        .into_iter()
        .map(|s| s.to_value())
        .collect())
}

const MAX_SAFE_INT: f64 = ((1u64 << 53) - 1) as f64;

fn advanced_fields_list(f: &Value) -> Result<Vec<ManualFieldSpec>, String> {
    if f.is_null() || js::is_undefined(f) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    let o = f.as_object();
    let p = o
        .and_then(|m| obj_defined(m, "protocol"))
        .and_then(Value::as_str);
    let ty = o
        .and_then(|m| obj_defined(m, "type"))
        .and_then(Value::as_str);
    let flow = o
        .and_then(|m| obj_defined(m, "flow"))
        .is_some_and(crate::port::js_str::is_truthy);
    let plugin = o
        .and_then(|m| obj_defined(m, "plugin"))
        .is_some_and(crate::port::js_str::is_truthy);
    let mux = o
        .and_then(|m| obj_defined(m, "mux"))
        .and_then(Value::as_str);
    let vision = flow_offered(f) && flow;
    let tls = uses_tls_bool(f)?;

    let mut fields = Vec::new();
    let count = |key: &'static str, range: (f64, f64), required: bool| {
        let mut s = ManualFieldSpec::plain(key);
        s.range = Some(range);
        s.required = Some(required);
        s
    };

    if p == Some("tuic") || (ty == Some("xhttp") && tls) {
        fields.push(ManualFieldSpec::plain("alpn"));
    }
    if tls {
        fields.push(ManualFieldSpec::plain("pinSHA256"));
        fields.push(ManualFieldSpec::plain("ech_config"));
    }
    if p == Some("ss") && plugin {
        fields.push(ManualFieldSpec::plain("plugin-opts"));
    }
    if p == Some("vless") {
        let mut pe = ManualFieldSpec::plain("packetEncoding");
        pe.options = Some(if vision {
            &["auto", "xudp"]
        } else {
            PACKET_ENCODINGS
        });
        fields.push(pe);
        if !vision {
            let mut mx = ManualFieldSpec::plain("mux");
            mx.options = Some(MUX_MODES);
            fields.push(mx);
        }
        if !vision && mux == Some("h2mux") {
            let mut pad = ManualFieldSpec::plain("padding");
            pad.flag = Some("true");
            fields.push(pad);
        }
        if !vision && mux == Some("xray") {
            let i16_min = f64::from(i16::MIN);
            let i16_max = f64::from(i16::MAX);
            if ty == Some("xhttp") {
                fields.push(count("concurrency", (i16_min, -1.0), true));
            } else {
                fields.push(count("concurrency", (i16_min, i16_max), false));
            }
            fields.push(count("xudpConcurrency", (i16_min, i16_max), false));
            let mut p443 = ManualFieldSpec::plain("xudpProxyUDP443");
            p443.options = Some(UDP443_POLICIES);
            fields.push(p443);
        }
    }
    if p == Some("hysteria2") {
        let u32_max = f64::from(u32::MAX);
        fields.push(count("upmbps", (0.0, u32_max), false));
        fields.push(count("downmbps", (0.0, u32_max), false));
        fields.push(ManualFieldSpec::plain("mport"));
        fields.push(count("mhop", (0.0, MAX_SAFE_INT), false));
    }
    if matches!(p, Some("hysteria2" | "tuic" | "juicity")) {
        let mtu_max = f64::from(u16::MAX - 8);
        fields.push(count("mtu", (1200.0, mtu_max), false));
    }
    if matches!(p, Some("hysteria2" | "tuic")) {
        fields.push(count("initStreamReceiveWindow", (0.0, MAX_SAFE_INT), false));
        fields.push(count("initConnReceiveWindow", (0.0, MAX_SAFE_INT), false));
    }
    if p == Some("hysteria2") {
        let mut d = ManualFieldSpec::plain("disablePathMTUDiscovery");
        d.flag = Some("1");
        fields.push(d);
    }
    if p == Some("anytls") {
        fields.push(count("min_idle_session", (0.0, f64::from(u16::MAX)), false));
        fields.push(count(
            "idle_session_check_interval",
            (0.0, MAX_SAFE_INT),
            false,
        ));
        fields.push(count("idle_session_timeout", (0.0, MAX_SAFE_INT), false));
    }
    Ok(fields)
}

pub fn advanced_fields(f: Value) -> Result<Vec<Value>, String> {
    Ok(advanced_fields_list(&f)?
        .into_iter()
        .map(|s| s.to_value())
        .collect())
}

pub fn settle(f: Value) -> Result<Value, String> {
    let mut next = match &f {
        Value::Object(m) => m.clone(),
        Value::Array(a) => {
            let mut m = Map::new();
            for (i, v) in a.iter().enumerate() {
                m.insert(i.to_string(), v.clone());
            }
            m
        }
        Value::String(s) => {
            let mut m = Map::new();
            for (i, u) in crate::utf16::units(s).into_iter().enumerate() {
                m.insert(i.to_string(), Value::String(String::from_utf16_lossy(&[u])));
            }
            m
        }
        _ => Map::new(),
    };
    if f.is_null() || js::is_undefined(&f) {
        return Err("TypeError: Cannot read properties of null (reading 'protocol')".to_string());
    }
    if !flow_offered(&f) {
        next.insert("flow".to_string(), Value::String(String::new()));
    }
    if next.get("flow").is_some_and(crate::port::js_str::is_truthy) {
        for key in [
            "mux",
            "padding",
            "concurrency",
            "xudpConcurrency",
            "xudpProxyUDP443",
        ] {
            next.insert(key.to_string(), Value::String(String::new()));
        }
        let pe = next.get("packetEncoding").and_then(Value::as_str);
        if !matches!(pe, Some("" | "auto" | "xudp")) {
            next.insert("packetEncoding".to_string(), Value::String(String::new()));
        }
    }
    Ok(Value::Object(next))
}

fn hops_valid(spec: &str) -> bool {
    let mut seen: HashSet<u32> = HashSet::new();
    for raw_part in spec.split(',') {
        let part = js_trim(raw_part);
        if part.is_empty() {
            continue;
        }
        let (low_s, high_s) = match part.find('-') {
            Some(dash) => (js_trim(&part[..dash]), js_trim(&part[dash + 1..])),
            None => (part, part),
        };
        if low_s.is_empty()
            || high_s.is_empty()
            || !low_s.bytes().all(|b| b.is_ascii_digit())
            || !high_s.bytes().all(|b| b.is_ascii_digit())
        {
            return false;
        }
        let Ok(low) = low_s.parse::<u32>() else {
            return false;
        };
        let Ok(high) = high_s.parse::<u32>() else {
            return false;
        };
        if low < 1 || high < low || high > 65535 {
            return false;
        }
        for port in low..=high {
            if !seen.insert(port) {
                return false;
            }
        }
    }
    true
}

fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 36 {
        return false;
    }
    for (i, &ch) in b.iter().enumerate() {
        if matches!(i, 8 | 13 | 18 | 23) {
            if ch != b'-' {
                return false;
            }
        } else if !ch.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn is_signed_int_with_suffix(s: &str) -> bool {
    let rest = s.strip_prefix('-').unwrap_or(s);
    let digit_count = rest.bytes().take_while(u8::is_ascii_digit).count();
    digit_count > 0 && rest[digit_count..].bytes().all(|b| b.is_ascii_alphabetic())
}

fn advanced_invalid(f: &Value, spec: &ManualFieldSpec) -> bool {
    let raw = f
        .as_object()
        .and_then(|o| obj_defined(o, spec.key))
        .filter(|v| !v.is_null())
        .and_then(Value::as_str)
        .unwrap_or("");
    let value = js_trim(raw);
    if value.is_empty() {
        return false;
    }
    if let Some((lo, hi)) = spec.range {
        if !is_signed_int_with_suffix(value) {
            return true;
        }
        let n = js_parse_int_10(value);
        return n < lo || n > hi;
    }
    if spec.key == "alpn" {
        let is_xhttp = f
            .as_object()
            .and_then(|o| obj_defined(o, "type"))
            .and_then(Value::as_str)
            == Some("xhttp");
        return is_xhttp
            && value.split(',').any(|item| {
                let t = js_trim(item);
                !t.is_empty() && t != "h2"
            });
    }
    spec.key == "mport" && !hops_valid(value)
}

pub fn manual_invalid(f: Value) -> Result<Vec<String>, String> {
    if f.is_null() || js::is_undefined(&f) {
        return Err("TypeError: Cannot read properties of null (reading 'port')".to_string());
    }
    let port_v = f
        .as_object()
        .and_then(|o| o.get("port"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let port = js_val_to_number(&port_v);
    let mut out = Vec::new();
    let host_s = f
        .as_object()
        .and_then(|o| o.get("host"))
        .map(js_to_string)
        .unwrap_or_else(|| "undefined".to_string());
    let host_ok = !host_s.is_empty()
        && !host_s
            .chars()
            .any(|c| is_js_ws(c) || matches!(c, '/' | '?' | '#' | '@' | '\\'));
    if !host_ok {
        out.push("host".to_string());
    }
    let port_ok =
        port.is_finite() && port.fract() == 0.0 && (1.0..=f64::from(u16::MAX)).contains(&port);
    if !port_ok {
        out.push("port".to_string());
    }
    let proto = f
        .as_object()
        .and_then(|o| obj_defined(o, "protocol"))
        .and_then(Value::as_str);
    let is_uuid_proto = matches!(proto, Some("vmess" | "vless" | "tuic" | "juicity"));
    let mut all_fields = manual_fields_list(&f)?;
    all_fields.extend(advanced_fields_list(&f)?);
    for field in all_fields {
        let fv = f.as_object().and_then(|o| o.get(field.key));
        let truthy = fv.is_some_and(crate::port::js_str::is_truthy);
        let req_bad = field.required == Some(true) && !truthy;
        let uuid_bad = field.key == "username"
            && is_uuid_proto
            && truthy
            && !is_uuid(&fv.map(js_to_string).unwrap_or_default());
        if req_bad || uuid_bad || advanced_invalid(&f, &field) {
            out.push(field.key.to_string());
        }
    }
    Ok(out)
}

pub fn manual_link(f: Value) -> Result<String, String> {
    if !manual_invalid(f.clone())?.is_empty() {
        return Ok(String::new());
    }
    let proto_val = f
        .as_object()
        .and_then(|o| o.get("protocol"))
        .cloned()
        .unwrap_or_else(js::undefined);
    let effective_proto = if js::is_undefined(&proto_val) {
        Value::String("vless".to_string())
    } else {
        proto_val
    };
    let mut fields = empty_node_map(effective_proto);
    for k in ["name", "host", "port"] {
        let v = f
            .as_object()
            .and_then(|o| o.get(k))
            .cloned()
            .unwrap_or_else(js::undefined);
        fields.insert(k.to_string(), v);
    }
    let mut all_fields = manual_fields_list(&f)?;
    all_fields.extend(advanced_fields_list(&f)?);
    for spec in all_fields {
        let v = f
            .as_object()
            .and_then(|o| o.get(spec.key))
            .cloned()
            .unwrap_or_else(js::undefined);
        fields.insert(spec.key.to_string(), v);
    }
    node_link(Value::Object(fields), None)
}
