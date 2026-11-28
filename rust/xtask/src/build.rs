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

fn resolve_doona_dir(root: &Path, src: &Path) -> Result<Option<PathBuf>, String> {
    if let Ok(explicit) = env::var("HONK_DOONA_DIR")
        && !explicit.is_empty()
    {
        return Ok(Some(PathBuf::from(explicit)));
    }

    let stage = src.join("target/doona-rust");
    let stage_lite = src.join("target/doona-rust-lite");
    let rebuild = env::var("REBUILD_WEBUI").ok().as_deref() == Some("1");
    if rebuild || !stage.join("index.html").is_file() || !stage_lite.join("index.html").is_file() {
        crate::webui::build_webui(root, Some(&stage))?;
    }
    if stage.is_dir() && stage_lite.is_dir() {
        return Ok(Some(stage));
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
