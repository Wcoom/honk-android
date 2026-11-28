//! M25 Smart Select app classifier & M34 JSON5/YAML/DAE/CIDR/Sing-Box config validator.

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledAppInfo {
    pub uid: u32,
    pub package_name: String,
    pub label: String,
    pub is_system: bool,
    pub has_internet_permission: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmartSelectMode {
    CnCommonApps,
    SystemApps,
    NoInternetApps,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationDiagnostic {
    pub severity: String,
    pub line: usize,
    pub message: String,
}

const CN_APP_PREFIXES: &[&str] = &[
    "com.tencent.",
    "com.eg.android.AlipayGphone",
    "com.taobao.",
    "com.tmall.",
    "com.jingdong.",
    "com.xunmeng.pinduoduo",
    "com.ss.android.ugc.aweme",
    "com.baidu.",
    "com.sina.weibo",
    "tv.danmaku.bili",
    "com.netease.",
    "com.sankuai.meituan",
    "com.xingin.xhs",
    "com.autonavi.minimap",
    "com.unionpay",
    "com.icbc",
    "com.chinamworld.",
];

/// M25 Smart Select classifier returning matching UIDs.
pub fn classify_smart_select(apps: &[InstalledAppInfo], mode: SmartSelectMode) -> Vec<u32> {
    let mut uids: Vec<u32> = apps
        .iter()
        .filter(|app| match mode {
            SmartSelectMode::CnCommonApps => CN_APP_PREFIXES
                .iter()
                .any(|p| app.package_name.starts_with(p)),
            SmartSelectMode::SystemApps => app.is_system,
            SmartSelectMode::NoInternetApps => !app.has_internet_permission,
        })
        .map(|a| a.uid)
        .collect();
    uids.sort_unstable();
    uids.dedup();
    uids
}

pub fn validate_cidr_v4(raw: &str) -> Result<(), String> {
    let (ip, prefix) = raw
        .trim()
        .split_once('/')
        .ok_or_else(|| format!("missing /prefix in IPv4 CIDR: {raw}"))?;
    ip.parse::<Ipv4Addr>()
        .map_err(|_| format!("invalid IPv4 address: {ip}"))?;
    let p: u8 = prefix
        .parse()
        .map_err(|_| format!("invalid IPv4 prefix: {prefix}"))?;
    if p > 32 {
        return Err(format!("IPv4 prefix out of range (0..=32): {p}"));
    }
    Ok(())
}

pub fn validate_cidr_v6(raw: &str) -> Result<(), String> {
    let (ip, prefix) = raw
        .trim()
        .split_once('/')
        .ok_or_else(|| format!("missing /prefix in IPv6 CIDR: {raw}"))?;
    ip.parse::<Ipv6Addr>()
        .map_err(|_| format!("invalid IPv6 address: {ip}"))?;
    let p: u8 = prefix
        .parse()
        .map_err(|_| format!("invalid IPv6 prefix: {prefix}"))?;
    if p > 128 {
        return Err(format!("IPv6 prefix out of range (0..=128): {p}"));
    }
    Ok(())
}

/// M34 Syntax & structural diagnostic validator for `.json`/`.jsonc`/`.yaml`/`.yml`/`.dae`.
pub fn validate_config_text(filename: &str, text: &str) -> Vec<ValidationDiagnostic> {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".json") || lower.ends_with(".jsonc") {
        match jsonc_parser::parse_to_serde_value::<serde_json::Value>(text, &Default::default()) {
            Ok(val) => {
                let mut diags = Vec::new();
                if (lower.contains("sing") || val.get("inbounds").is_some())
                    && val.get("outbounds").is_none()
                {
                    diags.push(ValidationDiagnostic {
                        severity: "warn".into(),
                        line: 1,
                        message: "sing-box config missing top-level `outbounds`".into(),
                    });
                }
                diags
            }
            Err(e) => vec![ValidationDiagnostic {
                severity: "error".into(),
                line: 1,
                message: format!("JSON/JSONC parse error: {e}"),
            }],
        }
    } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
        match serde_norway::from_str::<serde_norway::Value>(text) {
            Ok(_) => Vec::new(),
            Err(e) => vec![ValidationDiagnostic {
                severity: "error".into(),
                line: e.location().map(|l| l.line()).unwrap_or(1),
                message: format!("YAML parse error: {e}"),
            }],
        }
    } else if lower.ends_with(".dae") {
        validate_dae_braces(text)
    } else {
        Vec::new()
    }
}

fn validate_dae_braces(text: &str) -> Vec<ValidationDiagnostic> {
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    for (idx, line) in text.lines().enumerate() {
        let mut prev_escape = false;
        for ch in line.chars() {
            if !in_single && !in_double && ch == '#' {
                break;
            }
            if ch == '\'' && !in_double && !prev_escape {
                in_single = !in_single;
            } else if ch == '"' && !in_single && !prev_escape {
                in_double = !in_double;
            } else if !in_single && !in_double {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth < 0 {
                        return vec![ValidationDiagnostic {
                            severity: "error".into(),
                            line: idx + 1,
                            message: "unexpected closing brace `}`".into(),
                        }];
                    }
                }
            }
            prev_escape = ch == '\\' && !prev_escape;
        }
    }
    if depth != 0 {
        vec![ValidationDiagnostic {
            severity: "error".into(),
            line: text.lines().count().max(1),
            message: format!("unclosed block: missing {depth} closing brace(s) `}}`"),
        }]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_cidrs_configs_and_smart_select() {
        assert!(validate_cidr_v4("10.0.0.0/8").is_ok());
        assert!(validate_cidr_v4("10.0.0.0/33").is_err());
        assert!(validate_cidr_v6("fc00::/7").is_ok());
        assert!(validate_config_text("config.dae", "global { log_level: info }").is_empty());
        assert!(!validate_config_text("config.dae", "global { ").is_empty());
        assert!(validate_config_text("config.yaml", "mixed-port: 7890\n").is_empty());

        let apps = vec![
            InstalledAppInfo {
                uid: 10001,
                package_name: "com.tencent.mm".into(),
                label: "WeChat".into(),
                is_system: false,
                has_internet_permission: true,
            },
            InstalledAppInfo {
                uid: 10002,
                package_name: "org.example.offline".into(),
                label: "Offline".into(),
                is_system: false,
                has_internet_permission: false,
            },
        ];
        assert_eq!(classify_smart_select(&apps, SmartSelectMode::CnCommonApps), vec![10001]);
        assert_eq!(classify_smart_select(&apps, SmartSelectMode::NoInternetApps), vec![10002]);
    }
}
