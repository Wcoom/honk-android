//! M40 Sub-Store local module detector & `sub_store.env` parser (`z27.java` / `sn.java:500-520`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SUBSTORE_MODULE_DIR: &str = "/data/adb/modules/sub_store";
pub const SUBSTORE_ENV_PATH: &str = "/data/local/sub_store/scripts/sub_store.env";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubStoreEnv {
    pub installed: bool,
    pub backend_api_port: u16,
    pub backend_merge: bool,
    pub frontend_port: u16,
    pub frontend_backend_path: String,
    pub resolved_url: String,
}

pub fn parse_substore_env(raw: &str, installed: bool) -> SubStoreEnv {
    let mut vars = BTreeMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cleaned = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some((k, v)) = cleaned.split_once('=') {
            let val = v.trim().trim_matches(['"', '\'']).to_string();
            vars.insert(k.trim().to_string(), val);
        }
    }

    let backend_api_port = vars
        .get("SUB_STORE_BACKEND_API_PORT")
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let backend_merge = vars
        .get("SUB_STORE_BACKEND_MERGE")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    let frontend_port = vars
        .get("SUB_STORE_FRONTEND_PORT")
        .and_then(|v| v.parse().ok())
        .unwrap_or(if backend_merge { backend_api_port } else { 3001 });
    let frontend_backend_path = vars
        .get("SUB_STORE_FRONTEND_BACKEND_PATH")
        .cloned()
        .unwrap_or_default();

    let resolved_url = build_substore_url(
        frontend_port,
        backend_api_port,
        backend_merge,
        &frontend_backend_path,
    );

    SubStoreEnv {
        installed,
        backend_api_port,
        backend_merge,
        frontend_port,
        frontend_backend_path,
        resolved_url,
    }
}

pub fn build_substore_url(
    frontend_port: u16,
    backend_port: u16,
    backend_merge: bool,
    backend_path: &str,
) -> String {
    let path = backend_path.trim();
    let suffix = if path.is_empty() {
        String::new()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let api_port = if backend_merge {
        frontend_port
    } else {
        backend_port
    };
    format!("http://127.0.0.1:{frontend_port}/?api=http://127.0.0.1:{api_port}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_substore_env_and_builds_local_url() {
        let env = parse_substore_env(
            "SUB_STORE_BACKEND_API_PORT=3000\nSUB_STORE_FRONTEND_PORT=3001\nSUB_STORE_FRONTEND_BACKEND_PATH=/secret\n",
            true,
        );
        assert_eq!(
            env.resolved_url,
            "http://127.0.0.1:3001/?api=http://127.0.0.1:3000/secret"
        );
    }
}
