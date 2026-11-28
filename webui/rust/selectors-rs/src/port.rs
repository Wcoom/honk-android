//! The Rust port of `src/api/selectors.ts`.

#![forbid(unsafe_code)]

use serde_json::{Map, Value, json};
use url::Url;

use crate::{js, utf16};

// ---- JS value and string helpers ---------------------------------------------------------------

fn is_nullish(v: &Value) -> bool {
    v.is_null() || js::is_undefined(v)
}

fn nullish_label(v: &Value) -> &'static str {
    if v.is_null() { "null" } else { "undefined" }
}

fn check_object_coercible(v: &Value, prop: &str) -> Result<(), String> {
    if is_nullish(v) {
        return Err(format!(
            "TypeError: Cannot read properties of {} (reading '{prop}')",
            nullish_label(v)
        ));
    }
    Ok(())
}

fn prop_opt<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|o| o.get(key))
}

fn prop_or_undef(v: &Value, key: &str) -> Value {
    prop_opt(v, key).cloned().unwrap_or_else(js::undefined)
}

fn prop_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    prop_opt(v, key).and_then(Value::as_str)
}

fn marker<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object()
        .filter(|o| o.len() == 1)
        .and_then(|o| o.get(key))
}

fn is_truthy(v: &Value) -> bool {
    if is_nullish(v) {
        return false;
    }
    if let Some(n) = js::as_number(v) {
        return n != 0.0 && !n.is_nan();
    }
    if let Some(s) = marker(v, "$bigint").and_then(Value::as_str) {
        return s != "0";
    }
    if let Some(u) = marker(v, "$utf16").and_then(Value::as_array) {
        return !u.is_empty();
    }
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

fn trim_js(s: &str) -> &str {
    s.trim_matches(is_js_whitespace)
}

fn is_builtin_outbound(name: Option<&str>) -> bool {
    matches!(name, Some("direct" | "block"))
}

fn same_value_zero(a: &Value, b: &Value) -> bool {
    if let (Some(x), Some(y)) = (js::as_number(a), js::as_number(b)) {
        return x == y || (x.is_nan() && y.is_nan());
    }
    if let (Some(x), Some(y)) = (js::string_units(a), js::string_units(b)) {
        return x == y;
    }
    a == b
}

fn map_entries<'a>(v: &'a Value, name: &str) -> Result<&'a [Value], String> {
    check_object_coercible(v, "get")?;
    marker(v, "$map")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("TypeError: {name}.get is not a function"))
}

fn map_get(entries: &[Value], key: &Value) -> Option<Value> {
    let mut found = None;
    for pair in entries {
        if let Some(arr) = pair.as_array()
            && arr.len() >= 2
            && same_value_zero(&arr[0], key)
        {
            found = Some(arr[1].clone());
        }
    }
    found
}

fn js_to_string(v: &Value) -> String {
    if js::is_undefined(v) {
        return "undefined".into();
    }
    if v.is_null() {
        return "null".into();
    }
    if let Some(n) = js::as_number(v) {
        return js::number_to_string(n);
    }
    if let Some(s) = marker(v, "$bigint").and_then(Value::as_str) {
        return s.to_string();
    }
    match v {
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        _ => "[object Object]".into(),
    }
}

fn observation_rank(h: &Value) -> Result<i32, String> {
    check_object_coercible(h, "warmth")?;
    let warmth = match prop_str(h, "warmth") {
        Some("warm") => 0,
        Some("unknown") => 1,
        Some("mixed") => 2,
        Some("cold") => 3,
        _ => 1,
    };
    let measurement = match prop_str(h, "measurement") {
        Some("http_headers") => 0,
        Some("http_round_trip") => 1,
        Some("quic_handshake") => 2,
        Some("tcp_connect") => 3,
        Some("mixed") => 4,
        Some("unknown") => 5,
        Some("dns_round_trip") => 6,
        _ => 5,
    };
    Ok(warmth * 10 + measurement)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn parse_u32_exact(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

/// Parses an ECMAScript ISO-8601 date-time string into epoch milliseconds (or `NaN`).
fn parse_iso_date(s: &str) -> f64 {
    let b = s.trim().as_bytes();
    let (year, mut i) = if matches!(b.first(), Some(b'+' | b'-')) {
        if b.len() < 7 {
            return f64::NAN;
        }
        let Some(mag) = parse_u32_exact(&b[1..7]) else {
            return f64::NAN;
        };
        let sign = if b[0] == b'-' { -1 } else { 1 };
        (sign * i64::from(mag), 7)
    } else if b.len() >= 4 {
        let Some(y) = parse_u32_exact(&b[..4]) else {
            return f64::NAN;
        };
        (i64::from(y), 4)
    } else {
        return f64::NAN;
    };

    let mut month = 1u32;
    let mut day = 1u32;
    if i < b.len() && b[i] == b'-' {
        if i + 3 > b.len() {
            return f64::NAN;
        }
        let Some(m) = parse_u32_exact(&b[i + 1..i + 3]) else {
            return f64::NAN;
        };
        month = m;
        i += 3;
        if i < b.len() && b[i] == b'-' {
            if i + 3 > b.len() {
                return f64::NAN;
            }
            let Some(d) = parse_u32_exact(&b[i + 1..i + 3]) else {
                return f64::NAN;
            };
            day = d;
            i += 3;
        }
    }
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return f64::NAN;
    }

    let (mut hour, mut min, mut sec, mut ms, mut tz_ms) = (0i64, 0i64, 0i64, 0i64, 0i64);
    if i < b.len() {
        if !matches!(b[i], b'T' | b't' | b' ') {
            return f64::NAN;
        }
        i += 1;
        if i + 5 > b.len() || b[i + 2] != b':' {
            return f64::NAN;
        }
        let (Some(h), Some(m)) = (
            parse_u32_exact(&b[i..i + 2]),
            parse_u32_exact(&b[i + 3..i + 5]),
        ) else {
            return f64::NAN;
        };
        hour = i64::from(h);
        min = i64::from(m);
        i += 5;
        if i < b.len() && b[i] == b':' {
            if i + 3 > b.len() {
                return f64::NAN;
            }
            let Some(s_val) = parse_u32_exact(&b[i + 1..i + 3]) else {
                return f64::NAN;
            };
            sec = i64::from(s_val);
            i += 3;
            if i < b.len() && b[i] == b'.' {
                i += 1;
                let start = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                if i == start {
                    return f64::NAN;
                }
                let frac = &b[start..i];
                let d0 = i64::from(frac[0] - b'0');
                let d1 = i64::from(frac.get(1).copied().unwrap_or(b'0') - b'0');
                let d2 = i64::from(frac.get(2).copied().unwrap_or(b'0') - b'0');
                ms = d0 * 100 + d1 * 10 + d2;
            }
        }
        let valid_time =
            (hour < 24 && min < 60 && sec < 60) || (hour == 24 && min == 0 && sec == 0 && ms == 0);
        if !valid_time {
            return f64::NAN;
        }
        if i < b.len() {
            match b[i] {
                b'Z' | b'z' => i += 1,
                b'+' | b'-' => {
                    let sign = if b[i] == b'-' { -1 } else { 1 };
                    i += 1;
                    let (oh, om) = if i + 5 == b.len() && b[i + 2] == b':' {
                        (
                            parse_u32_exact(&b[i..i + 2]),
                            parse_u32_exact(&b[i + 3..i + 5]),
                        )
                    } else if i + 4 == b.len() {
                        (
                            parse_u32_exact(&b[i..i + 2]),
                            parse_u32_exact(&b[i + 2..i + 4]),
                        )
                    } else {
                        return f64::NAN;
                    };
                    let (Some(oh), Some(om)) = (oh, om) else {
                        return f64::NAN;
                    };
                    if oh > 23 || om > 59 {
                        return f64::NAN;
                    }
                    tz_ms = sign * (i64::from(oh) * 60 + i64::from(om)) * 60_000;
                    i = b.len();
                }
                _ => return f64::NAN,
            }
        }
    }
    if i != b.len() {
        return f64::NAN;
    }
    let total_ms = days_from_civil(year, month, day) * 86_400_000
        + ((hour * 3600 + min * 60 + sec) * 1000 + ms)
        - tz_ms;
    total_ms as f64
}

fn parse_u64_str(s: &str) -> Option<u128> {
    if s.len() > 20 || s.is_empty() {
        return None;
    }
    if s != "0" && (s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    s.parse::<u64>().ok().map(u128::from)
}

fn parse_u64_value(v: &Value) -> Result<Option<u128>, String> {
    if v.is_null() {
        return Ok(None);
    }
    if js::is_undefined(v) {
        return Err("TypeError: Cannot read properties of undefined (reading 'length')".into());
    }
    if let Some(s) = v.as_str() {
        return Ok(parse_u64_str(s));
    }
    Ok(parse_u64_str(&js_to_string(v)))
}

fn value_of_u64(v: &Value) -> Result<Option<u128>, String> {
    if let Some(s) = marker(v, "$bigint").and_then(Value::as_str) {
        if s.starts_with('-') {
            return Err("RangeError: Unsigned value must not be negative".into());
        }
        return Ok(s.parse::<u128>().ok());
    }
    parse_u64_value(v)
}

fn pct_u64(part: Option<u128>, whole: Option<u128>) -> Value {
    let (Some(p), Some(w)) = (part, whole) else {
        return Value::Null;
    };
    if w == 0 {
        return Value::from(0);
    }
    let clamped = p.min(w);
    let scaled = clamped
        .checked_mul(10_000)
        .and_then(|n| n.checked_add(w / 2))
        .map(|n| n / w)
        .unwrap_or_else(|| ((clamped as f64 / w as f64) * 10_000.0).round() as u128);
    js::number((scaled as f64) / 100.0)
}

fn bigint_json(n: Option<u128>) -> Value {
    match n {
        Some(v) => json!({ "$bigint": v.to_string() }),
        None => Value::Null,
    }
}

fn fold_families_vec(mut ordered: Vec<Value>) -> Result<Value, String> {
    if ordered.len() >= 2 {
        for row in &ordered {
            check_object_coercible(row, "ip_version")?;
        }
        ordered.sort_by_key(|row| usize::from(prop_str(row, "ip_version") != Some("ipv4")));
    }
    for row in &ordered {
        check_object_coercible(row, "state")?;
        if prop_str(row, "state") == Some("healthy")
            && prop_opt(row, "latency_ms").is_some_and(|v| !is_nullish(v))
        {
            return Ok(row.clone());
        }
    }
    for row in &ordered {
        if prop_str(row, "state") == Some("healthy") {
            return Ok(row.clone());
        }
    }
    for row in &ordered {
        if prop_str(row, "state") == Some("unavailable") {
            return Ok(row.clone());
        }
    }
    Ok(ordered.into_iter().next().unwrap_or_else(js::undefined))
}

fn pseudo_owner_ref(node: &Value) -> Result<&'static str, String> {
    check_object_coercible(node, "protocol")?;
    Ok(if is_builtin_outbound(prop_str(node, "protocol")) {
        "builtin"
    } else {
        "unattributed"
    })
}

// ---- Exported ports ----------------------------------------------------------------------------

/// Port of `safeHttpUrl` (`src/api/selectors.ts:24`).
pub fn safe_http_url(value: String) -> Result<bool, String> {
    if value.len() > 2048 || value.chars().any(|c| c == ',' || is_js_whitespace(c)) {
        return Ok(false);
    }
    let rest = if value.len() >= 7 && value.as_bytes()[..7].eq_ignore_ascii_case(b"http://") {
        &value[7..]
    } else if value.len() >= 8 && value.as_bytes()[..8].eq_ignore_ascii_case(b"https://") {
        &value[8..]
    } else {
        return Ok(false);
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') {
        return Ok(false);
    }
    Ok(match Url::parse(&value) {
        Ok(url) => url.host_str().is_some_and(|h| !h.is_empty()) && url.port() != Some(0),
        Err(_) => false,
    })
}

/// Port of `measured` (`src/api/selectors.ts:50`).
pub fn measured(h: Value) -> Result<Value, String> {
    check_object_coercible(&h, "error")?;
    Ok(Value::Bool(
        prop_str(&h, "error") != Some("no_node_address"),
    ))
}

/// Port of `preferredObservation` (`src/api/selectors.ts:51`).
pub fn preferred_observation(health: Value) -> Result<Value, String> {
    check_object_coercible(&health, "filter")?;
    let Some(items) = health.as_array() else {
        return Err("TypeError: health.filter is not a function".into());
    };
    let mut data = Vec::new();
    for h in items {
        check_object_coercible(h, "transport")?;
        if prop_str(h, "transport") == Some("tcp")
            && prop_str(h, "purpose") == Some("data")
            && prop_str(h, "error") != Some("no_node_address")
        {
            data.push(h.clone());
        }
    }
    let mut best: Option<i32> = None;
    let mut ranks = Vec::with_capacity(data.len());
    for h in &data {
        let r = observation_rank(h)?;
        best = Some(best.map_or(r, |b| b.min(r)));
        ranks.push(r);
    }
    let filtered: Vec<Value> = match best {
        Some(b) => data
            .into_iter()
            .zip(ranks)
            .filter_map(|(h, r)| (r == b).then_some(h))
            .collect(),
        None => Vec::new(),
    };
    fold_families_vec(filtered)
}

/// Port of `foldFamilies` (`src/api/selectors.ts:58`).
pub fn fold_families(rows: Value) -> Result<Value, String> {
    let items: Vec<Value> = if let Some(arr) = rows.as_array() {
        arr.clone()
    } else if let Some(s) = rows.as_str() {
        s.chars().map(|c| Value::String(c.to_string())).collect()
    } else if let Some(set) = marker(&rows, "$set").and_then(Value::as_array) {
        set.clone()
    } else if let Some(map) = marker(&rows, "$map").and_then(Value::as_array) {
        map.clone()
    } else {
        return Err("TypeError: rows is not iterable".into());
    };
    fold_families_vec(items)
}

/// Port of `preferredHealth` (`src/api/selectors.ts:67`).
pub fn preferred_health(node: Value) -> Result<Value, String> {
    check_object_coercible(&node, "health")?;
    preferred_observation(prop_or_undef(&node, "health"))
}

/// Port of `probeKinds` (`src/api/selectors.ts:75`).
pub fn probe_kinds(health: Value) -> Result<Value, String> {
    check_object_coercible(&health, "filter")?;
    let Some(items) = health.as_array() else {
        return Err(
            "TypeError: health.filter is not a function or its return value is not iterable".into(),
        );
    };
    struct Bucket {
        key: String,
        kind: &'static str,
        transport: Value,
        rows: Vec<Value>,
    }
    let mut buckets: Vec<Bucket> = Vec::new();
    for h in items {
        check_object_coercible(h, "error")?;
        if prop_str(h, "error") == Some("no_node_address") {
            continue;
        }
        let kind = if prop_str(h, "purpose") == Some("dns") {
            "DNS"
        } else if prop_str(h, "transport") == Some("udp") {
            "UDP"
        } else {
            let meas = prop_or_undef(h, "measurement");
            check_object_coercible(&meas, "startsWith")?;
            let Some(ms) = meas.as_str() else {
                return Err("TypeError: h.measurement.startsWith is not a function".into());
            };
            if ms.starts_with("http") {
                "HTTP"
            } else {
                "TCP"
            }
        };
        let transport = prop_or_undef(h, "transport");
        let key = format!("{kind}/{}", js_to_string(&transport));
        if let Some(bucket) = buckets.iter_mut().find(|b| b.key == key) {
            bucket.rows.push(h.clone());
        } else {
            buckets.push(Bucket {
                key,
                kind,
                transport,
                rows: vec![h.clone()],
            });
        }
    }

    let mut out = Vec::with_capacity(buckets.len());
    for bucket in buckets {
        let mut latest: Vec<(Value, f64)> = Vec::new();
        for h in &bucket.rows {
            let ipv = prop_or_undef(h, "ip_version");
            let ts = parse_iso_date(&js_to_string(&prop_or_undef(h, "observed_at")));
            if let Some((_, cur)) = latest.iter_mut().find(|(k, _)| same_value_zero(k, &ipv)) {
                *cur = if cur.is_nan() || ts.is_nan() {
                    f64::NAN
                } else {
                    cur.max(ts)
                };
            } else {
                latest.push((ipv, ts));
            }
        }
        let mut current = Vec::new();
        for h in &bucket.rows {
            let ipv = prop_or_undef(h, "ip_version");
            let ts = parse_iso_date(&js_to_string(&prop_or_undef(h, "observed_at")));
            let max_ts = latest
                .iter()
                .find(|(k, _)| same_value_zero(k, &ipv))
                .map_or(f64::NAN, |(_, v)| *v);
            if ts == max_ts {
                current.push(h.clone());
            }
        }
        let mut best: Option<i32> = None;
        let mut ranks = Vec::with_capacity(current.len());
        for h in &current {
            let r = observation_rank(h)?;
            best = Some(best.map_or(r, |b| b.min(r)));
            ranks.push(r);
        }
        let top: Vec<Value> = match best {
            Some(b) => current
                .into_iter()
                .zip(ranks)
                .filter_map(|(h, r)| (r == b).then_some(h))
                .collect(),
            None => Vec::new(),
        };
        let obs = fold_families_vec(top)?;
        out.push((bucket.kind, bucket.transport, obs));
    }

    let kind_ord = |k: &str| match k {
        "TCP" => 0,
        "HTTP" => 1,
        "UDP" => 2,
        _ => 3,
    };
    if out.len() >= 2 {
        out.sort_by(|a, b| {
            kind_ord(a.0).cmp(&kind_ord(b.0)).then_with(|| {
                let ta = a.1.as_str().unwrap_or("");
                let tb = b.1.as_str().unwrap_or("");
                ta.to_lowercase().cmp(&tb.to_lowercase()).then(ta.cmp(tb))
            })
        });
    }

    Ok(Value::Array(
        out.into_iter()
            .map(|(kind, transport, obs)| {
                let mut m = Map::new();
                m.insert("kind".into(), Value::String(kind.into()));
                m.insert("transport".into(), transport);
                m.insert("observation".into(), obs);
                Value::Object(m)
            })
            .collect(),
    ))
}

/// Port of `healthMillis` (`src/api/selectors.ts:94`).
pub fn health_millis(health: Option<Value>) -> Result<Value, String> {
    let Some(h) = health else {
        return Ok(js::undefined());
    };
    if prop_str(&h, "state") == Some("healthy")
        && let Some(lat) = prop_opt(&h, "latency_ms")
        && !is_nullish(lat)
    {
        return Ok(lat.clone());
    }
    Ok(js::undefined())
}

/// Port of `compareLatency` (`src/api/selectors.ts:96`).
pub fn compare_latency(a: Option<f64>, b: Option<f64>) -> Result<Value, String> {
    let diff = a.unwrap_or(f64::INFINITY) - b.unwrap_or(f64::INFINITY);
    let res = if diff == 0.0 || diff.is_nan() {
        0.0
    } else {
        diff
    };
    Ok(js::number(res))
}

/// Port of `outboundUsage` (`src/api/selectors.ts:98`).
pub fn outbound_usage(snapshot: Option<Value>) -> Result<Value, String> {
    let Some(snap) = snapshot.filter(is_truthy) else {
        return Ok(json!({ "rows": [], "total": null }));
    };
    let outbounds = prop_or_undef(&snap, "outbounds");
    check_object_coercible(&outbounds, "map")?;
    let Some(items) = outbounds.as_array() else {
        return Err("TypeError: snapshot.outbounds.map is not a function".into());
    };
    let mut dl_vals = Vec::with_capacity(items.len());
    let mut total: Option<u128> = Some(0);
    for row in items {
        check_object_coercible(row, "download_bytes")?;
        let raw_dl = prop_or_undef(row, "download_bytes");
        dl_vals.push(raw_dl);
    }
    for raw_dl in &dl_vals {
        if let Some(acc) = total {
            total = value_of_u64(raw_dl)?.map(|n| acc.saturating_add(n));
        }
    }
    let mut rows: Vec<(Option<u128>, Value)> = Vec::with_capacity(items.len());
    for (row, raw_dl) in items.iter().zip(dl_vals.iter()) {
        let bytes = parse_u64_value(raw_dl)?;
        let errors = parse_u64_value(&prop_or_undef(row, "errors"))?;
        let part = value_of_u64(raw_dl)?;
        let percent = pct_u64(part, total);
        let mut obj = Map::new();
        obj.insert("name".into(), prop_or_undef(row, "name"));
        obj.insert("kind".into(), prop_or_undef(row, "kind"));
        obj.insert("bytes".into(), bigint_json(bytes));
        obj.insert("errors".into(), bigint_json(errors));
        obj.insert("percent".into(), percent);
        rows.push((bytes, Value::Object(obj)));
    }
    rows.sort_by(|(a, _), (b, _)| match (a, b) {
        (Some(x), Some(y)) => y.cmp(x),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    let mut out = Map::new();
    out.insert(
        "rows".into(),
        Value::Array(rows.into_iter().map(|(_, v)| v).collect()),
    );
    out.insert("total".into(), bigint_json(total));
    Ok(Value::Object(out))
}

/// Port of `simulatedAddress` (`src/api/selectors.ts:112`).
pub fn simulated_address(result: Value) -> Result<Value, String> {
    check_object_coercible(&result, "type")?;
    let rtype = prop_str(&result, "type");
    if !matches!(rtype, Some("A" | "AAAA")) {
        return Ok(Value::Null);
    }
    let answers = prop_or_undef(&result, "answers");
    if is_nullish(&answers) {
        return Ok(Value::Null);
    }
    let Some(arr) = answers.as_array() else {
        return Err("TypeError: (result.answers ?? []).find is not a function".into());
    };
    for answer in arr {
        check_object_coercible(answer, "type")?;
        if prop_str(answer, "type") == rtype {
            let data = prop_or_undef(answer, "data");
            return Ok(if is_nullish(&data) { Value::Null } else { data });
        }
    }
    Ok(Value::Null)
}

/// Port of `ipLiteral` (`src/api/selectors.ts:117`).
pub fn ip_literal(text: String) -> Result<Option<String>, String> {
    let trimmed = trim_js(&text);
    let value = if trimmed.starts_with('[')
        && trimmed.ends_with(']')
        && trimmed.len() >= 3
        && !trimmed[1..trimmed.len() - 1].contains(']')
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let mut octets = value.split('.');
    let valid_v4 = (0..4).all(|_| {
        octets.next().is_some_and(|part| {
            !part.is_empty()
                && part.len() <= 3
                && (part == "0" || !part.starts_with('0'))
                && part.bytes().all(|b| b.is_ascii_digit())
                && part.parse::<u16>().is_ok_and(|n| n <= 255)
        })
    }) && octets.next().is_none();
    if valid_v4 {
        return Ok(Some(value.to_string()));
    }
    if !value.contains(':')
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() || b == b':' || b == b'.')
    {
        return Ok(None);
    }
    Ok(Url::parse(&format!("http://[{value}]/"))
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .and_then(|h| {
            h.strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .map(str::to_string)
        }))
}

/// Port of `pseudoOwner` (`src/api/selectors.ts:131`).
pub fn pseudo_owner(node: Value) -> Result<String, String> {
    pseudo_owner_ref(&node).map(str::to_string)
}

/// Port of `noNodeSources` (`src/api/selectors.ts:133`).
pub fn no_node_sources(providers: Value, nodes: Value) -> Result<bool, String> {
    if !is_truthy(&providers) || !is_truthy(&nodes) {
        return Ok(false);
    }
    let Some(prov_arr) = providers.as_array() else {
        return Err("TypeError: providers.some is not a function".into());
    };
    for provider in prov_arr {
        check_object_coercible(provider, "kind")?;
        if matches!(prop_str(provider, "kind"), Some("subscription" | "file")) {
            return Ok(false);
        }
    }
    let Some(node_arr) = nodes.as_array() else {
        return Err("TypeError: nodes.every is not a function".into());
    };
    for node in node_arr {
        if pseudo_owner_ref(node)? != "builtin" {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Port of `pseudoOwnerId` (`src/api/selectors.ts:141`).
pub fn pseudo_owner_id(kind: String, providers: Value) -> Result<String, String> {
    check_object_coercible(&providers, "some")?;
    let Some(arr) = providers.as_array() else {
        return Err("TypeError: providers.some is not a function".into());
    };
    let mut id = kind;
    loop {
        let mut collides = false;
        for provider in arr {
            check_object_coercible(provider, "id")?;
            if prop_str(provider, "id") == Some(id.as_str()) {
                collides = true;
                break;
            }
        }
        if collides {
            id.push('-');
        } else {
            return Ok(id);
        }
    }
}

/// Port of `nodeOwner` (`src/api/selectors.ts:147`).
pub fn node_owner(node: Value, providers: Value) -> Result<Value, String> {
    check_object_coercible(&node, "provider_id")?;
    if let Some(pid) = prop_opt(&node, "provider_id")
        && !is_nullish(pid)
    {
        return Ok(pid.clone());
    }
    let kind = pseudo_owner_ref(&node)?.to_string();
    Ok(Value::String(pseudo_owner_id(kind, providers)?))
}

/// Port of `sourceIp` (`src/api/selectors.ts:150`).
pub fn source_ip(src: Option<String>) -> Result<Option<String>, String> {
    let Some(s) = src.filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let inner = if s.starts_with('[') {
        match s.find(']') {
            Some(end) => s[1..end].to_string(),
            None => {
                let u = utf16::units(&s);
                if u.len() <= 2 {
                    String::new()
                } else {
                    utf16::slice(&u, 1, u.len() - 1)
                }
            }
        }
    } else if s.split(':').count() == 2 {
        s.split(':').next().unwrap_or("").to_string()
    } else {
        s
    };
    ip_literal(inner)
}

/// Port of `addressPort` (`src/api/selectors.ts:155`).
pub fn address_port(address: Option<String>) -> Result<Value, String> {
    let Some(s) = address else {
        return Ok(js::undefined());
    };
    let Some((host, port)) = s.rsplit_once(':') else {
        return Ok(js::undefined());
    };
    let port_ok = !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit());
    let bracketed = host.starts_with('[')
        && host.ends_with(']')
        && host.len() >= 3
        && !host[1..host.len() - 1].contains(']');
    let bare = !host.is_empty() && !host.contains(':');
    if port_ok && (bracketed || bare) {
        Ok(Value::String(port.to_string()))
    } else {
        Ok(js::undefined())
    }
}

/// Port of `chainNames` (`src/api/selectors.ts:162`).
pub fn chain_names(chain: Vec<String>, names: Option<Value>) -> Result<Value, String> {
    if chain.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let map = match names.as_ref().filter(|v| !is_nullish(v)) {
        None => None,
        Some(v) => Some(map_entries(v, "names?")?),
    };
    let out = chain
        .into_iter()
        .map(|id| {
            let key = Value::String(id);
            map.and_then(|entries| map_get(entries, &key))
                .filter(|v| !is_nullish(v))
                .unwrap_or(key)
        })
        .collect();
    Ok(Value::Array(out))
}

/// Port of `nodeSubscription` (`src/api/selectors.ts:175`).
pub fn node_subscription(row: Value, subscriptions: Option<Value>) -> Result<Value, String> {
    check_object_coercible(&row, "outbound")?;
    if is_builtin_outbound(prop_str(&row, "outbound")) {
        return Ok(Value::Null);
    }
    let Some(subs) = subscriptions.filter(|v| !is_nullish(v)) else {
        return Ok(Value::Null);
    };
    let chain = prop_or_undef(&row, "chain");
    check_object_coercible(&chain, "at")?;
    let leaf = if let Some(arr) = chain.as_array() {
        arr.last()
            .filter(|v| !is_nullish(v))
            .cloned()
            .unwrap_or_else(|| Value::String(String::new()))
    } else if let Some(units) = js::string_units(&chain) {
        units
            .last()
            .map(|&u| js::utf16_string(&[u]))
            .unwrap_or_else(|| Value::String(String::new()))
    } else {
        return Err("TypeError: row.chain.at is not a function".into());
    };
    let entries = map_entries(&subs, "subscriptions?")?;
    Ok(map_get(entries, &leaf)
        .filter(is_truthy)
        .unwrap_or(Value::Null))
}

/// Port of `lifecycleTone` (`src/api/selectors.ts:189`).
pub fn lifecycle_tone(state: Option<Value>) -> Result<Value, String> {
    let tone = match state.as_ref().and_then(Value::as_str) {
        Some("running") => "ok",
        Some("failed") => "err",
        _ => "warn",
    };
    Ok(Value::String(tone.into()))
}

/// Port of `memoryTone` (`src/api/selectors.ts:191`).
pub fn memory_tone(percent: f64) -> Result<String, String> {
    Ok(if percent > 90.0 {
        "err"
    } else if percent > 75.0 {
        "warn"
    } else {
        "ok"
    }
    .into())
}

/// Port of `closedAllTone` (`src/api/selectors.ts:193`).
pub fn closed_all_tone(tally: Value) -> Result<Value, String> {
    check_object_coercible(&tally, "closed")?;
    let tone = if !is_truthy(&prop_or_undef(&tally, "closed")) {
        "negative"
    } else if is_truthy(&prop_or_undef(&tally, "skipped")) {
        "info"
    } else {
        "positive"
    };
    Ok(Value::String(tone.into()))
}

/// Port of `connectionRows` (`src/api/selectors.ts:213`).
pub fn connection_rows(snapshot: Option<Value>) -> Result<Value, String> {
    let Some(snap) = snapshot.filter(is_truthy) else {
        return Ok(Value::Array(Vec::new()));
    };
    let tag_list = |key: &str, network: &str| -> Result<Vec<Value>, String> {
        let list = prop_or_undef(&snap, key);
        check_object_coercible(&list, "map")?;
        let Some(arr) = list.as_array() else {
            return Err(format!("TypeError: snapshot.{key}.map is not a function"));
        };
        let mut rows = Vec::with_capacity(arr.len());
        for c in arr {
            let mut m = match c {
                Value::Object(obj)
                    if !js::is_undefined(c)
                        && js::as_number(c).is_none()
                        && marker(c, "$utf16").is_none()
                        && marker(c, "$bigint").is_none() =>
                {
                    obj.clone()
                }
                Value::Array(elems) => elems
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i.to_string(), v.clone()))
                    .collect(),
                _ => return Err("TypeError: Invalid value used as weak map key".into()),
            };
            m.insert("network".into(), Value::String(network.into()));
            rows.push(Value::Object(m));
        }
        Ok(rows)
    };
    let mut out = tag_list("tcp", "tcp")?;
    out.extend(tag_list("udp", "udp")?);
    Ok(Value::Array(out))
}

/// Port of `routineGap` (`src/api/selectors.ts:231`).
pub fn routine_gap(event: Value) -> Result<bool, String> {
    check_object_coercible(&event, "event")?;
    if prop_str(&event, "event") != Some("flow.gap") {
        return Ok(false);
    }
    let data = prop_or_undef(&event, "data");
    if is_nullish(&data) {
        return Err(format!(
            "TypeError: Cannot destructure property 'reason' of 'event.data' as it is {}.",
            nullish_label(&data)
        ));
    }
    Ok(matches!(
        prop_str(&data, "reason"),
        Some("evicted" | "sampled" | "buffer_overflow")
    ))
}

/// Port of `shortId` (`src/api/selectors.ts:284`).
pub fn short_id(id: String) -> Result<Value, String> {
    let b = id.as_bytes();
    let is_uuid_prefix = b.len() >= 14
        && b[..8].iter().all(u8::is_ascii_hexdigit)
        && b[8] == b'-'
        && b[9..13].iter().all(u8::is_ascii_hexdigit)
        && b[13] == b'-';
    Ok(Value::String(if is_uuid_prefix {
        id[..8].to_string()
    } else {
        id
    }))
}

/// Port of `resolveSelectedLeaf` (`src/api/selectors.ts:286`).
pub fn resolve_selected_leaf(
    outbound: String,
    network: String,
    groups_by_name: Value,
    groups_by_id: Value,
    nodes_by_id: Value,
) -> Result<Value, String> {
    let by_name = map_entries(&groups_by_name, "groupsByName")?;
    let mut groups: Vec<Value> = Vec::new();
    let mut visited: Vec<Value> = Vec::new();
    let mut group = map_get(by_name, &Value::String(outbound));
    while let Some(g) = group.filter(is_truthy) {
        let id = prop_or_undef(&g, "id");
        if visited.iter().any(|v| same_value_zero(v, &id)) {
            break;
        }
        visited.push(id);
        let selection = prop_or_undef(&g, "selection");
        let member_key = if network == "udp" {
            "udp_member_id"
        } else {
            "tcp_member_id"
        };
        check_object_coercible(&selection, member_key)?;
        let member = prop_or_undef(&selection, member_key);
        groups.push(g);
        if !is_truthy(&member) {
            return Ok(json!({ "groups": groups, "member": null, "node": null }));
        }
        let by_id = map_entries(&groups_by_id, "groupsById")?;
        let next = map_get(by_id, &member);
        if !next.as_ref().is_some_and(is_truthy) {
            let nodes = map_entries(&nodes_by_id, "nodesById")?;
            let node = map_get(nodes, &member)
                .filter(|v| !is_nullish(v))
                .unwrap_or(Value::Null);
            return Ok(json!({ "groups": groups, "member": member, "node": node }));
        }
        group = next;
    }
    Ok(json!({ "groups": groups, "member": null, "node": null }))
}

/// Port of `interestingNotice` (`src/api/selectors.ts:330`).
pub fn interesting_notice(event: Value) -> Result<Value, String> {
    check_object_coercible(&event, "event")?;
    if matches!(
        prop_str(&event, "event"),
        Some("stream.ready" | "runtime.updated" | "flow.updated")
    ) {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(!routine_gap(event)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_http_url_rules() {
        assert!(safe_http_url("https://example.com/sub?a=1".into()).unwrap());
        assert!(safe_http_url("HTTP://1.1.1.1:8080/@path".into()).unwrap());
        assert!(safe_http_url("https://[2001:db8::1]:443/x".into()).unwrap());
        assert!(!safe_http_url("https://user@example.com/".into()).unwrap());
        assert!(!safe_http_url("http://example.com:0/".into()).unwrap());
        assert!(!safe_http_url("https://example.com/a,b".into()).unwrap());
        assert!(!safe_http_url("https://example.com/\u{FEFF}".into()).unwrap());
        assert!(!safe_http_url("ftp://example.com/".into()).unwrap());
        assert!(!safe_http_url(format!("https://a.co/{}", "a".repeat(2040))).unwrap());
    }

    #[test]
    fn ip_literal_and_source_ip_and_port() {
        assert_eq!(
            ip_literal(" 10.0.0.12 ".into()).unwrap(),
            Some("10.0.0.12".into())
        );
        assert_eq!(
            ip_literal("[2001:0db8::0001]".into()).unwrap(),
            Some("2001:db8::1".into())
        );
        assert_eq!(
            ip_literal("::ffff:192.0.2.128".into()).unwrap(),
            Some("::ffff:c000:280".into())
        );
        assert_eq!(ip_literal("01.2.3.4".into()).unwrap(), None);
        assert_eq!(ip_literal("256.0.0.1".into()).unwrap(), None);
        assert_eq!(
            source_ip(Some("149.154.167.220:443".into())).unwrap(),
            Some("149.154.167.220".into())
        );
        assert_eq!(
            source_ip(Some("[2001:db8::1]:8443".into())).unwrap(),
            Some("2001:db8::1".into())
        );
        assert_eq!(source_ip(Some("[::1".into())).unwrap(), Some("::".into()));
        assert_eq!(source_ip(None).unwrap(), None);
        assert_eq!(
            address_port(Some("149.154.167.220:443".into())).unwrap(),
            json!("443")
        );
        assert_eq!(
            address_port(Some("[2001:db8::1]:8443".into())).unwrap(),
            json!("8443")
        );
        assert!(js::is_undefined(
            &address_port(Some("2001:db8::1".into())).unwrap()
        ));
        assert!(js::is_undefined(&address_port(None).unwrap()));
    }

    #[test]
    fn health_selection_and_probe_kinds() {
        assert_eq!(measured(json!({"error": null})).unwrap(), json!(true));
        assert_eq!(
            measured(json!({"error": "no_node_address"})).unwrap(),
            json!(false)
        );
        assert!(measured(Value::Null).is_err());

        let rows = json!([
            {"transport": "tcp", "purpose": "data", "ip_version": "ipv6", "warmth": "warm", "measurement": "tcp_connect", "state": "healthy", "latency_ms": 50, "observed_at": "2026-11-28T05:03:21.833Z", "error": null},
            {"transport": "tcp", "purpose": "data", "ip_version": "ipv4", "warmth": "warm", "measurement": "http_headers", "state": "healthy", "latency_ms": 20, "observed_at": "2026-11-28T05:03:22.000Z", "error": null},
            {"transport": "udp", "purpose": "dns", "ip_version": "ipv4", "warmth": "warm", "measurement": "dns_round_trip", "state": "healthy", "latency_ms": 15, "observed_at": "2026-11-28T05:03:21.000Z", "error": null}
        ]);
        let pref = preferred_observation(rows.clone()).unwrap();
        assert_eq!(pref["measurement"], json!("http_headers"));
        assert_eq!(
            preferred_health(json!({"health": rows})).unwrap()["latency_ms"],
            json!(20)
        );
        assert_eq!(fold_families(json!("x")).unwrap(), json!("x"));
        assert!(fold_families(json!({})).is_err());
        assert_eq!(health_millis(Some(pref)).unwrap(), json!(20));
        assert!(js::is_undefined(&health_millis(None).unwrap()));

        let kinds = probe_kinds(rows).unwrap();
        let arr = kinds.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["kind"], json!("TCP"));
        assert_eq!(arr[1]["kind"], json!("HTTP"));
        assert_eq!(arr[2]["kind"], json!("DNS"));
    }

    #[test]
    fn latency_usage_and_dns() {
        assert_eq!(compare_latency(Some(1.0), Some(-2.5)).unwrap(), json!(3.5));
        assert_eq!(
            compare_latency(Some(f64::INFINITY), None).unwrap(),
            json!(0)
        );
        assert_eq!(
            compare_latency(Some(f64::NAN), Some(1.0)).unwrap(),
            json!(0)
        );

        assert_eq!(
            outbound_usage(None).unwrap(),
            json!({"rows": [], "total": null})
        );
        let usage = outbound_usage(Some(json!({
            "outbounds": [
                {"name": "b", "kind": "node", "download_bytes": "100", "errors": "0"},
                {"name": "a", "kind": "node", "download_bytes": "300", "errors": "2"},
                {"name": "c", "kind": "node", "download_bytes": null, "errors": null}
            ]
        })))
        .unwrap();
        assert_eq!(usage["total"], Value::Null);
        assert_eq!(usage["rows"][0]["name"], json!("a"));
        assert_eq!(usage["rows"][2]["name"], json!("c"));

        let usage2 = outbound_usage(Some(json!({
            "outbounds": [
                {"name": "a", "kind": "node", "download_bytes": "1", "errors": "0"},
                {"name": "b", "kind": "node", "download_bytes": "3", "errors": "0"}
            ]
        })))
        .unwrap();
        assert_eq!(usage2["total"], json!({"$bigint": "4"}));
        assert_eq!(usage2["rows"][0]["percent"], json!(75));

        assert_eq!(
            simulated_address(json!({"type": "A", "answers": [{"type": "CNAME", "data": "x"}, {"type": "A", "data": "1.1.1.1"}]})).unwrap(),
            json!("1.1.1.1")
        );
        assert_eq!(
            simulated_address(json!({"type": "TXT"})).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn owners_chains_tones_and_events() {
        assert_eq!(
            pseudo_owner(json!({"protocol": "direct"})).unwrap(),
            "builtin"
        );
        assert_eq!(
            pseudo_owner(json!({"protocol": "vless"})).unwrap(),
            "unattributed"
        );
        assert!(no_node_sources(json!([]), json!([{"protocol": "direct"}])).unwrap());
        assert!(!no_node_sources(json!([{"kind": "file"}]), json!([])).unwrap());
        assert!(!no_node_sources(json!({"a": 1}), Value::Null).unwrap());
        assert_eq!(
            node_owner(
                json!({"provider_id": null, "protocol": "block"}),
                json!([{"id": "builtin"}, {"id": "builtin-"}])
            )
            .unwrap(),
            json!("builtin--")
        );

        let names = json!({"$map": [["g1", "Proxy"], ["n1", "HK"]]});
        assert_eq!(
            chain_names(vec!["g1".into(), "n2".into()], Some(names.clone())).unwrap(),
            json!(["Proxy", "n2"])
        );
        assert_eq!(
            node_subscription(
                json!({"outbound": "proxy", "chain": ["g1", "n1"]}),
                Some(names)
            )
            .unwrap(),
            json!("HK")
        );
        assert_eq!(
            node_subscription(json!({"outbound": "direct", "chain": []}), Some(json!({}))).unwrap(),
            Value::Null
        );

        assert_eq!(lifecycle_tone(Some(json!("running"))).unwrap(), json!("ok"));
        assert_eq!(lifecycle_tone(Some(json!("failed"))).unwrap(), json!("err"));
        assert_eq!(lifecycle_tone(None).unwrap(), json!("warn"));
        assert_eq!(memory_tone(90.5).unwrap(), "err");
        assert_eq!(memory_tone(75.5).unwrap(), "warn");
        assert_eq!(memory_tone(75.0).unwrap(), "ok");
        assert_eq!(
            closed_all_tone(json!({"closed": 0, "skipped": 2})).unwrap(),
            json!("negative")
        );
        assert_eq!(
            closed_all_tone(json!({"closed": 2, "skipped": 1})).unwrap(),
            json!("info")
        );
        assert_eq!(
            closed_all_tone(json!({"closed": 2, "skipped": 0})).unwrap(),
            json!("positive")
        );

        let rows =
            connection_rows(Some(json!({"tcp": [{"id": "a"}], "udp": [{"id": "b"}]}))).unwrap();
        assert_eq!(
            rows,
            json!([{"id": "a", "network": "tcp"}, {"id": "b", "network": "udp"}])
        );

        let gap_ev = json!({"event": "flow.gap", "data": {"reason": "evicted"}});
        assert!(routine_gap(gap_ev.clone()).unwrap());
        assert_eq!(interesting_notice(gap_ev).unwrap(), json!(false));
        assert_eq!(
            interesting_notice(
                json!({"event": "flow.gap", "data": {"reason": "recording_changed"}})
            )
            .unwrap(),
            json!(true)
        );

        assert_eq!(
            short_id("e5af7786-442b-4f06-8ba5-ffa47f7dec70".into()).unwrap(),
            json!("e5af7786")
        );
        assert_eq!(short_id("gen-41".into()).unwrap(), json!("gen-41"));

        let g0 = json!({"id": "g0", "selection": {"tcp_member_id": "g1", "udp_member_id": null}});
        let g1 = json!({"id": "g1", "selection": {"tcp_member_id": "n1", "udp_member_id": null}});
        let n1 = json!({"id": "n1", "name": "HK"});
        let by_id = json!({"$map": [["g0", g0], ["g1", g1]]});
        let nodes = json!({"$map": [["n1", n1]]});
        let res =
            resolve_selected_leaf("g0".into(), "tcp".into(), by_id.clone(), by_id, nodes).unwrap();
        assert_eq!(res["member"], json!("n1"));
        assert_eq!(res["node"]["name"], json!("HK"));
    }
}
