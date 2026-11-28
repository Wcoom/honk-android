//! Root shell pool, `boxctl`/`boxbpf` SHA-256 atomic sync, `/proc/<pid>` sampling,
//! boot autostart script generation (M43), and 12-env-var shell hook runner (M32).

pub mod boot;
pub mod boxctl;
pub mod hooks;
pub mod procfs;
pub mod shell;
pub mod sync;

pub use boot::{install_boot_script, remove_boot_script, render_boot_script, BOOT_SCRIPT_PATH};
pub use boxctl::{BoxctlClient, BoxctlTimeouts};
pub use hooks::{build_hook_env, run_shell_hook, HookContext, HookRunResult};
pub use procfs::{
    parse_arp_table, parse_net_dev, parse_smaps_pss_kb, parse_stat_fields, sample_process,
    ArpClient, NetDevSample, ProcessMetrics,
};
pub use shell::{posix_quote, MockRootExecutor, RootExecutor, ShellOutput, SuShellPool};
pub use sync::{sha256_hex, sync_binary_atomic};

pub type RootResult<T> = std::result::Result<T, String>;
