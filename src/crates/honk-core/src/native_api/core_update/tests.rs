use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt as _;

use super::*;
use crate::control::ControlPlane;
use crate::ebpf::mock::MockEbpfBackend;

#[test]
fn version_comparison_handles_semver_prereleases_and_git_describe() {
    assert!(version_is_newer("v1.0.5", "v1.0.6"));
    assert!(version_is_newer("1.0.5", "v1.0.10"));
    assert!(version_is_newer("v1.0.6-rc.1", "v1.0.6"));
    assert!(version_is_newer("v1.0.6-alpha.1", "v1.0.6-beta.1"));
    assert!(version_is_newer("v1.0.5-3-gabcdef", "v1.0.6"));

    assert!(!version_is_newer("v1.0.6", "v1.0.6"));
    assert!(!version_is_newer("v1.0.6", "v1.0.5"));
    assert!(!version_is_newer("v1.0.6", "v1.0.6-rc.1"));
    assert!(!version_is_newer("v1.0.5-3-gabcdef", "v1.0.5"));
    assert!(!version_is_newer("v1.0.5-3-gabcdef-dirty", "v1.0.5"));
    assert!(!version_is_newer("v1.0.5", "not-a-version"));
}

#[test]
fn sha256sums_parser_handles_standard_binary_and_path_entries() {
    let digest = "25b98ff911a1f686e9316be70cd27fea397ad570a245b3b514f8576669130bd5";
    let body = format!(
        "# comment\n\
         880f17141e87192dbed165a871335a848e99ffe29e01e7c40d93c7d6525277c1  other.tar.gz\n\
         {} *dist/honk-android-arm64\n",
        digest.to_ascii_uppercase()
    );
    assert_eq!(
        parse_sha256sums_for_asset(&body, "honk-android-arm64").as_deref(),
        Some(digest)
    );
    assert_eq!(parse_sha256sums_for_asset(&body, "missing"), None);
    assert_eq!(
        parse_sha256sums_for_asset("short  honk-android-arm64\n", "honk-android-arm64"),
        None
    );
}

#[test]
fn elf_validator_checks_magic_class_endianness_and_machine() {
    let mut elf = vec![0u8; 64];
    elf[0..4].copy_from_slice(ELF_MAGIC);
    elf[4] = ELF_CLASS_64;
    elf[5] = ELF_DATA_LSB;
    elf[18..20].copy_from_slice(&EM_AARCH64.to_le_bytes());

    assert!(validate_elf_binary(&elf, Some(EM_AARCH64)).is_ok());
    assert!(validate_elf_binary(&elf, None).is_ok());

    let mut x86_elf = elf.clone();
    x86_elf[18..20].copy_from_slice(&0x003eu16.to_le_bytes());
    assert!(validate_elf_binary(&x86_elf, Some(EM_AARCH64)).is_err());

    let mut elf32 = elf.clone();
    elf32[4] = 1;
    assert!(validate_elf_binary(&elf32, None).is_err());
    assert!(validate_elf_binary(b"not an elf", None).is_err());
}

#[test]
fn extracts_release_tag_from_github_redirect_url() {
    assert_eq!(
        extract_tag_from_release_url(
            "https://github.com/Wcoom/honk-android/releases/tag/v1.0.6"
        )
        .as_deref(),
        Some("v1.0.6")
    );
    assert_eq!(
        extract_tag_from_release_url(
            "https://github.com/Wcoom/honk-android/releases/tag/v1.0.6/?foo=1#top"
        )
        .as_deref(),
        Some("v1.0.6")
    );
    assert_eq!(
        extract_tag_from_release_url("https://github.com/Wcoom/honk-android/releases"),
        None
    );
}

fn minimal_valid_arm64_elf(marker: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0u8; 64 + marker.len()];
    bytes[0..4].copy_from_slice(ELF_MAGIC);
    bytes[4] = ELF_CLASS_64;
    bytes[5] = ELF_DATA_LSB;
    bytes[18..20].copy_from_slice(&EM_AARCH64.to_le_bytes());
    bytes[64..].copy_from_slice(marker);
    bytes
}

async fn spawn_mock_http_server<F>(handler: F) -> (SocketAddr, tokio::task::JoinHandle<()>)
where
    F: Fn(&str) -> (u16, Vec<(String, String)>, Vec<u8>) + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handler = Arc::new(handler);
    let handle = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let handler = Arc::clone(&handler);
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let target = req
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");
                let (status, headers, body) = handler(target);
                let reason = match status {
                    200 => "OK",
                    302 => "Found",
                    403 => "Forbidden",
                    404 => "Not Found",
                    _ => "Status",
                };
                let mut head = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
                    body.len()
                );
                for (k, v) in headers {
                    head.push_str(&format!("{k}: {v}\r\n"));
                }
                head.push_str("\r\n");
                let _ = stream.write_all(head.as_bytes()).await;
                let _ = stream.write_all(&body).await;
                let _ = stream.shutdown().await;
            });
        }
    });
    (addr, handle)
}

async fn test_state_with_update_config(
    config: CoreUpdateConfig,
    secret: &str,
) -> (ControlPlane, Arc<NativeState>, tempfile::TempDir) {
    let mut cfg = honk_config::Config::default();
    cfg.experimental.native_api.enabled = true;
    cfg.experimental.native_api.secret = secret.to_owned();
    cfg.experimental.native_api.allow_anonymous_loopback = secret.is_empty();
    let dir = tempfile::tempdir().unwrap();
    cfg.global.data_dir = dir.path().to_string_lossy().into_owned();

    let router = crate::routing::Router::from_config(&cfg.routing).unwrap();
    let proxy_registry = Arc::new(crate::proxy::ProxyRegistry::default());
    let dns_router = Arc::new(crate::dns::routing::DnsRouter::new(&cfg.dns.routing).unwrap());
    let dns_upstream_pool = Arc::new(
        crate::dns::upstream_pool::UpstreamPool::new(&cfg.dns.upstream, Arc::clone(&dns_router))
            .unwrap(),
    );
    let dns_cache = Arc::new(tokio::sync::Mutex::new(crate::dns::cache::DnsCache::new(64)));
    let dns_forwarder = Arc::new(crate::dns::forwarder::DnsForwarder::new(
        dns_upstream_pool.clone(),
        dns_cache,
        dns_router,
    ));
    let mut control = ControlPlane::new_with_upstream_pool(
        cfg,
        Box::new(MockEbpfBackend::new()),
        router,
        proxy_registry,
        dns_forwarder,
        dns_upstream_pool,
    )
    .unwrap();
    let state = Arc::new(
        NativeState::with_core_update_for_test(
            &mut control,
            "127.0.0.1:9527".parse().unwrap(),
            CoreUpdateService::with_config(config),
        )
        .await
        .unwrap(),
    );
    (control, state, dir)
}

#[tokio::test]
async fn get_refresh_queries_api_and_caches_result() {
    let binary = minimal_valid_arm64_elf(b"honk-v99");
    let digest = sha256_hex(&binary);
    let digest_clone = digest.clone();
    let (addr, server) = spawn_mock_http_server(move |path| match path {
        "/releases/latest" => {
            let body = json!({
                "tag_name": "v99.0.0",
                "html_url": "https://example.invalid/releases/tag/v99.0.0",
                "published_at": "2026-11-24T01:00:00Z",
                "body": "Release highlights",
                "assets": [{
                    "name": "honk-android-arm64",
                    "size": binary.len(),
                    "digest": format!("sha256:{digest_clone}"),
                    "browser_download_url": "http://127.0.0.1:1/unused"
                }]
            })
            .to_string();
            (
                200,
                vec![("Content-Type".into(), "application/json".into())],
                body.into_bytes(),
            )
        }
        "/download/v99.0.0/SHA256SUMS.txt" => (404, vec![], b"missing".to_vec()),
        _ => (404, vec![], b"not found".to_vec()),
    })
    .await;

    let target_dir = tempfile::tempdir().unwrap();
    let target_bin = target_dir.path().join("honk");
    std::fs::write(&target_bin, b"old-binary").unwrap();

    let (_control, state, _data_dir) = test_state_with_update_config(
        CoreUpdateConfig {
            api_latest_url: format!("http://{addr}/releases/latest"),
            web_latest_url: format!("http://{addr}/web/latest"),
            download_base_url: format!("http://{addr}/download"),
            expected_elf_machine: Some(EM_AARCH64),
            verify_exec: false,
            skip_restart: true,
            target_path_override: Some(target_bin),
            ..CoreUpdateConfig::default()
        },
        "secret-token",
    )
    .await;

    let app = crate::native_api::router(Arc::clone(&state));
    let unrefreshed = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/x-honk/core/update")
                .header("Host", "127.0.0.1:9527")
                .header("Authorization", "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unrefreshed.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(unrefreshed.into_body(), 16384)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["current_version"], crate::VERSION);
    assert_eq!(body["can_apply"], true);
    assert_eq!(body["latest_version"], Value::Null);
    assert_eq!(body["has_update"], false);

    let refreshed = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/x-honk/core/update?refresh=1")
                .header("Host", "127.0.0.1:9527")
                .header("Authorization", "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refreshed.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(refreshed.into_body(), 16384)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["latest_version"], "v99.0.0");
    assert_eq!(body["has_update"], true);
    assert_eq!(body["source"], "github_api");
    assert_eq!(body["asset_sha256"], digest);

    server.abort();
}

#[tokio::test]
async fn apply_falls_back_to_web_redirect_verifies_sha256_and_replaces_binary() {
    let new_binary = minimal_valid_arm64_elf(b"new-v99-payload");
    let digest = sha256_hex(&new_binary);
    let sums = format!("{digest}  honk-android-arm64\n");
    let (addr, server) = spawn_mock_http_server(move |path| match path {
        "/api/latest" => (403, vec![], b"rate limit exceeded".to_vec()),
        "/web/latest" => (
            302,
            vec![(
                "Location".into(),
                "/Wcoom/honk-android/releases/tag/v99.1.0".into(),
            )],
            vec![],
        ),
        "/download/v99.1.0/SHA256SUMS.txt" => (200, vec![], sums.as_bytes().to_vec()),
        "/download/v99.1.0/honk-android-arm64" => (
            302,
            vec![("Location".into(), "/cdn/honk-v99.bin".into())],
            vec![],
        ),
        "/cdn/honk-v99.bin" => (200, vec![], new_binary.clone()),
        _ => (404, vec![], b"not found".to_vec()),
    })
    .await;

    let target_dir = tempfile::tempdir().unwrap();
    let target_bin = target_dir.path().join("honk");
    std::fs::write(&target_bin, b"old-honk").unwrap();

    let (_control, state, _data_dir) = test_state_with_update_config(
        CoreUpdateConfig {
            api_latest_url: format!("http://{addr}/api/latest"),
            web_latest_url: format!("http://{addr}/web/latest"),
            download_base_url: format!("http://{addr}/download"),
            expected_elf_machine: Some(EM_AARCH64),
            verify_exec: false,
            skip_restart: true,
            target_path_override: Some(target_bin.clone()),
            ..CoreUpdateConfig::default()
        },
        "",
    )
    .await;

    let app = crate::native_api::router(Arc::clone(&state));
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/x-honk/core/update")
                .header("Host", "127.0.0.1:9527")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 16384)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["status"], "updated");
    assert_eq!(body["installed_version"], "v99.1.0");
    assert_eq!(body["sha256"], digest);
    assert_eq!(
        sha256_hex(&std::fs::read(&target_bin).unwrap()),
        digest
    );

    server.abort();
}

#[tokio::test]
async fn apply_rejects_checksum_mismatch_without_touching_binary() {
    let new_binary = minimal_valid_arm64_elf(b"corrupted-payload");
    let wrong_digest = "0".repeat(64);
    let sums = format!("{wrong_digest}  honk-android-arm64\n");
    let (addr, server) = spawn_mock_http_server(move |path| match path {
        "/api/latest" => (403, vec![], b"rate limit".to_vec()),
        "/web/latest" => (
            302,
            vec![(
                "Location".into(),
                "/Wcoom/honk-android/releases/tag/v99.2.0".into(),
            )],
            vec![],
        ),
        "/download/v99.2.0/SHA256SUMS.txt" => (200, vec![], sums.as_bytes().to_vec()),
        "/download/v99.2.0/honk-android-arm64" => (200, vec![], new_binary.clone()),
        _ => (404, vec![], vec![]),
    })
    .await;

    let target_dir = tempfile::tempdir().unwrap();
    let target_bin = target_dir.path().join("honk");
    std::fs::write(&target_bin, b"intact-original").unwrap();

    let (_control, state, _data_dir) = test_state_with_update_config(
        CoreUpdateConfig {
            api_latest_url: format!("http://{addr}/api/latest"),
            web_latest_url: format!("http://{addr}/web/latest"),
            download_base_url: format!("http://{addr}/download"),
            expected_elf_machine: Some(EM_AARCH64),
            verify_exec: false,
            skip_restart: true,
            target_path_override: Some(target_bin.clone()),
            ..CoreUpdateConfig::default()
        },
        "",
    )
    .await;

    let app = crate::native_api::router(Arc::clone(&state));
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/x-honk/core/update")
                .header("Host", "127.0.0.1:9527")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 16384)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["error"]["details"]["stage"], "checksum_mismatch");
    assert_eq!(std::fs::read(&target_bin).unwrap(), b"intact-original");

    server.abort();
}
