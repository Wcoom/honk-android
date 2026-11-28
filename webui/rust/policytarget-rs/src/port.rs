//! The Rust port of `src/features/shared/policyTarget.ts`.

#![forbid(unsafe_code)]

use serde_json::{Value, json};

fn is_nullish(v: &Value) -> bool {
    v.is_null() || crate::js::is_undefined(v)
}

fn is_truthy(v: &Value) -> bool {
    !is_nullish(v)
        && crate::js::as_number(v).is_none_or(|n| n != 0.0 && !n.is_nan())
        && !matches!(v, Value::Bool(false))
        && !matches!(v, Value::String(s) if s.is_empty())
}

fn js_eq(a: &Value, b: &Value) -> bool {
    crate::js::as_number(a)
        .zip(crate::js::as_number(b))
        .map_or_else(|| a == b, |(x, y)| x == y)
}

fn check_obj(v: &Value, op: &str) -> Result<(), String> {
    if v.is_null() {
        Err(format!(
            "TypeError: Cannot read properties of null (reading '{op}')"
        ))
    } else if crate::js::is_undefined(v) {
        Err(format!(
            "TypeError: Cannot read properties of undefined (reading '{op}')"
        ))
    } else {
        Ok(())
    }
}

fn prop(obj: &Value, key: &str) -> Result<Value, String> {
    check_obj(obj, key)?;
    Ok(obj
        .as_object()
        .and_then(|m| m.get(key))
        .cloned()
        .unwrap_or_else(crate::js::undefined))
}

fn as_array<'a>(v: &'a Value, method: &str, name: &str) -> Result<&'a Vec<Value>, String> {
    check_obj(v, method)?;
    v.as_array()
        .ok_or_else(|| format!("TypeError: {name}.{method} is not a function"))
}

fn coalesce(first: Value, next: impl FnOnce() -> Result<Value, String>) -> Result<Value, String> {
    if is_nullish(&first) {
        next()
    } else {
        Ok(first)
    }
}

fn hub_selection_id(hub: &Value) -> Result<Value, String> {
    let sel = prop(hub, "selection")?;
    coalesce(prop(&sel, "tcp_member_id")?, || prop(&sel, "udp_member_id"))
}

fn has_field_eq(items: &[Value], key: &str, needle: &Value) -> Result<bool, String> {
    for item in items {
        if js_eq(&prop(item, key)?, needle) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn find_name_by_id(items: &[Value], id: &Value) -> Result<Option<Value>, String> {
    for item in items {
        if js_eq(&prop(item, "id")?, id) {
            return Ok(Some(prop(item, "name")?));
        }
    }
    Ok(None)
}

fn build_options(items: &[Value], current: &Value) -> Result<(Vec<Value>, bool, Value), String> {
    let mut options: Vec<Value> = items
        .iter()
        .map(|item| {
            let name = prop(item, "name")?;
            Ok(json!({ "id": name, "label": name }))
        })
        .collect::<Result<_, String>>()?;
    let is_global = prop(current, "mode")?.as_str() == Some("global");
    let target = if is_global {
        prop(current, "target")?
    } else {
        crate::js::undefined()
    };
    if is_global && is_truthy(&target) && !has_field_eq(&options, "id", &target)? {
        options.push(json!({ "id": target, "label": target }));
    }
    Ok((options, is_global, target))
}

/// Port of `findPolicyHub` (`src/features/shared/policyTarget.ts:16`).
pub fn find_policy_hub(
    groups: Vec<serde_json::Value>,
    fallback_name: String,
) -> Result<Option<serde_json::Value>, String> {
    let mut selectors = Vec::new();
    for (idx, group) in groups.iter().enumerate() {
        if prop(&prop(group, "policy")?, "kind")?.as_str() == Some("selector") {
            selectors.push(idx);
        }
    }
    if selectors.is_empty() {
        return Ok(None);
    }

    let mut ids = Vec::with_capacity(selectors.len());
    let mut degrees = Vec::with_capacity(selectors.len());
    for &s_idx in &selectors {
        let s_id = prop(&groups[s_idx], "id")?;
        let mut count = 0usize;
        for group in &groups {
            if js_eq(&prop(group, "id")?, &s_id) {
                continue;
            }
            let sel = prop(group, "selection")?;
            if js_eq(&prop(&sel, "tcp_member_id")?, &s_id)
                || js_eq(&prop(&sel, "udp_member_id")?, &s_id)
            {
                count += 1;
            }
        }
        ids.push(s_id);
        degrees.push(count);
    }

    let mut chain: Vec<Value> = Vec::new();
    let mut cursor = groups
        .iter()
        .position(|g| prop(g, "name").is_ok_and(|v| v.as_str() == Some(&fallback_name)));
    while let Some(c_idx) = cursor {
        let group = &groups[c_idx];
        if prop(&prop(group, "policy")?, "kind")?.as_str() != Some("selector") {
            break;
        }
        let c_id = prop(group, "id")?;
        if chain.iter().any(|seen| js_eq(seen, &c_id)) {
            break;
        }
        chain.push(c_id);
        let next_id = hub_selection_id(group)?;
        cursor = is_truthy(&next_id)
            .then(|| {
                selectors
                    .iter()
                    .zip(&ids)
                    .rev()
                    .find(|&(_, id)| js_eq(id, &next_id))
                    .map(|(&i, _)| i)
            })
            .flatten();
    }

    let (mut best, mut best_deg, mut best_chain) = (None, 0usize, false);
    for (pos, &s_idx) in selectors.iter().enumerate() {
        let deg = degrees[pos];
        if deg < 1 {
            continue;
        }
        let on_chain = chain.iter().any(|seen| js_eq(seen, &ids[pos]));
        if deg > best_deg || (deg == best_deg && on_chain && !best_chain) {
            (best, best_deg, best_chain) = (Some(s_idx), deg, on_chain);
        }
    }
    if let Some(idx) = best {
        return Ok(Some(groups[idx].clone()));
    }
    for &s_idx in &selectors {
        if prop(&groups[s_idx], "name")?.as_str() == Some(fallback_name.as_str()) {
            return Ok(Some(groups[s_idx].clone()));
        }
    }
    Ok(None)
}

/// Port of `resolvePolicyState` (`src/features/shared/policyTarget.ts:62`).
pub fn resolve_policy_state(arg0: serde_json::Value) -> Result<serde_json::Value, String> {
    if arg0.is_null() {
        return Err(
            "TypeError: Cannot destructure property 'current' of 'object null' as it is null."
                .into(),
        );
    }
    if crate::js::is_undefined(&arg0) {
        return Err(
            "TypeError: Cannot destructure property 'current' of 'undefined' as it is undefined."
                .into(),
        );
    }
    let current = prop(&arg0, "current")?;
    let groups_val = prop(&arg0, "groups")?;
    let hub = prop(&arg0, "hub")?;
    let hub_detail = prop(&arg0, "hubDetail")?;
    let rule_fallback = prop(&arg0, "ruleFallback")?;
    let saved_target = prop(&arg0, "savedTarget")?;

    let mut hub_members = Vec::new();
    if !is_nullish(&hub_detail) {
        let raw_members = prop(&hub_detail, "members")?;
        if !is_nullish(&raw_members) {
            for m in as_array(&raw_members, "filter", "members")? {
                if prop(m, "kind")?.as_str() == Some("group") {
                    hub_members.push(json!({ "id": prop(m, "id")?, "name": prop(m, "name")? }));
                }
            }
        }
    }

    let (hub_id, out_members, options, is_global, target, primary, uses_hub) =
        if is_truthy(&hub) && !hub_members.is_empty() {
            let rt_member = |net: &str| -> Result<Value, String> {
                if is_nullish(&hub_detail) {
                    return Ok(crate::js::undefined());
                }
                let net_sel = prop(&prop(&prop(&hub_detail, "runtime")?, "selection")?, net)?;
                if is_nullish(&net_sel) {
                    Ok(crate::js::undefined())
                } else {
                    prop(&net_sel, "member_id")
                }
            };
            let selected_id = coalesce(rt_member("tcp")?, || {
                coalesce(rt_member("udp")?, || hub_selection_id(&hub))
            })?;
            let first =
                find_name_by_id(&hub_members, &selected_id)?.unwrap_or_else(crate::js::undefined);
            let hub_active = coalesce(first, || {
                Ok(
                    find_name_by_id(as_array(&groups_val, "find", "groups")?, &selected_id)?
                        .filter(|v| !is_nullish(v))
                        .unwrap_or_else(|| Value::String(String::new())),
                )
            })?;
            let (options, is_global, target) = build_options(&hub_members, &current)?;
            (
                prop(&hub, "id")?,
                hub_members,
                options,
                is_global,
                target,
                hub_active,
                true,
            )
        } else {
            let groups = as_array(&groups_val, "map", "groups")?;
            let (options, is_global, target) = build_options(groups, &current)?;
            if is_truthy(&hub) {
                let selected_id = hub_selection_id(&hub)?;
                let hub_active = find_name_by_id(groups, &selected_id)?
                    .filter(|v| !is_nullish(v))
                    .unwrap_or_else(|| Value::String(String::new()));
                (
                    prop(&hub, "id")?,
                    Vec::new(),
                    options,
                    is_global,
                    target,
                    hub_active,
                    false,
                )
            } else {
                let fallback =
                    if is_truthy(&rule_fallback) && has_field_eq(groups, "name", &rule_fallback)? {
                        rule_fallback
                    } else {
                        Value::String(String::new())
                    };
                (
                    Value::Null,
                    Vec::new(),
                    options,
                    is_global,
                    target,
                    fallback,
                    false,
                )
            }
        };

    let valid_saved = if has_field_eq(&options, "id", &saved_target)? {
        saved_target
    } else {
        Value::String(String::new())
    };
    let active_target = if is_global {
        target
    } else {
        let first_opt = options
            .first()
            .map(|o| prop(o, "id"))
            .transpose()?
            .unwrap_or_else(crate::js::undefined);
        [primary, valid_saved, first_opt]
            .into_iter()
            .find(is_truthy)
            .unwrap_or_else(|| Value::String(String::new()))
    };
    Ok(json!({
        "hubId": hub_id, "hubMembers": out_members,
        "options": options, "activeTarget": active_target, "usesHub": uses_hub
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grp(id: &str, name: &str, kind: &str, sel: Option<&str>) -> Value {
        json!({
            "id": id, "name": name, "policy": { "kind": kind },
            "selection": { "tcp_member_id": sel, "udp_member_id": sel }
        })
    }

    #[test]
    fn finds_central_hub_and_breaks_ties_along_fallback_chain() {
        let groups = vec![
            grp("cdn", "CDN", "score", None),
            grp("a", "A", "selector", Some("cdn")),
            grp("b", "B", "selector", Some("cdn")),
            grp("svc", "Svc", "selector", Some("a")),
            grp("fb", "FB", "selector", Some("b")),
        ];
        assert_eq!(
            find_policy_hub(groups, "FB".into()).unwrap().unwrap()["id"],
            "b"
        );
    }

    #[test]
    fn falls_back_to_named_selector_when_in_degree_is_zero() {
        let flat = vec![
            grp("proxy", "proxy", "selector", Some("n1")),
            grp("auto", "auto", "urltest", None),
        ];
        assert_eq!(
            find_policy_hub(flat, "proxy".into()).unwrap().unwrap()["name"],
            "proxy"
        );
        assert!(find_policy_hub(vec![], "proxy".into()).unwrap().is_none());
    }

    #[test]
    fn errors_on_malformed_inputs() {
        assert!(
            find_policy_hub(vec![json!("a"), json!(1)], "x".into())
                .unwrap_err()
                .contains("kind")
        );
        assert!(
            resolve_policy_state(Value::Null)
                .unwrap_err()
                .contains("destructure")
        );
        assert!(resolve_policy_state(json!({})).unwrap_err().contains("map"));
    }

    #[test]
    fn resolves_hub_global_and_fallback_states() {
        let groups = vec![
            grp("cdn", "CDN", "score", None),
            grp("all", "ALL", "score", None),
            grp("hub", "Hub", "selector", Some("cdn")),
            grp("fb", "FB", "selector", Some("hub")),
        ];
        let detail = json!({
            "members": [{"id": "n1", "name": "n", "kind": "node"}, {"id": "cdn", "name": "CDN", "kind": "group"}, {"id": "all", "name": "ALL", "kind": "group"}],
            "runtime": {"selection": {"tcp": {"member_id": "cdn"}, "udp": {"member_id": "cdn"}}}
        });
        let st = resolve_policy_state(json!({
            "current": {"mode": "global", "target": "FB"}, "groups": groups,
            "hub": groups[2], "hubDetail": detail, "ruleFallback": "FB", "savedTarget": ""
        }))
        .unwrap();
        assert_eq!(st["hubId"], "hub");
        assert_eq!(st["usesHub"], true);
        assert_eq!(st["activeTarget"], "FB");
        assert_eq!(st["options"].as_array().unwrap().len(), 3);

        let no_hub = resolve_policy_state(json!({
            "current": {"mode": "rule"}, "groups": groups, "hub": null, "ruleFallback": "FB", "savedTarget": "ALL"
        }))
        .unwrap();
        assert_eq!(no_hub["hubId"], Value::Null);
        assert_eq!(no_hub["usesHub"], false);
        assert_eq!(no_hub["activeTarget"], "FB");
    }
}
