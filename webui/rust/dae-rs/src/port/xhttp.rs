//! Port of `src/dae/xhttp.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::{
    decode_uri_component, encode_uri_component, js_str_to_number, js_to_string, js_trim,
    obj_defined,
};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};

fn decode_param(raw: &str) -> Result<String, String> {
    decode_uri_component(&raw.replace('+', " "))
}

fn scalar(value: &str) -> Value {
    if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
        js::number(js_str_to_number(value))
    } else {
        Value::String(value.to_string())
    }
}

fn xhttp_text(value: Option<&Value>) -> Option<String> {
    match value {
        None | Some(Value::Null) => Some(String::new()),
        Some(v) if js::is_undefined(v) => Some(String::new()),
        Some(Value::String(s)) => Some(s.clone()),
        Some(v) if js::as_number(v).is_some() => Some(js_to_string(v)),
        _ => None,
    }
}

fn range_text(value: Option<&Value>) -> Option<String> {
    if let Some(Value::Object(o)) = value
        && let (Some(min), Some(max)) = (
            o.get("min").and_then(js::as_number),
            o.get("max").and_then(js::as_number),
        )
    {
        return Some(format!(
            "{}-{}",
            js::number_to_string(min),
            js::number_to_string(max)
        ));
    }
    xhttp_text(value)
}

fn put_json(target: &mut Map<String, Value>, key: &str, value: Value) {
    let empty = match &value {
        Value::Null => true,
        Value::Bool(false) => true,
        Value::String(s) => s.is_empty(),
        Value::Object(m) => m.is_empty() || js::is_undefined(&value),
        _ => false,
    };
    if empty {
        target.remove(key);
    } else {
        target.insert(key.to_string(), value);
    }
}

#[derive(Clone, Debug)]
struct DownloadFields {
    address: String,
    port: String,
    server_name: String,
    path: String,
    host: String,
}

#[derive(Clone, Debug)]
struct XhttpFields {
    mode: String,
    path: String,
    host: String,
    headers: Vec<(String, String)>,
    x_padding_bytes: String,
    no_grpc_header: bool,
    sc_max_each_post_bytes: String,
    sc_min_posts_interval_ms: String,
    download: Option<DownloadFields>,
}

impl XhttpFields {
    fn to_value(&self) -> Value {
        json!({
            "mode": self.mode,
            "path": self.path,
            "host": self.host,
            "headers": self.headers.iter().map(|(k, v)| json!([k, v])).collect::<Vec<_>>(),
            "xPaddingBytes": self.x_padding_bytes,
            "noGRPCHeader": self.no_grpc_header,
            "scMaxEachPostBytes": self.sc_max_each_post_bytes,
            "scMinPostsIntervalMs": self.sc_min_posts_interval_ms,
            "download": self.download.as_ref().map(|d| json!({
                "address": d.address,
                "port": d.port,
                "serverName": d.server_name,
                "path": d.path,
                "host": d.host,
            })),
        })
    }
}

struct ParsedXhttp {
    head: String,
    params: Vec<String>,
    fragment: String,
    extra: Map<String, Value>,
    current: XhttpFields,
}

fn parse_xhttp(link: &str) -> Option<ParsedXhttp> {
    let trimmed = js_trim(link);
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("vless://") || lower.starts_with("trojan://")) {
        return None;
    }
    let (before_frag, fragment) = match trimmed.find('#') {
        Some(h) => (&trimmed[..h], trimmed[h..].to_string()),
        None => (trimmed, String::new()),
    };
    let (head, query_str) = match before_frag.find('?') {
        Some(q) => (before_frag[..q].to_string(), &before_frag[q + 1..]),
        None => (before_frag.to_string(), ""),
    };
    let params: Vec<String> = query_str
        .split('&')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let mut query: HashMap<String, String> = HashMap::new();
    for param in &params {
        let raw_key = param.split('=').next().unwrap_or("");
        let key = decode_param(raw_key).ok()?;
        if !matches!(
            key.as_str(),
            "type" | "network" | "mode" | "path" | "host" | "extra"
        ) {
            continue;
        }
        if query.contains_key(&key) {
            return None;
        }
        let raw_val = match param.find('=') {
            Some(eq) => &param[eq + 1..],
            None => "",
        };
        query.insert(key, decode_param(raw_val).ok()?);
    }
    let transport = query
        .get("type")
        .or_else(|| query.get("network"))
        .map(String::as_str);
    if !matches!(transport, Some("xhttp" | "splithttp")) {
        return None;
    }
    let extra_val: Value = match query.get("extra") {
        None => Value::Object(Map::new()),
        Some(s) => serde_json::from_str(s).ok()?,
    };
    let extra = extra_val.as_object()?.clone();
    let headers_obj = match extra.get("headers") {
        None => Map::new(),
        Some(v) if js::is_undefined(v) => Map::new(),
        Some(Value::Object(m)) => m.clone(),
        _ => return None,
    };
    let mut headers = Vec::with_capacity(headers_obj.len());
    for (k, v) in headers_obj {
        headers.push((k, v.as_str()?.to_string()));
    }
    let download = match extra.get("downloadSettings") {
        None => None,
        Some(v) if js::is_undefined(v) => None,
        Some(Value::Object(dl)) => {
            let member = |group: &str, name: &str| -> Option<String> {
                let gv = dl.get(group);
                match gv {
                    None => Some(String::new()),
                    Some(v) if js::is_undefined(v) => Some(String::new()),
                    Some(Value::Object(go)) => xhttp_text(go.get(name)),
                    _ => None,
                }
            };
            Some(DownloadFields {
                address: xhttp_text(dl.get("address"))?,
                port: xhttp_text(dl.get("port"))?,
                server_name: member("tlsSettings", "serverName")?,
                path: member("xhttpSettings", "path")?,
                host: member("xhttpSettings", "host")?,
            })
        }
        _ => return None,
    };
    let current = XhttpFields {
        mode: match query.get("mode") {
            Some(s) => s.clone(),
            None => xhttp_text(extra.get("mode"))?,
        },
        path: match query.get("path") {
            Some(s) => s.clone(),
            None => xhttp_text(extra.get("path"))?,
        },
        host: match query.get("host") {
            Some(s) => s.clone(),
            None => xhttp_text(extra.get("host"))?,
        },
        headers,
        x_padding_bytes: range_text(extra.get("xPaddingBytes"))?,
        no_grpc_header: extra.get("noGRPCHeader") == Some(&Value::Bool(true)),
        sc_max_each_post_bytes: range_text(extra.get("scMaxEachPostBytes"))?,
        sc_min_posts_interval_ms: range_text(extra.get("scMinPostsIntervalMs"))?,
        download,
    };
    Some(ParsedXhttp {
        head,
        params,
        fragment,
        extra,
        current,
    })
}

pub fn read_xhttp(link: String) -> Result<Option<Value>, String> {
    Ok(parse_xhttp(&link).map(|p| p.current.to_value()))
}

fn duplicate_xhttp_headers_vec(headers: &Value) -> Result<Vec<usize>, String> {
    if headers.is_null() || js::is_undefined(headers) {
        return Err("TypeError: Cannot read properties of null (reading 'forEach')".to_string());
    }
    let Some(arr) = headers.as_array() else {
        return Err("TypeError: headers.forEach is not a function".to_string());
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut duplicates: Vec<usize> = Vec::new();
    for (index, item) in arr.iter().enumerate() {
        let name: String = match item {
            Value::Array(pair) => pair
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    "TypeError: Cannot read properties of undefined (reading 'trim')".to_string()
                })?
                .to_string(),
            Value::String(s) => s
                .chars()
                .next()
                .ok_or_else(|| {
                    "TypeError: Cannot read properties of undefined (reading 'trim')".to_string()
                })?
                .to_string(),
            _ => {
                return Err(format!(
                    "TypeError: {} is not iterable (cannot read property Symbol(Symbol.iterator))",
                    item
                ));
            }
        };
        let key = js_trim(&name).to_lowercase();
        if key.is_empty() {
            continue;
        }
        if !seen.insert(key) {
            duplicates.push(index);
        }
    }
    Ok(duplicates)
}

pub fn duplicate_xhttp_headers(headers: Value) -> Result<Value, String> {
    let dups = duplicate_xhttp_headers_vec(&headers)?;
    Ok(json!({ "$set": dups }))
}

pub fn write_xhttp(link: String, change: Value) -> Result<String, String> {
    let Some(mut parsed) = parse_xhttp(&link) else {
        return Ok(link);
    };
    let Some(co) = change.as_object() else {
        return Err("TypeError: Cannot read properties of null (reading 'headers')".to_string());
    };
    if let Some(hv) = obj_defined(co, "headers").filter(|v| crate::port::js_str::is_truthy(v))
        && !duplicate_xhttp_headers_vec(hv)?.is_empty()
    {
        return Ok(link);
    }
    let before = serde_json::to_string(&Value::Object(parsed.extra.clone())).unwrap_or_default();
    let mut changed = false;
    let mut set_param = |key: &str, value: &str| {
        let at = parsed.params.iter().position(|param| {
            decode_param(param.split('=').next().unwrap_or(""))
                .ok()
                .as_deref()
                == Some(key)
        });
        let enc = encode_uri_component(value).replace('\'', "%27");
        let param = format!("{key}={enc}");
        if value.is_empty() {
            let Some(i) = at else {
                return;
            };
            parsed.params.remove(i);
        } else if let Some(i) = at {
            parsed.params[i] = param;
        } else {
            parsed.params.push(param);
        }
        changed = true;
    };

    for (key, cur) in [
        ("mode", parsed.current.mode.as_str()),
        ("path", parsed.current.path.as_str()),
        ("host", parsed.current.host.as_str()),
    ] {
        if let Some(val_v) = obj_defined(co, key) {
            let val = val_v.as_str().unwrap_or("");
            if val != cur {
                set_param(key, val);
                parsed.extra.remove(key);
            }
        }
    }
    for (key, cur) in [
        ("xPaddingBytes", parsed.current.x_padding_bytes.as_str()),
        (
            "scMaxEachPostBytes",
            parsed.current.sc_max_each_post_bytes.as_str(),
        ),
        (
            "scMinPostsIntervalMs",
            parsed.current.sc_min_posts_interval_ms.as_str(),
        ),
    ] {
        if let Some(val_v) = obj_defined(co, key) {
            let val = val_v.as_str().unwrap_or("");
            if val != cur {
                put_json(&mut parsed.extra, key, scalar(js_trim(val)));
            }
        }
    }
    if let Some(v) = obj_defined(co, "noGRPCHeader") {
        let b = v.as_bool().unwrap_or(false);
        if b != parsed.current.no_grpc_header {
            put_json(&mut parsed.extra, "noGRPCHeader", Value::Bool(b));
        }
    }
    if let Some(hv) = obj_defined(co, "headers").filter(|v| crate::port::js_str::is_truthy(v))
        && let Some(arr) = hv.as_array()
    {
        let headers: Vec<(String, String)> = arr
            .iter()
            .filter_map(|row| {
                let p = row.as_array()?;
                let n = js_trim(p.first()?.as_str()?).to_string();
                let v = p.get(1)?.as_str()?.to_string();
                (!n.is_empty()).then_some((n, v))
            })
            .collect();
        if headers != parsed.current.headers {
            let mut hm = Map::new();
            for (k, v) in headers {
                hm.insert(k, Value::String(v));
            }
            put_json(&mut parsed.extra, "headers", Value::Object(hm));
        }
    }
    if let Some(dl_v) = obj_defined(co, "download") {
        if dl_v.is_null() {
            parsed.extra.remove("downloadSettings");
        } else if let Some(dlo) = dl_v.as_object() {
            let mut settings = match parsed.extra.get("downloadSettings") {
                Some(Value::Object(m)) => m.clone(),
                _ => {
                    let mut m = Map::new();
                    m.insert("network".to_string(), Value::String("xhttp".to_string()));
                    m.insert("security".to_string(), Value::String("tls".to_string()));
                    m
                }
            };
            let cur_dl = parsed.current.download.as_ref();
            for (key, cur_val) in [
                ("address", cur_dl.map(|d| d.address.as_str())),
                ("port", cur_dl.map(|d| d.port.as_str())),
            ] {
                if let Some(v) = obj_defined(dlo, key) {
                    let s = v.as_str().unwrap_or("");
                    if cur_val != Some(s) {
                        let written = if key == "port" {
                            scalar(s)
                        } else {
                            Value::String(s.to_string())
                        };
                        put_json(&mut settings, key, written);
                    }
                }
            }
            for (key, group, cur_val) in [
                (
                    "serverName",
                    "tlsSettings",
                    cur_dl.map(|d| d.server_name.as_str()),
                ),
                ("path", "xhttpSettings", cur_dl.map(|d| d.path.as_str())),
                ("host", "xhttpSettings", cur_dl.map(|d| d.host.as_str())),
            ] {
                if let Some(v) = obj_defined(dlo, key) {
                    let s = v.as_str().unwrap_or("");
                    if cur_val == Some(s) {
                        continue;
                    }
                    let mut nested = match settings.get(group) {
                        Some(Value::Object(m)) => m.clone(),
                        _ => Map::new(),
                    };
                    put_json(&mut nested, key, Value::String(s.to_string()));
                    put_json(&mut settings, group, Value::Object(nested));
                }
            }
            parsed
                .extra
                .insert("downloadSettings".to_string(), Value::Object(settings));
        }
    }

    let serialized =
        serde_json::to_string(&Value::Object(parsed.extra.clone())).unwrap_or_default();
    if serialized != before {
        if parsed.extra.is_empty() {
            set_param("extra", "");
        } else {
            set_param("extra", &serialized);
        }
    }

    if changed {
        let q = if parsed.params.is_empty() {
            String::new()
        } else {
            format!("?{}", parsed.params.join("&"))
        };
        Ok(format!("{}{q}{}", parsed.head, parsed.fragment))
    } else {
        Ok(link)
    }
}
