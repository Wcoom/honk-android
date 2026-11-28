//! Pure Rust replacements for `build.sh`, `build-boxctl.sh`, and `boxbpf` cross-compilation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_checked(cmd: &mut Command, label: &str) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("{label}: failed to spawn: {e}"))?;
    if !status.success() {
        return Err(format!("{label}: exited with {status}"));
    }
    Ok(())
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"))
}

fn configure_musl_cross_env(cmd: &mut Command, root: &Path) {
    let tool = root.join("toolchain");
    let home = home_dir();
    let old_path = env::var("PATH").unwrap_or_default();
    let new_path = format!(
        "{}:{}:{old_path}",
        tool.display(),
        home.join(".cargo/bin").display()
    );
    cmd.env("PATH", new_path);
    cmd.env(
        "CC_aarch64_unknown_linux_musl",
        tool.join("aarch64-linux-musl-gcc"),
    );
    cmd.env(
        "CXX_aarch64_unknown_linux_musl",
        tool.join("aarch64-linux-musl-g++"),
    );
    let ar = env::var("AR_aarch64_unknown_linux_musl")
        .unwrap_or_else(|_| "/usr/bin/llvm-ar-18".to_string());
    cmd.env("AR_aarch64_unknown_linux_musl", ar);
    cmd.env(
        "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER",
        tool.join("zig-link"),
    );
    let existing_flags = env::var("RUSTFLAGS").unwrap_or_default();
    cmd.env(
        "RUSTFLAGS",
        format!(
            "{existing_flags} --remap-path-prefix={}={} --remap-path-prefix={}={}",
            home.display(),
            "/build",
            root.display(),
            "/src"
        ),
    );
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    for entry in fs::read_dir(src).map_err(|e| format!("readdir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let ty = entry.file_type().map_err(|e| format!("filetype: {e}"))?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)
                .map_err(|e| format!("copy {}: {e}", target.display()))?;
        }
    }
    Ok(())
}

fn resolve_doona_dir(root: &Path, src: &Path) -> Result<Option<PathBuf>, String> {
    if let Ok(explicit) = env::var("HONK_DOONA_DIR")
        && !explicit.is_empty()
    {
        return Ok(Some(PathBuf::from(explicit)));
    }

    if env::var("FETCH_DOONA").ok().as_deref() == Some("1") {
        let script = src.join("ci/fetch-doona.sh");
        if script.is_file() {
            let out = Command::new(&script)
                .current_dir(src)
                .output()
                .map_err(|e| format!("fetch-doona: {e}"))?;
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() {
                    return Ok(Some(PathBuf::from(p)));
                }
            }
        }
    }

    let webui_dir = root.join("webui");
    if webui_dir.join("package.json").is_file() {
        let home = home_dir();
        let pnpm_bin = env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|p| p.join("pnpm"))
                    .find(|p| p.is_file())
            })
            .or_else(|| {
                let fallback = home.join(".local/bin/pnpm");
                fallback.is_file().then_some(fallback)
            });

        if let Some(pnpm) = pnpm_bin {
            if !webui_dir.join("node_modules").is_dir() {
                let mut c = Command::new(&pnpm);
                c.current_dir(&webui_dir)
                    .args(["install", "--frozen-lockfile"]);
                run_checked(&mut c, "pnpm install webui")?;
            }
            let lite_dir = webui_dir.join("apps/lite");
            if !lite_dir.join("node_modules").is_dir() {
                let mut c = Command::new(&pnpm);
                c.current_dir(&lite_dir)
                    .args(["install", "--frozen-lockfile"]);
                run_checked(&mut c, "pnpm install webui lite")?;
            }
            let rebuild = env::var("REBUILD_WEBUI").ok().as_deref() == Some("1");
            if rebuild
                || !webui_dir.join("dist/index.html").is_file()
                || !webui_dir.join("dist/.vite/bundled-packages.json").is_file()
            {
                let mut c = Command::new(&pnpm);
                c.current_dir(&webui_dir).arg("build");
                run_checked(&mut c, "pnpm build webui")?;
            }
            if rebuild
                || !lite_dir.join("dist/index.html").is_file()
                || !lite_dir.join("dist/.vite/bundled-packages.json").is_file()
            {
                let mut c = Command::new(&pnpm);
                c.current_dir(&webui_dir)
                    .args(["-C", &lite_dir.to_string_lossy(), "build"]);
                run_checked(&mut c, "pnpm build webui lite")?;
            }

            let stage = src.join("target/doona-embedded");
            for (dist_dir, out_dir) in [
                (webui_dir.join("dist"), stage.clone()),
                (
                    lite_dir.join("dist"),
                    src.join("target/doona-embedded-lite"),
                ),
            ] {
                if out_dir.exists() {
                    let _ = fs::remove_dir_all(&out_dir);
                }
                copy_dir_recursive(&dist_dir, &out_dir)?;
                let _ = fs::remove_dir_all(out_dir.join("fonts"));
                let _ = fs::remove_dir_all(out_dir.join(".vite"));
                for f in ["LICENSE", "NOTICE", "README.md", "README.en.md"] {
                    let _ = fs::copy(webui_dir.join(f), out_dir.join(f));
                }
                let mut notices = Command::new("node");
                notices
                    .current_dir(&webui_dir)
                    .arg("tools/notices.mjs")
                    .arg(&out_dir)
                    .arg(dist_dir.join(".vite/bundled-packages.json"));
                run_checked(&mut notices, "node tools/notices.mjs")?;
            }
            return Ok(Some(stage));
        }
    }

    if let Ok(entries) = fs::read_dir(src.join("target")) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("doona-") && !s.ends_with("-lite") {
                let p = entry.path();
                let lite = PathBuf::from(format!("{}-lite", p.display()));
                if p.is_dir() && lite.is_dir() {
                    return Ok(Some(p));
                }
            }
        }
    }

    Ok(None)
}

pub fn build_core(root: &Path, extra_args: &[String]) -> Result<(), String> {
    let src = root.join("src");
    let zig_lib = env::var("ZIG_LIB").unwrap_or_else(|_| "/usr/lib/zig0.16/lib".to_string());
    let musl_inc = format!(
        "-isystem {zig_lib}/libc/include/aarch64-linux-musl \
         -isystem {zig_lib}/libc/include/generic-musl \
         -isystem {zig_lib}/libc/include/any-linux-any"
    );

    let doona_dir = resolve_doona_dir(root, &src)?;
    let has_both_doona = doona_dir
        .as_ref()
        .is_some_and(|d| d.is_dir() && PathBuf::from(format!("{}-lite", d.display())).is_dir());
    let features = if has_both_doona {
        "ebpf,rprx,clash-api,native-ui,jemalloc"
    } else {
        "ebpf,rprx,clash-api,native-api,jemalloc"
    };

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&src);
    configure_musl_cross_env(&mut cmd, root);
    cmd.env("CMAKE_C_COMPILER_LAUNCHER", "");
    cmd.env("CMAKE_CXX_COMPILER_LAUNCHER", "");
    cmd.env(
        "BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_musl",
        &musl_inc,
    );
    cmd.env("BINDGEN_EXTRA_CLANG_ARGS", &musl_inc);
    if let Some(ref d) = doona_dir {
        cmd.env("HONK_DOONA_DIR", d);
    }
    cmd.args([
        "build",
        "--release",
        "-p",
        "honk-core",
        "--no-default-features",
        "--features",
        features,
        "--target",
        "aarch64-unknown-linux-musl",
    ]);
    cmd.args(extra_args);
    run_checked(&mut cmd, "cargo build honk-core")?;

    let out = src.join("target/aarch64-unknown-linux-musl/release/honk-core");
    let meta = fs::metadata(&out).map_err(|e| format!("missing output {}: {e}", out.display()))?;
    println!("\n================ 构建完成 ================");
    println!("{} ({} bytes)", out.display(), meta.len());
    Ok(())
}

pub fn build_boxctl(root: &Path, extra_args: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root.join("boxproxy"));
    configure_musl_cross_env(&mut cmd, root);
    cmd.args([
        "+1.99.1",
        "build",
        "--release",
        "--manifest-path",
        "boxctl/Cargo.toml",
        "--target",
        "aarch64-unknown-linux-musl",
    ]);
    cmd.args(extra_args);
    run_checked(&mut cmd, "cargo build boxctl")?;

    let out = root.join("boxproxy/boxctl/target/aarch64-unknown-linux-musl/release/boxctl");
    let meta = fs::metadata(&out).map_err(|e| format!("missing output {}: {e}", out.display()))?;
    println!("\n================ boxctl 构建完成 ================");
    println!("{} ({} bytes)", out.display(), meta.len());
    Ok(())
}

pub fn build_boxbpf(root: &Path, extra_args: &[String]) -> Result<(), String> {
    let app_dir = root.join("boxproxy/boxbpf/rust/app");
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&app_dir);
    configure_musl_cross_env(&mut cmd, root);
    cmd.args([
        "+1.100.0",
        "build",
        "--release",
        "--target",
        "aarch64-unknown-linux-musl",
    ]);
    cmd.args(extra_args);
    run_checked(&mut cmd, "cargo build pure-Rust boxbpf")?;

    let out = app_dir.join("target/aarch64-unknown-linux-musl/release/boxbpf");
    let meta = fs::metadata(&out).map_err(|e| format!("missing output {}: {e}", out.display()))?;
    println!("\n================ boxbpf (Rust) 构建完成 ================");
    println!("{} ({} bytes)", out.display(), meta.len());
    Ok(())
}
