//! Honk Native API client (`/api/v1/*` and `/api/v1/x-honk/*`, ADR-009).

use crate::ApiResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HonkTrafficPoint {
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub upload_bps: u64,
    #[serde(default)]
    pub download_bps: u64,
    #[serde(default)]
    pub total_upload: u64,
    #[serde(default)]
    pub total_download: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HonkRouteTraceRequest {
    pub domain: Option<String>,
    pub dst_ip: Option<String>,
    pub dst_port: Option<u16>,
    pub protocol: Option<String>,
    pub process_name: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HonkCoreUpdateResponse {
    #[serde(default)]
    pub current_version: String,
    #[serde(default)]
    pub latest_version: String,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub message: String,
}

pub struct HonkNativeClient {
    http: Client,
    base_url: String,
    token: String,
}

impl HonkNativeClient {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> ApiResult<Self> {
        static SHARED_HTTP: std::sync::OnceLock<Result<Client, String>> = std::sync::OnceLock::new();
        let http = SHARED_HTTP
            .get_or_init(|| {
                Client::builder()
                    .timeout(Duration::from_secs(15))
                    .pool_idle_timeout(Duration::from_secs(30))
                    .build()
                    .map_err(|e| format!("build http client failed: {e}"))
            })
            .clone()?;
        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: token.into(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.token.is_empty() {
            req
        } else {
            req.bearer_auth(&self.token)
        }
    }

    async fn get_json(&self, path: &str) -> ApiResult<Value> {
        let resp = self
            .auth(self.http.get(self.url(path)))
            .send()
            .await
            .map_err(|e| format!("GET {path} failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("GET {path} HTTP {}", resp.status()));
        }
        resp.json::<Value>()
            .await
            .map_err(|e| format!("decode {path} failed: {e}"))
    }

    async fn send_json(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> ApiResult<Value> {
        let mut req = self.auth(self.http.request(method.clone(), self.url(path)));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("{method} {path} failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("{method} {path} HTTP {}", resp.status()));
        }
        let text = resp.text().await.unwrap_or_default();
        if text.trim().is_empty() {
            Ok(Value::Null)
        } else {
            serde_json::from_str(&text).or(Ok(Value::String(text)))
        }
    }

    pub async fn runtime(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/runtime?detail=full").await
    }

    pub async fn runtime_memory(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/runtime/memory").await
    }

    pub async fn traffic_history(&self, window_seconds: u64, max_points: u32) -> ApiResult<Value> {
        self.get_json(&format!(
            "/api/v1/runtime/traffic/history?window_seconds={window_seconds}&max_points={max_points}"
        ))
        .await
    }

    pub async fn datapath(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/datapath").await
    }

    pub async fn groups(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/groups").await
    }

    pub async fn group_detail(&self, group_id: &str) -> ApiResult<Value> {
        self.get_json(&format!("/api/v1/groups/{group_id}")).await
    }

    pub async fn select_group_member(&self, group_id: &str, member: &str) -> ApiResult<Value> {
        self.send_json(
            reqwest::Method::PUT,
            &format!("/api/v1/groups/{group_id}/selection"),
            Some(serde_json::json!({ "member_id": member, "network": "both" })),
        )
        .await
    }

    pub async fn clear_group_selection(&self, group_id: &str) -> ApiResult<Value> {
        self.send_json(
            reqwest::Method::DELETE,
            &format!("/api/v1/groups/{group_id}/selection?network=both"),
            None,
        )
        .await
    }

    pub async fn probe_group(&self, group_id: &str) -> ApiResult<Value> {
        let key = format!(
            "box-probe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(1)
        );
        let payload = serde_json::json!({
            "target": { "type": "group", "group_id": group_id, "members": "leaves" },
            "kind": "http",
            "transport": ["tcp"],
            "ip_version": "any",
            "warmth": "warm",
        });
        let resp = self
            .auth(self.http.post(self.url("/api/v1/probes")))
            .header("Idempotency-Key", key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("POST /api/v1/probes failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("POST /api/v1/probes HTTP {}", resp.status()));
        }
        let text = resp.text().await.unwrap_or_default();
        serde_json::from_str(&text).or(Ok(Value::Null))
    }

    pub async fn nodes(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/nodes?limit=200").await
    }

    pub async fn probe_nodes(&self, payload: Value) -> ApiResult<Value> {
        self.send_json(reqwest::Method::POST, "/api/v1/probes", Some(payload))
            .await
    }

    pub async fn providers(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/providers?limit=200").await
    }

    pub async fn refresh_provider(&self, id: &str) -> ApiResult<Value> {
        let key = format!(
            "box-prov-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(1)
        );
        let resp = self
            .auth(self.http.post(self.url(&format!("/api/v1/providers/{id}/refresh"))))
            .header("Idempotency-Key", key)
            .send()
            .await
            .map_err(|e| format!("POST /api/v1/providers/{id}/refresh failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("POST /api/v1/providers/{id}/refresh HTTP {}", resp.status()));
        }
        let text = resp.text().await.unwrap_or_default();
        serde_json::from_str(&text).or(Ok(Value::Null))
    }

    pub async fn connections(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/connections?detail=full").await
    }

    pub async fn close_all_connections(&self) -> ApiResult<Value> {
        self.send_json(reqwest::Method::DELETE, "/api/v1/connections?all=true", None)
            .await
    }

    pub async fn close_connection(&self, id: &str) -> ApiResult<Value> {
        self.send_json(
            reqwest::Method::DELETE,
            &format!("/api/v1/connections/{id}"),
            None,
        )
        .await
    }

    pub async fn rules(&self) -> ApiResult<Value> {
        self.get_json("/api/v1/rules").await
    }

    pub async fn routing_trace(&self, req: &HonkRouteTraceRequest) -> ApiResult<Value> {
        let mut input = serde_json::Map::new();
        let net = req
            .protocol
            .as_deref()
            .unwrap_or("tcp")
            .to_ascii_lowercase();
        input.insert("network".into(), serde_json::json!(net));
        input.insert(
            "dst_port".into(),
            serde_json::json!(req.dst_port.unwrap_or(443)),
        );
        if let Some(d) = req.domain.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            input.insert("domain".into(), serde_json::json!(d));
        }
        if let Some(ip) = req.dst_ip.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            input.insert("dst_ip".into(), serde_json::json!(ip));
        }
        if let Some(p) = req.process_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            input.insert("pname".into(), serde_json::json!(p));
        }
        let body = serde_json::json!({ "input": Value::Object(input) });
        self.send_json(reqwest::Method::POST, "/api/v1/routing/trace", Some(body))
            .await
    }

    pub async fn reload(&self) -> ApiResult<Value> {
        let key = format!(
            "box-reload-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(1)
        );
        let resp = self
            .auth(self.http.post(self.url("/api/v1/operations/reload")))
            .header("Idempotency-Key", key)
            .send()
            .await
            .map_err(|e| format!("POST /api/v1/operations/reload failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("POST /api/v1/operations/reload HTTP {}", resp.status()));
        }
        let text = resp.text().await.unwrap_or_default();
        serde_json::from_str(&text).or(Ok(Value::Null))
    }

    pub async fn check_core_update(&self, refresh: bool) -> ApiResult<Value> {
        let path = if refresh {
            "/api/v1/x-honk/core/update?refresh=1"
        } else {
            "/api/v1/x-honk/core/update"
        };
        self.get_json(path).await
    }

    pub async fn apply_core_update(&self, force: bool) -> ApiResult<Value> {
        self.send_json(
            reqwest::Method::POST,
            "/api/v1/x-honk/core/update",
            Some(serde_json::json!({ "force": force })),
        )
        .await
    }

    pub async fn flush_dns_cache(&self) -> ApiResult<Value> {
        self.send_json(reqwest::Method::POST, "/api/v1/dns/cache/flush", None)
            .await
    }
}
