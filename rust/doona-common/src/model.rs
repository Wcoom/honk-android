//! Typed Rust models matching `honk-core` Native API (`/api/v1/*`) and extensions (`/api/v1/x-honk/*`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn value_u64(v: &Option<Value>) -> Option<u64> {
    match v.as_ref()? {
        Value::Number(n) => n.as_u64().or_else(|| n.as_f64().map(|f| f.max(0.0) as u64)),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

pub fn value_f64(v: &Option<Value>) -> Option<f64> {
    match v.as_ref()? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody,
    pub request_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiscoveryInfo {
    pub name: Option<String>,
    pub status: Option<String>,
    pub api_major: Option<u64>,
    pub base_path: Option<String>,
    pub auth: Option<AuthDiscovery>,
    pub links: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthDiscovery {
    pub mode: Option<String>,
    #[serde(default)]
    pub setup_required: bool,
    #[serde(default)]
    pub anonymous_loopback: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthSession {
    pub token: String,
    pub expires_at: Option<String>,
    pub username: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VersionInfo {
    pub api: Option<ApiVersion>,
    pub build: Option<BuildInfo>,
    pub engine: Option<EngineInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: Option<u64>,
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildInfo {
    pub built_at: Option<String>,
    pub revision: Option<String>,
    pub target: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub observed_at: Option<String>,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default)]
    pub limits: Value,
    #[serde(default)]
    pub resources: Value,
    #[serde(default)]
    pub extensions: Value,
}

impl Capabilities {
    pub fn resource_available(&self, key: &str) -> bool {
        self.resources
            .get(key)
            .and_then(|v| v.get("available"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn core_update_available(&self) -> bool {
        self.resources
            .get("x-honk")
            .or_else(|| self.extensions.get("x-honk"))
            .and_then(|v| v.get("core_update"))
            .and_then(|v| v.get("available"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn core_update_can_apply(&self) -> bool {
        self.resources
            .get("x-honk")
            .or_else(|| self.extensions.get("x-honk"))
            .and_then(|v| v.get("core_update"))
            .and_then(|v| v.get("can_apply"))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn can_select_groups(&self) -> bool {
        self.resources
            .get("groups")
            .and_then(|v| v.get("selection"))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }

    pub fn can_write_config(&self) -> bool {
        self.resources
            .get("config")
            .and_then(|v| v.get("writable").or_else(|| v.get("write")))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn memory_limit_reported(&self) -> bool {
        self.resources
            .get("runtime_memory")
            .and_then(|v| v.get("metrics"))
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|m| m.as_str() == Some("cgroup.limit_bytes")))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeLifecycle {
    pub state: Option<String>,
    pub started_at: Option<String>,
    pub uptime_seconds: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeGeneration {
    pub active_id: Option<String>,
    pub config_revision: Option<String>,
    pub state: Option<String>,
    pub activated_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeProcess {
    pub pid: Option<u32>,
    pub cpu_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrafficConnections {
    pub tcp: Option<u64>,
    pub udp: Option<u64>,
    pub total: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrafficBytes {
    pub upload: Option<Value>,
    pub download: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrafficRates {
    pub window_seconds: Option<f64>,
    pub upload_bytes_per_second: Option<Value>,
    pub download_bytes_per_second: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeTraffic {
    pub scope: Option<String>,
    pub observed_by: Option<String>,
    pub counter_since: Option<String>,
    pub sampled_at: Option<String>,
    #[serde(default)]
    pub connections: TrafficConnections,
    #[serde(default)]
    pub bytes: TrafficBytes,
    pub rates: Option<TrafficRates>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LastReload {
    pub operation_id: Option<String>,
    pub status: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<ApiErrorBody>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Degradation {
    pub code: Option<String>,
    pub message: Option<String>,
    pub component: Option<String>,
    pub since: Option<String>,
    pub details: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub observed_at: Option<String>,
    pub instance_id: Option<String>,
    #[serde(default)]
    pub lifecycle: RuntimeLifecycle,
    #[serde(default)]
    pub generation: RuntimeGeneration,
    pub datapath: Option<DatapathStatus>,
    #[serde(default)]
    pub traffic: RuntimeTraffic,
    #[serde(default)]
    pub process: RuntimeProcess,
    pub last_reload: Option<LastReload>,
    #[serde(default)]
    pub degradations: Vec<Degradation>,
}

impl RuntimeSnapshot {
    pub fn state_str(&self) -> &str {
        self.lifecycle.state.as_deref().unwrap_or("unknown")
    }

    pub fn uptime_secs(&self) -> Option<u64> {
        value_u64(&self.lifecycle.uptime_seconds)
    }

    pub fn download_rate(&self) -> f64 {
        self.traffic
            .rates
            .as_ref()
            .and_then(|r| value_f64(&r.download_bytes_per_second))
            .unwrap_or(0.0)
    }

    pub fn upload_rate(&self) -> f64 {
        self.traffic
            .rates
            .as_ref()
            .and_then(|r| value_f64(&r.upload_bytes_per_second))
            .unwrap_or(0.0)
    }

    pub fn total_download_bytes(&self) -> Option<u64> {
        value_u64(&self.traffic.bytes.download)
    }

    pub fn total_upload_bytes(&self) -> Option<u64> {
        value_u64(&self.traffic.bytes.upload)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrafficHistory {
    pub observed_at: Option<String>,
    pub window_seconds: Option<u64>,
    pub sampled_every_seconds: Option<u64>,
    #[serde(default)]
    pub samples: Vec<TrafficPoint>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrafficPoint {
    pub sampled_at: Option<String>,
    pub upload_bytes_per_second: Option<Value>,
    pub download_bytes_per_second: Option<Value>,
    pub connections: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryHistory {
    pub observed_at: Option<String>,
    pub window_seconds: Option<u64>,
    pub sampled_every_seconds: Option<u64>,
    #[serde(default)]
    pub samples: Vec<MemoryPoint>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryPoint {
    pub sampled_at: Option<String>,
    pub rss_bytes: Option<Value>,
    pub cgroup_current_bytes: Option<Value>,
    pub kernel_ebpf_bytes: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessMemory {
    pub rss_bytes: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CgroupEvents {
    pub high: Option<Value>,
    pub oom: Option<Value>,
    pub oom_kill: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CgroupMemory {
    pub scope: Option<String>,
    pub current_bytes: Option<Value>,
    pub limit_bytes: Option<Value>,
    pub events: Option<CgroupEvents>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KernelMemory {
    pub ebpf_bytes: Option<Value>,
    pub sampled_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeMemory {
    pub observed_at: Option<String>,
    pub process: Option<ProcessMemory>,
    pub cgroup: Option<CgroupMemory>,
    pub kernel: Option<KernelMemory>,
}

impl RuntimeMemory {
    pub fn rss(&self) -> Option<u64> {
        self.process.as_ref().and_then(|p| value_u64(&p.rss_bytes))
    }

    pub fn cgroup_current(&self) -> Option<u64> {
        self.cgroup
            .as_ref()
            .and_then(|c| value_u64(&c.current_bytes))
    }

    pub fn cgroup_limit(&self) -> Option<u64> {
        self.cgroup
            .as_ref()
            .and_then(|c| value_u64(&c.limit_bytes))
    }

    pub fn cgroup_scope(&self) -> Option<&str> {
        self.cgroup.as_ref().and_then(|c| c.scope.as_deref())
    }

    pub fn high(&self) -> Option<u64> {
        self.cgroup
            .as_ref()
            .and_then(|c| c.events.as_ref())
            .and_then(|e| value_u64(&e.high))
    }

    pub fn oom(&self) -> Option<u64> {
        self.cgroup
            .as_ref()
            .and_then(|c| c.events.as_ref())
            .and_then(|e| value_u64(&e.oom))
    }

    pub fn oom_kill(&self) -> Option<u64> {
        self.cgroup
            .as_ref()
            .and_then(|c| c.events.as_ref())
            .and_then(|e| value_u64(&e.oom_kill))
    }

    pub fn ebpf(&self) -> Option<u64> {
        self.kernel.as_ref().and_then(|k| value_u64(&k.ebpf_bytes))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DatapathRouting {
    pub state: Option<String>,
    pub generation_id: Option<String>,
    pub epoch: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DatapathAttachment {
    pub name: String,
    pub kind: Option<String>,
    pub interface: String,
    pub direction: String,
    pub state: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MapOccupancy {
    pub occupancy: Option<u64>,
    pub capacity: u64,
    #[serde(default)]
    pub occupancy_known: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DatapathMaps {
    pub state: Option<String>,
    pub conn_state: Option<MapOccupancy>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EbpfDetail {
    pub backend: Option<String>,
    pub programs: Option<String>,
    pub hooks: Option<String>,
    pub routing: Option<DatapathRouting>,
    pub health: Option<String>,
    pub last_error: Option<String>,
    pub checked_at: Option<String>,
    #[serde(default)]
    pub attachments: Vec<DatapathAttachment>,
    pub maps: Option<DatapathMaps>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DatapathError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DatapathStatus {
    pub observed_at: Option<String>,
    pub kind: Option<String>,
    pub state: Option<String>,
    pub visibility: Option<String>,
    pub ebpf: Option<EbpfDetail>,
    #[serde(default)]
    pub errors: Vec<DatapathError>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupPolicy {
    pub kind: String,
    pub native: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupSummarySelection {
    pub tcp_member_id: Option<String>,
    pub udp_member_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupSummary {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub member_count: usize,
    pub policy: GroupPolicy,
    #[serde(default)]
    pub selection: GroupSummarySelection,
    pub config_revision: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub kind: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupSelectionEntry {
    pub member_id: Option<String>,
    pub resolved_leaf_node_id: Option<String>,
    pub source: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupRuntimeSelection {
    pub tcp: Option<GroupSelectionEntry>,
    pub udp: Option<GroupSelectionEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupRuntime {
    #[serde(default)]
    pub selection: GroupRuntimeSelection,
    #[serde(default)]
    pub health: Vec<HealthObservation>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupCapabilities {
    #[serde(default)]
    pub can_select: bool,
    #[serde(default)]
    pub can_override: bool,
    #[serde(default)]
    pub supports_nested_groups: bool,
    #[serde(default)]
    pub probe_transports: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GroupDetail {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub policy: GroupPolicy,
    #[serde(default)]
    pub members: Vec<GroupMember>,
    #[serde(default)]
    pub runtime: GroupRuntime,
    #[serde(default)]
    pub capabilities: GroupCapabilities,
    #[serde(default)]
    pub config: Value,
    pub config_revision: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HealthObservation {
    pub member_id: Option<String>,
    pub resolved_leaf_node_id: Option<String>,
    pub transport: Option<String>,
    pub purpose: Option<String>,
    pub ip_version: Option<String>,
    pub warmth: Option<String>,
    pub measurement: Option<String>,
    pub state: Option<String>,
    pub latency_ms: Option<f64>,
    pub moving_avg_ms: Option<f64>,
    pub avg10_ms: Option<f64>,
    pub observed_at: Option<String>,
    pub error: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeItem {
    pub id: String,
    pub name: String,
    pub protocol: Option<String>,
    pub stream_transport: Option<String>,
    pub subscription_tag: Option<String>,
    pub provider_id: Option<String>,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub health: Vec<HealthObservation>,
}

impl NodeItem {
    pub fn best_latency_ms(&self) -> Option<f64> {
        self.health
            .iter()
            .filter(|h| {
                matches!(
                    h.state.as_deref(),
                    Some("healthy" | "alive" | "ok")
                )
            })
            .filter_map(|h| h.latency_ms.or(h.moving_avg_ms).or(h.avg10_ms))
            .fold(None, |acc, v| Some(acc.map_or(v, |a: f64| a.min(v))))
    }

    pub fn health_state(&self) -> &str {
        if self.health.iter().any(|h| {
            matches!(h.state.as_deref(), Some("alive" | "ok" | "healthy"))
        }) {
            "alive"
        } else if self.health.iter().any(|h| {
            matches!(
                h.state.as_deref(),
                Some("unavailable" | "dead" | "error" | "timeout" | "unreachable" | "unhealthy")
            )
        }) {
            "dead"
        } else {
            "unknown"
        }
    }
}

/// Resolves the effective display label for any group policy (`selector`, `score`, `urltest`,
/// `fallback`, `loadbalance`), including nested child groups and idle `urltest`/`fallback`/`loadbalance`
/// groups where `honk-core` reports `selection.tcp_member_id: null`.
pub fn resolve_group_effective_summary(
    group: &GroupSummary,
    detail: Option<&GroupDetail>,
    all_groups: &[GroupSummary],
    all_nodes: &[NodeItem],
    lang: crate::i18n::Lang,
) -> String {
    use crate::format::format_latency;
    use crate::i18n::Lang;

    let sel_id = detail
        .and_then(|d| d.runtime.selection.tcp.as_ref())
        .and_then(|s| s.member_id.as_deref())
        .or(group.selection.tcp_member_id.as_deref())
        .unwrap_or("");
    let leaf_id = detail
        .and_then(|d| d.runtime.selection.tcp.as_ref())
        .and_then(|s| s.resolved_leaf_node_id.as_deref())
        .unwrap_or("");

    // 1. Direct or overridden member selection present
    if !sel_id.is_empty() {
        if let Some(node) = all_nodes.iter().find(|n| n.id == sel_id) {
            return match node.best_latency_ms() {
                Some(ms) => format!("{} ({})", node.name, format_latency(Some(ms))),
                None => node.name.clone(),
            };
        }
        if let Some(child_grp) = all_groups.iter().find(|cg| cg.id == sel_id) {
            if !leaf_id.is_empty()
                && let Some(leaf) = all_nodes.iter().find(|n| n.id == leaf_id)
            {
                return match leaf.best_latency_ms() {
                    Some(ms) => format!("{} → {} ({})", child_grp.name, leaf.name, format_latency(Some(ms))),
                    None => format!("{} → {}", child_grp.name, leaf.name),
                };
            }
            if let Some(child_sel) = child_grp.selection.tcp_member_id.as_deref()
                && let Some(leaf) = all_nodes.iter().find(|n| n.id == child_sel)
            {
                return match leaf.best_latency_ms() {
                    Some(ms) => format!("{} → {} ({})", child_grp.name, leaf.name, format_latency(Some(ms))),
                    None => format!("{} → {}", child_grp.name, leaf.name),
                };
            }
            return child_grp.name.clone();
        }
        if let Some(m) = detail.and_then(|d| d.members.iter().find(|m| m.id == sel_id)) {
            return m.name.clone();
        }
    }

    // 2. Collect candidate leaf nodes in exact group member order when GroupDetail is available,
    //    or via `node.group_ids` membership fallback.
    let candidates: Vec<&NodeItem> = if let Some(det) = detail {
        let mut out = Vec::with_capacity(det.members.len());
        for m in &det.members {
            if m.kind == "node" {
                if let Some(n) = all_nodes.iter().find(|n| n.id == m.id) {
                    out.push(n);
                }
            } else if m.kind == "group" {
                for n in all_nodes.iter().filter(|n| n.group_ids.iter().any(|gid| gid == &m.id)) {
                    out.push(n);
                }
            }
        }
        out
    } else {
        all_nodes
            .iter()
            .filter(|n| n.group_ids.iter().any(|gid| gid == &group.id))
            .collect()
    };

    let kind = group.policy.kind.as_str();
    match kind {
        "urltest" | "min" | "min_moving_avg" | "min_avg10" | "score" => {
            let best = candidates
                .iter()
                .copied()
                .filter(|n| n.health_state() != "dead")
                .filter_map(|n| n.best_latency_ms().map(|ms| (n, ms)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((winner, ms)) = best {
                format!("{} ({})", winner.name, format_latency(Some(ms)))
            } else if let Some(first) = candidates.iter().copied().find(|n| n.health_state() != "dead").or_else(|| candidates.first().copied()) {
                let tag = match lang {
                    Lang::En => "unprobed",
                    Lang::ZhTw => "待測速",
                    Lang::ZhCn => "待测速",
                };
                format!("{} ({tag})", first.name)
            } else {
                "—".to_string()
            }
        }
        "fallback" | "failover" => {
            // Exact declaration priority order: first alive member, else first non-dead member.
            let primary = candidates
                .iter()
                .copied()
                .find(|n| n.health_state() == "alive")
                .or_else(|| candidates.iter().copied().find(|n| n.health_state() != "dead"))
                .or_else(|| candidates.first().copied());
            if let Some(node) = primary {
                match node.best_latency_ms() {
                    Some(ms) => format!("{} ({})", node.name, format_latency(Some(ms))),
                    None => node.name.clone(),
                }
            } else {
                "—".to_string()
            }
        }
        "loadbalance" | "random" | "rr" | "hash" => {
            let total = candidates.len().max(group.member_count);
            if total == 0 {
                return "—".to_string();
            }
            let alive_nodes: Vec<&NodeItem> = candidates
                .iter()
                .copied()
                .filter(|n| n.health_state() == "alive")
                .collect();
            let best_ms = alive_nodes
                .iter()
                .filter_map(|n| n.best_latency_ms())
                .fold(None, |acc, v| Some(acc.map_or(v, |a: f64| a.min(v))));
            if !alive_nodes.is_empty() {
                match (lang, best_ms) {
                    (Lang::En, Some(ms)) => format!("Balanced · {}/{total} up ({})", alive_nodes.len(), format_latency(Some(ms))),
                    (Lang::En, None) => format!("Balanced · {}/{total} up", alive_nodes.len()),
                    (Lang::ZhTw, Some(ms)) => format!("均衡 · {}/{total} 可用 ({})", alive_nodes.len(), format_latency(Some(ms))),
                    (Lang::ZhTw, None) => format!("均衡 · {}/{total} 可用", alive_nodes.len()),
                    (Lang::ZhCn, Some(ms)) => format!("均衡 · {}/{total} 可用 ({})", alive_nodes.len(), format_latency(Some(ms))),
                    (Lang::ZhCn, None) => format!("均衡 · {}/{total} 可用", alive_nodes.len()),
                }
            } else {
                let non_dead = candidates.iter().filter(|n| n.health_state() != "dead").count();
                let pool = if non_dead > 0 { non_dead } else { total };
                match lang {
                    Lang::En => format!("Balanced · {pool} nodes"),
                    Lang::ZhTw => format!("均衡 · {pool} 節點池"),
                    Lang::ZhCn => format!("均衡 · {pool} 节点池"),
                }
            }
        }
        _ => {
            if let Some(first) = candidates.first() {
                first.name.clone()
            } else {
                "—".to_string()
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeList {
    pub observed_at: Option<String>,
    #[serde(default)]
    pub nodes: Vec<NodeItem>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderItem {
    pub id: String,
    pub name: String,
    pub kind: Option<String>,
    #[serde(default, alias = "url_redacted")]
    pub url: Option<String>,
    pub status: Option<String>,
    pub node_count: Option<u64>,
    pub updated_at: Option<String>,
    pub last_refresh_at: Option<String>,
    pub next_refresh_at: Option<String>,
    pub error: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderList {
    #[serde(default)]
    pub providers: Vec<ProviderItem>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigSource {
    pub id: String,
    pub kind: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub sha256: Option<String>,
    pub content_sha256: Option<String>,
    pub writable: Option<bool>,
    pub read_only_reason: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EffectiveConfig {
    pub observed_at: Option<String>,
    pub generation_id: Option<String>,
    pub revision: Option<String>,
    #[serde(default)]
    pub sources: Vec<ConfigSource>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(rename = "x-honk")]
    pub x_honk: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigRevision {
    pub revision: u64,
    pub parent: Option<u64>,
    pub created_at: String,
    pub principal: String,
    pub origin: String,
    pub content_sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigRevisionList {
    pub active: Option<u64>,
    pub max_revisions: Option<u64>,
    #[serde(default)]
    pub revisions: Vec<ConfigRevision>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConnectionItem {
    pub id: String,
    pub flow_id: Option<String>,
    pub pname: Option<String>,
    pub state: Option<String>,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub domain: Option<String>,
    pub outbound: Option<String>,
    #[serde(default)]
    pub chain: Vec<String>,
    pub rule_expression: Option<String>,
    pub started_at: Option<String>,
    pub upload_bytes: Option<Value>,
    pub download_bytes: Option<Value>,
    pub upload_bytes_per_second: Option<Value>,
    pub download_bytes_per_second: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConnectionList {
    pub observed_at: Option<String>,
    pub instance_id: Option<String>,
    pub visibility: Option<String>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub tcp: Vec<ConnectionItem>,
    #[serde(default)]
    pub udp: Vec<ConnectionItem>,
    #[serde(default)]
    pub total_tcp: u64,
    #[serde(default)]
    pub total_udp: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OperationAccepted {
    pub operation_id: String,
    pub status: Option<String>,
    pub href: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OperationState {
    pub operation_id: String,
    pub kind: Option<String>,
    pub status: String,
    pub result: Option<Value>,
    pub error: Option<ApiErrorBody>,
}

impl OperationState {
    pub fn is_done(&self) -> bool {
        self.status == "succeeded" || self.status == "failed"
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CoreUpdateError {
    pub code: String,
    pub message: String,
}

fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CoreUpdateStatus {
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub observed_at: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub repo: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub current_version: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub current_revision: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub target: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub asset_name: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub executable_path: String,
    #[serde(default)]
    pub can_apply: bool,
    pub cannot_apply_reason: Option<String>,
    #[serde(default)]
    pub applying: bool,
    pub checked_at: Option<String>,
    pub source: Option<String>,
    pub latest_version: Option<String>,
    #[serde(default)]
    pub has_update: bool,
    pub release_url: Option<String>,
    pub published_at: Option<String>,
    pub notes: Option<String>,
    pub asset_size_bytes: Option<Value>,
    pub asset_sha256: Option<String>,
    pub last_error: Option<CoreUpdateError>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CoreUpdateApplyResult {
    pub status: String,
    pub restarting: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: Option<bool>,
    pub release_url: Option<String>,
    pub previous_version: Option<String>,
    pub installed_version: Option<String>,
    pub sha256: Option<String>,
    pub bytes: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<String>,
    pub observed_at: Option<String>,
    #[serde(default, alias = "ts")]
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiEventItem {
    pub id: String,
    pub event: String,
    pub data: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_honk_core_runtime_memory_with_all_eight_metrics() {
        let json = r#"{
            "observed_at": "2026-11-28T00:00:00Z",
            "process": { "rss_bytes": "85983232" },
            "cgroup": {
                "scope": "service",
                "current_bytes": "134217728",
                "limit_bytes": "536870912",
                "events": { "high": "2", "oom": "0", "oom_kill": "0" }
            },
            "kernel": { "ebpf_bytes": "25165824", "sampled_at": "2026-11-28T00:00:00Z" }
        }"#;
        let mem: RuntimeMemory = serde_json::from_str(json).unwrap();
        assert_eq!(mem.rss(), Some(85_983_232));
        assert_eq!(mem.cgroup_current(), Some(134_217_728));
        assert_eq!(mem.cgroup_limit(), Some(536_870_912));
        assert_eq!(mem.cgroup_scope(), Some("service"));
        assert_eq!(mem.high(), Some(2));
        assert_eq!(mem.oom(), Some(0));
        assert_eq!(mem.oom_kill(), Some(0));
        assert_eq!(mem.ebpf(), Some(25_165_824));
    }

    #[test]
    fn parses_honk_core_runtime_and_connections_and_datapath() {
        let rt: RuntimeSnapshot = serde_json::from_str(
            r#"{
                "observed_at": "2026-11-28T00:00:00Z",
                "instance_id": "inst-1",
                "lifecycle": { "state": "running", "started_at": "2026-11-28T00:00:00Z", "uptime_seconds": "420" },
                "generation": { "active_id": "g1", "config_revision": "abcdef123456", "state": "active", "activated_at": "2026-11-28T00:00:00Z" },
                "traffic": {
                    "scope": "visible",
                    "connections": { "tcp": 12, "udp": 3, "total": 15 },
                    "bytes": { "upload": "1024", "download": "4096" },
                    "rates": { "window_seconds": 1.0, "upload_bytes_per_second": "256", "download_bytes_per_second": "1024" }
                },
                "process": { "pid": 100, "cpu_percent": 3.5 },
                "degradations": []
            }"#,
        )
        .unwrap();
        assert_eq!(rt.state_str(), "running");
        assert_eq!(rt.uptime_secs(), Some(420));
        assert_eq!(rt.download_rate(), 1024.0);
        assert_eq!(rt.upload_rate(), 256.0);
        assert_eq!(rt.traffic.connections.total, Some(15));

        let conns: ConnectionList = serde_json::from_str(
            r#"{
                "observed_at": "2026-11-28T00:00:00Z",
                "truncated": false,
                "tcp": [{ "id": "c1", "domain": "example.com", "outbound": "proxy" }],
                "udp": [],
                "total_tcp": 1,
                "total_udp": 0
            }"#,
        )
        .unwrap();
        assert_eq!(conns.tcp.len(), 1);
        assert_eq!(conns.total_tcp, 1);
    }
}
