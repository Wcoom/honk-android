//! Pure Rust WebUI WASM build pipeline (`honk-xtask build-webui`).
//!
//! Compiles `rust/doona-wasm` (`/ui/`) and `rust/doona-lite-wasm` (`/ui/lite/`) to
//! `wasm32-unknown-unknown --release`, runs `wasm-bindgen --target web --no-typescript`,
//! bundles Doona's CSS stylesheets, emits content-hashed `assets/<name>-<8hex>.{wasm,js,css}`
//! matching `honk-core`'s `hashed_asset::is_hashed_asset` contract, and generates strict-CSP
//! `index.html` into `src/target/doona-rust` and `src/target/doona-rust-lite`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

fn hash8(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    for entry in fs::read_dir(src).map_err(|e| format!("readdir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&from).map_err(|e| format!("stat {}: {e}", from.display()))?;
        if meta.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if meta.is_file() {
            fs::copy(&from, &to)
                .map_err(|e| format!("copy {} -> {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}

fn concat_css(files: &[PathBuf]) -> Result<String, String> {
    let mut out = String::new();
    for path in files {
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(path)
            .map_err(|e| format!("read css {}: {e}", path.display()))?;
        for line in text.lines() {
            if line.trim_start().starts_with("@import ") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}

fn build_wasm_pkg(crate_dir: &Path, lib_name: &str, tmp_out: &Path) -> Result<(Vec<u8>, String), String> {
    let status = Command::new("cargo")
        .current_dir(crate_dir)
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()
        .map_err(|e| format!("cargo build wasm failed in {}: {e}", crate_dir.display()))?;
    if !status.success() {
        return Err(format!("cargo wasm32 build failed for {lib_name}"));
    }

    let wasm_path = crate_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(format!("{lib_name}.wasm"));
    if !wasm_path.is_file() {
        return Err(format!("expected wasm artifact not found: {}", wasm_path.display()));
    }

    if tmp_out.exists() {
        fs::remove_dir_all(tmp_out).map_err(|e| format!("clean {}: {e}", tmp_out.display()))?;
    }
    fs::create_dir_all(tmp_out).map_err(|e| format!("mkdir {}: {e}", tmp_out.display()))?;

    let status = Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "--no-typescript",
            "--out-dir",
            tmp_out.to_str().ok_or("non-utf8 out dir")?,
            wasm_path.to_str().ok_or("non-utf8 wasm path")?,
        ])
        .status()
        .map_err(|e| format!("wasm-bindgen failed for {lib_name}: {e}"))?;
    if !status.success() {
        return Err(format!("wasm-bindgen exited non-zero for {lib_name}"));
    }

    let bg_wasm = fs::read(tmp_out.join(format!("{lib_name}_bg.wasm")))
        .map_err(|e| format!("read bg wasm: {e}"))?;
    let js_glue = fs::read_to_string(tmp_out.join(format!("{lib_name}.js")))
        .map_err(|e| format!("read js glue: {e}"))?;
    Ok((bg_wasm, js_glue))
}

struct BundleSpec<'a> {
    out_dir: &'a Path,
    name: &'a str,
    title: &'a str,
    wasm_bytes: &'a [u8],
    js_glue: &'a str,
    css_text: &'a str,
    css_assets: &'a [(String, PathBuf)],
    public_dir: &'a Path,
    has_manifest: bool,
}

fn emit_bundle(spec: &BundleSpec<'_>) -> Result<(), String> {
    let BundleSpec {
        out_dir,
        name,
        title,
        wasm_bytes,
        js_glue,
        css_text,
        css_assets,
        public_dir,
        has_manifest,
    } = *spec;
    if out_dir.exists() {
        fs::remove_dir_all(out_dir).map_err(|e| format!("clean {}: {e}", out_dir.display()))?;
    }
    let assets_dir = out_dir.join("assets");
    fs::create_dir_all(&assets_dir).map_err(|e| format!("mkdir {}: {e}", assets_dir.display()))?;

    let mut final_css = css_text.to_string();
    for (css_ref, src_path) in css_assets {
        if !src_path.is_file() {
            continue;
        }
        let data = fs::read(src_path)
            .map_err(|e| format!("read css asset {}: {e}", src_path.display()))?;
        let h = hash8(&data);
        let stem = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("asset");
        let ext = src_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        let hashed_name = format!("{stem}-{h}.{ext}");
        fs::write(assets_dir.join(&hashed_name), &data)
            .map_err(|e| format!("write css asset {hashed_name}: {e}"))?;
        final_css = final_css.replace(css_ref, &format!("./{hashed_name}"));
    }

    let wasm_hash = hash8(wasm_bytes);
    let wasm_file = format!("{name}-{wasm_hash}.wasm");
    fs::write(assets_dir.join(&wasm_file), wasm_bytes)
        .map_err(|e| format!("write {wasm_file}: {e}"))?;

    let patched_js = js_glue.replace(&format!("{name}_bg.wasm"), &wasm_file);
    let js_hash = hash8(patched_js.as_bytes());
    let js_file = format!("{name}-{js_hash}.js");
    fs::write(assets_dir.join(&js_file), patched_js.as_bytes())
        .map_err(|e| format!("write {js_file}: {e}"))?;

    let boot_js = format!(
        "import init from './{js_file}';\ninit({{ module_or_path: new URL('./{wasm_file}', import.meta.url) }});\n"
    );
    let boot_hash = hash8(boot_js.as_bytes());
    let boot_file = format!("boot-{boot_hash}.js");
    fs::write(assets_dir.join(&boot_file), boot_js.as_bytes())
        .map_err(|e| format!("write {boot_file}: {e}"))?;

    let css_hash = hash8(final_css.as_bytes());
    let css_file = format!("{name}-{css_hash}.css");
    fs::write(assets_dir.join(&css_file), final_css.as_bytes())
        .map_err(|e| format!("write {css_file}: {e}"))?;

    if public_dir.join("logo.svg").is_file() {
        fs::copy(public_dir.join("logo.svg"), out_dir.join("logo.svg"))
            .map_err(|e| format!("copy logo.svg: {e}"))?;
    }
    if has_manifest && public_dir.join("manifest.webmanifest").is_file() {
        fs::copy(
            public_dir.join("manifest.webmanifest"),
            out_dir.join("manifest.webmanifest"),
        )
        .map_err(|e| format!("copy manifest.webmanifest: {e}"))?;
    }
    copy_dir_recursive(&public_dir.join("icons"), &out_dir.join("icons"))?;

    let manifest_tag = if has_manifest {
        "\n<link rel=\"manifest\" href=\"./manifest.webmanifest\">"
    } else {
        ""
    };
    let html = format!(
        r#"<!doctype html>
<html lang="zh-CN" dir="ltr" data-scheme="light" data-family="rose-pine" data-flavour="moon">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' http: https: ws: wss:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https: http:; font-src 'self'">
<title>{title}</title>
<meta name="description" content="Pure Rust WASM web UI for honk native API.">
<link rel="icon" href="./logo.svg" type="image/svg+xml">{manifest_tag}
<meta name="darkreader-lock">
<link rel="apple-touch-icon" sizes="180x180" href="./icons/apple-touch-icon.png">
<meta name="mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-capable" content="yes">
<link rel="stylesheet" href="./assets/{css_file}">
</head>
<body>
<div id="root"></div>
<script type="module" src="./assets/{boot_file}"></script>
</body>
</html>
"#
    );
    fs::write(out_dir.join("index.html"), html.as_bytes())
        .map_err(|e| format!("write index.html: {e}"))?;

    println!(
        "Emitted {} ({wasm_file}: {} KiB, {css_file}: {} KiB)",
        out_dir.display(),
        wasm_bytes.len() / 1024,
        css_text.len() / 1024
    );
    Ok(())
}

const LITE_LAYOUT_CSS: &str = r#"
:root {
  --lite-bg: var(--color-background-body, #ffffff);
  --lite-surface: var(--color-background-surface, #f7f7f9);
  --lite-text: var(--color-text-primary, #1b1b1e);
  --lite-muted: var(--color-text-secondary, #6b6b76);
  --lite-border: var(--color-border, #e2e2e8);
  --lite-accent: #286983;
}
:root[data-color-mode='dark'] {
  --lite-bg: #1b1b1e;
  --lite-surface: #242429;
  --lite-text: #ececf0;
  --lite-muted: #a0a0ad;
  --lite-border: #363640;
  --lite-accent: #3e8fb0;
}
html, body { background: var(--lite-bg); color: var(--lite-text); margin: 0; font-family: system-ui, -apple-system, sans-serif; }
.lite-shell { display: grid; grid-template-columns: 224px minmax(0, 1fr); min-height: 100dvh; }
@media (max-width: 720px) {
  .lite-shell { grid-template-columns: 1fr; }
  .lite-sidenav { flex-direction: row; align-items: center; justify-content: space-between; border-right: none; border-bottom: 1px solid var(--lite-border); padding: 8px 14px; min-height: 48px; }
  .lite-nav-list, .lite-sidenav > .lite-nav-btn, .lite-status-row { display: none; }
  .lite-sidenav-footer { border-top: none; padding-top: 0; flex-direction: row; }
}
.lite-sidenav { background: var(--lite-surface); border-right: 1px solid var(--lite-border); padding: 14px; display: flex; flex-direction: column; gap: 10px; }
.lite-brand { display: flex; align-items: center; gap: 8px; }
.lite-nav-list { display: flex; flex-direction: column; gap: 4px; flex: 1; }
.lite-nav-link, .lite-nav-btn { display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; border-radius: 8px; text-decoration: none; color: inherit; background: transparent; border: 1px solid transparent; cursor: pointer; font-size: 14px; }
.lite-nav-link[aria-current='page'] { background: var(--lite-accent); color: #fff; }
.lite-sidenav-footer { border-top: 1px solid var(--lite-border); padding-top: 10px; display: flex; flex-direction: column; gap: 8px; }
.lite-main { padding: 20px 20px 104px; max-width: 1120px; width: 100%; box-sizing: border-box; }
.lite-page { display: flex; flex-direction: column; gap: 16px; }
.lite-head h1 { margin: 0; font-size: 24px; letter-spacing: -0.025em; font-weight: 700; }
.lite-head, .lite-between { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
.lite-card { background: var(--lite-surface); border: 1px solid var(--lite-border); border-radius: 16px; padding: 16px; }
.lite-grid { display: flex; flex-direction: column; gap: 16px; }
.lite-member-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; margin-top: 12px; }
.lite-member { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border-radius: 10px; border: 1px solid var(--lite-border); background: var(--lite-bg); color: inherit; cursor: pointer; font-variant-numeric: tabular-nums; transition: transform 140ms cubic-bezier(0.32,0.72,0,1), background-color 180ms ease, border-color 180ms ease; }
.lite-member:active { transform: scale(0.975); }
.lite-member.active { border-color: var(--lite-accent); background: var(--lite-accent); color: #fff; }
.lite-btn { padding: 8px 14px; border-radius: 10px; border: 1px solid var(--lite-border); background: var(--lite-bg); color: inherit; cursor: pointer; text-decoration: none; font-size: 13px; transition: transform 140ms cubic-bezier(0.32,0.72,0,1), opacity 160ms ease; }
.lite-btn:active { transform: scale(0.97); }
.lite-btn.primary { background: var(--lite-accent); color: #fff; border-color: var(--lite-accent); font-weight: 600; }
.lite-btn.sm { padding: 5px 10px; font-size: 12px; }
.lite-input { padding: 8px 12px; border-radius: 10px; border: 1px solid var(--lite-border); background: var(--lite-bg); color: inherit; font-size: 13px; box-sizing: border-box; }
.lite-field { display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
.lite-badge { display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; border: 1px solid var(--lite-border); font-variant-numeric: tabular-nums; }
.lite-muted { color: var(--lite-muted); font-size: 12px; }
.lite-table { width: 100%; border-collapse: collapse; font-size: 13px; font-variant-numeric: tabular-nums; }
.lite-table th, .lite-table td { text-align: left; padding: 8px; border-bottom: 1px solid var(--lite-border); }
.lite-pre { overflow: auto; max-height: 360px; font-size: 12px; background: var(--lite-bg); padding: 12px; border-radius: 10px; }
.lite-center { min-height: 100dvh; display: flex; align-items: center; justify-content: center; padding: 24px; }
.lite-modal-bg { position: fixed; inset: 0; background: rgba(15,23,42,0.38); backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 900; animation: lite-fade 180ms cubic-bezier(0.32,0.72,0,1) both; }
.lite-modal { width: min(440px, 92vw); max-height: 80vh; overflow: auto; border-radius: 22px; animation: lite-sheet 260ms cubic-bezier(0.32,0.72,0,1) both; }
.lite-draft-bar { position: fixed; bottom: 88px; left: 50%; transform: translateX(-50%); background: var(--lite-surface); border: 1px solid var(--lite-accent); border-radius: 16px; padding: 10px 16px; display: flex; align-items: center; gap: 14px; z-index: 800; box-shadow: 0 10px 28px rgba(0,0,0,0.22); }
.lite-toast { display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-radius: 12px; border: 1px solid var(--lite-border); background: var(--lite-surface); margin-bottom: 12px; animation: lite-sheet 240ms cubic-bezier(0.32,0.72,0,1) both; }
.lite-dot { width: 8px; height: 8px; border-radius: 999px; background: #2da44e; display: inline-block; margin-right: 6px; }
.lite-status-row { display: flex; align-items: center; font-size: 12px; }
@media (prefers-reduced-motion: no-preference) {
  .lite-route-stage[data-nav-dir='tab-forward'][data-nav-parity='0'] { animation: lite-tab-fwd-a 250ms cubic-bezier(0.32,0.72,0,1) both; }
  .lite-route-stage[data-nav-dir='tab-forward'][data-nav-parity='1'] { animation: lite-tab-fwd-b 250ms cubic-bezier(0.32,0.72,0,1) both; }
  .lite-route-stage[data-nav-dir='tab-backward'][data-nav-parity='0'] { animation: lite-tab-bwd-a 250ms cubic-bezier(0.32,0.72,0,1) both; }
  .lite-route-stage[data-nav-dir='tab-backward'][data-nav-parity='1'] { animation: lite-tab-bwd-b 250ms cubic-bezier(0.32,0.72,0,1) both; }
  .lite-route-stage[data-nav-dir='fade'] { animation: lite-fade 200ms cubic-bezier(0.32,0.72,0,1) both; }
}
@keyframes lite-tab-fwd-a { from { opacity: 0; transform: translate3d(24px,0,0) scale(0.985); } to { opacity: 1; transform: translate3d(0,0,0) scale(1); } }
@keyframes lite-tab-fwd-b { from { opacity: 0; transform: translate3d(24px,0,0) scale(0.985); } to { opacity: 1; transform: translate3d(0,0,0) scale(1); } }
@keyframes lite-tab-bwd-a { from { opacity: 0; transform: translate3d(-24px,0,0) scale(0.985); } to { opacity: 1; transform: translate3d(0,0,0) scale(1); } }
@keyframes lite-tab-bwd-b { from { opacity: 0; transform: translate3d(-24px,0,0) scale(0.985); } to { opacity: 1; transform: translate3d(0,0,0) scale(1); } }
@keyframes lite-fade { from { opacity: 0; } to { opacity: 1; } }
@keyframes lite-sheet { from { opacity: 0; transform: translate3d(0,14px,0) scale(0.94); } to { opacity: 1; transform: translate3d(0,0,0) scale(1); } }
.lite-lg-dock-wrap { position: fixed; left: 0; right: 0; bottom: 0; z-index: 40; display: flex; justify-content: center; padding: 8px 16px calc(12px + env(safe-area-inset-bottom)); pointer-events: none; }
@media (min-width: 721px) { .lite-lg-dock-wrap { display: none !important; } .lite-main { padding-bottom: 32px; } }
.lite-lg-dock { position: relative; width: min(460px, 100%); height: 58px; border-radius: 999px; pointer-events: auto; touch-action: pan-y; user-select: none; -webkit-user-select: none; box-shadow: 0 12px 32px rgba(15,23,42,0.16), 0 2px 8px rgba(15,23,42,0.08); isolation: isolate; }
.lite-lg-surface { position: absolute; inset: 0; border-radius: inherit; background: color-mix(in srgb, var(--lite-surface) 70%, transparent); backdrop-filter: blur(18px) saturate(180%); -webkit-backdrop-filter: blur(18px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.72), inset 0 -1px 0 rgba(0,0,0,0.08); pointer-events: none; overflow: hidden; }
@supports (backdrop-filter: url(#lite-lg-bar)) { @media (hover: hover) and (pointer: fine) { .lite-lg-surface { backdrop-filter: url(#lite-lg-bar) blur(10px) saturate(165%); } } }
:root[data-color-mode='dark'] .lite-lg-surface { background: color-mix(in srgb, var(--lite-surface) 64%, rgba(12,14,20,0.48)); box-shadow: inset 0 1px 0 rgba(255,255,255,0.24), inset 0 -1px 0 rgba(255,255,255,0.06); }
.lite-lg-track { position: relative; z-index: 1; display: flex; align-items: center; width: 100%; height: 100%; padding: 4px; box-sizing: border-box; }
.lite-lg-droplet { position: absolute; top: 4px; left: 4px; height: 50px; border-radius: 999px; background: color-mix(in srgb, var(--lite-accent) 18%, rgba(255,255,255,0.32)); backdrop-filter: blur(8px) saturate(185%); -webkit-backdrop-filter: blur(8px) saturate(185%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.85), 0 4px 12px color-mix(in srgb, var(--lite-accent) 24%, rgba(0,0,0,0.14)); pointer-events: none; will-change: transform; }
@supports (backdrop-filter: url(#lite-lg-droplet)) { .lite-lg-droplet { backdrop-filter: url(#lite-lg-droplet) blur(3px) saturate(165%); } }
.lite-lg-tab { position: relative; z-index: 2; flex: 1 1 0; min-width: 0; height: 50px; display: flex; align-items: center; justify-content: center; text-decoration: none; color: var(--lite-muted); font-size: 13px; font-weight: 500; border-radius: 999px; transition: color 200ms cubic-bezier(0.32,0.72,0,1); }
.lite-lg-tab[aria-current='page'] { color: var(--lite-accent); font-weight: 700; }
"#;

pub fn build_webui(repo_root: &Path, out_base: Option<&Path>) -> Result<(), String> {
    let target_full = out_base
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join("src/target/doona-rust"));
    let mut lite_name = target_full
        .file_name()
        .ok_or("invalid output dir")?
        .to_os_string();
    lite_name.push("-lite");
    let target_lite = target_full.with_file_name(lite_name);

    let tmp_dir = repo_root.join("src/target/wasm-bindgen-tmp");

    // 1. Build Doona Full WASM (`/ui/`)
    let (full_wasm, full_js) = build_wasm_pkg(
        &repo_root.join("rust/doona-wasm"),
        "doona_wasm",
        &tmp_dir.join("full"),
    )?;
    let full_assets_root = repo_root.join("rust/doona-wasm/assets");
    let styles_root = full_assets_root.join("styles");
    let full_css_files = vec![
        full_assets_root.join("flags.css"),
        styles_root.join("palettes/rose-pine.css"),
        styles_root.join("palettes/catppuccin.css"),
        styles_root.join("palettes/nord.css"),
        styles_root.join("palettes/antd.css"),
        styles_root.join("palettes/arco.css"),
        styles_root.join("palettes/semi.css"),
        styles_root.join("palettes/kary.css"),
        styles_root.join("palettes/glass.css"),
        styles_root.join("palettes/qiangguo.css"),
        styles_root.join("palettes/halloween.css"),
        styles_root.join("foundations.css"),
        styles_root.join("shell-search.css"),
        styles_root.join("cards-dashboard.css"),
        styles_root.join("buttons-menus.css"),
        styles_root.join("indicators-charts.css"),
        styles_root.join("dialogs-search.css"),
        styles_root.join("page-layout.css"),
        styles_root.join("nodes.css"),
        styles_root.join("fields.css"),
        styles_root.join("tables-forms.css"),
        styles_root.join("motion.css"),
        styles_root.join("segmented.css"),
        styles_root.join("toasts.css"),
        styles_root.join("tags-bars.css"),
        styles_root.join("feature-details.css"),
        styles_root.join("interaction-states.css"),
        styles_root.join("tabs-panels.css"),
        styles_root.join("charts.css"),
        styles_root.join("dashboard.css"),
        styles_root.join("glass.css"),
        styles_root.join("routing.css"),
        styles_root.join("select-box.css"),
        styles_root.join("table-cards.css"),
        styles_root.join("textarea.css"),
        styles_root.join("widgets.css"),
        styles_root.join("liquid-ios.css"),
    ];
    let full_css = concat_css(&full_css_files)?;
    let mut full_css_assets: Vec<(String, PathBuf)> = vec![(
        "./assets/TwemojiCountryFlags.woff2".to_string(),
        full_assets_root.join("fonts/TwemojiCountryFlags.woff2"),
    )];
    for sub in ["halloween", "qiangguo"] {
        let dir = styles_root.join("palettes").join(sub);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file()
                    && let Some(fname) = p.file_name().and_then(|s| s.to_str())
                {
                    full_css_assets.push((format!("./{sub}/{fname}"), p));
                }
            }
        }
    }
    emit_bundle(&BundleSpec {
        out_dir: &target_full,
        name: "doona_wasm",
        title: "doona",
        wasm_bytes: &full_wasm,
        js_glue: &full_js,
        css_text: &full_css,
        css_assets: &full_css_assets,
        public_dir: &full_assets_root.join("public"),
        has_manifest: true,
    })?;

    // 2. Build Doona Lite WASM (`/ui/lite/`)
    let (lite_wasm, lite_js) = build_wasm_pkg(
        &repo_root.join("rust/doona-lite-wasm"),
        "doona_lite_wasm",
        &tmp_dir.join("lite"),
    )?;
    let lite_assets_root = repo_root.join("rust/doona-lite-wasm/assets");
    let lite_css_files = vec![
        lite_assets_root.join("site.css"),
        lite_assets_root.join("modeTokens.css"),
        lite_assets_root.join("styles.css"),
        lite_assets_root.join("ruleList.css"),
    ];
    let mut lite_css = concat_css(&lite_css_files)?;
    lite_css.push_str(LITE_LAYOUT_CSS);
    emit_bundle(&BundleSpec {
        out_dir: &target_lite,
        name: "doona_lite_wasm",
        title: "doona lite",
        wasm_bytes: &lite_wasm,
        js_glue: &lite_js,
        css_text: &lite_css,
        css_assets: &[],
        public_dir: &lite_assets_root.join("public"),
        has_manifest: false,
    })?;

    let _ = fs::remove_dir_all(&tmp_dir);
    println!(
        "Pure-Rust WebUI ready:\n  HONK_DOONA_DIR={}\n  Lite sibling:  {}",
        target_full.display(),
        target_lite.display()
    );
    Ok(())
}
