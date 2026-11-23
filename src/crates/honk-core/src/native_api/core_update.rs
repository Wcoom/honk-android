//! Core self-check and atomic binary update against `Wcoom/honk-android` GitHub Releases.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use axum::Json;
use axum::extract::Request;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use honk_outbound::group::ScoreContinuation;
use parking_lot::Mutex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::time::Instant;

use super::{
    ApiError, ErrorCode, NativeState, body, error, parse_query, timestamp, types::RequestId,
};
use crate::download_route::{self, Detour, Failed, Outbounds};
use crate::marked_http::{self, Deadline, Reply};

pub(super) const GITHUB_REPO: &str = "Wcoom/honk-android";
const DEFAULT_API_LATEST_URL: &str =
    "https://api.github.com/repos/Wcoom/honk-android/releases/latest";
const DEFAULT_WEB_LATEST_URL: &str = "https://github.com/Wcoom/honk-android/releases/latest";
const DEFAULT_DOWNLOAD_BASE_URL: &str =
    "https://github.com/Wcoom/honk-android/releases/download";
const DEFAULT_ASSET_NAME: &str = "honk-android-arm64";
const CHECKSUM_FILE_NAME: &str = "SHA256SUMS.txt";

const CHECK_TIMEOUT: Duration = Duration::from_secs(20);
const CHECKSUM_TIMEOUT: Duration = Duration::from_secs(15);
const DOWNLOAD_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TOTAL_TIMEOUT: Duration = Duration::from_secs(600);
const MAX_API_JSON_BYTES: usize = 512 * 1024;
const MAX_CHECKSUM_BYTES: usize = 64 * 1024;
const MAX_CORE_BINARY_BYTES: usize = 128 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LSB: u8 = 1;
const EM_AARCH64: u16 = 0x00b7;

#[derive(Clone, Debug)]
pub(crate) struct CoreUpdateConfig {
    pub(crate) repo: &'static str,
    pub(crate) api_latest_url: String,
    pub(crate) web_latest_url: String,
    pub(crate) download_base_url: String,
    pub(crate) asset_name: &'static str,
    pub(crate) expected_elf_machine: Option<u16>,
    pub(crate) verify_exec: bool,
    pub(crate) skip_restart: bool,
    pub(crate) target_path_override: Option<PathBuf>,
}

impl Default for CoreUpdateConfig {
    fn default() -> Self {
        Self {
            repo: GITHUB_REPO,
            api_latest_url: DEFAULT_API_LATEST_URL.to_owned(),
            web_latest_url: DEFAULT_WEB_LATEST_URL.to_owned(),
            download_base_url: DEFAULT_DOWNLOAD_BASE_URL.to_owned(),
            asset_name: DEFAULT_ASSET_NAME,
            expected_elf_machine: current_expected_elf_machine(),
            verify_exec: true,
            skip_restart: false,
            target_path_override: None,
        }
    }
}

fn current_expected_elf_machine() -> Option<u16> {
    if cfg!(target_arch = "aarch64") {
        Some(EM_AARCH64)
    } else {
        None
    }
}

fn current_arch_supported() -> bool {
    cfg!(target_arch = "aarch64")
}

#[derive(Clone, Debug, Serialize)]
struct CachedError {
    code: &'static str,
    message: String,
}

#[derive(Clone, Debug)]
struct ReleaseMetadata {
    source: &'static str,
    tag_name: String,
    release_url: Option<String>,
    published_at: Option<String>,
    notes: Option<String>,
    asset_url: String,
    checksum_url: Option<String>,
    asset_size: Option<u64>,
    asset_sha256: Option<String>,
}

#[derive(Default)]
struct CachedCheck {
    checked_at: Option<SystemTime>,
    latest: Option<ReleaseMetadata>,
    last_error: Option<CachedError>,
    installed_version: Option<String>,
}

pub(crate) struct CoreUpdateService {
    config: CoreUpdateConfig,
    cache: Mutex<CachedCheck>,
    check_lock: tokio::sync::Mutex<()>,
    apply_lock: tokio::sync::Mutex<()>,
    applying: AtomicBool,
}

impl Default for CoreUpdateService {
    fn default() -> Self {
        Self::with_config(CoreUpdateConfig::default())
    }
}

impl CoreUpdateService {
    pub(crate) fn with_config(config: CoreUpdateConfig) -> Self {
        Self {
            config,
            cache: Mutex::new(CachedCheck::default()),
            check_lock: tokio::sync::Mutex::new(()),
            apply_lock: tokio::sync::Mutex::new(()),
            applying: AtomicBool::new(false),
        }
    }

    pub(super) fn capability(&self, admin_authorized: bool) -> Value {
        let (_, can_apply, reason) = self.inspect_target(admin_authorized);
        json!({
            "available": true,
            "can_apply": can_apply,
            "cannot_apply_reason": reason,
            "repo": self.config.repo,
            "asset": self.config.asset_name,
            "target": crate::TARGET,
        })
    }

    fn inspect_target(&self, admin_authorized: bool) -> (Option<PathBuf>, bool, Option<&'static str>) {
        let exe = self
            .config
            .target_path_override
            .clone()
            .or_else(resolve_current_executable_path);
        let Some(exe_path) = exe else {
            return (None, false, Some("executable_unavailable"));
        };
        if !admin_authorized {
            return (Some(exe_path), false, Some("admin_authorization_required"));
        }
        if self.config.target_path_override.is_none() && !current_arch_supported() {
            return (Some(exe_path), false, Some("unsupported_target"));
        }
        let Some(parent) = exe_path.parent() else {
            return (Some(exe_path), false, Some("executable_dir_unavailable"));
        };
        if !dir_is_writable(parent) {
            return (Some(exe_path), false, Some("executable_dir_read_only"));
        }
        (Some(exe_path), true, None)
    }

    fn status_json(&self, state: &NativeState) -> Value {
        let admin = state.settings.admin_authorized();
        let (exe_path, can_apply, cannot_apply_reason) = self.inspect_target(admin);
        let applying = self.applying.load(Ordering::Acquire);
        let cache = self.cache.lock();
        let active_version = cache
            .installed_version
            .as_deref()
            .unwrap_or(crate::VERSION)
            .to_owned();
        let latest = cache.latest.as_ref();
        let has_update = latest
            .is_some_and(|rel| version_is_newer(&active_version, &rel.tag_name));
        json!({
            "observed_at": timestamp(SystemTime::now()),
            "repo": self.config.repo,
            "current_version": crate::VERSION,
            "current_revision": (!crate::REVISION.is_empty()).then_some(crate::REVISION),
            "target": crate::TARGET,
            "asset_name": self.config.asset_name,
            "executable_path": exe_path.as_ref().map(|p| p.display().to_string()),
            "can_apply": can_apply,
            "cannot_apply_reason": cannot_apply_reason,
            "applying": applying,
            "checked_at": cache.checked_at.map(timestamp),
            "source": latest.map(|rel| rel.source),
            "latest_version": latest.map(|rel| rel.tag_name.as_str()),
            "has_update": has_update,
            "release_url": latest.and_then(|rel| rel.release_url.as_deref()),
            "published_at": latest.and_then(|rel| rel.published_at.as_deref()),
            "notes": latest.and_then(|rel| rel.notes.as_deref()),
            "asset_size_bytes": latest.and_then(|rel| rel.asset_size.map(|n| n.to_string())),
            "asset_sha256": latest.and_then(|rel| rel.asset_sha256.as_deref()),
            "last_error": cache.last_error.as_ref(),
        })
    }

    async fn check_latest(&self, state: &NativeState) -> Result<ReleaseMetadata, UpdateFailure> {
        let _guard = self.check_lock.lock().await;
        let result = fetch_latest_release(state, &self.config).await;
        let mut cache = self.cache.lock();
        cache.checked_at = Some(SystemTime::now());
        match &result {
            Ok(metadata) => {
                cache.latest = Some(metadata.clone());
                cache.last_error = None;
            }
            Err(failure) => {
                cache.last_error = Some(CachedError {
                    code: failure.stage,
                    message: failure.detail.clone(),
                });
            }
        }
        result
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyBody {
    #[serde(default)]
    force: bool,
}

pub(super) async fn get(
    state: &NativeState,
    uri: &Uri,
    id: &RequestId,
) -> Result<Response, ApiError> {
    let query = parse_query(uri, &["refresh"], id)?;
    let refresh = match query.get("refresh").map(String::as_str) {
        None | Some("0" | "false") => false,
        Some("1" | "true") => true,
        Some(_) => {
            return Err(error(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                "Invalid query parameters",
                id,
            ));
        }
    };
    if refresh {
        state
            .core_update
            .check_latest(state)
            .await
            .map_err(|failure| failure.into_api_error())?;
    }
    Ok(Json(state.core_update.status_json(state)).into_response())
}

pub(super) async fn apply(
    state: &Arc<NativeState>,
    request: Request,
    id: &RequestId,
) -> Result<Response, ApiError> {
    parse_query(request.uri(), &[], id)?;
    if !state.settings.admin_authorized() {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            ErrorCode::PermissionDenied,
            "Core updates require administrative authorization",
            None,
        ));
    }
    let bytes = body::buffered(request.into_body()).await;
    let params = if bytes.is_empty() {
        ApplyBody { force: false }
    } else {
        body::decode(&bytes, || {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                "Invalid core update request body",
                None,
            )
        })?
    };

    let (exe_path, can_apply, cannot_apply_reason) =
        state.core_update.inspect_target(state.settings.admin_authorized());
    let Some(exe_path) = exe_path.filter(|_| can_apply) else {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            ErrorCode::StateConflict,
            "Core self-update cannot be applied in this environment",
            None,
        )
        .with_details(json!({"reason": cannot_apply_reason.unwrap_or("unavailable")})));
    };

    let Ok(_apply_guard) = state.core_update.apply_lock.try_lock() else {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            ErrorCode::StateConflict,
            "A core update is already in progress",
            None,
        ));
    };

    struct ApplyingReset<'a>(&'a AtomicBool);
    impl Drop for ApplyingReset<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }
    state.core_update.applying.store(true, Ordering::Release);
    let _applying_reset = ApplyingReset(&state.core_update.applying);

    let metadata = state
        .core_update
        .check_latest(state)
        .await
        .map_err(|failure| failure.into_api_error())?;

    let has_update = version_is_newer(crate::VERSION, &metadata.tag_name);
    if !has_update && !params.force {
        return Ok(Json(json!({
            "status": "up_to_date",
            "restarting": false,
            "current_version": crate::VERSION,
            "latest_version": metadata.tag_name,
            "has_update": false,
            "release_url": metadata.release_url,
        }))
        .into_response());
    }

    let expected_sha256 = resolve_expected_sha256(state, &state.core_update.config, &metadata)
        .await
        .map_err(|failure| failure.into_api_error())?;

    let downloaded = download_release_asset(state, &metadata.asset_url)
        .await
        .map_err(|failure| failure.into_api_error())?;

    let actual_sha256 = sha256_hex(&downloaded);
    if !actual_sha256.eq_ignore_ascii_case(&expected_sha256) {
        return Err(UpdateFailure::new(
            "checksum_mismatch",
            format!("expected {expected_sha256}, got {actual_sha256}"),
        )
        .into_api_error());
    }

    validate_elf_binary(&downloaded, state.core_update.config.expected_elf_machine)
        .map_err(|failure| failure.into_api_error())?;

    let staged_path = stage_path_for(&exe_path)?;
    struct StagedCleanup(Option<PathBuf>);
    impl Drop for StagedCleanup {
        fn drop(&mut self) {
            if let Some(path) = self.0.take() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    let mut cleanup = StagedCleanup(Some(staged_path.clone()));

    let exe_clone = exe_path.clone();
    let staged_clone = staged_path.clone();
    let verify_exec = state.core_update.config.verify_exec;
    let payload_len = downloaded.len();
    tokio::task::spawn_blocking(move || -> Result<(), UpdateFailure> {
        write_staged_executable(&staged_clone, &downloaded)?;
        drop(downloaded);
        if verify_exec {
            verify_staged_binary(&staged_clone)?;
        }
        preserve_owner_if_root(&exe_clone, &staged_clone);
        std::fs::rename(&staged_clone, &exe_clone).map_err(|error| {
            UpdateFailure::new(
                "install_failed",
                format!("atomic replace {} failed: {error}", exe_clone.display()),
            )
        })?;
        sync_parent_dir(&exe_clone);
        Ok(())
    })
    .await
    .map_err(|error| UpdateFailure::new("install_failed", error.to_string()).into_api_error())?
    .map_err(|failure| failure.into_api_error())?;

    cleanup.0 = None;
    {
        let mut cache = state.core_update.cache.lock();
        cache.installed_version = Some(metadata.tag_name.clone());
        if let Some(rel) = cache.latest.as_mut() {
            rel.asset_sha256.get_or_insert_with(|| actual_sha256.clone());
        }
    }

    let restarting = !state.core_update.config.skip_restart;
    if restarting {
        let restart_exe = exe_path.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            if let Err(error) = restart_self(&restart_exe) {
                tracing::error!(%error, "failed to launch updated honk-core process");
            }
        });
    }

    Ok(Json(json!({
        "status": if restarting { "restarting" } else { "updated" },
        "restarting": restarting,
        "previous_version": crate::VERSION,
        "installed_version": metadata.tag_name,
        "sha256": actual_sha256,
        "bytes": payload_len.to_string(),
        "release_url": metadata.release_url,
    }))
    .into_response())
}

#[derive(Debug)]
struct UpdateFailure {
    stage: &'static str,
    detail: String,
}

impl UpdateFailure {
    fn new(stage: &'static str, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }

    fn into_api_error(self) -> ApiError {
        let (status, code, msg): (StatusCode, ErrorCode, &'static str) = match self.stage {
            "checksum_mismatch" | "checksum_unavailable" | "invalid_binary" | "asset_missing"
            | "no_release_found" => (
                StatusCode::BAD_GATEWAY,
                ErrorCode::TemporarilyUnavailable,
                "Core update release validation failed",
            ),
            "install_failed" | "self_check_failed" => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::TemporarilyUnavailable,
                "Failed to install updated core binary",
            ),
            _ => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::TemporarilyUnavailable,
                "Failed to fetch core update information",
            ),
        };
        ApiError::new(status, code, msg, None)
            .with_details(json!({"stage": self.stage, "reason": self.detail}))
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: Option<String>,
    published_at: Option<String>,
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    size: Option<u64>,
    digest: Option<String>,
    browser_download_url: String,
}

async fn fetch_latest_release(
    state: &NativeState,
    config: &CoreUpdateConfig,
) -> Result<ReleaseMetadata, UpdateFailure> {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        format!("honk-core/{}", crate::VERSION)
            .parse()
            .expect("valid header"),
    );
    headers.insert(
        http::header::ACCEPT,
        "application/vnd.github+json".parse().expect("valid header"),
    );

    let api_url = Url::parse(&config.api_latest_url)
        .map_err(|e| UpdateFailure::new("invalid_url", e.to_string()))?;
    let api_reply = fetch_with_fallback(
        state,
        &api_url,
        &headers,
        Instant::now() + CHECK_TIMEOUT,
        MAX_API_JSON_BYTES,
        true,
        false,
    )
    .await;

    match api_reply {
        Ok(reply) if reply.status == StatusCode::OK => {
            let release: GithubRelease = serde_json::from_slice(&reply.body)
                .map_err(|e| UpdateFailure::new("invalid_response", e.to_string()))?;
            if release.draft || release.tag_name.trim().is_empty() {
                return Err(UpdateFailure::new("no_release_found", "no published release"));
            }
            let asset = release
                .assets
                .iter()
                .find(|a| a.name == config.asset_name)
                .ok_or_else(|| {
                    UpdateFailure::new(
                        "asset_missing",
                        format!("release {} lacks asset {}", release.tag_name, config.asset_name),
                    )
                })?;
            let checksum_url = release
                .assets
                .iter()
                .find(|a| a.name == CHECKSUM_FILE_NAME)
                .map(|a| a.browser_download_url.clone())
                .or_else(|| {
                    Some(format!(
                        "{}/{}/{}",
                        config.download_base_url.trim_end_matches('/'),
                        release.tag_name,
                        CHECKSUM_FILE_NAME
                    ))
                });
            let mut sha256 = asset
                .digest
                .as_deref()
                .and_then(|d| d.strip_prefix("sha256:"))
                .filter(|hex| is_valid_sha256_hex(hex))
                .map(str::to_ascii_lowercase);
            if sha256.is_none()
                && let Some(ref sum_url) = checksum_url
                && let Ok(parsed_sum) =
                    fetch_sha256_from_sums(state, sum_url, config.asset_name).await
            {
                sha256 = Some(parsed_sum);
            }
            let _ = release.prerelease;
            return Ok(ReleaseMetadata {
                source: "github_api",
                tag_name: release.tag_name,
                release_url: release.html_url,
                published_at: release.published_at,
                notes: release.body,
                asset_url: asset.browser_download_url.clone(),
                checksum_url,
                asset_size: asset.size,
                asset_sha256: sha256,
            });
        }
        Ok(reply)
            if matches!(
                reply.status,
                StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
            ) =>
        {
            tracing::info!(
                status = reply.status.as_u16(),
                "GitHub API rate-limited; falling back to releases/latest redirect"
            );
        }
        Ok(reply) if reply.status == StatusCode::NOT_FOUND => {
            return Err(UpdateFailure::new(
                "no_release_found",
                "GitHub release endpoint returned 404",
            ));
        }
        Ok(reply) => {
            return Err(UpdateFailure::new(
                download_route::status_stage(reply.status),
                format!("HTTP {}", reply.status),
            ));
        }
        Err(err) => return Err(err),
    }

    fetch_latest_via_web_redirect(state, config).await
}

async fn fetch_latest_via_web_redirect(
    state: &NativeState,
    config: &CoreUpdateConfig,
) -> Result<ReleaseMetadata, UpdateFailure> {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        format!("honk-core/{}", crate::VERSION)
            .parse()
            .expect("valid header"),
    );
    let web_url = Url::parse(&config.web_latest_url)
        .map_err(|e| UpdateFailure::new("invalid_url", e.to_string()))?;
    let reply = fetch_with_fallback(
        state,
        &web_url,
        &headers,
        Instant::now() + CHECK_TIMEOUT,
        64 * 1024,
        false,
        true,
    )
    .await?;

    if !marked_http::followed_redirect(reply.status) {
        return Err(UpdateFailure::new(
            "http_status_rejected",
            format!("expected redirect from releases/latest, got {}", reply.status),
        ));
    }
    let location = reply
        .location
        .as_ref()
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| UpdateFailure::new("no_release_found", "redirect without Location"))?;
    let next = web_url
        .join(location)
        .map_err(|e| UpdateFailure::new("invalid_response", e.to_string()))?;
    let tag_name = extract_tag_from_release_url(next.as_str()).ok_or_else(|| {
        UpdateFailure::new(
            "no_release_found",
            format!("no release tag in redirect target {}", next),
        )
    })?;
    let base = config.download_base_url.trim_end_matches('/');
    let checksum_url = format!("{base}/{tag_name}/{CHECKSUM_FILE_NAME}");
    let sha256 = fetch_sha256_from_sums(state, &checksum_url, config.asset_name)
        .await
        .ok();
    Ok(ReleaseMetadata {
        source: "github_web",
        tag_name: tag_name.clone(),
        release_url: Some(next.into()),
        published_at: None,
        notes: None,
        asset_url: format!("{base}/{tag_name}/{}", config.asset_name),
        checksum_url: Some(checksum_url),
        asset_size: None,
        asset_sha256: sha256,
    })
}

fn extract_tag_from_release_url(url: &str) -> Option<String> {
    let (_, after) = url.split_once("/releases/tag/")?;
    let tag = after.split(['?', '#']).next()?.trim_matches('/');
    (!tag.is_empty()).then(|| tag.to_owned())
}

async fn resolve_expected_sha256(
    state: &NativeState,
    config: &CoreUpdateConfig,
    metadata: &ReleaseMetadata,
) -> Result<String, UpdateFailure> {
    let from_sums = match metadata.checksum_url.as_deref() {
        Some(url) => match fetch_sha256_from_sums(state, url, config.asset_name).await {
            Ok(hex) => Some(hex),
            Err(err) if metadata.asset_sha256.is_some() => {
                tracing::debug!(error = ?err, "SHA256SUMS.txt fetch failed; using API digest");
                None
            }
            Err(err) => return Err(err),
        },
        None => None,
    };

    match (from_sums, metadata.asset_sha256.as_deref()) {
        (Some(sum), Some(api)) => {
            if !sum.eq_ignore_ascii_case(api) {
                return Err(UpdateFailure::new(
                    "checksum_mismatch",
                    "SHA256SUMS.txt disagrees with release asset digest",
                ));
            }
            Ok(sum)
        }
        (Some(sum), None) => Ok(sum),
        (None, Some(api)) => Ok(api.to_ascii_lowercase()),
        (None, None) => Err(UpdateFailure::new(
            "checksum_unavailable",
            "release provides neither SHA256SUMS.txt nor asset digest",
        )),
    }
}

async fn fetch_sha256_from_sums(
    state: &NativeState,
    url: &str,
    asset_name: &str,
) -> Result<String, UpdateFailure> {
    let parsed = Url::parse(url).map_err(|e| UpdateFailure::new("invalid_url", e.to_string()))?;
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        format!("honk-core/{}", crate::VERSION)
            .parse()
            .expect("valid header"),
    );
    let reply = fetch_with_fallback(
        state,
        &parsed,
        &headers,
        Instant::now() + CHECKSUM_TIMEOUT,
        MAX_CHECKSUM_BYTES,
        true,
        false,
    )
    .await?;
    if reply.status != StatusCode::OK {
        return Err(UpdateFailure::new(
            "checksum_unavailable",
            format!("checksum file returned HTTP {}", reply.status),
        ));
    }
    let text = std::str::from_utf8(&reply.body)
        .map_err(|_| UpdateFailure::new("checksum_unavailable", "checksum file is not UTF-8"))?;
    parse_sha256sums_for_asset(text, asset_name).ok_or_else(|| {
        UpdateFailure::new(
            "checksum_unavailable",
            format!("no SHA-256 entry for {asset_name}"),
        )
    })
}

async fn download_release_asset(
    state: &NativeState,
    url: &str,
) -> Result<Arc<[u8]>, UpdateFailure> {
    let parsed = Url::parse(url).map_err(|e| UpdateFailure::new("invalid_url", e.to_string()))?;
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::USER_AGENT,
        format!("honk-core/{}", crate::VERSION)
            .parse()
            .expect("valid header"),
    );
    let started = Instant::now();
    let deadline = Deadline {
        headers: started + DOWNLOAD_IDLE_TIMEOUT,
        idle: Some((DOWNLOAD_IDLE_TIMEOUT, started + DOWNLOAD_TOTAL_TIMEOUT)),
    };
    let reply = fetch_with_fallback(
        state,
        &parsed,
        &headers,
        deadline,
        MAX_CORE_BINARY_BYTES,
        true,
        false,
    )
    .await?;
    if reply.status != StatusCode::OK {
        return Err(UpdateFailure::new(
            download_route::status_stage(reply.status),
            format!("download returned HTTP {}", reply.status),
        ));
    }
    Ok(reply.body)
}

async fn fetch_with_fallback(
    state: &NativeState,
    initial_url: &Url,
    base_headers: &http::HeaderMap,
    deadline: impl Into<Deadline>,
    max_bytes: usize,
    follow_redirects: bool,
    allow_error_status_body: bool,
) -> Result<Reply, UpdateFailure> {
    let deadline = deadline.into();
    let outbounds = Outbounds {
        router: &state.traffic_router,
        config: &state.config,
        group_manager: &state.group_manager,
        proxy_registry: &state.proxy_registry,
        runtime_registry: &state.runtime_registry,
    };
    let mut url = initial_url.clone();
    let mut headers = base_headers.clone();
    let mut continuation: Option<ScoreContinuation> = None;

    let mut redirect_count = 0usize;
    loop {
        let req = download_route::Request {
            url: &url,
            headers: &headers,
            wants_body: if allow_error_status_body {
                |status, headers| {
                    !(marked_http::followed_redirect(status)
                        && headers.contains_key(http::header::LOCATION))
                }
            } else {
                |status, headers| {
                    status.is_success()
                        && !(marked_http::followed_redirect(status)
                            && headers.contains_key(http::header::LOCATION))
                }
            },
            deadline,
            max_bytes,
            bootstrap: None,
        };
        let reply = match outbounds
            .fetch(
                Detour::Routing,
                "core_update.download",
                "core update",
                &req,
                &mut continuation,
            )
            .await
        {
            Ok((reply, _, _)) => reply,
            Err(Failed::Route(_)) | Err(Failed::Stage(_, Some(_))) => {
                // Group has no usable node or proxied dial failed; try direct once.
                download_route::fetch_direct(&req)
                    .await
                    .map_err(|stage| UpdateFailure::new(stage, format!("fetch failed at {stage}")))?
            }
            Err(Failed::Stage(stage, None)) => {
                return Err(UpdateFailure::new(stage, format!("fetch failed at {stage}")));
            }
        };

        if follow_redirects
            && marked_http::followed_redirect(reply.status)
            && let Some(location) = reply.location.as_ref().and_then(|v| v.to_str().ok())
        {
            if redirect_count >= MAX_REDIRECTS {
                return Err(UpdateFailure::new(
                    "too_many_redirects",
                    "exceeded redirect limit",
                ));
            }
            redirect_count += 1;
            let next = url
                .join(location)
                .map_err(|e| UpdateFailure::new("invalid_redirect", e.to_string()))?;
            if !matches!(next.scheme(), "https" | "http") {
                return Err(UpdateFailure::new(
                    "invalid_redirect",
                    "unsupported redirect scheme",
                ));
            }
            if next.origin() != url.origin() {
                headers.remove(http::header::AUTHORIZATION);
                headers.remove(http::header::COOKIE);
            }
            url = next;
            continue;
        }
        return Ok(reply);
    }
}

fn is_valid_sha256_hex(hex: &str) -> bool {
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

pub(crate) fn parse_sha256sums_for_asset(text: &str, asset_name: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let Some(hex) = parts.next() else { continue };
        let Some(rest) = parts.next() else { continue };
        if !is_valid_sha256_hex(hex) {
            continue;
        }
        let candidate = rest.trim_start().trim_start_matches('*').trim();
        let file_name = Path::new(candidate)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(candidate);
        if candidate == asset_name || file_name == asset_name {
            return Some(hex.to_ascii_lowercase());
        }
    }
    None
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedVersion {
    core: [u64; 3],
    prerelease: Vec<String>,
    is_post_tag_dev: bool,
}

fn parse_version(raw: &str) -> Option<ParsedVersion> {
    let trimmed = raw.trim();
    let trimmed = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let no_build = trimmed.split('+').next().unwrap_or(trimmed);
    let (core_str, suffix) = match no_build.split_once('-') {
        Some((core, rest)) => (core, Some(rest)),
        None => (no_build, None),
    };
    let mut nums = core_str.split('.');
    let major = nums.next()?.parse::<u64>().ok()?;
    let minor = nums.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = nums.next().unwrap_or("0").parse::<u64>().ok()?;
    if nums.next().is_some() {
        return None;
    }

    let mut is_post_tag_dev = false;
    let mut prerelease = Vec::new();
    if let Some(rest) = suffix {
        if is_git_describe_suffix(rest) {
            is_post_tag_dev = true;
        } else if !rest.is_empty() {
            prerelease = rest.split('.').map(str::to_string).collect();
        }
    }
    Some(ParsedVersion {
        core: [major, minor, patch],
        prerelease,
        is_post_tag_dev,
    })
}

fn is_git_describe_suffix(suffix: &str) -> bool {
    let mut parts = suffix.split('-');
    let Some(commits) = parts.next() else { return false };
    let Some(hash_part) = parts.next() else { return false };
    let third = parts.next();
    if let Some(extra) = third
        && extra != "dirty"
    {
        return false;
    }
    if parts.next().is_some() {
        return false;
    }
    !commits.is_empty()
        && commits.bytes().all(|b| b.is_ascii_digit())
        && hash_part
            .strip_prefix('g')
            .is_some_and(|hex| hex.len() >= 4 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn compare_prerelease(a: &[String], b: &[String]) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.is_empty(), b.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            for (pa, pb) in a.iter().zip(b.iter()) {
                let ord = match (pa.parse::<u64>(), pb.parse::<u64>()) {
                    (Ok(na), Ok(nb)) => na.cmp(&nb),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => pa.cmp(pb),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            a.len().cmp(&b.len())
        }
    }
}

pub(crate) fn version_is_newer(current: &str, latest: &str) -> bool {
    if current.trim() == latest.trim() {
        return false;
    }
    let (Some(cur), Some(lat)) = (parse_version(current), parse_version(latest)) else {
        return false;
    };
    if lat.core != cur.core {
        return lat.core > cur.core;
    }
    if cur.is_post_tag_dev && lat.prerelease.is_empty() && !lat.is_post_tag_dev {
        // Post-tag git describe build (e.g. v1.0.5-3-gabc) already includes tag v1.0.5.
        return false;
    }
    compare_prerelease(&lat.prerelease, &cur.prerelease) == std::cmp::Ordering::Greater
}

fn validate_elf_binary(
    bytes: &[u8],
    expected_machine: Option<u16>,
) -> Result<(), UpdateFailure> {
    if bytes.len() < 64
        || &bytes[0..4] != ELF_MAGIC
        || bytes[4] != ELF_CLASS_64
        || bytes[5] != ELF_DATA_LSB
    {
        return Err(UpdateFailure::new(
            "invalid_binary",
            "downloaded file is not a valid 64-bit little-endian ELF executable",
        ));
    }
    if let Some(expected) = expected_machine {
        let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
        if machine != expected {
            return Err(UpdateFailure::new(
                "invalid_binary",
                format!("unexpected ELF machine architecture: 0x{machine:04x}"),
            ));
        }
    }
    Ok(())
}

fn resolve_current_executable_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let text = exe.to_string_lossy();
    let cleaned = text
        .strip_suffix(" (deleted)")
        .map(PathBuf::from)
        .unwrap_or(exe);
    cleaned.is_absolute().then_some(cleaned)
}

fn dir_is_writable(dir: &Path) -> bool {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt as _;
    let Ok(cstr) = CString::new(dir.as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: NUL-terminated path passed to POSIX access(2).
    unsafe { libc::access(cstr.as_ptr(), libc::W_OK | libc::X_OK) == 0 }
}

fn stage_path_for(exe_path: &Path) -> Result<PathBuf, ApiError> {
    let parent = exe_path.parent().ok_or_else(|| {
        UpdateFailure::new("install_failed", "executable has no parent directory").into_api_error()
    })?;
    let name = exe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("honk");
    Ok(parent.join(format!(".{name}.update.{}", std::process::id())))
}

fn write_staged_executable(staged: &Path, bytes: &[u8]) -> Result<(), UpdateFailure> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o755)
        .open(staged)
        .map_err(|error| {
            UpdateFailure::new(
                "install_failed",
                format!("create staged binary {} failed: {error}", staged.display()),
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        UpdateFailure::new(
            "install_failed",
            format!("write staged binary failed: {error}"),
        )
    })?;
    file.sync_all().map_err(|error| {
        UpdateFailure::new(
            "install_failed",
            format!("fsync staged binary failed: {error}"),
        )
    })?;
    Ok(())
}

fn verify_staged_binary(staged: &Path) -> Result<(), UpdateFailure> {
    let output = std::process::Command::new(staged)
        .arg("--version")
        .output()
        .map_err(|error| {
            UpdateFailure::new(
                "self_check_failed",
                format!("staged --version exec failed: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(UpdateFailure::new(
            "self_check_failed",
            format!("staged --version exited with {}", output.status),
        ));
    }
    Ok(())
}

fn preserve_owner_if_root(existing: &Path, staged: &Path) {
    use std::os::unix::fs::MetadataExt as _;
    // SAFETY: geteuid has no preconditions.
    if unsafe { libc::geteuid() } != 0 {
        return;
    }
    let Ok(meta) = std::fs::metadata(existing) else {
        return;
    };
    let Ok(cstr) = std::ffi::CString::new(std::os::unix::ffi::OsStrExt::as_bytes(
        staged.as_os_str(),
    )) else {
        return;
    };
    // SAFETY: NUL-terminated path passed to chown(2).
    unsafe {
        libc::chown(cstr.as_ptr(), meta.uid(), meta.gid());
    }
}

fn sync_parent_dir(exe_path: &Path) {
    if let Some(parent) = exe_path.parent()
        && let Ok(dir) = std::fs::File::open(parent)
    {
        let _ = dir.sync_all();
    }
}

fn open_max_fd_limit() -> libc::c_int {
    let mut rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: rlim points to valid rlimit storage.
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) };
    if rc == 0 && rlim.rlim_cur > 0 {
        rlim.rlim_cur.min(65536) as libc::c_int
    } else {
        4096
    }
}

fn resolve_run_dir(exe_path: &Path) -> Option<PathBuf> {
    if let Some(env_run) = std::env::var_os("HONK_RUN_DIR").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(env_run));
    }
    exe_path
        .parent()
        .and_then(Path::parent)
        .map(|box_dir| box_dir.join("run"))
        .filter(|run| run.is_dir())
}

fn restart_self(exe_path: &Path) -> anyhow::Result<()> {
    let in_systemd = std::env::var_os("INVOCATION_ID").is_some_and(|v| !v.is_empty())
        && std::env::var_os("HONK_RUN_DIR").is_none_or(|v| v.is_empty());
    if in_systemd {
        tracing::info!("core updated; sending SIGTERM for systemd restart");
        // SAFETY: sending SIGTERM to self triggers clean shutdown.
        unsafe { libc::kill(libc::getpid(), libc::SIGTERM) };
        return Ok(());
    }

    let args: Vec<std::ffi::OsString> = crate::startup_args()
        .map(|slice| slice.to_vec())
        .unwrap_or_else(|| std::env::args_os().skip(1).collect());
    let run_dir = resolve_run_dir(exe_path);
    let log_name = format!(
        "{}.log",
        exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("honk")
    );
    let log_stdio = run_dir
        .as_ref()
        .map(|dir| dir.join(&log_name))
        .filter(|path| path.is_file())
        .and_then(|path| OpenOptions::new().create(true).append(true).open(path).ok());

    let mut cmd = std::process::Command::new(exe_path);
    cmd.args(&args).stdin(std::process::Stdio::null());
    if let Some(log_file) = log_stdio {
        let stderr_file = log_file.try_clone()?;
        cmd.stdout(std::process::Stdio::from(log_file))
            .stderr(std::process::Stdio::from(stderr_file));
    } else {
        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    let max_fd = open_max_fd_limit();
    // SAFETY: pre_exec calls only async-signal-safe syscalls (setsid, syscall(close_range), close).
    unsafe {
        use std::os::unix::process::CommandExt as _;
        cmd.pre_exec(move || {
            let _ = libc::setsid();
            #[cfg(target_os = "linux")]
            let closed = libc::syscall(libc::SYS_close_range, 3usize, libc::c_uint::MAX as usize, 0usize);
            #[cfg(not(target_os = "linux"))]
            let closed = -1;
            if closed != 0 {
                for fd in 3..max_fd {
                    libc::close(fd);
                }
            }
            Ok(())
        });
    }

    let child = cmd.spawn()?;
    if let Some(dir) = run_dir {
        let box_pid = dir.join("box.pid");
        if box_pid.is_file() {
            let tmp_pid = dir.join(".box.pid.update.tmp");
            if std::fs::write(&tmp_pid, child.id().to_string()).is_ok() {
                let _ = std::fs::rename(&tmp_pid, &box_pid);
            }
        }
    }
    tracing::info!(
        child_pid = child.id(),
        path = %exe_path.display(),
        "spawned updated honk-core successor; draining current instance"
    );
    // SAFETY: sending SIGTERM to self triggers clean shutdown and lock release.
    unsafe { libc::kill(libc::getpid(), libc::SIGTERM) };
    Ok(())
}

#[cfg(test)]
mod tests;
