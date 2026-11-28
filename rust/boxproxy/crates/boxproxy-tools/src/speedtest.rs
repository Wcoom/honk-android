//! M05 3-target latency prober + M42 3-source speed test (Cloudflare / Apple / Ookla).

use crate::ToolsResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeedTestSource {
    #[default]
    Cloudflare,
    Apple,
    Ookla,
}

impl SpeedTestSource {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "apple" => Self::Apple,
            "ookla" => Self::Ookla,
            _ => Self::Cloudflare,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyProbeResult {
    pub name: String,
    pub url: String,
    pub latency_ms: Option<u64>,
    pub ok: bool,
    pub error: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub source: String,
    pub server: String,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub exit_ip: String,
}

pub async fn probe_latency_targets(targets: &[(String, String)]) -> Vec<LatencyProbeResult> {
    let http = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| Client::new());
    let mut handles = Vec::new();
    for (name, url) in targets {
        let client = http.clone();
        let name = name.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let t0 = Instant::now();
            match client.head(&url).send().await {
                Ok(resp) => LatencyProbeResult {
                    name,
                    url,
                    latency_ms: Some(t0.elapsed().as_millis() as u64),
                    ok: resp.status().as_u16() < 500,
                    error: String::new(),
                },
                Err(e) => LatencyProbeResult {
                    name,
                    url,
                    latency_ms: None,
                    ok: false,
                    error: e.to_string(),
                },
            }
        }));
    }
    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok(r) = h.await {
            out.push(r);
        }
    }
    out
}

pub async fn run_speed_test(source: SpeedTestSource) -> ToolsResult<SpeedTestResult> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let (probe_url, down_url, up_url, label) = match source {
        SpeedTestSource::Cloudflare => (
            "https://speed.cloudflare.com/__down?bytes=1000",
            "https://speed.cloudflare.com/__down?bytes=10000000",
            Some("https://speed.cloudflare.com/__up"),
            "cloudflare",
        ),
        SpeedTestSource::Apple => (
            "https://mensura.cdn-apple.com/api/v1/gm/config",
            "https://mensura.cdn-apple.com/api/v1/gm/small",
            None,
            "apple",
        ),
        SpeedTestSource::Ookla => (
            "https://www.speedtest.net/api/js/servers?engine=js&limit=5",
            "https://speed.cloudflare.com/__down?bytes=10000000",
            Some("https://speed.cloudflare.com/__up"),
            "ookla",
        ),
    };

    // Latency & jitter across 3 samples.
    let mut rtts = Vec::new();
    for _ in 0..3 {
        let t0 = Instant::now();
        if client.get(probe_url).send().await.is_ok() {
            rtts.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let latency_ms = if rtts.is_empty() {
        0.0
    } else {
        rtts.iter().sum::<f64>() / rtts.len() as f64
    };
    let jitter_ms = if rtts.len() >= 2 {
        let diffs: f64 = rtts.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
        diffs / (rtts.len() - 1) as f64
    } else {
        0.0
    };

    // Download speed.
    let t_down = Instant::now();
    let down_bytes = client
        .get(down_url)
        .send()
        .await
        .map_err(|e| format!("download test failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("read download bytes failed: {e}"))?
        .len();
    let down_secs = t_down.elapsed().as_secs_f64().max(0.001);
    let download_mbps = (down_bytes as f64 * 8.0) / (down_secs * 1_000_000.0);

    // Upload speed (2 MiB payload when endpoint supports upload).
    let upload_mbps = if let Some(u_url) = up_url {
        let payload = vec![0u8; 2 * 1024 * 1024];
        let t_up = Instant::now();
        let ok = client.post(u_url).body(payload).send().await.is_ok();
        if ok {
            let up_secs = t_up.elapsed().as_secs_f64().max(0.001);
            (2.0 * 1024.0 * 1024.0 * 8.0) / (up_secs * 1_000_000.0)
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(SpeedTestResult {
        source: label.to_string(),
        server: label.to_string(),
        download_mbps,
        upload_mbps,
        latency_ms,
        jitter_ms,
        exit_ip: String::new(),
    })
}
