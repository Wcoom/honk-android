//! M30–M35 runtime files, JS override scripts, proxy providers YAML, shell scripts, and logs.

use crate::StoreResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

pub const MAX_JS_SCRIPT_BYTES: usize = 512 * 1024;
pub const JS_SCRIPT_TEMPLATE: &str =
    "function main(config, profileName) {\n  return config;\n}\n";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub relative_path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsScriptItem {
    pub name: String,
    pub relative_path: String,
    pub enabled: bool,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellScriptItem {
    pub name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub is_pre_start: bool,
    pub is_post_stop: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub modified_ms: u64,
}

/// M30 Mihomo/Meow proxy provider override entry (`run/state/config/mihomo-proxy-providers.yaml`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyProviderEntry {
    pub name: String,
    /// `"http"` | `"file"` | `"inline"`
    pub source_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub interval: u64,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub exclude_filter: String,
    #[serde(default)]
    pub health_check_enabled: bool,
    #[serde(default)]
    pub health_check_url: String,
    #[serde(default)]
    pub health_check_interval: u64,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub override_yaml: String,
}

pub struct RuntimeFileStore {
    box_home: PathBuf,
}

impl RuntimeFileStore {
    pub fn new(box_home: impl Into<PathBuf>) -> Self {
        Self {
            box_home: box_home.into(),
        }
    }

    pub fn ensure_layout(&self) -> StoreResult<()> {
        for rel in [
            "bin",
            "run",
            "run/state/locks",
            "run/state/config",
            "run/state/cache",
            "run/state/history",
            "run/state/monitor",
            "run/state/runtime",
            "run/state/cnip",
            "run/state/ebpf",
            "honk",
            "mihomo",
            "mihomo/dashboard",
            "mihomo/scripts",
            "meow",
            "meow/scripts",
            "sing-box",
            "sing-box/scripts",
            "xray",
            "v2fly",
            "hysteria",
            "scripts",
        ] {
            fs::create_dir_all(self.box_home.join(rel))
                .map_err(|e| format!("create dir {rel} failed: {e}"))?;
        }
        Ok(())
    }

    pub fn resolve_safe(&self, rel: &str) -> StoreResult<PathBuf> {
        let mut clean = PathBuf::new();
        for comp in Path::new(rel.trim_start_matches('/')).components() {
            match comp {
                Component::Normal(p) => clean.push(p),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(format!("path traversal rejected: {rel}"));
                }
            }
        }
        Ok(self.box_home.join(clean))
    }

    pub fn list_dir(&self, rel: &str) -> StoreResult<Vec<FileEntry>> {
        let dir = self.resolve_safe(rel)?;
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("read_dir failed: {e}"))?.flatten() {
            let path = entry.path();
            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified_ms = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let name = entry.file_name().to_string_lossy().to_string();
            let relative_path = path
                .strip_prefix(&self.box_home)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            items.push(FileEntry {
                relative_path,
                name,
                is_dir,
                size_bytes,
                modified_ms,
            });
        }
        items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        Ok(items)
    }

    pub fn read_text(&self, rel: &str) -> StoreResult<String> {
        let path = self.resolve_safe(rel)?;
        fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))
    }

    pub fn write_text(&self, rel: &str, content: &str) -> StoreResult<()> {
        let path = self.resolve_safe(rel)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, content).map_err(|e| format!("write temp failed: {e}"))?;
        fs::rename(&tmp, &path).map_err(|e| format!("rename failed: {e}"))
    }

    pub fn rename(&self, from_rel: &str, to_rel: &str) -> StoreResult<()> {
        let src = self.resolve_safe(from_rel)?;
        let dst = self.resolve_safe(to_rel)?;
        fs::rename(&src, &dst).map_err(|e| format!("rename failed: {e}"))
    }

    pub fn delete(&self, rel: &str) -> StoreResult<()> {
        let path = self.resolve_safe(rel)?;
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| format!("remove dir failed: {e}"))
        } else if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("remove file failed: {e}"))
        } else {
            Ok(())
        }
    }

    // -------------------------------------------------------------------------
    // M31: JS override scripts (`<core>/scripts/*.js` + `run/state/config/<core>-scripts.yaml`)
    // -------------------------------------------------------------------------

    pub fn list_js_scripts(&self, core: &str) -> StoreResult<Vec<JsScriptItem>> {
        let enabled_order = self.read_enabled_js_scripts(core)?;
        let dir = self.resolve_safe(&format!("{core}/scripts"))?;
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut by_name = BTreeMap::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("read scripts dir failed: {e}"))?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".js") {
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                by_name.insert(name, size_bytes);
            }
        }
        let mut out = Vec::new();
        for name in &enabled_order {
            if let Some(size_bytes) = by_name.remove(name) {
                out.push(JsScriptItem {
                    relative_path: format!("{core}/scripts/{name}"),
                    name: name.clone(),
                    enabled: true,
                    size_bytes,
                });
            }
        }
        for (name, size_bytes) in by_name {
            out.push(JsScriptItem {
                relative_path: format!("{core}/scripts/{name}"),
                name,
                enabled: false,
                size_bytes,
            });
        }
        Ok(out)
    }

    pub fn save_js_script(&self, core: &str, name: &str, content: &str) -> StoreResult<()> {
        if content.len() > MAX_JS_SCRIPT_BYTES {
            return Err("JS script exceeds 512 KiB limit".to_string());
        }
        let filename = if name.ends_with(".js") {
            name.to_string()
        } else {
            format!("{name}.js")
        };
        self.write_text(&format!("{core}/scripts/{filename}"), content)
    }

    pub fn read_enabled_js_scripts(&self, core: &str) -> StoreResult<Vec<String>> {
        let manifest = self.box_home.join(format!("run/state/config/{core}-scripts.yaml"));
        if !manifest.is_file() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&manifest).unwrap_or_default();
        Ok(serde_norway::from_str::<Vec<String>>(&text).unwrap_or_default())
    }

    pub fn set_enabled_js_scripts(&self, core: &str, ordered: &[String]) -> StoreResult<()> {
        let yaml = serde_norway::to_string(ordered)
            .map_err(|e| format!("format scripts yaml failed: {e}"))?;
        self.write_text(&format!("run/state/config/{core}-scripts.yaml"), &yaml)
    }

    // -------------------------------------------------------------------------
    // M30: Mihomo / Meow proxy providers YAML
    // -------------------------------------------------------------------------

    pub fn load_proxy_providers(&self) -> StoreResult<Vec<ProxyProviderEntry>> {
        let path = self.box_home.join("run/state/config/mihomo-proxy-providers.yaml");
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&path).map_err(|e| format!("read providers yaml failed: {e}"))?;
        Ok(serde_norway::from_str(&text).unwrap_or_default())
    }

    pub fn save_proxy_providers(&self, entries: &[ProxyProviderEntry]) -> StoreResult<()> {
        let yaml = serde_norway::to_string(entries)
            .map_err(|e| format!("format providers yaml failed: {e}"))?;
        self.write_text("run/state/config/mihomo-proxy-providers.yaml", &yaml)
    }

    // -------------------------------------------------------------------------
    // M32: Shell hook scripts (`files/box/scripts/*.sh`, reject binary)
    // -------------------------------------------------------------------------

    pub fn list_shell_scripts(
        &self,
        pre_start: &str,
        post_stop: &str,
    ) -> StoreResult<Vec<ShellScriptItem>> {
        let dir = self.box_home.join("scripts");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("read scripts dir failed: {e}"))?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".sh") {
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                out.push(ShellScriptItem {
                    is_pre_start: pre_start == name,
                    is_post_stop: post_stop == name,
                    relative_path: format!("scripts/{name}"),
                    name,
                    size_bytes,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn save_shell_script(&self, name: &str, raw: &[u8]) -> StoreResult<()> {
        if raw.contains(&0) {
            return Err("binary content rejected for shell script".to_string());
        }
        let text =
            std::str::from_utf8(raw).map_err(|_| "shell script must be valid UTF-8".to_string())?;
        let filename = if name.ends_with(".sh") {
            name.to_string()
        } else {
            format!("{name}.sh")
        };
        self.write_text(&format!("scripts/{filename}"), text)
    }

    // -------------------------------------------------------------------------
    // M35: Logs (`files/box/run/*.log`)
    // -------------------------------------------------------------------------

    pub fn list_logs(&self) -> StoreResult<Vec<LogFileInfo>> {
        let dir = self.box_home.join("run");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("read run dir failed: {e}"))?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".log") {
                let meta = entry.metadata().ok();
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified_ms = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                out.push(LogFileInfo {
                    name,
                    size_bytes,
                    modified_ms,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn read_log_tail(&self, name: &str, max_lines: usize) -> StoreResult<String> {
        if !name.ends_with(".log") || name.contains('/') || name.contains("..") {
            return Err(format!("invalid log filename: {name}"));
        }
        let path = self.box_home.join("run").join(name);
        if !path.is_file() {
            return Ok(String::new());
        }
        let text = fs::read_to_string(&path).map_err(|e| format!("read log failed: {e}"))?;
        let lines: Vec<&str> = text.lines().collect();
        let start = lines.len().saturating_sub(max_lines);
        Ok(lines[start..].join("\n"))
    }

    pub fn clear_log(&self, name: &str) -> StoreResult<()> {
        if !name.ends_with(".log") || name.contains('/') || name.contains("..") {
            return Err(format!("invalid log filename: {name}"));
        }
        let path = self.box_home.join("run").join(name);
        fs::write(&path, "").map_err(|e| format!("clear log failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn enforces_js_limit_and_rejects_binary_shell_scripts() {
        let dir = tempdir().unwrap();
        let store = RuntimeFileStore::new(dir.path());
        store.ensure_layout().unwrap();

        let big = "a".repeat(MAX_JS_SCRIPT_BYTES + 1);
        assert!(store.save_js_script("mihomo", "big", &big).is_err());

        store
            .save_js_script("mihomo", "rewrite", JS_SCRIPT_TEMPLATE)
            .unwrap();
        store
            .set_enabled_js_scripts("mihomo", &["rewrite.js".into()])
            .unwrap();
        let js = store.list_js_scripts("mihomo").unwrap();
        assert_eq!(js.len(), 1);
        assert!(js[0].enabled);

        assert!(store.save_shell_script("bad", b"\x7fELF\x00").is_err());
        store
            .save_shell_script("hook", b"#!/system/bin/sh\necho ok\n")
            .unwrap();
        let sh = store.list_shell_scripts("hook.sh", "").unwrap();
        assert_eq!(sh.len(), 1);
        assert!(sh[0].is_pre_start);
    }
}
