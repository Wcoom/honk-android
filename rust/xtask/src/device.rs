//! Pure Rust replacements for `setup-cgroup.sh`, `vtest.sh`, `measure.sh`, and `deploy_honk_android.sh`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rusqlite::Connection;

const CGROUP_HOOKS: [(&str, u32); 9] = [
    ("cgroup_sock_ops", 19),
    ("cgroup_inet_sock_create", 29),
    ("cgroup_inet4_connect", 33),
    ("cgroup_inet6_connect", 34),
    ("cgroup_udp4_sendmsg", 37),
    ("cgroup_udp6_sendmsg", 38),
    ("cgroup_inet4_bind", 31),
    ("cgroup_inet6_bind", 32),
    ("cgroup_inet_sock_release", 30),
];

fn adb_cmd(serial: Option<&str>) -> Command {
    let mut c = Command::new("adb");
    if let Some(s) = serial {
        c.args(["-s", s]);
    }
    c
}

fn adb_shell_su(serial: Option<&str>, script: &str) -> Result<String, String> {
    let mut child = adb_cmd(serial)
        .args(["shell", "su", "-c", "sh -s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn adb shell su: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(script.as_bytes())
            .map_err(|e| format!("write su script: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait adb shell su: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(format!(
            "adb su exited with {}:\n{stdout}{stderr}",
            output.status
        ));
    }
    Ok(stdout)
}

fn adb_push(serial: Option<&str>, local: &Path, remote: &str) -> Result<(), String> {
    let status = adb_cmd(serial)
        .args(["push"])
        .arg(local)
        .arg(remote)
        .status()
        .map_err(|e| format!("adb push: {e}"))?;
    if !status.success() {
        return Err(format!("adb push {} -> {remote} failed", local.display()));
    }
    Ok(())
}

pub fn setup_cgroup(serial: Option<&str>) -> Result<(), String> {
    let mut script = String::from(
        "CGROUP=/sys/fs/cgroup\n\
         upgrade() {\n\
           t=$1; id=$2\n\
           if bpftool cgroup show $CGROUP 2>/dev/null | grep -q \"$t .*multi\"; then\n\
             echo \"  $t 已是 multi，跳过\"\n\
             return 0\n\
           fi\n\
           bpftool cgroup detach $CGROUP $t id $id >/dev/null 2>&1\n\
           if bpftool cgroup attach $CGROUP $t id $id multi >/dev/null 2>&1; then\n\
             echo \"  $t -> multi OK\"\n\
           else\n\
             echo \"  $t 升级失败（程序 $id 可能已变，重新查 id 后重试）\"\n\
           fi\n\
         }\n\
         echo '== 升级 honk 需要的 cgroup hook 到 multi 模式 =='\n",
    );
    for (hook, id) in CGROUP_HOOKS {
        script.push_str(&format!("upgrade {hook} {id}\n"));
    }
    script.push_str(
        "echo\necho '== 结果 =='\nbpftool cgroup show $CGROUP | grep -vE '^ID|^Cgroup' || true\n",
    );
    let out = adb_shell_su(serial, &script)?;
    print!("{out}");
    Ok(())
}

pub fn vtest(root: &Path, cfg: &str, timeout: u32, serial: Option<&str>) -> Result<(), String> {
    let ebpf_dir = root.join("src/crates/honk-ebpf");
    let target_dir = ebpf_dir.join("target");
    let status = Command::new("cargo")
        .current_dir(&ebpf_dir)
        .env_remove("RUSTFLAGS")
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "+nightly-2026-10-20",
            "build",
            "--release",
            "-Zbuild-std=core",
            "--target",
            "bpfel-unknown-none",
        ])
        .status()
        .map_err(|e| format!("cargo build honk-ebpf: {e}"))?;
    if !status.success() {
        return Err(format!("honk-ebpf build failed: {status}"));
    }
    let obj = target_dir.join("bpfel-unknown-none/release/honk-ebpf");
    if !obj.is_file() {
        return Err(format!("missing eBPF object: {}", obj.display()));
    }
    adb_push(serial, &obj, "/data/local/tmp/honk-ebpf.o")?;
    let script = format!(
        "BASE=/data/local/tmp/honk\n\
         export HONK_RUN_DIR=$BASE/run SSL_CERT_FILE=$BASE/ssl SSL_CERT_DIR=$BASE/ssl\n\
         cd $BASE && timeout {timeout} /data/local/tmp/honk-core-a64 -c '{cfg}' -b /data/local/tmp/honk-ebpf.o > $BASE/v.log 2>&1\n\
         echo rc=$?\n\
         grep -E 'ERROR|Error|stack|processed|is running|Attached|listening|insn' $BASE/v.log | cut -c1-400 | tail -12 || true\n"
    );
    let out = adb_shell_su(serial, &script)?;
    print!("{out}");
    Ok(())
}

pub fn measure(root: &Path, serial: Option<&str>) -> Result<(), String> {
    crate::build::build_core(root, &[])?;
    let bin = root.join("src/target/aarch64-unknown-linux-musl/release/honk-core");
    adb_push(serial, &bin, "/data/local/tmp/honk-core-a64")?;
    let out = adb_shell_su(
        serial,
        "BASE=/data/local/tmp/honk\n\
         export HONK_RUN_DIR=$BASE/run SSL_CERT_FILE=$BASE/ssl SSL_CERT_DIR=$BASE/ssl\n\
         cd $BASE && timeout 90 /data/local/tmp/honk-core-a64 -c /data/local/tmp/honk-min.dae > $BASE/m.log 2>&1 || true\n\
         grep -oE 'processed [0-9]+ insns|Processed [0-9]+ insn|is running|fatal error[^/]*' $BASE/m.log | tail -3 || true\n",
    )?;
    print!("{out}");
    Ok(())
}

pub struct DeployOpts {
    pub honk: Option<PathBuf>,
    pub boxctl: Option<PathBuf>,
    pub boxbpf: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub geoip: Option<PathBuf>,
    pub geosite: Option<PathBuf>,
    pub pkg: String,
    pub configure_db: bool,
    pub boot_autostart: bool,
    pub start: bool,
    pub serial: Option<String>,
}

pub fn patch_box_db(
    db_path: &Path,
    configure_db: bool,
    boot_autostart: bool,
) -> Result<(), String> {
    let conn =
        Connection::open(db_path).map_err(|e| format!("open sqlite {}: {e}", db_path.display()))?;
    let has_table = |name: &str| -> bool {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |_| Ok(()),
        )
        .is_ok()
    };
    if has_table("runtime_profile") {
        if configure_db {
            conn.execute(
                "UPDATE runtime_profile SET core_name='honk', mode='ebpf', config_name='config.dae' WHERE id=1",
                [],
            )
            .map_err(|e| format!("update runtime_profile: {e}"))?;
        }
        if boot_autostart {
            conn.execute(
                "UPDATE runtime_profile SET boot_auto_start=1 WHERE id=1",
                [],
            )
            .map_err(|e| format!("update boot_auto_start: {e}"))?;
        }
    }
    if configure_db && has_table("settings") {
        conn.execute(
            "UPDATE settings SET value='honk' WHERE key IN ('core','proxy_core')",
            [],
        )
        .map_err(|e| format!("update settings core: {e}"))?;
        conn.execute(
            "UPDATE settings SET value='ebpf' WHERE key IN ('mode','proxy_mode')",
            [],
        )
        .map_err(|e| format!("update settings mode: {e}"))?;
        conn.execute(
            "UPDATE settings SET value='config.dae' WHERE key IN ('config_file','honk_config_file')",
            [],
        )
        .map_err(|e| format!("update settings config: {e}"))?;
    }
    Ok(())
}

pub fn deploy(opts: &DeployOpts) -> Result<(), String> {
    let serial = opts.serial.as_deref();
    let box_dir = format!("/data/user/0/{}/files/box", opts.pkg);

    println!(
        "[1/5] Checking ADB connection and root environment (target package: {})...",
        opts.pkg
    );
    let check_out = adb_shell_su(
        serial,
        &format!(
            "set -e\n\
             ARCH=$(uname -m)\n\
             KVER=$(uname -r)\n\
             echo \"  Device arch=$ARCH kernel=$KVER uid=$(id -u)\"\n\
             if [ \"$ARCH\" != \"aarch64\" ]; then echo \"ERROR: Expected aarch64, got $ARCH\" >&2; exit 1; fi\n\
             if [ ! -d '{box_dir}' ]; then echo \"ERROR: {box_dir} not found\" >&2; exit 1; fi\n"
        ),
    )?;
    print!("{check_out}");

    println!("[2/5] Stopping active core if running...");
    let _ = adb_shell_su(
        serial,
        &format!(
            "if [ -x '{box_dir}/bin/boxctl' ] && [ -f '{box_dir}/box.db' ]; then\n\
               '{box_dir}/bin/boxctl' --db '{box_dir}/box.db' down || true\n\
             fi\n"
        ),
    );

    println!("[3/5] Pushing selected artifacts to /data/local/tmp...");
    let mut install_cmds = format!(
        "set -e\nBOX='{box_dir}'\nOWNER=$(stat -c '%u:%g' \"$BOX\")\nmkdir -p \"$BOX/bin\" \"$BOX/honk/data\"\n"
    );
    let pushes: [(&Option<PathBuf>, &str, &str, &str); 6] = [
        (
            &opts.honk,
            "/data/local/tmp/honk_deploy_bin",
            "$BOX/bin/honk",
            "755",
        ),
        (
            &opts.boxctl,
            "/data/local/tmp/boxctl_deploy_bin",
            "$BOX/bin/boxctl",
            "755",
        ),
        (
            &opts.boxbpf,
            "/data/local/tmp/boxbpf_deploy_bin",
            "$BOX/bin/boxbpf",
            "755",
        ),
        (
            &opts.config,
            "/data/local/tmp/honk_deploy_config.dae",
            "$BOX/honk/config.dae",
            "644",
        ),
        (
            &opts.geoip,
            "/data/local/tmp/honk_deploy_geoip.dat",
            "$BOX/honk/data/geoip.dat",
            "644",
        ),
        (
            &opts.geosite,
            "/data/local/tmp/honk_deploy_geosite.dat",
            "$BOX/honk/data/geosite.dat",
            "644",
        ),
    ];
    for (local, tmp_remote, dest, mode) in pushes {
        if let Some(p) = local {
            if !p.is_file() {
                return Err(format!("Missing artifact file: {}", p.display()));
            }
            adb_push(serial, p, tmp_remote)?;
            install_cmds.push_str(&format!(
                "cp '{tmp_remote}' \"{dest}\"\nchmod {mode} \"{dest}\"\nchown \"$OWNER\" \"{dest}\"\nrm -f '{tmp_remote}'\n"
            ));
        }
    }
    install_cmds.push_str(
        "chown -R \"$OWNER\" \"$BOX/honk\"\n\
         if [ -x \"$BOX/bin/honk\" ] && [ -f \"$BOX/honk/config.dae\" ]; then\n\
           echo '  Validating config.dae...'\n\
           \"$BOX/bin/honk\" -t -d \"$BOX/honk/data\" -c \"$BOX/honk/config.dae\"\n\
         fi\n",
    );

    println!("[4/5] Installing files, fixing ownership/permissions, and validating config...");
    let inst_out = adb_shell_su(serial, &install_cmds)?;
    print!("{inst_out}");

    if opts.configure_db || opts.boot_autostart {
        let tmp_db =
            std::env::temp_dir().join(format!("honk_deploy_box_{}.db", std::process::id()));
        adb_shell_su(
            serial,
            &format!(
                "if [ -f '{box_dir}/box.db' ]; then\n\
                   cp '{box_dir}/box.db' /data/local/tmp/honk_deploy_box.db\n\
                   chmod 644 /data/local/tmp/honk_deploy_box.db\n\
                 fi\n"
            ),
        )?;
        let pull_ok = adb_cmd(serial)
            .args(["pull", "/data/local/tmp/honk_deploy_box.db"])
            .arg(&tmp_db)
            .status()
            .is_ok_and(|s| s.success());
        if pull_ok {
            patch_box_db(&tmp_db, opts.configure_db, opts.boot_autostart)?;
            adb_push(serial, &tmp_db, "/data/local/tmp/honk_deploy_box.db")?;
            adb_shell_su(
                serial,
                &format!(
                    "OWNER=$(stat -c '%u:%g' '{box_dir}')\n\
                     am force-stop '{}' >/dev/null 2>&1 || true\n\
                     rm -f '{box_dir}/box.db-wal' '{box_dir}/box.db-shm' '{box_dir}/box.db-journal'\n\
                     cp /data/local/tmp/honk_deploy_box.db '{box_dir}/box.db'\n\
                     chown \"$OWNER\" '{box_dir}/box.db'\n\
                     chmod 600 '{box_dir}/box.db'\n\
                     rm -f /data/local/tmp/honk_deploy_box.db\n",
                    opts.pkg
                ),
            )?;
        }
        let _ = fs::remove_file(&tmp_db);
    }

    if opts.boot_autostart {
        let (boot_script, boot_tag) = if opts.pkg == "com.boxproxy.honk" {
            ("/data/adb/service.d/boxproxy-honk.sh", "BoxProxyHonkBoot")
        } else {
            ("/data/adb/service.d/boxproxy-box.sh", "BoxProxyBoot")
        };
        let install_boot = format!(
            "set -e\nmkdir -p /data/adb/service.d\n\
             cat > '{boot_script}.tmp' <<'BOOTEOF'\n\
             #!/system/bin/sh\n(\n\
               export PATH=\"/data/adb/magisk:/data/adb/ksu/bin:/data/adb/ap/bin:/system/bin:/system/xbin:$PATH\"\n\
               export BOXCTL_STDOUT_LOG_LINE=1\n\
               run_dir='{box_dir}/run'\n\
               log_file='{box_dir}/run/boot.log'\n\
               boxctl='{box_dir}/bin/boxctl'\n\
               db='{box_dir}/box.db'\n\
               timestamp() {{ date '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo '-'; }}\n\
               boot_log() {{ echo \"$(timestamp) [Info] $*\" >> \"$log_file\" 2>/dev/null || true; }}\n\
               log -t {boot_tag} '开机自启：准备启动服务' 2>/dev/null || true\n\
               while [ \"$(getprop init.svc.bootanim 2>/dev/null)\" != \"stopped\" ]; do sleep 10; done\n\
               while [ ! -x \"$boxctl\" ] || [ ! -f \"$db\" ]; do sleep 2; done\n\
               mkdir -p \"$run_dir\" 2>/dev/null || exit 1\n\
               : > \"$log_file\" 2>/dev/null || exit 1\n\
               boot_log '开机自启：准备启动服务'\n\
               \"$boxctl\" --db \"$db\" boot >> \"$log_file\" 2>&1\n\
               start_code=$?\n\
               if [ \"$start_code\" -eq 0 ]; then boot_log '开机自启：服务启动命令已完成'; else boot_log \"启动失败: exit code $start_code\"; fi\n\
             ) >/dev/null 2>&1 &\nexit 0\nBOOTEOF\n\
             chmod 755 '{boot_script}.tmp'\nchown root:root '{boot_script}.tmp' 2>/dev/null || true\nmv '{boot_script}.tmp' '{boot_script}'\n"
        );
        adb_shell_su(serial, &install_boot)?;
    }

    if opts.start {
        println!("[5/5] Starting honk via boxctl and running quick health check...");
        let start_out = adb_shell_su(
            serial,
            &format!(
                "set -e\n\
                 '{box_dir}/bin/boxctl' --db '{box_dir}/box.db' up\n\
                 sleep 3\n\
                 '{box_dir}/bin/boxctl' --db '{box_dir}/box.db' status\n\
                 curl -sS --max-time 3 http://127.0.0.1:9090/version && echo '' || true\n\
                 curl -sI --max-time 3 http://127.0.0.1:9527/ui/ | head -n 5 || true\n"
            ),
        )?;
        print!("{start_out}");
    } else {
        println!("[5/5] Deployment complete (pass --start to start service automatically).");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_runtime_profile_in_sqlite_without_python() {
        let p = std::env::temp_dir().join(format!("xtask_db_test_{}.db", std::process::id()));
        let _ = fs::remove_file(&p);
        {
            let conn = Connection::open(&p).unwrap();
            conn.execute_batch(
                "CREATE TABLE runtime_profile (id INTEGER PRIMARY KEY, core_name TEXT, mode TEXT, config_name TEXT, boot_auto_start INTEGER);\
                 INSERT INTO runtime_profile VALUES (1, 'mihomo', 'tproxy', 'config.yaml', 0);",
            )
            .unwrap();
        }
        patch_box_db(&p, true, true).unwrap();
        let conn = Connection::open(&p).unwrap();
        let row: (String, String, String, i64) = conn
            .query_row(
                "SELECT core_name, mode, config_name, boot_auto_start FROM runtime_profile WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        let _ = fs::remove_file(&p);
        assert_eq!(row, ("honk".into(), "ebpf".into(), "config.dae".into(), 1));
    }
}
