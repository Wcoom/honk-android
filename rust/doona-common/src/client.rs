//! Pure-Rust browser Fetch API client for `honk-core` Native API (`/api/v1/*`) & `/api/v1/x-honk/*`.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::DeserializeOwned;
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
use crate::model::ApiErrorEnvelope;
use crate::model::{
    AuthSession, Capabilities, ConfigRevisionList, ConfigSource, ConnectionList,
    CoreUpdateApplyResult, CoreUpdateStatus, DatapathStatus, DiscoveryInfo, EffectiveConfig,
    GroupDetail, GroupSummary, MemoryHistory, NodeList, OperationAccepted, OperationState,
    ProviderList, RuntimeMemory, RuntimeSnapshot, TrafficHistory, VersionInfo,
};

static NEXT_IDEMPOTENCY_SEQ: AtomicU64 = AtomicU64::new(1);

fn fresh_idempotency_key(prefix: &str) -> String {
    let seq = NEXT_IDEMPOTENCY_SEQ.fetch_add(1, Ordering::Relaxed);
    #[cfg(target_arch = "wasm32")]
    let ts = js_sys::Date::now() as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let ts = 0u64;
    format!("{prefix}-{ts}-{seq}")
}

fn encode_query_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct ApiClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        let trimmed = base_url.into().trim_end_matches('/').to_string();
        Self {
            base_url: trimmed,
            token: token.filter(|t| !t.trim().is_empty()),
        }
    }

    pub fn endpoint(&self, path: &str) -> String {
        if self.base_url.is_empty() {
            path.to_string()
        } else {
            format!("{}{path}", self.base_url)
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn request_raw<B: Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        content_type: Option<&str>,
        if_match: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<(String, Option<String>), String> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Headers, Request, RequestCache, RequestInit, Response};

        let window = web_sys::window().ok_or_else(|| "missing browser window".to_string())?;
        let opts = RequestInit::new();
        opts.set_method(method);
        opts.set_cache(RequestCache::NoStore);

        let headers = Headers::new().map_err(|e| format!("{e:?}"))?;
        let _ = headers.set("Accept", "application/json");
        if let Some(ref tok) = self.token {
            let _ = headers.set("Authorization", &format!("Bearer {tok}"));
        }
        if let Some(etag) = if_match {
            let _ = headers.set("If-Match", etag);
        }
        if let Some(key) = idempotency_key {
            let _ = headers.set("Idempotency-Key", key);
        }
        if let Some(b) = body {
            let text = serde_json::to_string(b).map_err(|e| e.to_string())?;
            let _ = headers.set("Content-Type", content_type.unwrap_or("application/json"));
            opts.set_body(&wasm_bindgen::JsValue::from_str(&text));
        }
        opts.set_headers(&headers);

        let url = self.endpoint(path);
        let req = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{e:?}"))?;
        let resp_val = JsFuture::from(window.fetch_with_request(&req))
            .await
            .map_err(|e| format!("network error: {e:?}"))?;
        let resp: Response = resp_val.dyn_into().map_err(|_| "invalid Response".to_string())?;
        let status = resp.status();
        let etag = resp.headers().get("ETag").ok().flatten();
        let text_val = JsFuture::from(resp.text().map_err(|e| format!("{e:?}"))?)
            .await
            .map_err(|e| format!("read body failed: {e:?}"))?;
        let text = text_val.as_string().unwrap_or_default();

        if !resp.ok() {
            if let Ok(env) = serde_json::from_str::<ApiErrorEnvelope>(&text) {
                return Err(format!(
                    "HTTP {status} ({}): {}",
                    env.error.code, env.error.message
                ));
            }
            return Err(format!("HTTP {status}: {text}"));
        }
        Ok((text, etag))
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn request_raw<B: Serialize>(
        &self,
        _method: &str,
        path: &str,
        _body: Option<&B>,
        _content_type: Option<&str>,
        _if_match: Option<&str>,
        _idempotency_key: Option<&str>,
    ) -> Result<(String, Option<String>), String> {
        Err(format!(
            "browser fetch unavailable on host target for {path}"
        ))
    }

    async fn request_json<B: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        content_type: Option<&str>,
        if_match: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<R, String> {
        let (text, _) = self
            .request_raw(method, path, body, content_type, if_match, idempotency_key)
            .await?;
        if text.trim().is_empty() {
            return serde_json::from_str("null")
                .map_err(|e| format!("empty body decode failed on {path}: {e}"));
        }
        serde_json::from_str(&text).map_err(|e| format!("JSON decode failed on {path}: {e}"))
    }

    pub async fn get_json<R: DeserializeOwned>(&self, path: &str) -> Result<R, String> {
        self.request_json::<(), R>("GET", path, None, None, None, None)
            .await
    }

    pub async fn post_json<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, String> {
        self.request_json("POST", path, Some(body), None, None, None)
            .await
    }

    pub async fn post_idempotent<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        prefix: &str,
    ) -> Result<R, String> {
        let key = fresh_idempotency_key(prefix);
        self.request_json("POST", path, Some(body), None, None, Some(&key))
            .await
    }

    pub async fn post_empty_idempotent<R: DeserializeOwned>(
        &self,
        path: &str,
        prefix: &str,
    ) -> Result<R, String> {
        let key = fresh_idempotency_key(prefix);
        self.request_json::<(), R>("POST", path, None, None, None, Some(&key))
            .await
    }

    pub async fn put_json<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        if_match: Option<&str>,
    ) -> Result<R, String> {
        self.request_json("PUT", path, Some(body), None, if_match, None)
            .await
    }

    pub async fn patch_json<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        content_type: Option<&str>,
        if_match: Option<&str>,
    ) -> Result<R, String> {
        self.request_json("PATCH", path, Some(body), content_type, if_match, None)
            .await
    }

    pub async fn delete_json<R: DeserializeOwned>(&self, path: &str) -> Result<R, String> {
        self.request_json::<(), R>("DELETE", path, None, None, None, None)
            .await
    }

    pub async fn discovery(&self) -> Result<DiscoveryInfo, String> {
        self.get_json("/api").await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthSession, String> {
        let body = serde_json::json!({ "username": username, "password": password });
        self.post_json("/api/v1/auth/login", &body).await
    }

    pub async fn setup_auth(&self, username: &str, password: &str) -> Result<AuthSession, String> {
        let body = serde_json::json!({ "username": username, "password": password });
        self.post_json("/api/v1/auth/setup", &body).await
    }

    pub async fn logout(&self) -> Result<serde_json::Value, String> {
        self.post_json("/api/v1/auth/logout", &serde_json::json!({}))
            .await
    }

    pub async fn version(&self) -> Result<VersionInfo, String> {
        self.get_json("/api/v1/version").await
    }

    pub async fn capabilities(&self) -> Result<Capabilities, String> {
        self.get_json("/api/v1/capabilities").await
    }

    pub async fn runtime(&self) -> Result<RuntimeSnapshot, String> {
        self.get_json("/api/v1/runtime?detail=full").await
    }

    pub async fn runtime_outbounds(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/runtime/outbounds").await
    }

    pub async fn traffic_history(
        &self,
        window_seconds: u64,
        max_points: usize,
    ) -> Result<TrafficHistory, String> {
        self.get_json(&format!(
            "/api/v1/runtime/traffic/history?window_seconds={window_seconds}&max_points={max_points}"
        ))
        .await
    }

    pub async fn memory_history(
        &self,
        window_seconds: u64,
        max_points: usize,
    ) -> Result<MemoryHistory, String> {
        self.get_json(&format!(
            "/api/v1/runtime/memory/history?window_seconds={window_seconds}&max_points={max_points}"
        ))
        .await
    }

    pub async fn runtime_memory(&self) -> Result<RuntimeMemory, String> {
        self.get_json("/api/v1/runtime/memory").await
    }

    pub async fn datapath(&self) -> Result<DatapathStatus, String> {
        self.get_json("/api/v1/datapath?detail=full").await
    }

    pub async fn groups(&self) -> Result<Vec<GroupSummary>, String> {
        self.get_json("/api/v1/groups").await
    }

    pub async fn group(&self, id: &str) -> Result<GroupDetail, String> {
        self.get_json(&format!("/api/v1/groups/{id}")).await
    }

    pub async fn select_group(
        &self,
        group_id: &str,
        member_id: &str,
        network: &str,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({ "member_id": member_id, "network": network });
        self.put_json(&format!("/api/v1/groups/{group_id}/selection"), &body, None)
            .await
    }

    pub async fn clear_group_override(
        &self,
        group_id: &str,
        network: &str,
    ) -> Result<serde_json::Value, String> {
        self.delete_json(&format!(
            "/api/v1/groups/{group_id}/selection?network={network}"
        ))
        .await
    }

    pub async fn group_config(
        &self,
        group_id: &str,
    ) -> Result<(serde_json::Value, Option<String>), String> {
        let (text, etag) = self
            .request_raw::<()>(
                "GET",
                &format!("/api/v1/groups/{group_id}/config"),
                None,
                None,
                None,
                None,
            )
            .await?;
        let val: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("decode group config failed: {e}"))?;
        Ok((val, etag))
    }

    pub async fn patch_group_config(
        &self,
        group_id: &str,
        ops: serde_json::Value,
        if_match: &str,
    ) -> Result<OperationAccepted, String> {
        let key = fresh_idempotency_key("grp-patch");
        self.request_json(
            "PATCH",
            &format!("/api/v1/groups/{group_id}/config"),
            Some(&ops),
            Some("application/json-patch+json"),
            Some(if_match),
            Some(&key),
        )
        .await
    }

    pub async fn nodes(&self) -> Result<NodeList, String> {
        self.get_json("/api/v1/nodes?limit=200").await
    }

    pub async fn node_detail(&self, id: &str) -> Result<serde_json::Value, String> {
        self.get_json(&format!("/api/v1/nodes/{id}")).await
    }

    pub async fn create_node(&self, name: &str, link: &str) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({ "name": name, "link": link });
        self.post_idempotent("/api/v1/nodes", &body, "node-add").await
    }

    pub async fn delete_node(&self, id: &str) -> Result<serde_json::Value, String> {
        let key = fresh_idempotency_key("node-del");
        self.request_json::<(), _>(
            "DELETE",
            &format!("/api/v1/nodes/{id}"),
            None,
            None,
            None,
            Some(&key),
        )
        .await
    }

    pub async fn providers(&self) -> Result<ProviderList, String> {
        self.get_json("/api/v1/providers?limit=200").await
    }

    pub async fn provider_detail(&self, id: &str) -> Result<serde_json::Value, String> {
        self.get_json(&format!("/api/v1/providers/{id}")).await
    }

    pub async fn create_provider(
        &self,
        name: &str,
        url: &str,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({
            "name": name,
            "kind": "subscription",
            "url": url,
        });
        self.post_idempotent("/api/v1/providers", &body, "prov-add")
            .await
    }

    pub async fn delete_provider(&self, id: &str) -> Result<serde_json::Value, String> {
        let key = fresh_idempotency_key("prov-del");
        self.request_json::<(), _>(
            "DELETE",
            &format!("/api/v1/providers/{id}"),
            None,
            None,
            None,
            Some(&key),
        )
        .await
    }

    pub async fn refresh_provider(&self, id: &str) -> Result<OperationAccepted, String> {
        self.post_empty_idempotent(&format!("/api/v1/providers/{id}/refresh"), "prov-ref")
            .await
    }

    pub async fn connections(&self) -> Result<ConnectionList, String> {
        self.get_json("/api/v1/connections?detail=full").await
    }

    pub async fn close_connection(&self, id: &str) -> Result<serde_json::Value, String> {
        self.delete_json(&format!("/api/v1/connections/{id}")).await
    }

    pub async fn close_all_connections(&self) -> Result<serde_json::Value, String> {
        self.delete_json("/api/v1/connections?all=true").await
    }

    pub async fn flows(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/flows?limit=100&detail=full").await
    }

    pub async fn flow_detail(&self, id: &str) -> Result<serde_json::Value, String> {
        self.get_json(&format!("/api/v1/flows/{id}")).await
    }

    pub async fn rules(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/rules").await
    }

    pub async fn dns_rules(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/dns/rules").await
    }

    pub async fn dns_cache(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/dns/cache?detail=full&limit=200")
            .await
    }

    pub async fn dns_log(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/dns/log?limit=200").await
    }

    pub async fn dns_query(
        &self,
        domain: &str,
        types: &[&str],
    ) -> Result<serde_json::Value, String> {
        self.dns_query_mode(domain, types, "normal").await
    }

    pub async fn dns_query_mode(
        &self,
        domain: &str,
        types: &[&str],
        cache_mode: &str,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({
            "domain": domain,
            "type": types,
            "cache_mode": cache_mode,
        });
        self.post_json("/api/v1/dns/query?detail=full", &body).await
    }

    pub async fn delete_dns_cache_entry(&self, entry_id: &str) -> Result<serde_json::Value, String> {
        self.delete_json(&format!("/api/v1/dns/cache/{entry_id}"))
            .await
    }

    pub async fn delete_dns_cache_name(&self, name: &str) -> Result<serde_json::Value, String> {
        let enc = encode_query_component(name);
        self.delete_json(&format!("/api/v1/dns/cache?name={enc}"))
            .await
    }

    pub async fn flush_dns_cache(&self) -> Result<serde_json::Value, String> {
        self.post_json("/api/v1/dns/cache/flush", &serde_json::json!({}))
            .await
    }

    pub async fn routing_trace(
        &self,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.post_json("/api/v1/routing/trace", &serde_json::json!({ "input": input }))
            .await
    }

    pub async fn config(&self) -> Result<EffectiveConfig, String> {
        self.get_json("/api/v1/config").await
    }

    pub async fn config_source(&self, source_id: &str) -> Result<(ConfigSource, Option<String>), String> {
        let (text, etag) = self
            .request_raw::<()>("GET", &format!("/api/v1/config/sources/{source_id}"), None, None, None, None)
            .await?;
        let source: ConfigSource =
            serde_json::from_str(&text).map_err(|e| format!("decode source failed: {e}"))?;
        Ok((source, etag))
    }

    pub async fn replace_config_source(
        &self,
        source_id: &str,
        content: &str,
        if_match: &str,
    ) -> Result<OperationAccepted, String> {
        let key = fresh_idempotency_key("cfg-put");
        let body = serde_json::json!({ "content": content });
        self.request_json(
            "PUT",
            &format!("/api/v1/config/sources/{source_id}"),
            Some(&body),
            None,
            Some(if_match),
            Some(&key),
        )
        .await
    }

    pub async fn create_config_source(
        &self,
        path: &str,
        content: &str,
    ) -> Result<OperationAccepted, String> {
        let body = serde_json::json!({ "path": path, "content": content });
        self.post_idempotent("/api/v1/config/sources", &body, "cfg-new")
            .await
    }

    pub async fn validate_config(
        &self,
        sources: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.post_json("/api/v1/config/validate", &sources).await
    }

    pub async fn config_revisions(&self) -> Result<ConfigRevisionList, String> {
        self.get_json("/api/v1/x-honk/config/revisions").await
    }

    pub async fn activate_config_revision(&self, rev: u64) -> Result<OperationAccepted, String> {
        self.post_idempotent(
            &format!("/api/v1/x-honk/config/revisions/{rev}/activate"),
            &serde_json::json!({}),
            "cfg-act",
        )
        .await
    }

    pub async fn import_config(&self) -> Result<OperationAccepted, String> {
        self.import_config_replace(true).await
    }

    pub async fn import_config_replace(&self, replace: bool) -> Result<OperationAccepted, String> {
        self.post_idempotent(
            "/api/v1/x-honk/config/import",
            &serde_json::json!({ "replace": replace }),
            "cfg-imp",
        )
        .await
    }

    pub async fn export_config_text(&self) -> Result<String, String> {
        let (text, _) = self
            .request_raw::<()>("GET", "/api/v1/x-honk/config/export", None, None, None, None)
            .await?;
        Ok(text)
    }

    pub async fn operation(&self, id: &str) -> Result<OperationState, String> {
        self.get_json(&format!("/api/v1/operations/{id}")).await
    }

    pub async fn start_reload(&self) -> Result<OperationAccepted, String> {
        self.post_idempotent("/api/v1/operations/reload", &serde_json::json!({}), "reload")
            .await
    }

    pub async fn start_suspend(&self) -> Result<OperationAccepted, String> {
        self.post_idempotent("/api/v1/operations/suspend", &serde_json::json!({}), "suspend")
            .await
    }

    pub async fn start_resume(&self) -> Result<OperationAccepted, String> {
        self.post_idempotent("/api/v1/operations/resume", &serde_json::json!({}), "resume")
            .await
    }

    pub async fn runtime_settings(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/runtime/settings").await
    }

    pub async fn patch_runtime_settings(
        &self,
        patch: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.patch_json("/api/v1/runtime/settings", &patch, None, None)
            .await
    }

    pub async fn geodata(&self) -> Result<serde_json::Value, String> {
        self.get_json("/api/v1/geodata").await
    }

    pub async fn update_geodata(&self) -> Result<OperationAccepted, String> {
        self.post_empty_idempotent("/api/v1/geodata/update", "geo-upd")
            .await
    }

    pub async fn core_update_status(&self, refresh: bool) -> Result<CoreUpdateStatus, String> {
        let path = if refresh {
            "/api/v1/x-honk/core/update?refresh=1"
        } else {
            "/api/v1/x-honk/core/update"
        };
        self.get_json(path).await
    }

    pub async fn apply_core_update(&self, force: bool) -> Result<CoreUpdateApplyResult, String> {
        let body = if force {
            serde_json::json!({ "force": true })
        } else {
            serde_json::json!({})
        };
        self.post_json("/api/v1/x-honk/core/update", &body).await
    }

    pub async fn probe_group(&self, group_id: &str) -> Result<OperationAccepted, String> {
        let body = serde_json::json!({
            "target": { "type": "group", "group_id": group_id, "members": "leaves" },
            "kind": "http",
            "transport": ["tcp"],
            "ip_version": "any",
            "warmth": "warm",
        });
        self.post_idempotent("/api/v1/probes", &body, "probe-grp")
            .await
    }

    pub async fn probe_node(&self, node_id: &str) -> Result<OperationAccepted, String> {
        let body = serde_json::json!({
            "target": { "type": "node", "node_id": node_id },
            "kind": "http",
            "transport": ["tcp"],
            "ip_version": "any",
            "warmth": "warm",
        });
        self.post_idempotent("/api/v1/probes", &body, "probe-nd")
            .await
    }
}
