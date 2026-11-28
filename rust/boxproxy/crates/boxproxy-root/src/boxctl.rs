//! Typed `boxctl` command wrapper matching Section 3.3 timeouts & subcommands.

use crate::shell::{posix_quote, RootExecutor, ShellOutput};
use crate::RootResult;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct BoxctlTimeouts {
    pub up: Duration,
    pub down: Duration,
    pub restart: Duration,
    pub config_sync: Duration,
    pub config_sync_manual: Duration,
    pub mode_renew: Duration,
    pub resource_apply: Duration,
    pub network_match_apply: Duration,
    pub monitor: Duration,
    pub clashapi: Duration,
}

impl Default for BoxctlTimeouts {
    fn default() -> Self {
        Self {
            up: Duration::from_secs(75),
            down: Duration::from_secs(50),
            restart: Duration::from_secs(125),
            config_sync: Duration::from_secs(15),
            config_sync_manual: Duration::from_secs(60),
            mode_renew: Duration::from_secs(15),
            resource_apply: Duration::from_secs(10),
            network_match_apply: Duration::from_secs(15),
            monitor: Duration::from_secs(10),
            clashapi: Duration::from_secs(8),
        }
    }
}

pub struct BoxctlClient {
    boxctl_bin: PathBuf,
    db_path: PathBuf,
    timeouts: BoxctlTimeouts,
}

impl BoxctlClient {
    pub fn new(box_home: &Path) -> Self {
        Self {
            boxctl_bin: box_home.join("bin/boxctl"),
            db_path: box_home.join("box.db"),
            timeouts: BoxctlTimeouts::default(),
        }
    }

    pub fn build_command(&self, subcmd: &str) -> String {
        format!(
            "{} --db {} {}",
            posix_quote(&self.boxctl_bin.to_string_lossy()),
            posix_quote(&self.db_path.to_string_lossy()),
            subcmd
        )
    }

    async fn run(&self, exec: &dyn RootExecutor, subcmd: &str, timeout: Duration) -> RootResult<ShellOutput> {
        let cmd = self.build_command(subcmd);
        let out = exec.exec(&cmd, timeout).await?;
        if out.ok() {
            Ok(out)
        } else {
            Err(format!("boxctl {subcmd} failed (rc={}): {}", out.exit_code, out.error_text()))
        }
    }

    pub async fn up(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "up", self.timeouts.up).await
    }

    pub async fn down(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "down", self.timeouts.down).await
    }

    pub async fn restart(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "restart", self.timeouts.restart).await
    }

    pub async fn config_sync(&self, exec: &dyn RootExecutor, manual: bool) -> RootResult<ShellOutput> {
        let t = if manual {
            self.timeouts.config_sync_manual
        } else {
            self.timeouts.config_sync
        };
        self.run(exec, "config sync", t).await
    }

    pub async fn mode_renew(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "mode renew", self.timeouts.mode_renew).await
    }

    pub async fn resource_apply(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "resource apply", self.timeouts.resource_apply).await
    }

    pub async fn network_match_apply(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "network-match apply", self.timeouts.network_match_apply)
            .await
    }

    pub async fn monitor(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "monitor", self.timeouts.monitor).await
    }

    pub async fn clashapi_init(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "clashapi init", self.timeouts.clashapi).await
    }

    pub async fn clashapi_clear(&self, exec: &dyn RootExecutor) -> RootResult<ShellOutput> {
        self.run(exec, "clashapi clear", self.timeouts.clashapi).await
    }
}
