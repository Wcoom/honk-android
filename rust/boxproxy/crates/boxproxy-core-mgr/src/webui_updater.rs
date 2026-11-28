//! M39 Local WebUI dashboard ZIP updater (`hm7.java` / `g08.java`).

use crate::MgrResult;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use zip::ZipArchive;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebUiUpdateResult {
    pub target_dir: String,
    pub files_extracted: usize,
}

pub fn extract_webui_zip(zip_bytes: &[u8], target_dir: &Path) -> MgrResult<WebUiUpdateResult> {
    let mut archive =
        ZipArchive::new(Cursor::new(zip_bytes)).map_err(|e| format!("open webui zip failed: {e}"))?;
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("create webui dir {} failed: {e}", target_dir.display()))?;

    // Detect single top-level folder (e.g. `dist/`) so `index.html` lands directly in `target_dir`.
    let strip_prefix = detect_single_top_dir(&mut archive);
    let mut files_extracted = 0usize;

    for idx in 0..archive.len() {
        let mut entry = archive
            .by_index(idx)
            .map_err(|e| format!("read webui entry {idx} failed: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let raw_name = entry.name().replace('\\', "/");
        let rel = if let Some(prefix) = &strip_prefix {
            raw_name.strip_prefix(prefix).unwrap_or(&raw_name)
        } else {
            &raw_name
        };
        let safe = sanitize_rel_path(rel)?;
        let dst = target_dir.join(safe);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
        }
        let mut out = File::create(&dst).map_err(|e| format!("create {} failed: {e}", dst.display()))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("extract failed: {e}"))?;
        files_extracted += 1;
    }

    Ok(WebUiUpdateResult {
        target_dir: target_dir.to_string_lossy().to_string(),
        files_extracted,
    })
}

fn detect_single_top_dir<R: std::io::Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<String> {
    let mut common: Option<String> = None;
    for idx in 0..archive.len() {
        let Ok(entry) = archive.by_index(idx) else {
            continue;
        };
        let name = entry.name().replace('\\', "/");
        let (first, _) = name.split_once('/')?;
        let prefix = format!("{first}/");
        match &common {
            None => common = Some(prefix),
            Some(c) if c == &prefix => {}
            Some(_) => return None,
        }
    }
    common
}

fn sanitize_rel_path(raw: &str) -> MgrResult<PathBuf> {
    let mut clean = PathBuf::new();
    for comp in Path::new(raw).components() {
        match comp {
            Component::Normal(p) => clean.push(p),
            Component::CurDir => {}
            _ => return Err(format!("zip-slip path rejected in webui zip: {raw}")),
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("empty path in webui zip".to_string());
    }
    Ok(clean)
}
