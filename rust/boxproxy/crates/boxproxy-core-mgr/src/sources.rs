//! All 9 built-in core update sources (Section 3.5.1) + GitHub Release checker & atomic installer.

use crate::elf::{extract_core_binary, verify_release_sha256, PackageFormat};
use crate::version::{compare_versions, VersionCompareMode, VersionOrder};
use crate::MgrResult;
use boxproxy_root::{posix_quote, BoxctlClient, RootExecutor};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreUpdateSource {
    pub id: String,
    pub name: String,
    pub target_core: String,
    pub repo: String,
    pub tag_filter: Option<String>,
    pub include_prerelease: bool,
    pub asset_regex: String,
    pub package_format: PackageFormat,
    pub binary_name: String,
    pub compare_mode: VersionCompareMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub source_id: String,
    pub target_core: String,
    pub local_version: String,
    pub remote_version: String,
    pub asset_name: String,
    pub download_url: String,
    pub sha256_expected: Option<String>,
    pub update_available: bool,
}

/// Exact 9 built-in core update sources from `com.boxproxy.honk` v214 (`ki1.java`).
pub fn builtin_core_sources(mihomo_stable: bool, sing_box_stable: bool) -> Vec<CoreUpdateSource> {
    vec![
        CoreUpdateSource {
            id: "honk".into(),
            name: "Honk (Wcoom/honk-android)".into(),
            target_core: "honk".into(),
            repo: "Wcoom/honk-android".into(),
            tag_filter: None,
            include_prerelease: false,
            asset_regex: r"^honk-android-arm64$".into(),
            package_format: PackageFormat::Direct,
            binary_name: "honk".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "mihomo".into(),
            name: "Mihomo (MetaCubeX/mihomo)".into(),
            target_core: "mihomo".into(),
            repo: "MetaCubeX/mihomo".into(),
            tag_filter: if mihomo_stable {
                None
            } else {
                Some("Prerelease-Alpha".into())
            },
            include_prerelease: !mihomo_stable,
            asset_regex: if mihomo_stable {
                r"^mihomo-android-arm64-v8-v?[A-Za-z0-9._-]+\.gz$".into()
            } else {
                r"^mihomo-android-arm64-v8-alpha-[A-Za-z0-9._-]+\.gz$".into()
            },
            package_format: PackageFormat::Gzip,
            binary_name: "mihomo".into(),
            compare_mode: if mihomo_stable {
                VersionCompareMode::Version
            } else {
                VersionCompareMode::Build
            },
        },
        CoreUpdateSource {
            id: "mihomo_smart".into(),
            name: "Mihomo Smart (vernesong/mihomo)".into(),
            target_core: "mihomo".into(),
            repo: "vernesong/mihomo".into(),
            tag_filter: Some("Prerelease-Alpha".into()),
            include_prerelease: true,
            asset_regex: r"^mihomo-android-arm64-v8-alpha-smart-[A-Fa-f0-9]+\.gz$".into(),
            package_format: PackageFormat::Gzip,
            binary_name: "mihomo".into(),
            compare_mode: VersionCompareMode::Build,
        },
        CoreUpdateSource {
            id: "meow".into(),
            name: "Meow (meow-rs/meow-rs)".into(),
            target_core: "meow".into(),
            repo: "meow-rs/meow-rs".into(),
            tag_filter: None,
            include_prerelease: false,
            asset_regex: r"^meow-.+-aarch64-unknown-linux-musl\.tar\.gz$".into(),
            package_format: PackageFormat::TarGz,
            binary_name: "meow".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "sing-box".into(),
            name: "Sing-Box (SagerNet/sing-box)".into(),
            target_core: "sing-box".into(),
            repo: "SagerNet/sing-box".into(),
            tag_filter: None,
            include_prerelease: !sing_box_stable,
            asset_regex: r"^sing-box-.+-android-arm64\.tar\.gz$".into(),
            package_format: PackageFormat::TarGz,
            binary_name: "sing-box".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "sing-box_ref1nd".into(),
            name: "Sing-Box reF1nd".into(),
            target_core: "sing-box".into(),
            repo: "reF1nd/sing-box-releases".into(),
            tag_filter: None,
            include_prerelease: !sing_box_stable,
            asset_regex: r"^sing-box-.+-android-arm64\.tar\.gz$".into(),
            package_format: PackageFormat::TarGz,
            binary_name: "sing-box".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "xray".into(),
            name: "Xray-core (XTLS/Xray-core)".into(),
            target_core: "xray".into(),
            repo: "XTLS/Xray-core".into(),
            tag_filter: None,
            include_prerelease: true,
            asset_regex: r"^Xray-android-arm64-v8a\.zip$".into(),
            package_format: PackageFormat::Zip,
            binary_name: "xray".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "v2fly".into(),
            name: "V2Fly (v2fly/v2ray-core)".into(),
            target_core: "v2fly".into(),
            repo: "v2fly/v2ray-core".into(),
            tag_filter: None,
            include_prerelease: true,
            asset_regex: r"^v2ray-android-arm64-v8a\.zip$".into(),
            package_format: PackageFormat::Zip,
            binary_name: "v2ray".into(),
            compare_mode: VersionCompareMode::Version,
        },
        CoreUpdateSource {
            id: "hysteria".into(),
            name: "Hysteria (apernet/hysteria)".into(),
            target_core: "hysteria".into(),
            repo: "apernet/hysteria".into(),
            tag_filter: None,
            include_prerelease: true,
            asset_regex: r"hysteria-android-arm64$".into(),
            package_format: PackageFormat::Direct,
            binary_name: "hysteria".into(),
            compare_mode: VersionCompareMode::Version,
        },
    ]
}

pub fn apply_download_prefix(url: &str, enabled: bool, prefix: &str) -> String {
    let p = prefix.trim();
    if !enabled || p.is_empty() || url.starts_with(p) {
        return url.to_string();
    }
    if p.ends_with('/') {
        format!("{p}{url}")
    } else {
        format!("{p}/{url}")
    }
}

pub async fn check_github_source(
    source: &CoreUpdateSource,
    local_version: &str,
    github_token: Option<&str>,
) -> MgrResult<UpdateCheckResult> {
    let http = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("BoxProxy-Honk-Rust")
        .build()
        .map_err(|e| format!("build http client failed: {e}"))?;

    let api_url = if let Some(tag) = &source.tag_filter {
        format!("https://api.github.com/repos/{}/releases/tags/{tag}", source.repo)
    } else if source.include_prerelease {
        format!("https://api.github.com/repos/{}/releases?per_page=10", source.repo)
    } else {
        format!("https://api.github.com/repos/{}/releases/latest", source.repo)
    };

    let mut req = http.get(&api_url);
    if let Some(tok) = github_token.filter(|t| !t.trim().is_empty()) {
        req = req.bearer_auth(tok.trim());
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("fetch release {} failed: {e}", source.repo))?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API HTTP {}", resp.status()));
    }
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    let release = if let Some(arr) = body.as_array() {
        arr.first().cloned().ok_or("no releases found")?
    } else {
        body
    };

    let remote_version = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let re = Regex::new(&source.asset_regex)
        .map_err(|e| format!("invalid asset regex {}: {e}", source.asset_regex))?;
    let assets = release
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or("release has no assets")?;

    let mut matched_asset = None;
    for a in assets {
        let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if re.is_match(name) {
            let dl = a
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let digest = a
                .get("digest")
                .and_then(|v| v.as_str())
                .and_then(|d| d.strip_prefix("sha256:"))
                .map(ToOwned::to_owned);
            matched_asset = Some((name.to_string(), dl.to_string(), digest));
            break;
        }
    }

    let (asset_name, download_url, sha256_expected) =
        matched_asset.ok_or_else(|| format!("no matching arm64 asset in {}", source.repo))?;
    let order = compare_versions(local_version, &remote_version, source.compare_mode);
    let update_available = matches!(order, VersionOrder::Older) || local_version.trim().is_empty();

    Ok(UpdateCheckResult {
        source_id: source.id.clone(),
        target_core: source.target_core.clone(),
        local_version: local_version.to_string(),
        remote_version,
        asset_name,
        download_url,
        sha256_expected,
        update_available,
    })
}

/// Atomic root core binary install (`hi1.java:102-155`), with automatic restart if core is running.
#[allow(clippy::too_many_arguments)]
pub async fn install_core_binary_atomic(
    exec: &dyn RootExecutor,
    box_home: &Path,
    target_core: &str,
    package_bytes: &[u8],
    format: PackageFormat,
    expected_bin_name: &str,
    expected_sha256: Option<&str>,
    restart_if_running: bool,
) -> MgrResult<usize> {
    if let Some(sha) = expected_sha256 {
        verify_release_sha256(package_bytes, sha)?;
    }
    let elf_bytes = extract_core_binary(package_bytes, format, expected_bin_name)?;

    let cache_dir = box_home.join("run/state/cache");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("create cache dir failed: {e}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let staged = cache_dir.join(format!(".{target_core}.{nonce}.staged"));
    fs::write(&staged, &elf_bytes).map_err(|e| format!("write staged core failed: {e}"))?;

    let bin_dir = box_home.join("bin");
    let new_path = bin_dir.join(format!(".{target_core}.new"));
    let dst_path = bin_dir.join(target_core);

    let cmd = format!(
        "mkdir -p {bin} && cp -f {src} {new} && (chown root:net_admin {new} 2>/dev/null || true) && chmod 0755 {new} && mv -f {new} {dst}",
        bin = posix_quote(&bin_dir.to_string_lossy()),
        src = posix_quote(&staged.to_string_lossy()),
        new = posix_quote(&new_path.to_string_lossy()),
        dst = posix_quote(&dst_path.to_string_lossy()),
    );
    let out = exec.exec(&cmd, Duration::from_secs(30)).await?;
    let _ = fs::remove_file(&staged);
    if !out.ok() {
        return Err(format!("atomic core install failed: {}", out.error_text()));
    }

    if restart_if_running {
        let boxctl = BoxctlClient::new(box_home);
        boxctl.restart(exec).await?;
    }

    Ok(elf_bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_all_nine_builtin_sources_with_valid_regex() {
        let sources = builtin_core_sources(true, true);
        assert_eq!(sources.len(), 9);
        for s in &sources {
            assert!(Regex::new(&s.asset_regex).is_ok(), "bad regex for {}", s.id);
        }
        assert_eq!(
            apply_download_prefix("https://github.com/a/b", true, "https://ghproxy.net"),
            "https://ghproxy.net/https://github.com/a/b"
        );
    }
}
