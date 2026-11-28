//! Port of `src/dae/vocab.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::port::js_str::js_trim;
use serde_json::Value;

pub fn policy_kind_str(native: &str) -> Option<&'static str> {
    let first = native.split('(').next().unwrap_or("");
    let base = js_trim(first).to_ascii_lowercase();
    match base.as_str() {
        "select" | "selector" | "fixed" | "min" | "random" => Some("selector"),
        "urltest" | "min_moving_avg" | "min_avg10" | "min_last_delay" => Some("urltest"),
        "roundrobin" | "round_robin" | "loadbalance" | "balance" => Some("loadbalance"),
        "fallback" => Some("fallback"),
        "score" => Some("score"),
        _ => None,
    }
}

pub fn policy_kind(native: String) -> Result<Option<Value>, String> {
    match policy_kind_str(&native) {
        Some(k) => Ok(Some(Value::String(k.to_string()))),
        None => Ok(Some(js::undefined())),
    }
}

pub fn is_builtin_outbound_str(name: Option<&str>) -> bool {
    matches!(name, Some("direct" | "block"))
}

pub fn is_builtin_outbound(name: Option<String>) -> Result<bool, String> {
    Ok(is_builtin_outbound_str(name.as_deref()))
}
