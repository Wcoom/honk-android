//! Endpoint & secret discovery for Honk Native API and Clash API (`cr0.java` parity + ADR-009).

use boxproxy_schema::CoreKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKind {
    HonkNative,
    ClashApi,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub kind: ApiKind,
    pub base_url: String,
    pub secret: String,
    pub clash_fallback_url: Option<String>,
}

pub fn discover_endpoint(
    core: CoreKind,
    box_home: &Path,
    custom_enabled: bool,
    custom_endpoint: &str,
    custom_secret: &str,
) -> ApiEndpoint {
    if custom_enabled && !custom_endpoint.trim().is_empty() {
        let base = normalize_base_url(custom_endpoint);
        let kind = if core.prefers_honk_native_api() && base.contains(":9527") {
            ApiKind::HonkNative
        } else {
            ApiKind::ClashApi
        };
        return ApiEndpoint {
            kind,
            base_url: base,
            secret: custom_secret.trim().to_string(),
            clash_fallback_url: None,
        };
    }

    let startup_cfg = box_home.join("run/state/config/startup-config");
    let raw = fs::read_to_string(&startup_cfg).unwrap_or_default();

    match core {
        CoreKind::Honk => {
            let (clash_url, clash_secret) = parse_yaml_or_dae_controller(&raw);
            let native_url = parse_honk_native_listen(&raw)
                .unwrap_or_else(|| "http://127.0.0.1:9527".to_string());
            ApiEndpoint {
                kind: ApiKind::HonkNative,
                base_url: native_url,
                secret: clash_secret,
                clash_fallback_url: Some(
                    clash_url.unwrap_or_else(|| "http://127.0.0.1:9090".to_string()),
                ),
            }
        }
        CoreKind::Mihomo | CoreKind::Meow => {
            let (url, secret) = parse_yaml_or_dae_controller(&raw);
            ApiEndpoint {
                kind: ApiKind::ClashApi,
                base_url: url.unwrap_or_else(|| "http://127.0.0.1:9090".to_string()),
                secret,
                clash_fallback_url: None,
            }
        }
        CoreKind::SingBox => {
            let (url, secret) = parse_singbox_controller(&raw);
            ApiEndpoint {
                kind: ApiKind::ClashApi,
                base_url: url.unwrap_or_else(|| "http://127.0.0.1:9090".to_string()),
                secret,
                clash_fallback_url: None,
            }
        }
        CoreKind::Xray | CoreKind::V2fly | CoreKind::Hysteria => ApiEndpoint {
            kind: ApiKind::Unsupported,
            base_url: String::new(),
            secret: String::new(),
            clash_fallback_url: None,
        },
    }
}

pub fn normalize_base_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if let Some(port) = trimmed.strip_prefix(':') {
        format!("http://127.0.0.1:{port}")
    } else if let Some(rest) = trimmed.strip_prefix("0.0.0.0:") {
        format!("http://127.0.0.1:{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("[::]:") {
        format!("http://127.0.0.1:{rest}")
    } else {
        format!("http://{trimmed}")
    }
}

fn parse_yaml_or_dae_controller(raw: &str) -> (Option<String>, String) {
    let mut url = None;
    let mut secret = String::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let val = v
                .split('#')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(['"', '\''])
                .to_string();
            if (key == "external-controller" || key == "external_controller") && !val.is_empty() {
                url = Some(normalize_base_url(&val));
            } else if key == "secret" && secret.is_empty() {
                secret = val;
            }
        }
    }
    (url, secret)
}

fn parse_honk_native_listen(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("native_api") || trimmed.starts_with("listen:") {
            if let Some((_, rhs)) = trimmed.split_once(':') {
                let val = rhs
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches(['"', '\'']);
                if val.contains(':') && val.contains("9527") {
                    return Some(normalize_base_url(val));
                }
            }
        }
    }
    None
}

fn parse_singbox_controller(raw: &str) -> (Option<String>, String) {
    let Ok(val) = jsonc_parser::parse_to_serde_value::<serde_json::Value>(raw, &Default::default()) else {
        return (None, String::new());
    };
    let clash_api = val.get("experimental").and_then(|e| e.get("clash_api"));
    let url = clash_api
        .and_then(|c| c.get("external_controller"))
        .and_then(|v| v.as_str())
        .map(normalize_base_url);
    let secret = clash_api
        .and_then(|c| c.get("secret"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (url, secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_honk_native_api_for_honk_core() {
        let ep = discover_endpoint(CoreKind::Honk, Path::new("/nonexistent"), false, "", "");
        assert_eq!(ep.kind, ApiKind::HonkNative);
        assert_eq!(ep.base_url, "http://127.0.0.1:9527");
        assert_eq!(ep.clash_fallback_url.as_deref(), Some("http://127.0.0.1:9090"));
    }

    #[test]
    fn normalizes_wildcard_controllers_to_loopback() {
        assert_eq!(normalize_base_url("0.0.0.0:9090"), "http://127.0.0.1:9090");
        assert_eq!(normalize_base_url(":9527"), "http://127.0.0.1:9527");
    }
}
