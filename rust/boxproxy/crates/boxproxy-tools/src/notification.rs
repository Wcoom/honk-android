//! M57/M58 12-placeholder persistent notification renderer (`hd0.java` parity).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationRenderVars {
    pub status: String,
    pub pid: String,
    pub core: String,
    pub mode: String,
    pub config: String,
    pub uptime: String,
    pub cpu: String,
    pub memory: String,
    pub upload: String,
    pub download: String,
    pub total_upload: String,
    pub total_download: String,
}

pub fn render_notification_template(template: &str, v: &NotificationRenderVars) -> String {
    template
        .replace("{status}", &v.status)
        .replace("{pid}", &v.pid)
        .replace("{core}", &v.core)
        .replace("{mode}", &v.mode)
        .replace("{config}", &v.config)
        .replace("{uptime}", &v.uptime)
        .replace("{cpu}", &v.cpu)
        .replace("{memory}", &v.memory)
        .replace("{upload}", &v.upload)
        .replace("{download}", &v.download)
        .replace("{totalUpload}", &v.total_upload)
        .replace("{total_upload}", &v.total_upload)
        .replace("{totalDownload}", &v.total_download)
        .replace("{total_download}", &v.total_download)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_all_twelve_notification_placeholders() {
        let vars = NotificationRenderVars {
            status: "Running".into(),
            pid: "4200".into(),
            core: "honk".into(),
            mode: "ebpf".into(),
            config: "config.dae".into(),
            uptime: "01:12:34".into(),
            cpu: "1.2%".into(),
            memory: "82 MB".into(),
            upload: "12 KB/s".into(),
            download: "1.5 MB/s".into(),
            total_upload: "20 MB".into(),
            total_download: "350 MB".into(),
        };
        let rendered = render_notification_template(
            "{status} #{pid} {core}/{mode} ({config}) {uptime} ↑{upload} ↓{download} ∑↑{totalUpload} ∑↓{totalDownload} CPU {cpu} RAM {memory}",
            &vars,
        );
        assert!(!rendered.contains('{'));
        assert!(rendered.contains("honk/ebpf"));
        assert!(rendered.contains("82 MB"));
    }
}
