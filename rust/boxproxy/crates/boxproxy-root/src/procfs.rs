//! `/proc/<pid>` sampling (M02/M57), `/proc/net/arp` hotspot client scan (M28), and `/proc/net/dev` (M03).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub running: bool,
    pub pid: Option<u32>,
    pub pss_mb: f64,
    pub rss_mb: f64,
    pub cpu_percent: f32,
    pub cpu_core: Option<u32>,
    pub uptime_secs: u64,
    pub cmdline: String,
    pub total_cpu_ticks: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArpClient {
    pub ip: String,
    pub mac: String,
    pub device: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDevSample {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub fn sample_process(box_home: &Path, prev: Option<(u64, f64)>, elapsed_secs: f64) -> ProcessMetrics {
    let pid_path = box_home.join("run/box.pid");
    let Ok(pid_str) = fs::read_to_string(&pid_path) else {
        return ProcessMetrics::default();
    };
    let Ok(pid) = pid_str.trim().parse::<u32>() else {
        return ProcessMetrics::default();
    };

    let proc_dir = Path::new("/proc").join(pid.to_string());
    if !proc_dir.is_dir() {
        return ProcessMetrics::default();
    }

    let cmdline = fs::read(proc_dir.join("cmdline"))
        .map(|b| {
            String::from_utf8_lossy(&b)
                .replace('\0', " ")
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    if cmdline.is_empty() {
        return ProcessMetrics::default();
    }

    let stat_text = fs::read_to_string(proc_dir.join("stat")).unwrap_or_default();
    let (ticks, start_ticks, cpu_core) = parse_stat_fields(&stat_text).unwrap_or((0, 0, None));

    let smaps = fs::read_to_string(proc_dir.join("smaps_rollup")).unwrap_or_default();
    let status = fs::read_to_string(proc_dir.join("status")).unwrap_or_default();
    let rss_kb = parse_status_vmrss_kb(&status).unwrap_or(0);
    let pss_kb = parse_smaps_pss_kb(&smaps).unwrap_or(rss_kb);

    let sys_uptime = fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0);
    let hz = 100.0_f64;
    let uptime_secs = (sys_uptime - (start_ticks as f64 / hz)).max(0.0) as u64;

    let cpu_percent = match prev {
        Some((prev_ticks, _)) if elapsed_secs > 0.05 && ticks >= prev_ticks => {
            let delta_secs = (ticks - prev_ticks) as f64 / hz;
            ((delta_secs / elapsed_secs) * 100.0).clamp(0.0, 800.0) as f32
        }
        _ => 0.0,
    };

    ProcessMetrics {
        running: true,
        pid: Some(pid),
        pss_mb: pss_kb as f64 / 1024.0,
        rss_mb: rss_kb as f64 / 1024.0,
        cpu_percent,
        cpu_core,
        uptime_secs,
        cmdline,
        total_cpu_ticks: ticks,
    }
}

/// Parse `/proc/<pid>/stat` (fields 14 utime, 15 stime, 22 starttime, 39 processor).
pub fn parse_stat_fields(stat: &str) -> Option<(u64, u64, Option<u32>)> {
    let close_paren = stat.rfind(')')?;
    let after = stat.get(close_paren + 1..)?.trim();
    let fields: Vec<&str> = after.split_whitespace().collect();
    // Field 3 (state) is index 0 after ')'.
    // So field N is at index N - 3.
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let starttime: u64 = fields.get(19)?.parse().ok()?;
    let processor: Option<u32> = fields.get(36).and_then(|s| s.parse().ok());
    Some((utime.saturating_add(stime), starttime, processor))
}

pub fn parse_smaps_pss_kb(text: &str) -> Option<u64> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Pss:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

pub fn parse_status_vmrss_kb(text: &str) -> Option<u64> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

pub fn parse_arp_table(text: &str) -> Vec<ArpClient> {
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 6 {
            let mac = cols[3].to_ascii_uppercase();
            if mac != "00:00:00:00:00:00" {
                out.push(ArpClient {
                    ip: cols[0].to_string(),
                    mac,
                    device: cols[5].to_string(),
                });
            }
        }
    }
    out
}

pub fn parse_net_dev(text: &str) -> NetDevSample {
    let mut total = NetDevSample::default();
    for line in text.lines().skip(2) {
        let Some((iface, rest)) = line.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        if iface == "lo" || iface.starts_with("tun") || iface.starts_with("meta") {
            continue;
        }
        let cols: Vec<&str> = rest.split_whitespace().collect();
        if cols.len() >= 9 {
            total.rx_bytes = total.rx_bytes.saturating_add(cols[0].parse().unwrap_or(0));
            total.tx_bytes = total.tx_bytes.saturating_add(cols[8].parse().unwrap_or(0));
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_proc_stat_smaps_and_arp() {
        let mut fake = String::from("1234 (honk-core) S ");
        // fields 4..40 (37 fields)
        for i in 4..=40 {
            let val = match i {
                14 => "250",
                15 => "50",
                22 => "1000",
                39 => "6",
                _ => "0",
            };
            fake.push_str(val);
            fake.push(' ');
        }
        let (ticks, start, cpu) = parse_stat_fields(&fake).unwrap();
        assert_eq!(ticks, 300);
        assert_eq!(start, 1000);
        assert_eq!(cpu, Some(6));
        assert_eq!(parse_smaps_pss_kb("Rss: 90000 kB\nPss: 83968 kB\n"), Some(83968));

        let arp = "IP address       HW type     Flags       HW address            Mask     Device\n\
                   192.168.43.10    0x1         0x2         aa:bb:cc:dd:ee:ff     *        ap0\n";
        let clients = parse_arp_table(arp);
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].mac, "AA:BB:CC:DD:EE:FF");
    }
}
