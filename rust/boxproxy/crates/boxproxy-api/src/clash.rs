//! Full Clash API HTTP client (`mihomo`, `meow`, `sing-box`, plus fallback for `honk`).

use crate::ApiResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClashConfig {
    #[serde(default)]
    pub mode: String,
    #[serde(default, alias = "mode-list", alias = "mode_list", alias = "modeList")]
    pub mode_list: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClashProxyGroup {
    pub name: String,
    #[serde(rename = "type", default)]
    pub group_type: String,
    #[serde(default)]
    pub now: String,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub latency_ms: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClashConnectionsSnapshot {
    #[serde(rename = "downloadTotal", default)]
    pub download_total: u64,
    #[serde(rename = "uploadTotal", default)]
    pub upload_total: u64,
    #[serde(default)]
    pub connections: Vec<Value>,
}

pub struct ClashApiClient {
    http: Client,
    base_url: String,
    secret: String,
}

impl ClashApiClient {
    pub fn new(base_url: impl Into<String>, secret: impl Into<String>) -> ApiResult<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| format!("build clash http client failed: {e}"))?;
        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            secret: secret.into(),
        })
    }

    fn req(&self, method: reqwest::Method, path: &str, timeout: Duration) -> reqwest::RequestBuilder {
        let mut r = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .timeout(timeout);
        if !self.secret.is_empty() {
            r = r.bearer_auth(&self.secret);
        }
        r
    }

    pub async fn get_configs(&self) -> ApiResult<ClashConfig> {
        let resp = self
            .req(reqwest::Method::GET, "/configs", Duration::from_secs(7))
            .send()
            .await
            .map_err(|e| format!("GET /configs failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("GET /configs HTTP {}", resp.status()));
        }
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn set_mode(&self, mode: &str) -> ApiResult<()> {
        let resp = self
            .req(reqwest::Method::PATCH, "/configs", Duration::from_secs(7))
            .json(&serde_json::json!({ "mode": mode }))
            .send()
            .await
            .map_err(|e| format!("PATCH /configs failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("PATCH /configs HTTP {}", resp.status()))
        }
    }

    pub async fn reload_config(&self) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::PUT,
                "/configs?force=true",
                Duration::from_secs(15),
            )
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| format!("PUT /configs?force=true failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("PUT /configs?force=true HTTP {}", resp.status()))
        }
    }

    pub async fn get_proxies(&self) -> ApiResult<BTreeMap<String, Value>> {
        let resp = self
            .req(reqwest::Method::GET, "/proxies", Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("GET /proxies failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("GET /proxies HTTP {}", resp.status()));
        }
        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        let map = body
            .get("proxies")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        Ok(map)
    }

    pub async fn select_proxy(&self, group: &str, node: &str) -> ApiResult<()> {
        let enc_group = url_encode(group);
        let resp = self
            .req(
                reqwest::Method::PUT,
                &format!("/proxies/{enc_group}"),
                Duration::from_secs(10),
            )
            .json(&serde_json::json!({ "name": node }))
            .send()
            .await
            .map_err(|e| format!("PUT /proxies/{group} failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("PUT /proxies/{group} HTTP {}", resp.status()))
        }
    }

    pub async fn test_proxy_delay(
        &self,
        node: &str,
        url: &str,
        timeout_ms: u32,
    ) -> ApiResult<u32> {
        let path = format!(
            "/proxies/{}/delay?timeout={timeout_ms}&url={}",
            url_encode(node),
            url_encode(url)
        );
        let resp = self
            .req(
                reqwest::Method::GET,
                &path,
                Duration::from_millis(timeout_ms as u64 + 2000),
            )
            .send()
            .await
            .map_err(|e| format!("delay test {node} failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("delay test {node} HTTP {}", resp.status()));
        }
        let v: Value = resp.json().await.map_err(|e| e.to_string())?;
        v.get("delay")
            .and_then(|d| d.as_u64())
            .map(|d| d as u32)
            .ok_or_else(|| "missing delay field".to_string())
    }

    pub async fn test_group_delay(
        &self,
        group: &str,
        url: &str,
        timeout_ms: u32,
    ) -> ApiResult<BTreeMap<String, u32>> {
        let path = format!(
            "/group/{}/delay?timeout={timeout_ms}&url={}",
            url_encode(group),
            url_encode(url)
        );
        let resp = self
            .req(
                reqwest::Method::GET,
                &path,
                Duration::from_millis(timeout_ms as u64 + 5000),
            )
            .send()
            .await
            .map_err(|e| format!("group delay {group} failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("group delay {group} HTTP {}", resp.status()));
        }
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_proxy_providers(&self) -> ApiResult<Value> {
        let resp = self
            .req(
                reqwest::Method::GET,
                "/providers/proxies",
                Duration::from_secs(25),
            )
            .send()
            .await
            .map_err(|e| format!("GET /providers/proxies failed: {e}"))?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn update_proxy_provider(&self, name: &str) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::PUT,
                &format!("/providers/proxies/{}", url_encode(name)),
                Duration::from_secs(60),
            )
            .send()
            .await
            .map_err(|e| format!("PUT /providers/proxies/{name} failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("update provider {name} HTTP {}", resp.status()))
        }
    }

    pub async fn healthcheck_proxy_provider(&self, name: &str) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::GET,
                &format!("/providers/proxies/{}/healthcheck", url_encode(name)),
                Duration::from_secs(60),
            )
            .send()
            .await
            .map_err(|e| format!("healthcheck {name} failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("healthcheck {name} HTTP {}", resp.status()))
        }
    }

    pub async fn get_rules(&self) -> ApiResult<Value> {
        let resp = self
            .req(reqwest::Method::GET, "/rules", Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("GET /rules failed: {e}"))?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_rule_providers(&self) -> ApiResult<Value> {
        let resp = self
            .req(
                reqwest::Method::GET,
                "/providers/rules",
                Duration::from_secs(15),
            )
            .send()
            .await
            .map_err(|e| format!("GET /providers/rules failed: {e}"))?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn update_rule_provider(&self, name: &str) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::PUT,
                &format!("/providers/rules/{}", url_encode(name)),
                Duration::from_secs(45),
            )
            .send()
            .await
            .map_err(|e| format!("PUT /providers/rules/{name} failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("update rule provider {name} HTTP {}", resp.status()))
        }
    }

    pub async fn get_connections(&self) -> ApiResult<ClashConnectionsSnapshot> {
        let resp = self
            .req(reqwest::Method::GET, "/connections", Duration::from_secs(7))
            .send()
            .await
            .map_err(|e| format!("GET /connections failed: {e}"))?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn close_all_connections(&self) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::DELETE,
                "/connections",
                Duration::from_secs(7),
            )
            .send()
            .await
            .map_err(|e| format!("DELETE /connections failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("DELETE /connections HTTP {}", resp.status()))
        }
    }

    pub async fn close_connection(&self, id: &str) -> ApiResult<()> {
        let resp = self
            .req(
                reqwest::Method::DELETE,
                &format!("/connections/{}", url_encode(id)),
                Duration::from_secs(7),
            )
            .send()
            .await
            .map_err(|e| format!("DELETE /connections/{id} failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("DELETE /connections/{id} HTTP {}", resp.status()))
        }
    }
}

fn url_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(&mut out, "%{b:02X}");
            }
        }
    }
    out
}
