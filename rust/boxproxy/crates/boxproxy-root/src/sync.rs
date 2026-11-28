//! SHA-256 verified atomic binary sync for `boxctl` and `boxbpf` (Section 3.2).

use crate::shell::{posix_quote, RootExecutor};
use crate::RootResult;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Sync embedded asset bytes to `dst` if length or SHA-256 differs, then ensure `0755`.
pub async fn sync_binary_atomic(
    exec: &dyn RootExecutor,
    asset_bytes: &[u8],
    dst: &Path,
) -> RootResult<bool> {
    let want_sha = sha256_hex(asset_bytes);
    if let Ok(existing) = fs::read(dst) {
        if existing.len() == asset_bytes.len() && sha256_hex(&existing) == want_sha {
            let cmd = format!("chmod 0755 {}", posix_quote(&dst.to_string_lossy()));
            let _ = exec.exec(&cmd, Duration::from_secs(5)).await;
            return Ok(false);
        }
    }

    let parent = dst.parent().ok_or("binary destination has no parent dir")?;
    fs::create_dir_all(parent).map_err(|e| format!("create bin dir failed: {e}"))?;
    let name = dst
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bin");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(".{name}.{nonce}.tmp"));
    fs::write(&tmp, asset_bytes).map_err(|e| format!("write tmp binary failed: {e}"))?;

    let verify = fs::read(&tmp).map_err(|e| format!("readback tmp binary failed: {e}"))?;
    if sha256_hex(&verify) != want_sha {
        let _ = fs::remove_file(&tmp);
        return Err("binary readback SHA-256 mismatch".to_string());
    }

    if fs::rename(&tmp, dst).is_err() {
        let cmd = format!(
            "mv -f {} {} && chmod 0755 {}",
            posix_quote(&tmp.to_string_lossy()),
            posix_quote(&dst.to_string_lossy()),
            posix_quote(&dst.to_string_lossy()),
        );
        let out = exec.exec(&cmd, Duration::from_secs(15)).await?;
        if !out.ok() {
            let _ = fs::remove_file(&tmp);
            return Err(format!("root replace binary failed: {}", out.error_text()));
        }
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dst, fs::Permissions::from_mode(0o755));
        }
        let cmd = format!(
            "chown root:net_admin {} 2>/dev/null || true; chmod 0755 {}",
            posix_quote(&dst.to_string_lossy()),
            posix_quote(&dst.to_string_lossy())
        );
        let _ = exec.exec(&cmd, Duration::from_secs(5)).await;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::MockRootExecutor;
    use tempfile::tempdir;

    #[tokio::test]
    async fn sync_skips_when_sha256_matches() {
        let dir = tempdir().unwrap();
        let dst = dir.path().join("bin/boxctl");
        let mock = MockRootExecutor::default();

        assert!(sync_binary_atomic(&mock, b"hello-boxctl", &dst).await.unwrap());
        assert!(!sync_binary_atomic(&mock, b"hello-boxctl", &dst).await.unwrap());
        assert!(sync_binary_atomic(&mock, b"hello-boxctl-v2", &dst).await.unwrap());
    }
}
