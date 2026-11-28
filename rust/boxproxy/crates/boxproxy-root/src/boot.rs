//! M43 Magisk / KernelSU / APatch `service.d` boot autostart script generator (Section 3.4).

use crate::shell::{posix_quote, RootExecutor};
use crate::RootResult;
use std::path::Path;
use std::time::Duration;

pub const BOOT_SCRIPT_PATH: &str = "/data/adb/service.d/boxproxy-honk.sh";

pub fn render_boot_script(box_home: &Path) -> String {
    let home = box_home.to_string_lossy();
    let boxctl = box_home.join("bin/boxctl");
    let db = box_home.join("box.db");
    let log_file = box_home.join("run/boot.log");
    format!(
        "#!/system/bin/sh\n\
         export PATH=\"/data/adb/magisk:/data/adb/ksu/bin:/data/adb/ap/bin:/system/bin:/system/xbin:$PATH\"\n\
         export BOXCTL_STDOUT_LOG_LINE=1\n\
         export BOX_HOME={home_q}\n\
         boxctl={boxctl_q}\n\
         db={db_q}\n\
         log_file={log_q}\n\
         (\n\
           until [ \"$(getprop init.svc.bootanim 2>/dev/null)\" = \"stopped\" ]; do\n\
             sleep 10\n\
           done\n\
           until [ -x \"$boxctl\" ] && [ -f \"$db\" ]; do\n\
             sleep 2\n\
           done\n\
           mkdir -p \"$(dirname \"$log_file\")\"\n\
           : > \"$log_file\"\n\
           \"$boxctl\" --db \"$db\" boot >> \"$log_file\" 2>&1\n\
         ) &\n\
         exit 0\n",
        home_q = posix_quote(&home),
        boxctl_q = posix_quote(&boxctl.to_string_lossy()),
        db_q = posix_quote(&db.to_string_lossy()),
        log_q = posix_quote(&log_file.to_string_lossy()),
    )
}

pub async fn install_boot_script(exec: &dyn RootExecutor, box_home: &Path) -> RootResult<()> {
    let script = render_boot_script(box_home);
    let tmp = "/data/adb/service.d/.boxproxy-honk.sh.tmp";
    let cmd = format!(
        "mkdir -p /data/adb/service.d && cat <<'__BOX_BOOT_EOF__' > {tmp_q}\n{script}__BOX_BOOT_EOF__\n\
         chmod 0755 {tmp_q} && chown root:root {tmp_q} && mv -f {tmp_q} {dst_q} && test -x {dst_q}",
        tmp_q = posix_quote(tmp),
        dst_q = posix_quote(BOOT_SCRIPT_PATH),
    );
    let out = exec.exec(&cmd, Duration::from_secs(10)).await?;
    if out.ok() {
        Ok(())
    } else {
        Err(format!("install boot script failed: {}", out.error_text()))
    }
}

pub async fn remove_boot_script(exec: &dyn RootExecutor) -> RootResult<()> {
    let cmd = format!("rm -f {}", posix_quote(BOOT_SCRIPT_PATH));
    let out = exec.exec(&cmd, Duration::from_secs(5)).await?;
    if out.ok() {
        Ok(())
    } else {
        Err(format!("remove boot script failed: {}", out.error_text()))
    }
}
