//! Byte, rate, latency, duration, DNS cache, and SSE event formatters plus pure-Rust SVG sparklines.

use serde_json::Value;

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut val = bytes as f64;
    let mut idx = 0usize;
    while val >= 1024.0 && idx + 1 < UNITS.len() {
        val /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{bytes} B")
    } else if val >= 100.0 {
        format!("{val:.0} {}", UNITS[idx])
    } else if val >= 10.0 {
        format!("{val:.1} {}", UNITS[idx])
    } else {
        format!("{val:.2} {}", UNITS[idx])
    }
}

pub fn format_rate(bps: f64) -> String {
    if !bps.is_finite() || bps <= 0.0 {
        return "0 B/s".to_string();
    }
    format!("{}/s", format_bytes(bps.round() as u64))
}

pub fn format_latency(ms: Option<f64>) -> String {
    match ms {
        Some(v) if v.is_finite() && v >= 0.0 => format!("{} ms", v.round() as u64),
        _ => "—".to_string(),
    }
}

pub fn format_duration_secs(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3_600;
    let m = (secs % 3_600) / 60;
    let s = secs % 60;
    if d > 0 {
        format!("{d}d {h}h {m}m")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Pure-Rust smooth monotone SVG line + area path generator replacing `d3-shape` sparklines.
pub fn sparkline_paths(values: &[f64], width: f64, height: f64) -> (String, String) {
    if values.is_empty() || width <= 0.0 || height <= 0.0 {
        return (String::new(), String::new());
    }
    let max_v = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .fold(1.0_f64, f64::max);
    let pad = 2.0;
    let inner_h = (height - pad * 2.0).max(1.0);
    let n = values.len();
    let step = if n > 1 { width / (n - 1) as f64 } else { width };

    let pts: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let clamped = if v.is_finite() { v.max(0.0) } else { 0.0 };
            let x = if n == 1 { width } else { i as f64 * step };
            let y = height - pad - (clamped / max_v) * inner_h;
            (x, y)
        })
        .collect();

    let mut line = format!("M{:.2},{:.2}", pts[0].0, pts[0].1);
    for w in pts.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        let mx = (x0 + x1) * 0.5;
        line.push_str(&format!(" C{mx:.2},{y0:.2} {mx:.2},{y1:.2} {x1:.2},{y1:.2}"));
    }
    let last_x = pts.last().map_or(width, |p| p.0);
    let area = format!("{line} L{last_x:.2},{height:.2} L{:.2},{height:.2} Z", pts[0].0);
    (line, area)
}

/// Parses an ISO-8601 / RFC-3339 UTC timestamp (`YYYY-MM-DDTHH:MM:SS[.fff]Z`) into Unix seconds.
pub fn parse_rfc3339_utc_secs(ts: &str) -> Option<i64> {
    let s = ts.trim();
    if s.len() < 19 {
        return None;
    }
    let year: i64 = s.get(0..4)?.parse().ok()?;
    let month: i64 = s.get(5..7)?.parse().ok()?;
    let day: i64 = s.get(8..10)?.parse().ok()?;
    let hour: i64 = s.get(11..13)?.parse().ok()?;
    let min: i64 = s.get(14..16)?.parse().ok()?;
    let sec: i64 = s.get(17..19)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Howard Hinnant civil-to-days (UTC proleptic Gregorian)
    let y = year - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hour * 3_600 + min * 60 + sec)
}

/// Computes remaining DNS cache TTL from `honk-core` `/api/v1/dns/cache?detail=full` entry JSON.
pub fn format_dns_cache_ttl(entry: &Value, observed_at: Option<&str>) -> String {
    for k in ["remaining_ttl_seconds", "ttl_seconds", "ttl"] {
        if let Some(secs) = entry.get(k).and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
        }) {
            return format_duration_secs(secs);
        }
    }
    if let Some(exp_str) = entry.get("expires_at").and_then(Value::as_str)
        && let Some(exp_s) = parse_rfc3339_utc_secs(exp_str)
        && let Some(obs_s) = observed_at.and_then(parse_rfc3339_utc_secs)
    {
        let diff = exp_s - obs_s;
        return if diff <= 0 {
            "已过期".to_string()
        } else {
            format_duration_secs(diff as u64)
        };
    }
    if let Some(ans_ttl) = entry
        .get("answers")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|a| a.get("ttl"))
        .and_then(Value::as_u64)
    {
        return format_duration_secs(ans_ttl);
    }
    "—".to_string()
}

/// Formats DNS cache `answers` (preferring terminal records matching `qtype` over CNAMEs) or negative status.
pub fn format_dns_cache_answers(entry: &Value, qtype: &str) -> String {
    if let Some(arr) = entry
        .get("answers")
        .or_else(|| entry.get("records"))
        .and_then(Value::as_array)
        && !arr.is_empty()
    {
        let mut primary = Vec::new();
        let mut secondary = Vec::new();
        for r in arr {
            let r_type = r
                .get("type")
                .or_else(|| r.get("rtype"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let raw = r
                .get("data")
                .or_else(|| r.get("value"))
                .or_else(|| r.get("rdata"))
                .or_else(|| r.get("target"))
                .and_then(Value::as_str)
                .or_else(|| r.as_str())
                .unwrap_or("")
                .trim_end_matches('.');
            if raw.is_empty() {
                continue;
            }
            if r_type.eq_ignore_ascii_case(qtype) {
                primary.push(raw.to_string());
            } else {
                secondary.push(format!(
                    "{}{}",
                    if r_type.is_empty() {
                        String::new()
                    } else {
                        format!("{r_type}: ")
                    },
                    raw
                ));
            }
        }
        let chosen = if !primary.is_empty() { primary } else { secondary };
        if !chosen.is_empty() {
            let total = chosen.len();
            let shown: Vec<_> = chosen.into_iter().take(2).collect();
            return if total > 2 {
                format!("{} (+{})", shown.join(", "), total - 2)
            } else {
                shown.join(", ")
            };
        }
    }

    for k in ["status", "rcode", "outcome"] {
        if let Some(st) = entry.get(k).and_then(Value::as_str)
            && !st.is_empty()
        {
            return match st {
                "NXDOMAIN" => "NXDOMAIN (不存在)".to_string(),
                "NODATA" => "NODATA (无记录)".to_string(),
                "NOERROR" => "NOERROR (空应答)".to_string(),
                other => other.to_string(),
            };
        }
    }
    "—".to_string()
}

/// Formats an SSE `/api/v1/events` frame into `(hh_mm_ss, readable_summary, optional_flow_id)`.
pub fn format_sse_event_row(kind: &str, raw_data: &str) -> (String, String, Option<String>) {
    let Ok(val) = serde_json::from_str::<Value>(raw_data) else {
        return (String::new(), raw_data.to_string(), None);
    };
    let time_str = val
        .get("observed_at")
        .or_else(|| val.get("timestamp"))
        .and_then(Value::as_str)
        .and_then(|s| s.split_once('T').map(|(_, rest)| rest.trim_end_matches('Z')))
        .unwrap_or("")
        .to_string();

    let short_uuid = |s: &str| -> String { s.split('-').next().unwrap_or(s).to_string() };

    let res_id = val.get("resource_id").and_then(Value::as_str);
    let rev = val.get("revision").and_then(Value::as_u64);
    let href = val.get("href").and_then(Value::as_str).unwrap_or("");
    let flow_id = if href.starts_with("/api/v1/flows/") || kind.starts_with("flow.") {
        res_id.map(str::to_string)
    } else {
        None
    };

    let summary = match kind {
        "stream.ready" => {
            let inst = val
                .get("instance_id")
                .and_then(Value::as_str)
                .map(short_uuid)
                .unwrap_or_default();
            if inst.is_empty() {
                "实时事件流已就绪".to_string()
            } else {
                format!("实时事件流已就绪 · 实例 {inst}")
            }
        }
        k if k.starts_with("flow.") => {
            let fid = res_id.map(short_uuid).unwrap_or_else(|| "—".to_string());
            let action = match k {
                "flow.created" => "新建连接流",
                "flow.closed" => "关闭连接流",
                _ => "连接流更新",
            };
            match rev {
                Some(r) => format!("{action} · {fid} · 修订 #{r}"),
                None => format!("{action} · {fid}"),
            }
        }
        k if k.starts_with("group.") => {
            let gid = val
                .get("group_name")
                .or_else(|| val.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| res_id.map(short_uuid))
                .unwrap_or_else(|| "策略组".to_string());
            format!("策略状态变更 · {gid}")
        }
        k if k.starts_with("operation.") => {
            let oid = res_id
                .or_else(|| val.get("operation_id").and_then(Value::as_str))
                .map(short_uuid)
                .unwrap_or_else(|| "—".to_string());
            let st = val.get("status").and_then(Value::as_str).unwrap_or("");
            if st.is_empty() {
                format!("异步操作 · {oid}")
            } else {
                format!("异步操作 {oid} · {st}")
            }
        }
        _ => {
            if let Some(obj) = val.as_object() {
                let mut parts = Vec::new();
                for (k, v) in obj {
                    if matches!(k.as_str(), "instance_id" | "observed_at" | "timestamp" | "href") {
                        continue;
                    }
                    let vs = match v {
                        Value::String(s) => short_uuid(s),
                        other => other.to_string(),
                    };
                    parts.push(format!("{k}: {vs}"));
                }
                if parts.is_empty() {
                    "事件已触发".to_string()
                } else {
                    parts.join(" · ")
                }
            } else {
                raw_data.to_string()
            }
        }
    };

    (time_str, summary, flow_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_bytes_rates_and_sparklines() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(127_533_056), "122 MB");
        assert_eq!(format_bytes(26_210_456), "25.0 MB");
        assert_eq!(format_rate(0.0), "0 B/s");
        assert_eq!(format_rate(2048.0), "2.00 KB/s");
        let (line, area) = sparkline_paths(&[0.0, 10.0, 5.0, 20.0], 120.0, 32.0);
        assert!(line.starts_with("M0.00,"));
        assert!(area.ends_with("Z"));
    }

    #[test]
    fn formats_honk_dns_cache_answers_ttl_and_sse_events() {
        let pos = json!({
            "domain": "support.weixin.qq.com.",
            "type": "A",
            "status": "NOERROR",
            "expires_at": "2026-11-28T14:17:26.607Z",
            "answers": [
                { "type": "CNAME", "data": "minorshort.weixin.qq.com.", "ttl": 600 },
                { "type": "A", "data": "117.185.24.162", "ttl": 600 },
                { "type": "A", "data": "223.109.215.124", "ttl": 600 }
            ]
        });
        assert_eq!(
            format_dns_cache_answers(&pos, "A"),
            "117.185.24.162, 223.109.215.124"
        );
        assert_eq!(
            format_dns_cache_ttl(&pos, Some("2026-11-28T14:14:54.535Z")),
            "2m 32s"
        );

        let neg = json!({
            "domain": "appcfg.v.qq.com.",
            "type": "AAAA",
            "status": "NXDOMAIN",
            "expires_at": "2026-11-28T14:15:10.000Z",
            "answers": []
        });
        assert_eq!(format_dns_cache_answers(&neg, "AAAA"), "NXDOMAIN (不存在)");
        assert_eq!(
            format_dns_cache_ttl(&neg, Some("2026-11-28T14:14:54.000Z")),
            "16s"
        );

        let (tm, summary, fid) = format_sse_event_row(
            "flow.updated",
            r#"{"instance_id":"dbfc8ce6-8c29","observed_at":"2026-11-28T14:15:25.146Z","resource_id":"03751f2c-82ee-40c3-bbbe-96507cf240e4","revision":20,"href":"/api/v1/flows/03751f2c-82ee-40c3-bbbe-96507cf240e4"}"#,
        );
        assert_eq!(tm, "14:15:25.146");
        assert_eq!(summary, "连接流更新 · 03751f2c · 修订 #20");
        assert_eq!(fid.as_deref(), Some("03751f2c-82ee-40c3-bbbe-96507cf240e4"));
    }
}
