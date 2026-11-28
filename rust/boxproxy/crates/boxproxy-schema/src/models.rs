//! Strongly-typed domain models for all 21 `box.db` v15 tables and network rule AST.

use serde::{Deserialize, Serialize};
use std::fmt;

/// All 7 supported proxy cores in `com.boxproxy.honk` v214 & `io.honk.android`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreKind {
    #[default]
    Honk,
    Mihomo,
    Meow,
    SingBox,
    Xray,
    V2fly,
    Hysteria,
}

impl CoreKind {
    pub const ALL: &'static [CoreKind] = &[
        Self::Honk,
        Self::Mihomo,
        Self::Meow,
        Self::SingBox,
        Self::Xray,
        Self::V2fly,
        Self::Hysteria,
    ];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "honk" => Ok(Self::Honk),
            "mihomo" => Ok(Self::Mihomo),
            "meow" => Ok(Self::Meow),
            "sing-box" | "sing_box" | "singbox" => Ok(Self::SingBox),
            "xray" => Ok(Self::Xray),
            "v2fly" => Ok(Self::V2fly),
            "hysteria" => Ok(Self::Hysteria),
            other => Err(format!("unknown core: {other}")),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Honk => "honk",
            Self::Mihomo => "mihomo",
            Self::Meow => "meow",
            Self::SingBox => "sing-box",
            Self::Xray => "xray",
            Self::V2fly => "v2fly",
            Self::Hysteria => "hysteria",
        }
    }

    pub const fn config_extensions(self) -> &'static [&'static str] {
        match self {
            Self::Honk => &[".dae"],
            Self::Mihomo | Self::Meow | Self::Hysteria => &[".yaml", ".yml"],
            Self::SingBox => &[".json", ".jsonc", ".yaml", ".yml"],
            Self::Xray | Self::V2fly => &[".json"],
        }
    }

    pub const fn supports_ebpf(self) -> bool {
        matches!(self, Self::Honk | Self::Mihomo | Self::Meow | Self::SingBox)
    }

    pub const fn uses_runtime_startup_config(self) -> bool {
        matches!(self, Self::Mihomo | Self::Meow | Self::SingBox)
    }

    pub const fn supports_config_override(self) -> bool {
        matches!(self, Self::Honk | Self::Mihomo | Self::Meow | Self::SingBox)
    }

    pub const fn supports_js_scripts(self) -> bool {
        matches!(self, Self::Mihomo | Self::Meow | Self::SingBox)
    }

    pub const fn supports_clash_api(self) -> bool {
        matches!(self, Self::Honk | Self::Mihomo | Self::Meow | Self::SingBox)
    }

    pub const fn prefers_honk_native_api(self) -> bool {
        matches!(self, Self::Honk)
    }

    pub const fn allow_script_only_start(self) -> bool {
        matches!(self, Self::Mihomo | Self::Meow | Self::SingBox)
    }
}

impl fmt::Display for CoreKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    Tun,
    Tproxy,
    #[default]
    Ebpf,
    Redirect,
    Mixed,
    Enhance,
}

impl NetworkMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tun" => Ok(Self::Tun),
            "tproxy" => Ok(Self::Tproxy),
            "ebpf" => Ok(Self::Ebpf),
            "redirect" => Ok(Self::Redirect),
            "mixed" => Ok(Self::Mixed),
            "enhance" => Ok(Self::Enhance),
            other => Err(format!("invalid network mode: {other}")),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tun => "tun",
            Self::Tproxy => "tproxy",
            Self::Ebpf => "ebpf",
            Self::Redirect => "redirect",
            Self::Mixed => "mixed",
            Self::Enhance => "enhance",
        }
    }

    pub const fn uses_tun(self) -> bool {
        matches!(self, Self::Tun | Self::Mixed)
    }
}

impl fmt::Display for NetworkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    #[default]
    Core,
    Blacklist,
    Whitelist,
}

impl ProxyMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "core" => Ok(Self::Core),
            "blacklist" | "black" => Ok(Self::Blacklist),
            "whitelist" | "white" => Ok(Self::Whitelist),
            other => Err(format!("invalid proxy mode: {other}")),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Blacklist => "blacklist",
            Self::Whitelist => "whitelist",
        }
    }
}

impl fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ipv6Mode {
    #[default]
    Enable,
    Bypass,
    Disable,
}

impl Ipv6Mode {
    pub fn from_storage(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "enable" | "enabled" | "true" | "1" => Self::Enable,
            "disable" | "disabled" | "system_disable" | "off" => Self::Disable,
            "bypass" | "bypassed" | "false" | "0" => Self::Bypass,
            _ => Self::Enable,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enable => "enable",
            Self::Bypass => "bypass",
            Self::Disable => "disable",
        }
    }
}

impl fmt::Display for Ipv6Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CnipMode {
    #[default]
    Ipset,
    Ebpf,
}

impl CnipMode {
    pub fn from_storage(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("ebpf") {
            Self::Ebpf
        } else {
            Self::Ipset
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ipset => "ipset",
            Self::Ebpf => "ebpf",
        }
    }
}

impl fmt::Display for CnipMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Table 1: `runtime_profile` (exact columns matching `box.db` v15 on OnePlus 13).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfile {
    pub core_name: String,
    pub mode: String,
    pub proxy_mode: String,
    pub auto_sync_config: bool,
    pub performance_mode: bool,
    pub clean_vendor_firewall: bool,
    pub ipv6_mode: String,
    pub config_name: String,
    pub quic: String,
    pub mihomo_dns_forward: String,
    pub proxy_tcp: bool,
    pub proxy_udp: bool,
    pub dns_hijack_tcp: bool,
    pub dns_hijack_udp: bool,
    pub dns_hijack_mode: String,
    pub cgroup_memcg: bool,
    pub memcg_limit: String,
    pub taskset_cpu: bool,
    pub allow_cpu: String,
    pub cgroup_blkio: bool,
    pub weight: String,
    pub bypass_cn: bool,
    pub boot_auto_start: bool,
    /// Optional legacy / override columns (preserved when reading older v1-v3 DBs or `app_settings`).
    #[serde(default)]
    pub tproxy_port: String,
    #[serde(default)]
    pub redir_port: String,
    #[serde(default)]
    pub mihomo_dns_port: String,
    #[serde(default)]
    pub tun_device: String,
    #[serde(default)]
    pub fake_ip_range: String,
    #[serde(default)]
    pub fake_ip6_range: String,
}

impl Default for RuntimeProfile {
    fn default() -> Self {
        Self {
            core_name: "honk".to_string(),
            mode: "ebpf".to_string(),
            proxy_mode: "core".to_string(),
            auto_sync_config: true,
            performance_mode: false,
            clean_vendor_firewall: false,
            ipv6_mode: "enable".to_string(),
            config_name: "config.dae".to_string(),
            quic: "enable".to_string(),
            mihomo_dns_forward: "enable".to_string(),
            proxy_tcp: true,
            proxy_udp: true,
            dns_hijack_tcp: true,
            dns_hijack_udp: true,
            dns_hijack_mode: "tproxy".to_string(),
            cgroup_memcg: false,
            memcg_limit: "100M".to_string(),
            taskset_cpu: false,
            allow_cpu: "0-7".to_string(),
            cgroup_blkio: false,
            weight: String::new(),
            bypass_cn: false,
            boot_auto_start: false,
            tproxy_port: String::new(),
            redir_port: String::new(),
            mihomo_dns_port: String::new(),
            tun_device: String::new(),
            fake_ip_range: String::new(),
            fake_ip6_range: String::new(),
        }
    }
}

/// Table 8: `cnip_settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CnipSettings {
    pub bypass_cnip: bool,
    pub cnip_mode: String,
    pub bypass_ipv4: bool,
    pub bypass_ipv6: bool,
    pub ipv4_file: String,
    pub ipv4_url: String,
    pub ipv6_file: String,
    pub ipv6_url: String,
    pub ipv4_source: String,
    pub ipv6_source: String,
    pub ipv4_additions: String,
    pub ipv6_additions: String,
}

impl Default for CnipSettings {
    fn default() -> Self {
        Self {
            bypass_cnip: false,
            cnip_mode: "ipset".to_string(),
            bypass_ipv4: true,
            bypass_ipv6: false,
            ipv4_file: String::new(),
            ipv4_url: crate::defaults::DEFAULT_CNIP_IPV4_URL.to_string(),
            ipv6_file: String::new(),
            ipv6_url: crate::defaults::DEFAULT_CNIP_IPV6_URL.to_string(),
            ipv4_source: "url".to_string(),
            ipv6_source: "url".to_string(),
            ipv4_additions: String::new(),
            ipv6_additions: String::new(),
        }
    }
}

/// Table 14: `network_match_settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkMatchSettings {
    pub network_control_enabled: bool,
    pub use_on_wifi_disconnect: bool,
    pub use_on_wifi_connect: bool,
    pub network_enable_on_unmatched: bool,
    pub network_enable_on_matched: bool,
    pub matched_action: String,
    pub unmatched_action: String,
    pub enable_ssid_matching: bool,
    pub enable_network_control_log: bool,
    pub list_mode: String,
    pub switch_config: bool,
    pub matched_config_name: String,
    pub unmatched_config_name: String,
    pub matched_script_name: String,
    pub unmatched_script_name: String,
}

impl Default for NetworkMatchSettings {
    fn default() -> Self {
        Self {
            network_control_enabled: false,
            use_on_wifi_disconnect: true,
            use_on_wifi_connect: true,
            network_enable_on_unmatched: true,
            network_enable_on_matched: true,
            matched_action: "enable".to_string(),
            unmatched_action: "stop".to_string(),
            enable_ssid_matching: false,
            enable_network_control_log: true,
            list_mode: "blacklist".to_string(),
            switch_config: false,
            matched_config_name: String::new(),
            unmatched_config_name: String::new(),
            matched_script_name: String::new(),
            unmatched_script_name: String::new(),
        }
    }
}

/// Table 13: `network_match_conditions`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkMatchCondition {
    pub id: i64,
    #[serde(rename = "type")]
    pub condition_type: String,
    pub value: String,
    pub enabled: bool,
    pub sort_order: i64,
}

/// Table 17: `network_rule_groups`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRuleGroup {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub priority: i64,
    pub expression: String,
    #[serde(default)]
    pub actions: Vec<NetworkRuleAction>,
}

/// Table 16: `network_rule_actions`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRuleAction {
    pub id: i64,
    pub group_id: i64,
    pub sort_order: i64,
    #[serde(rename = "type")]
    pub action_type: String,
    pub target: String,
    pub continue_on_error: bool,
}

/// Table 21: `shared_network_settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedNetworkSettings {
    pub mac_filter: bool,
    pub mac_mode: String,
}

impl Default for SharedNetworkSettings {
    fn default() -> Self {
        Self {
            mac_filter: false,
            mac_mode: "blacklist".to_string(),
        }
    }
}

/// Snapshot of live network environment used to evaluate `network_rule_groups` AST.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkObservation {
    pub wifi_connected: bool,
    pub wifi_ssid: String,
    pub wifi_bssid: String,
    pub cellular_connected: bool,
    pub mcc_mnc: String,
    pub sim1_mcc_mnc: String,
    pub sim2_mcc_mnc: String,
    pub active_sim_slot: Option<u8>,
}

/// Boolean AST node stored as JSON in `network_rule_groups.expression`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRuleExpr {
    pub kind: String,
    #[serde(default)]
    pub op: String,
    #[serde(default)]
    pub children: Vec<NetworkRuleExpr>,
    #[serde(rename = "type", default)]
    pub cond_type: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
}

impl NetworkRuleExpr {
    /// Evaluate this AST node against a live `NetworkObservation`.
    /// Returns `None` if disabled or has no enabled children (neutral element).
    pub fn evaluate(&self, obs: &NetworkObservation) -> Option<bool> {
        if self.disabled {
            return None;
        }
        match self.kind.trim().to_ascii_lowercase().as_str() {
            "leaf" => Some(eval_leaf(&self.cond_type, &self.value, obs)),
            "op" => {
                let evaluated: Vec<bool> = self
                    .children
                    .iter()
                    .filter_map(|child| child.evaluate(obs))
                    .collect();
                if evaluated.is_empty() {
                    return None;
                }
                match self.op.trim().to_ascii_lowercase().as_str() {
                    "and" => Some(evaluated.into_iter().all(|v| v)),
                    "or" => Some(evaluated.into_iter().any(|v| v)),
                    "not" => Some(!evaluated[0]),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

fn eval_leaf(cond_type: &str, expected: &str, obs: &NetworkObservation) -> bool {
    let expected_trim = expected.trim();
    match cond_type.trim().to_ascii_lowercase().as_str() {
        "wifi" => obs.wifi_connected,
        "wifi_ssid" => {
            obs.wifi_connected
                && !expected_trim.is_empty()
                && obs.wifi_ssid.trim() == expected_trim
        }
        "wifi_bssid" => {
            obs.wifi_connected
                && !expected_trim.is_empty()
                && obs.wifi_bssid.trim().eq_ignore_ascii_case(expected_trim)
        }
        "cellular" => obs.cellular_connected,
        "mcc_mnc" => {
            obs.cellular_connected
                && !expected_trim.is_empty()
                && (obs.mcc_mnc.trim() == expected_trim
                    || obs.sim1_mcc_mnc.trim() == expected_trim
                    || obs.sim2_mcc_mnc.trim() == expected_trim)
        }
        "sim1" => {
            obs.cellular_connected
                && obs.active_sim_slot == Some(1)
                && (expected_trim.is_empty() || obs.sim1_mcc_mnc.trim() == expected_trim)
        }
        "sim2" => {
            obs.cellular_connected
                && obs.active_sim_slot == Some(2)
                && (expected_trim.is_empty() || obs.sim2_mcc_mnc.trim() == expected_trim)
        }
        _ => false,
    }
}

/// Evaluate priority-sorted `network_rule_groups` and return the first matching group.
pub fn match_first_rule_group<'a>(
    groups: &'a [NetworkRuleGroup],
    obs: &NetworkObservation,
) -> Option<&'a NetworkRuleGroup> {
    let mut sorted: Vec<&NetworkRuleGroup> = groups.iter().filter(|g| g.enabled).collect();
    sorted.sort_by_key(|g| (g.priority, g.id));
    for group in sorted {
        let expr_str = group.expression.trim();
        if expr_str.is_empty() {
            continue;
        }
        if let Ok(ast) = serde_json::from_str::<NetworkRuleExpr>(expr_str) {
            if ast.evaluate(obs) == Some(true) {
                return Some(group);
            }
        }
    }
    None
}
