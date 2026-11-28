//! The Rust port of `apps/lite/src/groupHealth.ts`.

#![forbid(unsafe_code)]

use serde_json::{Map, Value, json};

use crate::js;

fn is_nullish(v: &Value) -> bool {
    v.is_null() || js::is_undefined(v)
}

fn check_obj(v: &Value, prop: &str) -> Result<(), String> {
    if is_nullish(v) {
        let kind = if v.is_null() { "null" } else { "undefined" };
        return Err(format!(
            "TypeError: Cannot read properties of {kind} (reading '{prop}')"
        ));
    }
    Ok(())
}

fn prop_opt<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    v.as_object().and_then(|o| o.get(key))
}

fn prop(v: &Value, key: &str) -> Value {
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

fn eq_js(a: &Value, b: &Value, same_value_zero: bool) -> bool {
    if js::is_undefined(a) && js::is_undefined(b) {
        return true;
    }
    if let (Some(x), Some(y)) = (js::as_number(a), js::as_number(b)) {
        return x == y || (same_value_zero && x.is_nan() && y.is_nan());
    }
    if let (Some(x), Some(y)) = (js::string_units(a), js::string_units(b)) {
        return x == y;
    }
    if same_value_zero {
        return a == b;
    }
    if let (Some(x), Some(y)) = (marker(a, "$bigint"), marker(b, "$bigint")) {
        return x == y;
    }
    matches!((a, b), (Value::Null, Value::Null))
        || matches!((a, b), (Value::Bool(x), Value::Bool(y)) if x == y)
}

fn is_js_ws(u: u16) -> bool {
    matches!(
        u,
        0x0009..=0x000D | 0x0020 | 0x00A0 | 0x1680 | 0x2000..=0x200A
            | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF
    )
}

fn map_set(entries: &mut Vec<(Value, Value)>, key: Value, val: Value) {
    let key = if js::as_number(&key) == Some(0.0) {
        Value::from(0)
    } else {
        key
    };
    if let Some((_, slot)) = entries.iter_mut().find(|(k, _)| eq_js(k, &key, true)) {
        *slot = val;
    } else {
        entries.push((key, val));
    }
}

fn map_get(entries: &[(Value, Value)], key: &Value) -> Option<Value> {
    entries
        .iter()
        .find(|(k, _)| eq_js(k, key, true))
        .map(|(_, v)| v.clone())
}

fn preferred_obs(health: &Value) -> Result<Value, String> {
    check_obj(health, "filter")?;
    let Some(items) = health.as_array() else {
        return Err("TypeError: health.filter is not a function".into());
    };
    let mut ranked: Vec<(i32, &Value)> = Vec::new();
    for h in items {
        check_obj(h, "transport")?;
        if prop_str(h, "transport") == Some("tcp")
            && prop_str(h, "purpose") == Some("data")
            && prop_str(h, "error") != Some("no_node_address")
        {
            let w = match prop_str(h, "warmth") {
                Some("warm") => 0,
                Some("mixed") => 2,
                Some("cold") => 3,
                _ => 1,
            };
            let m = match prop_str(h, "measurement") {
                Some("http_headers") => 0,
                Some("http_round_trip") => 1,
                Some("quic_handshake") => 2,
                Some("tcp_connect") => 3,
                Some("mixed") => 4,
                Some("dns_round_trip") => 6,
                _ => 5,
            };
            ranked.push((w * 10 + m, h));
        }
    }
    let Some(best) = ranked.iter().map(|(r, _)| *r).min() else {
        return Ok(js::undefined());
    };
    let mut top: Vec<&Value> = ranked
        .into_iter()
        .filter(|(r, _)| *r == best)
        .map(|(_, h)| h)
        .collect();
    top.sort_by_key(|h| prop_str(h, "ip_version") != Some("ipv4"));
    let pick = top
        .iter()
        .find(|h| {
            prop_str(h, "state") == Some("healthy")
                && prop_opt(h, "latency_ms").is_some_and(|v| !is_nullish(v))
        })
        .or_else(|| top.iter().find(|h| prop_str(h, "state") == Some("healthy")))
        .or_else(|| {
            top.iter()
                .find(|h| prop_str(h, "state") == Some("unavailable"))
        })
        .or_else(|| top.first());
    Ok(pick.map_or_else(js::undefined, |&h| h.clone()))
}

fn fallback_get(fallback: &Value, key: &Value) -> Result<Value, String> {
    check_obj(fallback, "get")?;
    let Some(entries) = marker(fallback, "$map").and_then(Value::as_array) else {
        return Err("TypeError: fallback.get is not a function".into());
    };
    let mut found = None;
    for pair in entries {
        if let Some(a) = pair.as_array()
            && a.len() >= 2
            && eq_js(&a[0], key, true)
        {
            found = Some(a[1].clone());
        }
    }
    Ok(found.unwrap_or_else(js::undefined))
}

fn member_health_vec(group: &Value, fallback: &Value) -> Result<Vec<Value>, String> {
    if !is_truthy(group) {
        return Ok(Vec::new());
    }
    let runtime = prop(group, "runtime");
    check_obj(&runtime, "health")?;
    let health = prop(&runtime, "health");
    let obs_items: Vec<Value> = if let Some(a) = health.as_array() {
        a.clone()
    } else if let Some(s) = health.as_str() {
        s.chars().map(|c| Value::String(c.to_string())).collect()
    } else if let Some(a) = marker(&health, "$set")
        .or_else(|| marker(&health, "$map"))
        .and_then(Value::as_array)
    {
        a.clone()
    } else {
        return Err("TypeError: group.runtime.health is not iterable".into());
    };
    let mut grouped: Vec<(Value, Vec<Value>)> = Vec::new();
    for obs in obs_items {
        check_obj(&obs, "member_id")?;
        let mid = prop(&obs, "member_id");
        if let Some((_, list)) = grouped.iter_mut().find(|(k, _)| eq_js(k, &mid, true)) {
            list.push(obs);
        } else {
            grouped.push((mid, vec![obs]));
        }
    }
    let members = prop(group, "members");
    check_obj(&members, "map")?;
    let Some(member_arr) = members.as_array() else {
        return Err("TypeError: group.members.map is not a function".into());
    };
    let mut out = Vec::with_capacity(member_arr.len());
    for member in member_arr {
        check_obj(member, "id")?;
        let mid = prop(member, "id");
        let list = grouped
            .iter()
            .find(|(k, _)| eq_js(k, &mid, true))
            .map(|(_, v)| Value::Array(v.clone()))
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let pref = preferred_obs(&list)?;
        let resolved = if !is_nullish(&pref) {
            pref
        } else {
            fallback_get(fallback, &mid)?
        };
        let mut obj = match member {
            Value::Object(m)
                if !js::is_undefined(member)
                    && js::as_number(member).is_none()
                    && marker(member, "$utf16").is_none()
                    && marker(member, "$bigint").is_none() =>
            {
                m.clone()
            }
            Value::Array(es) => es
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v.clone()))
                .collect(),
            _ => Map::new(),
        };
        obj.insert("health".into(), resolved);
        out.push(Value::Object(obj));
    }
    Ok(out)
}

/// Port of `memberName` (`apps/lite/src/groupHealth.ts:8`).
pub fn member_name(group: Value, id: Value, unknown: Value) -> Result<Value, String> {
    check_obj(&group, "members")?;
    let members = prop(&group, "members");
    check_obj(&members, "find")?;
    let Some(arr) = members.as_array() else {
        return Err("TypeError: group.members.find is not a function".into());
    };
    for member in arr {
        check_obj(member, "id")?;
        if eq_js(&prop(member, "id"), &id, false) {
            let name = prop(member, "name");
            check_obj(&name, "trim")?;
            let Some(units) = js::string_units(&name) else {
                return Err(
                    "TypeError: group.members.find(...)?.name.trim is not a function".into(),
                );
            };
            if let Some(s) = units.iter().position(|&u| !is_js_ws(u)) {
                let e = units.iter().rposition(|&u| !is_js_ws(u)).unwrap_or(s);
                return Ok(js::utf16_string(&units[s..=e]));
            }
            break;
        }
    }
    Ok(if is_truthy(&id) { id } else { unknown })
}

/// Port of `healthFallback` (`apps/lite/src/groupHealth.ts:12`).
pub fn health_fallback(nodes: Value, groups: Value) -> Result<Value, String> {
    check_obj(&nodes, "map")?;
    let Some(node_arr) = nodes.as_array() else {
        return Err("TypeError: nodes.map is not a function".into());
    };
    let mut node_map = Vec::with_capacity(node_arr.len());
    for node in node_arr {
        check_obj(node, "id")?;
        map_set(
            &mut node_map,
            prop(node, "id"),
            preferred_obs(&prop(node, "health"))?,
        );
    }
    check_obj(&groups, "map")?;
    let Some(group_arr) = groups.as_array() else {
        return Err("TypeError: groups.map is not a function".into());
    };
    let (mut mapped, mut by_id) = (
        Vec::with_capacity(group_arr.len()),
        Vec::with_capacity(group_arr.len()),
    );
    for item in group_arr {
        check_obj(item, "members")?;
        check_obj(&prop(item, "members"), "length")?;
        let runtime = prop(item, "runtime");
        check_obj(&runtime, "selection")?;
        let sel = prop(&runtime, "selection");
        check_obj(&sel, "tcp")?;
        let tcp = prop(&sel, "tcp");
        let tcp_id = if is_nullish(&tcp) {
            Value::Null
        } else {
            let m = prop(&tcp, "member_id");
            if is_nullish(&m) { Value::Null } else { m }
        };
        let gid = prop(item, "id");
        map_set(&mut by_id, gid.clone(), tcp_id.clone());
        mapped.push((gid, tcp_id));
    }
    let mut out = node_map.clone();
    for (gid, mut cur) in mapped {
        let mut seen = vec![gid.clone()];
        while is_truthy(&cur) && !seen.iter().any(|s| eq_js(s, &cur, true)) {
            let Some(next) = map_get(&by_id, &cur) else {
                break;
            };
            seen.push(cur);
            cur = next;
        }
        if is_truthy(&cur)
            && let Some(h) = map_get(&node_map, &cur)
        {
            map_set(&mut out, gid, h);
        }
    }
    Ok(
        json!({ "$map": out.into_iter().map(|(k, v)| Value::Array(vec![k, v])).collect::<Vec<_>>() }),
    )
}

/// Port of `observedMembers` (`apps/lite/src/groupHealth.ts:23`).
pub fn observed_members(group: Value, fallback: Value) -> Result<Value, String> {
    Ok(Value::Array(member_health_vec(&group, &fallback)?))
}

/// Port of `groupStatus` (`apps/lite/src/groupHealth.ts:26`).
pub fn group_status(group: Value, fallback: Value) -> Result<Value, String> {
    let observed = member_health_vec(&group, &fallback)?;
    let current = if observed.is_empty() {
        None
    } else {
        let sel = prop(&prop(&group, "runtime"), "selection");
        check_obj(&sel, "tcp")?;
        let tcp = prop(&sel, "tcp");
        let tcp_id = if is_nullish(&tcp) {
            js::undefined()
        } else {
            prop(&tcp, "member_id")
        };
        observed
            .iter()
            .find(|m| eq_js(&prop(m, "id"), &tcp_id, false))
    };
    let cur_health = current.and_then(|m| prop_opt(m, "health"));
    let state = cur_health.and_then(|h| prop_str(h, "state"));
    let status = if state == Some("unavailable") {
        "unavailable"
    } else if observed
        .iter()
        .any(|m| prop_opt(m, "health").and_then(|h| prop_str(h, "state")) == Some("unavailable"))
    {
        "warning"
    } else if state == Some("healthy") {
        "healthy"
    } else {
        "unknown"
    };
    let latency = cur_health
        .filter(|_| state == Some("healthy"))
        .and_then(|h| prop_opt(h, "latency_ms"))
        .filter(|v| !is_nullish(v))
        .cloned()
        .unwrap_or(Value::Null);
    Ok(json!({ "status": status, "latency": latency }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(
        member_id: &str,
        state: &str,
        latency_ms: Option<u64>,
        warmth: &str,
        measurement: &str,
        ip_version: &str,
    ) -> Value {
        json!({
            "member_id": member_id,
            "transport": "tcp",
            "purpose": "data",
            "warmth": warmth,
            "measurement": measurement,
            "ip_version": ip_version,
            "state": state,
            "latency_ms": latency_ms,
            "error": null
        })
    }

    #[test]
    fn member_name_resolution_and_errors() {
        let group = json!({
            "members": [
                {"id": "n1", "name": "  Hong Kong \u{FEFF} "},
                {"id": "n2", "name": "   \t\n  "},
                {"id": "n3", "name": "Tokyo"}
            ]
        });
        assert_eq!(
            member_name(group.clone(), json!("n1"), json!("Unknown")).unwrap(),
            json!("Hong Kong")
        );
        assert_eq!(
            member_name(group.clone(), json!("n2"), json!("Unknown")).unwrap(),
            json!("n2")
        );
        assert_eq!(
            member_name(group.clone(), json!("missing"), json!("Unknown")).unwrap(),
            json!("missing")
        );
        assert_eq!(
            member_name(group.clone(), json!(""), json!("Unknown")).unwrap(),
            json!("Unknown")
        );
        assert_eq!(
            member_name(group.clone(), Value::Null, json!("Unknown")).unwrap(),
            json!("Unknown")
        );
        assert_eq!(
            member_name(group, js::undefined(), json!("Unknown")).unwrap(),
            json!("Unknown")
        );
        assert!(member_name(Value::Null, json!("a"), json!("u")).is_err());
        assert!(member_name(json!({}), json!("a"), json!("u")).is_err());
    }

    #[test]
    fn health_fallback_resolves_nested_groups_and_cycles() {
        let fast = obs("hk", "healthy", Some(12), "warm", "http_headers", "ipv4");
        let slow = obs("hk", "healthy", Some(90), "cold", "tcp_connect", "ipv4");
        let nodes = json!([
            {"id": "hk", "health": [slow, fast]},
            {"id": "untested", "health": []}
        ]);
        let mk_group = |id: &str, tcp: Option<&str>| {
            json!({
                "id": id,
                "members": [],
                "runtime": {
                    "selection": {"tcp": tcp.map(|m| json!({"member_id": m})), "udp": null},
                    "health": []
                }
            })
        };
        let groups = json!([
            mk_group("outer", Some("inner")),
            mk_group("inner", Some("hk")),
            mk_group("loop", Some("back")),
            mk_group("back", Some("loop")),
            mk_group("idle", None),
            mk_group("empty_node", Some("untested"))
        ]);
        let fb = health_fallback(nodes, groups).unwrap();
        assert_eq!(fallback_get(&fb, &json!("hk")).unwrap(), fast);
        assert_eq!(fallback_get(&fb, &json!("outer")).unwrap(), fast);
        assert_eq!(fallback_get(&fb, &json!("inner")).unwrap(), fast);
        let pairs = fb["$map"].as_array().unwrap();
        assert!(
            pairs
                .iter()
                .any(|p| p[0] == json!("empty_node") && js::is_undefined(&p[1]))
        );
        assert!(!pairs.iter().any(|p| p[0] == json!("loop")));
        assert!(!pairs.iter().any(|p| p[0] == json!("idle")));
        assert!(health_fallback(json!("x"), json!([])).is_err());
        assert!(health_fallback(json!([]), Value::Null).is_err());
    }

    #[test]
    fn observed_members_prefers_group_health_then_fallback() {
        let headers = obs(
            "ranked",
            "healthy",
            Some(25),
            "warm",
            "http_headers",
            "ipv4",
        );
        let connect = obs("ranked", "unavailable", None, "warm", "tcp_connect", "ipv4");
        let fb_obs = obs(
            "fallback",
            "healthy",
            Some(10),
            "warm",
            "tcp_connect",
            "ipv4",
        );
        let mut udp_obs = obs(
            "fallback",
            "unavailable",
            None,
            "warm",
            "tcp_connect",
            "ipv4",
        );
        udp_obs["transport"] = json!("udp");
        let mut ignored = obs(
            "fallback",
            "unavailable",
            None,
            "warm",
            "http_headers",
            "ipv4",
        );
        ignored["error"] = json!("no_node_address");
        let group = json!({
            "members": [
                {"id": "ranked", "name": "R"},
                {"id": "fallback", "name": "F"},
                {"id": "untested", "name": "U"}
            ],
            "runtime": {"health": [connect, headers, udp_obs, ignored]}
        });
        let fb = json!({ "$map": [["ranked", fb_obs], ["fallback", fb_obs]] });
        let out = observed_members(group, fb).unwrap();
        let arr = out.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["health"], headers);
        assert_eq!(arr[1]["health"], fb_obs);
        assert!(js::is_undefined(&arr[2]["health"]));
        assert_eq!(observed_members(Value::Null, json!({})).unwrap(), json!([]));
        assert_eq!(observed_members(json!(0), Value::Null).unwrap(), json!([]));
        assert!(observed_members(json!({}), json!({})).is_err());
    }

    #[test]
    fn group_status_all_states() {
        let healthy = obs("a", "healthy", Some(42), "warm", "http_headers", "ipv4");
        let down_b = obs("b", "unavailable", None, "warm", "http_headers", "ipv4");
        let down_a = obs("a", "unavailable", None, "warm", "http_headers", "ipv4");
        let mk = |tcp: Option<&str>, health: Value| {
            json!({
                "members": [{"id": "a", "name": "A"}, {"id": "b", "name": "B"}],
                "runtime": {
                    "selection": {"tcp": tcp.map(|m| json!({"member_id": m})), "udp": null},
                    "health": health
                }
            })
        };
        let empty_fb = json!({ "$map": [] });

        assert_eq!(
            group_status(mk(Some("a"), json!([healthy])), empty_fb.clone()).unwrap(),
            json!({"status": "healthy", "latency": 42})
        );
        assert_eq!(
            group_status(mk(Some("a"), json!([healthy, down_b])), empty_fb.clone()).unwrap(),
            json!({"status": "warning", "latency": 42})
        );
        assert_eq!(
            group_status(mk(Some("a"), json!([down_a])), empty_fb.clone()).unwrap(),
            json!({"status": "unavailable", "latency": null})
        );
        assert_eq!(
            group_status(mk(Some("a"), json!([])), empty_fb.clone()).unwrap(),
            json!({"status": "unknown", "latency": null})
        );
        assert_eq!(
            group_status(mk(Some("a"), json!([down_b])), empty_fb).unwrap(),
            json!({"status": "warning", "latency": null})
        );
        assert_eq!(
            group_status(Value::Null, Value::Null).unwrap(),
            json!({"status": "unknown", "latency": null})
        );
        assert_eq!(
            group_status(json!(0), json!({})).unwrap(),
            json!({"status": "unknown", "latency": null})
        );
    }

    #[test]
    fn fold_families_prefers_ipv4_and_healthy_with_latency() {
        let v6_ok = obs("m", "healthy", Some(30), "warm", "tcp_connect", "ipv6");
        let v4_ok = obs("m", "healthy", Some(50), "warm", "tcp_connect", "ipv4");
        let v4_no_lat = obs("m", "healthy", None, "warm", "tcp_connect", "ipv4");
        let v4_down = obs("m", "unavailable", None, "warm", "tcp_connect", "ipv4");
        assert_eq!(preferred_obs(&json!([v6_ok, v4_ok])).unwrap(), v4_ok);
        assert_eq!(preferred_obs(&json!([v4_no_lat, v6_ok])).unwrap(), v6_ok);
        assert_eq!(
            preferred_obs(&json!([v4_down, v4_no_lat])).unwrap(),
            v4_no_lat
        );
        assert_eq!(preferred_obs(&json!([v4_down])).unwrap(), v4_down);
    }
}
