//! Pure Rust build, verification, and Android deployment CLI replacing repo Shell/Python scripts.

mod build;
mod device;
mod stackmap;
mod webui;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "honk-xtask", about = "Pure Rust tooling for honk-android")]
struct Cli {
    /// Repository root directory (defaults to current directory or workspace ancestor)
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Port of `stackmap.py`: analyze reachable BPF subprograms and chained stack depths
    Stackmap { obj: PathBuf, entry: String },
    /// Port of `build.sh`: cross-compile static ARM64 `honk-core`
    BuildCore {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cargo_args: Vec<String>,
    },
    /// Port of `build-boxctl.sh`: cross-compile static ARM64 `boxctl`
    BuildBoxctl {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cargo_args: Vec<String>,
    },
    /// Cross-compile pure Rust static ARM64 `boxbpf`
    BuildBoxbpf {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cargo_args: Vec<String>,
    },
    /// Build pure-Rust WASM Doona (`/ui/`) and Doona Lite (`/ui/lite/`) bundles for `honk-core`
    BuildWebui {
        /// Output directory for full Doona bundle (lite goes to `<out>-lite`)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Port of `setup-cgroup.sh`: upgrade Android root cgroup BPF hooks to multi mode
    SetupCgroup {
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Port of `vtest.sh`: build eBPF object and run verifier check on device
    Vtest {
        #[arg(long, default_value = "/data/local/tmp/honk-min.dae")]
        config: String,
        #[arg(long, default_value_t = 60)]
        timeout: u32,
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Port of `measure.sh`: build `honk-core` and measure TC verifier cost on device
    Measure {
        #[arg(short, long)]
        serial: Option<String>,
    },
    /// Port of `deploy_honk_android.sh`: deploy artifacts, patch `box.db`, optional autostart
    Deploy {
        #[arg(long)]
        honk: Option<PathBuf>,
        #[arg(long)]
        boxctl: Option<PathBuf>,
        #[arg(long)]
        boxbpf: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        geoip: Option<PathBuf>,
        #[arg(long)]
        geosite: Option<PathBuf>,
        #[arg(long, default_value = "com.boxproxy.honk")]
        pkg: String,
        #[arg(long)]
        configure_db: bool,
        #[arg(long)]
        boot_autostart: bool,
        #[arg(long)]
        start: bool,
        #[arg(short, long)]
        serial: Option<String>,
    },
}

fn find_repo_root(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let mut cur = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    loop {
        if cur.join("src/Cargo.toml").is_file() && cur.join("boxproxy").is_dir() {
            return Ok(cur);
        }
        if !cur.pop() {
            return Err("could not locate honk-android repository root; pass --root".to_string());
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let res = match cli.cmd {
        Cmd::Stackmap { obj, entry } => stackmap::run_stackmap(&obj, &entry),
        Cmd::BuildCore { cargo_args } => {
            find_repo_root(cli.root).and_then(|r| build::build_core(&r, &cargo_args))
        }
        Cmd::BuildBoxctl { cargo_args } => {
            find_repo_root(cli.root).and_then(|r| build::build_boxctl(&r, &cargo_args))
        }
        Cmd::BuildBoxbpf { cargo_args } => {
            find_repo_root(cli.root).and_then(|r| build::build_boxbpf(&r, &cargo_args))
        }
        Cmd::BuildWebui { out } => {
            find_repo_root(cli.root).and_then(|r| webui::build_webui(&r, out.as_deref()))
        }
        Cmd::SetupCgroup { serial } => device::setup_cgroup(serial.as_deref()),
        Cmd::Vtest {
            config,
            timeout,
            serial,
        } => find_repo_root(cli.root)
            .and_then(|r| device::vtest(&r, &config, timeout, serial.as_deref())),
        Cmd::Measure { serial } => {
            find_repo_root(cli.root).and_then(|r| device::measure(&r, serial.as_deref()))
        }
        Cmd::Deploy {
            honk,
            boxctl,
            boxbpf,
            config,
            geoip,
            geosite,
            pkg,
            configure_db,
            boot_autostart,
            start,
            serial,
        } => device::deploy(&device::DeployOpts {
            honk,
            boxctl,
            boxbpf,
            config,
            geoip,
            geosite,
            pkg,
            configure_db,
            boot_autostart,
            start,
            serial,
        }),
    };

    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
