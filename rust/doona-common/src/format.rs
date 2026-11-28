//! Byte, rate, latency, duration formatters and pure-Rust SVG sparkline/area path builder (replaces `d3-shape`).

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
