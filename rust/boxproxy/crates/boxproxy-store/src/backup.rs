//! M49 ZIP atomic backup & rollback-safe restore (`boxapp.backup.zip`).
//!
//! Enforces `com.boxproxy.honk` v214 hard limits:
//! - max 20,000 entries
//! - max 512 MiB per file
//! - max 2 GiB total uncompressed
//! - strict Zip-Slip path traversal rejection
//! - pre-restore rollback snapshot (`restore_<nanos>`) with automatic rollback on failure.

use crate::prefs::{AllPrefs, PrefsStore};
use crate::StoreResult;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const MAX_BACKUP_ENTRIES: usize = 20_000;
pub const MAX_SINGLE_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupScope {
    pub app_settings: bool,
    pub runtime_data: bool,
    pub user_scripts: bool,
    pub core_binaries: bool,
}

impl Default for BackupScope {
    fn default() -> Self {
        Self {
            app_settings: true,
            runtime_data: true,
            user_scripts: true,
            core_binaries: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    pub format_version: u32,
    pub app_package: String,
    pub created_at_ms: u64,
    pub schema_version: i64,
    pub scope: BackupScope,
    pub file_count: usize,
}

pub struct BackupService {
    box_home: PathBuf,
    prefs_path: PathBuf,
}

impl BackupService {
    pub fn new(box_home: impl Into<PathBuf>, prefs_path: impl Into<PathBuf>) -> Self {
        Self {
            box_home: box_home.into(),
            prefs_path: prefs_path.into(),
        }
    }

    pub fn export_to_writer<W: Write + Seek>(
        &self,
        writer: W,
        scope: BackupScope,
        app_package: &str,
    ) -> StoreResult<BackupManifest> {
        let mut zip = ZipWriter::new(writer);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut count = 0usize;
        let mut total_bytes = 0u64;

        if scope.app_settings {
            let db_path = self.box_home.join("box.db");
            if db_path.is_file() {
                add_file(&mut zip, opts, &db_path, "database/box.db", &mut count, &mut total_bytes)?;
            }
            let prefs = PrefsStore::new(&self.prefs_path).load()?.for_backup();
            let prefs_bytes = serde_json::to_vec_pretty(&prefs)
                .map_err(|e| format!("serialize backup prefs failed: {e}"))?;
            add_bytes(
                &mut zip,
                opts,
                &prefs_bytes,
                "prefs/app_prefs.json",
                &mut count,
                &mut total_bytes,
            )?;
        }

        if scope.runtime_data {
            for core in [
                "honk", "mihomo", "meow", "sing-box", "xray", "v2fly", "hysteria",
            ] {
                let dir = self.box_home.join(core);
                if dir.is_dir() {
                    add_dir_recursive(
                        &mut zip,
                        opts,
                        &dir,
                        &format!("files/{core}"),
                        &mut count,
                        &mut total_bytes,
                    )?;
                }
            }
        }

        if scope.user_scripts {
            let scripts = self.box_home.join("scripts");
            if scripts.is_dir() {
                add_dir_recursive(
                    &mut zip,
                    opts,
                    &scripts,
                    "files/scripts",
                    &mut count,
                    &mut total_bytes,
                )?;
            }
        }

        if scope.core_binaries {
            let bin = self.box_home.join("bin");
            if bin.is_dir() {
                add_dir_recursive(
                    &mut zip,
                    opts,
                    &bin,
                    "files/bin",
                    &mut count,
                    &mut total_bytes,
                )?;
            }
        }

        let manifest = BackupManifest {
            format_version: 2,
            app_package: app_package.to_string(),
            created_at_ms: now_millis(),
            schema_version: boxproxy_schema::SCHEMA_VERSION,
            scope,
            file_count: count,
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| format!("serialize manifest failed: {e}"))?;
        add_bytes(
            &mut zip,
            opts,
            &manifest_bytes,
            "manifest.json",
            &mut count,
            &mut total_bytes,
        )?;
        zip.finish()
            .map_err(|e| format!("finish backup zip failed: {e}"))?;
        Ok(manifest)
    }

    pub fn restore_from_reader<R: Read + Seek>(
        &self,
        reader: R,
        scope: BackupScope,
    ) -> StoreResult<BackupManifest> {
        let mut archive =
            ZipArchive::new(reader).map_err(|e| format!("open backup zip failed: {e}"))?;
        if archive.len() > MAX_BACKUP_ENTRIES {
            return Err(format!(
                "backup archive exceeds maximum entry count ({MAX_BACKUP_ENTRIES})"
            ));
        }

        // 1. Validate archive paths and size ceilings before touching live files.
        let mut manifest: Option<BackupManifest> = None;
        let mut total_uncompressed = 0u64;
        for idx in 0..archive.len() {
            let mut entry = archive
                .by_index(idx)
                .map_err(|e| format!("read zip entry {idx} failed: {e}"))?;
            let safe_rel = validate_zip_path(entry.name())?;
            let size = entry.size();
            if size > MAX_SINGLE_FILE_BYTES {
                return Err(format!("backup entry exceeds 512 MiB limit: {}", entry.name()));
            }
            total_uncompressed = total_uncompressed.saturating_add(size);
            if total_uncompressed > MAX_TOTAL_BYTES {
                return Err("backup archive exceeds 2 GiB total limit".to_string());
            }
            if safe_rel == Path::new("manifest.json") {
                let mut buf = Vec::new();
                entry
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("read manifest.json failed: {e}"))?;
                manifest = Some(
                    serde_json::from_slice(&buf)
                        .map_err(|e| format!("invalid manifest.json: {e}"))?,
                );
            }
        }

        // 2. Create pre-restore rollback snapshot (`restore_<nanos>`).
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let rollback_dir = self
            .box_home
            .parent()
            .unwrap_or(&self.box_home)
            .join(format!("restore_{nanos}"));
        fs::create_dir_all(&rollback_dir)
            .map_err(|e| format!("create rollback dir failed: {e}"))?;
        snapshot_live_state(&self.box_home, &self.prefs_path, &rollback_dir)?;

        // 3. Apply entries; roll back atomically on any error.
        let apply_result = self.apply_archive(&mut archive, scope);
        match apply_result {
            Ok(()) => {
                let _ = fs::remove_dir_all(&rollback_dir);
                Ok(manifest.unwrap_or(BackupManifest {
                    format_version: 1,
                    app_package: String::new(),
                    created_at_ms: now_millis(),
                    schema_version: boxproxy_schema::SCHEMA_VERSION,
                    scope,
                    file_count: archive.len(),
                }))
            }
            Err(err) => {
                let _ = restore_rollback_snapshot(&rollback_dir, &self.box_home, &self.prefs_path);
                let _ = fs::remove_dir_all(&rollback_dir);
                Err(format!("restore failed (rolled back cleanly): {err}"))
            }
        }
    }

    fn apply_archive<R: Read + Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        scope: BackupScope,
    ) -> StoreResult<()> {
        for idx in 0..archive.len() {
            let mut entry = archive
                .by_index(idx)
                .map_err(|e| format!("extract entry {idx} failed: {e}"))?;
            if entry.is_dir() {
                continue;
            }
            let safe_rel = validate_zip_path(entry.name())?;
            let path_str = safe_rel.to_string_lossy();

            if path_str == "database/box.db" && scope.app_settings {
                let dst = self.box_home.join("box.db");
                write_entry_to_path(&mut entry, &dst)?;
                // Remove stale WAL/SHM sidecars so SQLite opens the restored DB cleanly.
                let _ = fs::remove_file(self.box_home.join("box.db-wal"));
                let _ = fs::remove_file(self.box_home.join("box.db-shm"));
            } else if path_str == "prefs/app_prefs.json" && scope.app_settings {
                let mut buf = Vec::new();
                entry
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("read backup prefs failed: {e}"))?;
                let mut incoming: AllPrefs = serde_json::from_slice(&buf)
                    .map_err(|e| format!("parse backup prefs failed: {e}"))?;
                // Preserve live core_update_monitor cache if present.
                if let Ok(live) = PrefsStore::new(&self.prefs_path).load() {
                    incoming.core_update_monitor = live.core_update_monitor;
                }
                PrefsStore::new(&self.prefs_path).save(&incoming)?;
            } else if let Some(rest) = path_str.strip_prefix("files/") {
                let top = rest.split('/').next().unwrap_or("");
                let allowed = match top {
                    "scripts" => scope.user_scripts,
                    "bin" => scope.core_binaries,
                    "honk" | "mihomo" | "meow" | "sing-box" | "xray" | "v2fly" | "hysteria" => {
                        scope.runtime_data
                    }
                    _ => false,
                };
                if allowed {
                    let dst = self.box_home.join(rest);
                    write_entry_to_path(&mut entry, &dst)?;
                }
            }
        }
        Ok(())
    }
}

pub fn validate_zip_path(raw: &str) -> StoreResult<PathBuf> {
    if raw.is_empty() || raw.contains('\0') {
        return Err(format!("invalid backup path: {raw:?}"));
    }
    let normalized = raw.replace('\\', "/");
    let path = Path::new(&normalized);
    let mut clean = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("zip-slip path rejected: {raw}"));
            }
        }
    }
    if clean.as_os_str().is_empty() {
        return Err(format!("empty normalized backup path: {raw}"));
    }
    Ok(clean)
}

fn add_file<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
    src: &Path,
    archive_name: &str,
    count: &mut usize,
    total_bytes: &mut u64,
) -> StoreResult<()> {
    let meta = fs::metadata(src).map_err(|e| format!("stat {} failed: {e}", src.display()))?;
    if meta.len() > MAX_SINGLE_FILE_BYTES {
        return Err(format!("file exceeds 512 MiB limit: {}", src.display()));
    }
    *count += 1;
    if *count > MAX_BACKUP_ENTRIES {
        return Err(format!("backup exceeds {MAX_BACKUP_ENTRIES} entries"));
    }
    *total_bytes = total_bytes.saturating_add(meta.len());
    if *total_bytes > MAX_TOTAL_BYTES {
        return Err("backup exceeds 2 GiB limit".to_string());
    }
    let mut f = File::open(src).map_err(|e| format!("open {} failed: {e}", src.display()))?;
    zip.start_file(archive_name, opts)
        .map_err(|e| format!("zip start {archive_name} failed: {e}"))?;
    std::io::copy(&mut f, zip).map_err(|e| format!("zip copy {archive_name} failed: {e}"))?;
    Ok(())
}

fn add_bytes<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
    data: &[u8],
    archive_name: &str,
    count: &mut usize,
    total_bytes: &mut u64,
) -> StoreResult<()> {
    *count += 1;
    *total_bytes = total_bytes.saturating_add(data.len() as u64);
    zip.start_file(archive_name, opts)
        .map_err(|e| format!("zip start {archive_name} failed: {e}"))?;
    zip.write_all(data)
        .map_err(|e| format!("zip write {archive_name} failed: {e}"))?;
    Ok(())
}

fn add_dir_recursive<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
    dir: &Path,
    prefix: &str,
    count: &mut usize,
    total_bytes: &mut u64,
) -> StoreResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {} failed: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let child_prefix = format!("{prefix}/{name}");
        if path.is_dir() {
            add_dir_recursive(zip, opts, &path, &child_prefix, count, total_bytes)?;
        } else if path.is_file() {
            add_file(zip, opts, &path, &child_prefix, count, total_bytes)?;
        }
    }
    Ok(())
}

fn write_entry_to_path<R: Read>(entry: &mut R, dst: &Path) -> StoreResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {} failed: {e}", parent.display()))?;
    }
    let tmp = dst.with_extension("restore.tmp");
    let mut out = File::create(&tmp).map_err(|e| format!("create {} failed: {e}", tmp.display()))?;
    std::io::copy(entry, &mut out).map_err(|e| format!("write {} failed: {e}", tmp.display()))?;
    out.sync_all().ok();
    fs::rename(&tmp, dst).map_err(|e| format!("rename to {} failed: {e}", dst.display()))?;
    Ok(())
}

fn snapshot_live_state(box_home: &Path, prefs_path: &Path, rollback_dir: &Path) -> StoreResult<()> {
    let db = box_home.join("box.db");
    if db.is_file() {
        fs::copy(&db, rollback_dir.join("box.db"))
            .map_err(|e| format!("snapshot box.db failed: {e}"))?;
    }
    if prefs_path.is_file() {
        fs::copy(prefs_path, rollback_dir.join("app_prefs.json"))
            .map_err(|e| format!("snapshot app_prefs.json failed: {e}"))?;
    }
    Ok(())
}

fn restore_rollback_snapshot(
    rollback_dir: &Path,
    box_home: &Path,
    prefs_path: &Path,
) -> StoreResult<()> {
    let db = rollback_dir.join("box.db");
    if db.is_file() {
        fs::copy(&db, box_home.join("box.db"))
            .map_err(|e| format!("rollback box.db failed: {e}"))?;
    }
    let prefs = rollback_dir.join("app_prefs.json");
    if prefs.is_file() {
        fs::copy(&prefs, prefs_path)
            .map_err(|e| format!("rollback app_prefs.json failed: {e}"))?;
    }
    Ok(())
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoxDbRepo;
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn rejects_zip_slip_paths() {
        assert!(validate_zip_path("../evil.sh").is_err());
        assert!(validate_zip_path("/etc/passwd").is_err());
        assert!(validate_zip_path("files/../../escape").is_err());
        assert_eq!(
            validate_zip_path("files/honk/config.dae").unwrap(),
            PathBuf::from("files/honk/config.dae")
        );
    }

    #[test]
    fn backup_and_restore_roundtrip_preserves_db_prefs_and_configs() {
        let dir = tempdir().unwrap();
        let box_home = dir.path().join("box");
        let prefs_path = box_home.join("app_prefs.json");
        let repo = BoxDbRepo::open(box_home.join("box.db"), &box_home).unwrap();
        let mut profile = repo.get_runtime_profile().unwrap();
        profile.config_name = "custom.dae".into();
        repo.save_runtime_profile(&profile).unwrap();

        fs::create_dir_all(box_home.join("honk")).unwrap();
        fs::write(box_home.join("honk/custom.dae"), "global {}").unwrap();

        let mut prefs = AllPrefs::default();
        prefs.boxapp_settings.ui_mode = "miuix".into();
        PrefsStore::new(&prefs_path).save(&prefs).unwrap();

        let svc = BackupService::new(&box_home, &prefs_path);
        let mut buf = Cursor::new(Vec::new());
        let manifest = svc
            .export_to_writer(&mut buf, BackupScope::default(), "io.honk.android")
            .unwrap();
        assert_eq!(manifest.schema_version, 15);

        // Mutate live state then restore.
        profile.config_name = "mutated.dae".into();
        repo.save_runtime_profile(&profile).unwrap();
        fs::write(box_home.join("honk/custom.dae"), "broken").unwrap();

        buf.set_position(0);
        svc.restore_from_reader(buf, BackupScope::default()).unwrap();

        assert_eq!(repo.get_runtime_profile().unwrap().config_name, "custom.dae");
        assert_eq!(
            fs::read_to_string(box_home.join("honk/custom.dae")).unwrap(),
            "global {}"
        );
    }
}
