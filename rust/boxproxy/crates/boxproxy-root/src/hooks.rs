//! M32 Shell lifecycle hook runner injecting all 12 `BOX_*` environment variables (Section 3.5.4).

use crate::shell::{posix_quote, RootExecutor};
use crate::RootResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct HookContext {
    pub event: String,
    pub box_home: PathBuf,
    pub core: String,
    pub config_name: String,
    pub mode: String,
    pub proxy_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookRunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub at_millis: u64,
}

/// Exact 12 `BOX_*` environment variables required by `com.boxproxy.honk` (`wr6.java:8-45`).
pub fn build_hook_env(ctx: &HookContext) -> Vec<(&'static str, String)> {
    let core_path = ctx.box_home.join("bin").join(&ctx.core);
    let config_path = ctx.box_home.join(&ctx.core).join(&ctx.config_name);
    let startup_config = ctx.box_home.join("run/state/config/startup-config");
    let scripts_dir = ctx.box_home.join("scripts");
    let run_dir = ctx.box_home.join("run");
    let state_dir = ctx.box_home.join("run/state");

    vec![
        ("BOX_EVENT", ctx.event.clone()),
        ("BOX_HOME", ctx.box_home.to_string_lossy().to_string()),
        ("BOX_CORE", ctx.core.clone()),
        ("BOX_CORE_PATH", core_path.to_string_lossy().to_string()),
        ("BOX_CONFIG", ctx.config_name.clone()),
        ("BOX_CONFIG_PATH", config_path.to_string_lossy().to_string()),
        (
            "BOX_STARTUP_CONFIG_PATH",
            startup_config.to_string_lossy().to_string(),
        ),
        ("BOX_MODE", ctx.mode.clone()),
        ("BOX_PROXY_MODE", ctx.proxy_mode.clone()),
        ("BOX_SCRIPTS_DIR", scripts_dir.to_string_lossy().to_string()),
        ("BOX_RUN_DIR", run_dir.to_string_lossy().to_string()),
        ("BOX_STATE_DIR", state_dir.to_string_lossy().to_string()),
    ]
}

pub async fn run_shell_hook(
    exec: &dyn RootExecutor,
    script_path: &Path,
    ctx: &HookContext,
) -> RootResult<HookRunResult> {
    let mut cmd = String::new();
    for (k, v) in build_hook_env(ctx) {
        cmd.push_str(k);
        cmd.push('=');
        cmd.push_str(&posix_quote(&v));
        cmd.push(' ');
    }
    cmd.push_str("sh ");
    cmd.push_str(&posix_quote(&script_path.to_string_lossy()));

    let out = exec.exec(&cmd, Duration::from_secs(60)).await?;
    let at_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Ok(HookRunResult {
        exit_code: out.exit_code,
        stdout: out.stdout,
        stderr: out.stderr,
        at_millis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_all_twelve_box_env_vars() {
        let envs = build_hook_env(&HookContext {
            event: "pre_start".into(),
            box_home: PathBuf::from("/data/user/0/io.honk.android/files/box"),
            core: "honk".into(),
            config_name: "config.dae".into(),
            mode: "ebpf".into(),
            proxy_mode: "core".into(),
        });
        assert_eq!(envs.len(), 12);
        assert_eq!(envs[0], ("BOX_EVENT", "pre_start".into()));
        assert_eq!(envs[11].0, "BOX_STATE_DIR");
    }
}
