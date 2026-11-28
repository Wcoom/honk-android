//! Routing Hub pages: Policy Groups selector, probe & RFC 6902 policy editor,
//! Nodes & Subscriptions manager with detail drawer & confirmed deletion, Rules & Routing Trace.

use std::collections::{HashMap, HashSet};

use doona_common::format::format_latency;
use doona_common::model::{GroupDetail, resolve_group_effective_summary};
use leptos::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen_futures::spawn_local;

use crate::state::use_app_state;
use crate::ui::{ConfirmSheet, StatusLight, copy_to_clipboard};

#[derive(Clone)]
struct GroupConfigEditor {
    group_id: String,
    group_name: String,
    etag: String,
    initial: Value,
    policy_kind: String,
    check_url: String,
    tolerance: String,
    idle_timeout: String,
    interrupt_connections: bool,
}

#[component]
pub fn PoliciesPage() -> impl IntoView {
    let state = use_app_state();
    let details = RwSignal::new(HashMap::<String, GroupDetail>::new());
    let collapsed_groups = RwSignal::new(HashSet::<String>::new());
    let collapse_seeded = RwSignal::new(false);
    let group_search = RwSignal::new(String::new());
    let busy_group = RwSignal::new(None::<String>);
    let editor = RwSignal::new(None::<GroupConfigEditor>);
    let editor_saving = RwSignal::new(false);

    // Load full details for all groups when group list or reload_nonce updates,
    // and initialize large groups (>12 members) as collapsed by default for clean browsing.
    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        let gs = state.groups.get();
        if !gs.is_empty() && !collapse_seeded.get_untracked() {
            let mut initial = HashSet::new();
            for g in &gs {
                if g.member_count > 12 {
                    initial.insert(g.id.clone());
                }
            }
            collapsed_groups.set(initial);
            collapse_seeded.set(true);
        }
        let api = state.client();
        for g in gs {
            let gid = g.id.clone();
            let api_c = api.clone();
            spawn_local(async move {
                if let Ok(det) = api_c.group(&gid).await {
                    details.update(|map| {
                        map.insert(gid, det);
                    });
                }
            });
        }
    });

    let save_group_policy = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(ed) = editor.get_untracked() else {
            return;
        };
        let mut ops: Vec<Value> = Vec::new();
        let orig_kind = ed
            .initial
            .get("policy")
            .and_then(|p| p.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("selector");
        if ed.policy_kind != orig_kind {
            ops.push(json!({
                "op": "replace",
                "path": "/policy",
                "value": { "kind": ed.policy_kind, "native": ed.policy_kind }
            }));
        }

        let cfg_obj = ed.initial.get("config").cloned().unwrap_or_else(|| json!({}));
        let orig_url = cfg_obj
            .get("check_url")
            .and_then(Value::as_str)
            .unwrap_or("");
        let next_url = ed.check_url.trim();
        if next_url != orig_url {
            if next_url.is_empty() {
                if !orig_url.is_empty() {
                    ops.push(json!({ "op": "remove", "path": "/config/check_url" }));
                }
            } else {
                ops.push(json!({
                    "op": "replace",
                    "path": "/config/check_url",
                    "value": next_url,
                }));
            }
        }

        if ed.policy_kind == "urltest" {
            let orig_tol = cfg_obj.get("tolerance").and_then(Value::as_u64);
            let next_tol = ed.tolerance.trim().parse::<u64>().ok();
            if next_tol != orig_tol {
                match next_tol {
                    Some(v) => ops.push(json!({
                        "op": "replace",
                        "path": "/config/tolerance",
                        "value": v,
                    })),
                    None if orig_tol.is_some() => {
                        ops.push(json!({ "op": "remove", "path": "/config/tolerance" }));
                    }
                    None => {}
                }
            }
        } else if cfg_obj.get("tolerance").is_some_and(|v| !v.is_null()) && orig_kind == "urltest" {
            ops.push(json!({ "op": "remove", "path": "/config/tolerance" }));
        }

        let orig_idle = cfg_obj.get("idle_timeout").and_then(Value::as_u64);
        let next_idle = ed.idle_timeout.trim().parse::<u64>().ok();
        if next_idle != orig_idle {
            match next_idle {
                Some(v) => ops.push(json!({
                    "op": "replace",
                    "path": "/config/idle_timeout",
                    "value": v,
                })),
                None if orig_idle.is_some() => {
                    ops.push(json!({ "op": "remove", "path": "/config/idle_timeout" }));
                }
                None => {}
            }
        }

        let orig_int = cfg_obj
            .get("interrupt_connections")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if ed.interrupt_connections != orig_int {
            ops.push(json!({
                "op": "replace",
                "path": "/config/interrupt_connections",
                "value": ed.interrupt_connections,
            }));
        }

        if ops.is_empty() {
            editor.set(None);
            return;
        }

        editor_saving.set(true);
        let api = state.client();
        let gid = ed.group_id.clone();
        let etag = ed.etag.clone();
        spawn_local(async move {
            match api.patch_group_config(&gid, Value::Array(ops), &etag).await {
                Ok(op) => {
                    editor.set(None);
                    state.poll_operation(op.operation_id, state.t("ui.saved"));
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            editor_saving.set(false);
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <input
                    class="rp-input"
                    type="search"
                    style="max-width:260px;"
                    placeholder=move || state.t("ui.search")
                    prop:value=move || group_search.get()
                    on:input=move |ev| group_search.set(event_target_value(&ev))
                />
                <div class="rp-cluster" style="gap:8px;">
                    <button
                        type="button"
                        class="rp-btn secondary sm"
                        on:click=move |_| collapsed_groups.set(HashSet::new())
                    >
                        "全部展开"
                    </button>
                    <button
                        type="button"
                        class="rp-btn secondary sm"
                        on:click=move |_| {
                            let all_ids: HashSet<String> = state.groups.get_untracked().into_iter().map(|g| g.id).collect();
                            collapsed_groups.set(all_ids);
                        }
                    >
                        "全部收起"
                    </button>
                </div>
            </div>

            {move || {
                let all_gs = state.groups.get();
                let q = group_search.get().trim().to_lowercase();
                let det_map = details.get();
                let nodes = state.nodes.get().map(|n| n.nodes).unwrap_or_default();
                let lang = state.lang.get();
                let gs: Vec<_> = all_gs
                    .iter()
                    .filter(|g| q.is_empty() || g.name.to_lowercase().contains(&q) || g.policy.kind.to_lowercase().contains(&q))
                    .cloned()
                    .collect();
                if gs.is_empty() {
                    return view! { <section class="rp-card"><p class="rp-empty">{state.t("ui.empty")}</p></section> }.into_any();
                }
                gs.into_iter().map(|g| {
                    let gid = g.id.clone();
                    let gid_toggle = g.id.clone();
                    let gid_collapsed_chk = g.id.clone();
                    let gid_chevron_chk = g.id.clone();
                    let gid_probe_dis = g.id.clone();
                    let gid_probe = g.id.clone();
                    let gid_clear = g.id.clone();
                    let gid_edit = g.id.clone();
                    let gname_edit = g.name.clone();
                    let det = det_map.get(&gid).cloned();
                    let effective_label = resolve_group_effective_summary(
                        &g,
                        det.as_ref(),
                        &all_gs,
                        &nodes,
                        lang,
                    );
                    let selected_id = det
                        .as_ref()
                        .and_then(|d| d.runtime.selection.tcp.as_ref())
                        .and_then(|s| s.member_id.clone())
                        .or_else(|| g.selection.tcp_member_id.clone())
                        .unwrap_or_default();
                    let can_select = det
                        .as_ref()
                        .is_some_and(|d| d.capabilities.can_select || d.capabilities.can_override);

                    view! {
                        <section class="rp-card">
                            <div class="rp-between" style="flex-wrap:wrap;gap:8px;align-items:center;">
                                <div
                                    class="rp-cluster"
                                    role="button"
                                    tabindex="0"
                                    style="cursor:pointer;flex:1 1 200px;min-width:0;user-select:none;"
                                    on:click=move |_| {
                                        let id = gid_toggle.clone();
                                        collapsed_groups.update(|set| {
                                            if !set.remove(&id) {
                                                set.insert(id);
                                            }
                                        });
                                    }
                                >
                                    <span
                                        class="rp-chevron"
                                        data-collapsed=move || collapsed_groups.get().contains(&gid_chevron_chk).then_some("true")
                                    >
                                        "▾"
                                    </span>
                                    <h2 class="rp-h3" style="margin:0;">{g.name.clone()}</h2>
                                    <span class="rp-badge">{g.policy.kind.clone()}</span>
                                    <span class="rp-label">{format!("{} 成员", g.member_count)}</span>
                                    <span class="rp-badge" style="background:color-mix(in srgb,var(--rp-accent) 16%,transparent);color:var(--rp-text);max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">
                                        {effective_label}
                                    </span>
                                </div>
                                <div class="rp-cluster" on:click=move |ev| ev.stop_propagation()>
                                    <button
                                        type="button"
                                        class="rp-btn secondary sm"
                                        disabled=move || busy_group.get().as_deref() == Some(gid_probe_dis.as_str())
                                        on:click=move |_| {
                                            let id = gid_probe.clone();
                                            busy_group.set(Some(id.clone()));
                                            let api = state.client();
                                            spawn_local(async move {
                                                match api.probe_group(&id).await {
                                                    Ok(op) => {
                                                        state.poll_operation(op.operation_id, state.t("pol.probeStarted"));
                                                    }
                                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                }
                                                busy_group.set(None);
                                            });
                                        }
                                    >
                                        {move || state.t("pol.probe")}
                                    </button>
                                    <button
                                        type="button"
                                        class="rp-btn secondary sm"
                                        on:click=move |_| {
                                            let id = gid_edit.clone();
                                            let name = gname_edit.clone();
                                            let api = state.client();
                                            spawn_local(async move {
                                                match api.group_config(&id).await {
                                                    Ok((val, etag_opt)) => {
                                                        let etag = etag_opt.unwrap_or_else(|| {
                                                            let rev = val.get("revision").and_then(Value::as_str).unwrap_or("*");
                                                            format!("\"{rev}\"")
                                                        });
                                                        let kind = val
                                                            .get("policy")
                                                            .and_then(|p| p.get("kind"))
                                                            .and_then(Value::as_str)
                                                            .unwrap_or("selector")
                                                            .to_string();
                                                        let cfg = val.get("config").cloned().unwrap_or_else(|| json!({}));
                                                        let check_url = cfg
                                                            .get("check_url")
                                                            .and_then(Value::as_str)
                                                            .unwrap_or("")
                                                            .to_string();
                                                        let tolerance = cfg
                                                            .get("tolerance")
                                                            .and_then(Value::as_u64)
                                                            .map(|v| v.to_string())
                                                            .unwrap_or_default();
                                                        let idle_timeout = cfg
                                                            .get("idle_timeout")
                                                            .and_then(Value::as_u64)
                                                            .map(|v| v.to_string())
                                                            .unwrap_or_default();
                                                        let interrupt_connections = cfg
                                                            .get("interrupt_connections")
                                                            .and_then(Value::as_bool)
                                                            .unwrap_or(false);
                                                        editor.set(Some(GroupConfigEditor {
                                                            group_id: id,
                                                            group_name: name,
                                                            etag,
                                                            initial: val,
                                                            policy_kind: kind,
                                                            check_url,
                                                            tolerance,
                                                            idle_timeout,
                                                            interrupt_connections,
                                                        }));
                                                    }
                                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                }
                                            });
                                        }
                                    >
                                        "策略设置"
                                    </button>
                                    {det.as_ref().is_some_and(|d| d.capabilities.can_override).then(|| view! {
                                        <button
                                            type="button"
                                            class="rp-btn quiet sm"
                                            on:click=move |_| {
                                                let id = gid_clear.clone();
                                                let api = state.client();
                                                spawn_local(async move {
                                                    match api.clear_group_override(&id, "both").await {
                                                        Ok(_) => {
                                                            if let Ok(d) = api.group(&id).await {
                                                                details.update(|m| { m.insert(id, d); });
                                                            }
                                                            if let Ok(all) = api.groups().await {
                                                                state.groups.set(all);
                                                            }
                                                        }
                                                        Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                    }
                                                });
                                            }
                                        >
                                            {move || state.t("pol.clearOverride")}
                                        </button>
                                    })}
                                </div>
                            </div>

                            <div
                                class="rp-collapse-grid"
                                data-collapsed=move || collapsed_groups.get().contains(&gid_collapsed_chk).then_some("true")
                            >
                                <div class="rp-collapse-inner">
                                    <div class="rp-nodes" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(190px,1fr));gap:8px;">
                                        {det.map(|d| d.members.into_iter().map(|m| {
                                            let is_sel = m.id == selected_id;
                                            let mid = m.id.clone();
                                            let gid_sel = gid.clone();
                                            let lat = nodes
                                                .iter()
                                                .find(|n| n.id == m.id)
                                                .and_then(|n| n.best_latency_ms())
                                                .or_else(|| {
                                                    // If member is a nested group, show its resolved latency if available
                                                    all_gs
                                                        .iter()
                                                        .find(|cg| cg.id == m.id)
                                                        .and_then(|cg| cg.selection.tcp_member_id.as_ref())
                                                        .and_then(|leaf_id| nodes.iter().find(|n| &n.id == leaf_id))
                                                        .and_then(|n| n.best_latency_ms())
                                                });
                                            let cls = if is_sel { "rp-node rp-btn accent" } else { "rp-node rp-btn secondary" };
                                            view! {
                                                <button
                                                    type="button"
                                                    class=cls
                                                    disabled=!can_select
                                                    style="display:flex;justify-content:space-between;align-items:center;padding:8px 10px;text-align:left;"
                                                    on:click=move |_| {
                                                        let api = state.client();
                                                        let g_id = gid_sel.clone();
                                                        let m_id = mid.clone();
                                                        spawn_local(async move {
                                                            match api.select_group(&g_id, &m_id, "both").await {
                                                                Ok(_) => {
                                                                    if let Ok(refreshed) = api.group(&g_id).await {
                                                                        details.update(|map| { map.insert(g_id, refreshed); });
                                                                    }
                                                                    if let Ok(all) = api.groups().await {
                                                                        state.groups.set(all);
                                                                    }
                                                                }
                                                                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                                                            }
                                                        });
                                                    }
                                                >
                                                    <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{m.name}</span>
                                                    <span class="rp-latency" style="font-size:11px;margin-left:6px;opacity:0.85;">{format_latency(lat)}</span>
                                                </button>
                                            }
                                        }).collect_view())}
                                    </div>
                                </div>
                            </div>
                        </section>
                    }
                }).collect_view().into_any()
            }}

            <Show when=move || editor.get().is_some()>
                <div
                    class="rp-ios-sheet-backdrop"
                    on:click=move |_| editor.set(None)
                >
                    <form
                        class="rp-dialog rp-card rp-ios-sheet"
                        style="width:min(460px,92vw);display:flex;flex-direction:column;gap:12px;"
                        on:click=move |ev| ev.stop_propagation()
                        on:submit=save_group_policy
                    >
                        <div class="rp-between">
                            <h3 class="rp-h3">{move || editor.get().map(|e| format!("策略设置 · {}", e.group_name)).unwrap_or_default()}</h3>
                            <button type="button" class="rp-btn quiet sm" on:click=move |_| editor.set(None)>"×"</button>
                        </div>
                        <label class="rp-field">
                            <span class="rp-label">"调度策略 (Policy)"</span>
                            <select
                                class="rp-input"
                                prop:value=move || editor.get().map(|e| e.policy_kind).unwrap_or_default()
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    editor.update(|opt| {
                                        if let Some(ed) = opt {
                                            ed.policy_kind = v;
                                        }
                                    });
                                }
                            >
                                <option value="selector">"Selector (手动选择)"</option>
                                <option value="urltest">"URLTest (自动低延迟)"</option>
                                <option value="fallback">"Fallback (故障转移)"</option>
                                <option value="loadbalance">"LoadBalance (负载均衡)"</option>
                                <option value="score">"Score (综合评分)"</option>
                            </select>
                        </label>
                        <label class="rp-field">
                            <span class="rp-label">"健康探测 URL (Check URL)"</span>
                            <input
                                class="rp-input"
                                placeholder="https://www.gstatic.com/generate_204"
                                prop:value=move || editor.get().map(|e| e.check_url).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    editor.update(|opt| {
                                        if let Some(ed) = opt {
                                            ed.check_url = v;
                                        }
                                    });
                                }
                            />
                        </label>
                        <Show when=move || editor.get().is_some_and(|e| e.policy_kind == "urltest")>
                            <label class="rp-field">
                                <span class="rp-label">"切换容差 ms (仅 URLTest)"</span>
                                <input
                                    class="rp-input"
                                    type="number"
                                    placeholder="50"
                                    prop:value=move || editor.get().map(|e| e.tolerance).unwrap_or_default()
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        editor.update(|opt| {
                                            if let Some(ed) = opt {
                                                ed.tolerance = v;
                                            }
                                        });
                                    }
                                />
                            </label>
                        </Show>
                        <label class="rp-field">
                            <span class="rp-label">"空闲超时秒数 (Idle Timeout)"</span>
                            <input
                                class="rp-input"
                                type="number"
                                placeholder="(默认)"
                                prop:value=move || editor.get().map(|e| e.idle_timeout).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    editor.update(|opt| {
                                        if let Some(ed) = opt {
                                            ed.idle_timeout = v;
                                        }
                                    });
                                }
                            />
                        </label>
                        <label class="rp-between" style="cursor:pointer;">
                            <span class="rp-label">"切换节点时中断已有连接"</span>
                            <input
                                type="checkbox"
                                prop:checked=move || editor.get().is_some_and(|e| e.interrupt_connections)
                                on:change=move |ev| {
                                    let c = event_target_checked(&ev);
                                    editor.update(|opt| {
                                        if let Some(ed) = opt {
                                            ed.interrupt_connections = c;
                                        }
                                    });
                                }
                            />
                        </label>
                        <div class="rp-ios-sheet-actions">
                            <button type="button" class="rp-btn secondary" on:click=move |_| editor.set(None)>
                                {move || state.t("ui.cancel")}
                            </button>
                            <button type="submit" class="rp-btn accent" disabled=move || editor_saving.get()>
                                {move || state.t("config.save")}
                            </button>
                        </div>
                    </form>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn NodesPage() -> impl IntoView {
    let state = use_app_state();
    let init_tab = crate::state::query_param(&state.query.get_untracked(), "tab")
        .filter(|t| t == "subscriptions")
        .unwrap_or_else(|| "nodes".to_string());
    let tab = RwSignal::new(init_tab);
    let tab_seq = RwSignal::new(0u64);
    Effect::new(move |_| {
        if let Some(qtab) = crate::state::query_param(&state.query.get(), "tab")
            && (qtab == "nodes" || qtab == "subscriptions")
            && tab.get_untracked() != qtab
        {
            tab_seq.update(|s| *s += 1);
            tab.set(qtab);
        }
    });
    let filter = RwSignal::new(String::new());
    let new_name = RwSignal::new(String::new());
    let new_link = RwSignal::new(String::new());
    let sub_name = RwSignal::new(String::new());
    let sub_url = RwSignal::new(String::new());
    let inspected_node = RwSignal::new(None::<Value>);
    let del_target = RwSignal::new(None::<(String, String, bool)>); // (id, name, is_provider)
    let del_confirm = RwSignal::new(false);

    let add_node = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = new_name.get_untracked().trim().to_string();
        let link = new_link.get_untracked().trim().to_string();
        if name.is_empty() || link.is_empty() {
            return;
        }
        let api = state.client();
        spawn_local(async move {
            match api.create_node(&name, &link).await {
                Ok(_) => {
                    new_name.set(String::new());
                    new_link.set(String::new());
                    state.toast("positive", state.t("ui.saved"), None);
                    state.refresh_all();
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
        });
    };

    let add_sub = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = sub_name.get_untracked().trim().to_string();
        let url = sub_url.get_untracked().trim().to_string();
        if name.is_empty() || url.is_empty() {
            return;
        }
        let api = state.client();
        spawn_local(async move {
            match api.create_provider(&name, &url).await {
                Ok(_) => {
                    sub_name.set(String::new());
                    sub_url.set(String::new());
                    state.toast("positive", state.t("ui.saved"), None);
                    state.refresh_all();
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
        });
    };

    let confirm_delete = Callback::new(move |()| {
        let Some((id, _, is_prov)) = del_target.get_untracked() else {
            return;
        };
        let api = state.client();
        spawn_local(async move {
            let res = if is_prov {
                api.delete_provider(&id).await
            } else {
                api.delete_node(&id).await
            };
            match res {
                Ok(_) => {
                    state.toast("positive", "已删除", None);
                    state.refresh_all();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-seg" data-size="M">
                    <button
                        type="button"
                        class="rp-btn"
                        data-size="M"
                        data-selected=move || (tab.get() == "nodes").then_some("true")
                        on:click=move |_| {
                            if tab.get_untracked() != "nodes" {
                                tab_seq.update(|s| *s += 1);
                                tab.set("nodes".to_string());
                            }
                        }
                    >
                        {move || state.t("nav.nodes")}
                    </button>
                    <button
                        type="button"
                        class="rp-btn"
                        data-size="M"
                        data-selected=move || (tab.get() == "subscriptions").then_some("true")
                        on:click=move |_| {
                            if tab.get_untracked() != "subscriptions" {
                                tab_seq.update(|s| *s += 1);
                                tab.set("subscriptions".to_string());
                            }
                        }
                    >
                        {move || state.t("nodes.subscriptions")}
                    </button>
                </div>
                <div class="rp-cluster">
                    <input
                        class="rp-input"
                        type="search"
                        placeholder=move || state.t("ui.search")
                        prop:value=move || filter.get()
                        on:input=move |ev| filter.set(event_target_value(&ev))
                    />
                    <Show when=move || tab.get() == "nodes">
                        <button
                            type="button"
                            class="rp-btn secondary sm"
                            on:click=move |_| {
                                let gs = state.groups.get_untracked();
                                let api = state.client();
                                spawn_local(async move {
                                    let mut last_op = None;
                                    let mut last_err = None;
                                    for g in gs.iter().take(6) {
                                        match api.probe_group(&g.id).await {
                                            Ok(op) => last_op = Some(op.operation_id),
                                            Err(e) => last_err = Some(e),
                                        }
                                    }
                                    if let Some(op_id) = last_op {
                                        state.poll_operation(op_id, state.t("pol.probeStarted"));
                                    } else if let Some(e) = last_err {
                                        state.toast("negative", state.t("ui.failed"), Some(e));
                                    }
                                });
                            }
                        >
                            "全部测速"
                        </button>
                    </Show>
                </div>
            </div>

            {move || inspected_node.get().map(|nd| {
                let id = nd.get("id").and_then(Value::as_str).unwrap_or("—").to_string();
                let id_copy = id.clone();
                let name = nd.get("name").and_then(Value::as_str).unwrap_or("—").to_string();
                let proto = nd.get("protocol").and_then(Value::as_str).unwrap_or("—").to_string();
                let addr = nd.get("address").or_else(|| nd.get("server")).and_then(Value::as_str).unwrap_or("—").to_string();
                let sub = nd.get("subscription_tag").and_then(Value::as_str).unwrap_or("inline").to_string();
                let groups_cnt = nd.get("group_ids").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                view! {
                    <div class="rp-ios-sheet-backdrop" on:click=move |_| inspected_node.set(None)>
                        <section class="rp-card rp-ios-sheet" on:click=move |ev| ev.stop_propagation()>
                            <div class="rp-between">
                                <div class="rp-cluster">
                                    <strong>{name}</strong>
                                    <span class="rp-badge">{proto}</span>
                                </div>
                                <div class="rp-cluster">
                                    <button
                                        type="button"
                                        class="rp-btn secondary sm"
                                        on:click=move |_| {
                                            copy_to_clipboard(&id_copy);
                                            state.toast("positive", "已复制节点 ID", None);
                                        }
                                    >
                                        "复制 ID"
                                    </button>
                                    <button type="button" class="rp-btn quiet sm" on:click=move |_| inspected_node.set(None)>"×"</button>
                                </div>
                            </div>
                            <div class="rp-kv">
                                <div><span class="k">"ID"</span><span class="v"><code>{id}</code></span></div>
                                <div><span class="k">"服务器地址"</span><span class="v"><code>{addr}</code></span></div>
                                <div><span class="k">"来源"</span><span class="v">{sub}</span></div>
                                <div><span class="k">"所属策略组"</span><span class="v">{format!("{groups_cnt} 个")}</span></div>
                            </div>
                        </section>
                    </div>
                }
            })}

            {move || {
                let is_sub = tab.get() == "subscriptions";
                let parity = (tab_seq.get_untracked() % 2).to_string();
                let inner = if is_sub {
                    view! {
                        <div style="display:flex;flex-direction:column;gap:12px;">
                        <section class="rp-card">
                            <form class="rp-cluster" on:submit=add_sub style="flex-wrap:wrap;">
                                <input class="rp-input" placeholder=move || state.t("ui.name") prop:value=move || sub_name.get() on:input=move |ev| sub_name.set(event_target_value(&ev)) />
                                <input class="rp-input" placeholder="https://..." style="flex:1;min-width:220px;" prop:value=move || sub_url.get() on:input=move |ev| sub_url.set(event_target_value(&ev)) />
                                <button type="submit" class="rp-btn accent sm">{move || state.t("ui.add")}</button>
                            </form>
                        </section>
                        <section class="rp-card" style="overflow:auto;">
                            {move || {
                                let subs = state.providers.get();
                                if subs.is_empty() {
                                    return view! { <p class="rp-empty">{state.t("act.noSubscriptions")}</p> }.into_any();
                                }
                                view! {
                                    <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                        <thead>
                                            <tr>
                                                <th style="text-align:left;padding:6px 8px;">{state.t("ui.name")}</th>
                                                <th style="text-align:left;padding:6px 8px;">"Kind"</th>
                                                <th style="text-align:right;padding:6px 8px;">"Nodes"</th>
                                                <th style="text-align:right;padding:6px 8px;">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {subs.into_iter().map(|p| {
                                                let is_inline = p.id == "inline" || p.kind.as_deref() == Some("inline");
                                                let disp_name = if is_inline && p.name == "inline" {
                                                    "内置节点 (Inline)".to_string()
                                                } else {
                                                    p.name.clone()
                                                };
                                                let pid_ref = p.id.clone();
                                                let pid_del = p.id.clone();
                                                let pname_del = p.name.clone();
                                                view! {
                                                    <tr>
                                                        <td style="padding:6px 8px;"><strong>{disp_name}</strong></td>
                                                        <td style="padding:6px 8px;"><span class="rp-badge">{p.kind.unwrap_or_else(|| "subscription".into())}</span></td>
                                                        <td style="text-align:right;padding:6px 8px;">{p.node_count.map(|c| c.to_string()).unwrap_or_else(|| "—".into())}</td>
                                                        <td style="text-align:right;padding:6px 8px;">
                                                            {(!is_inline).then(|| view! {
                                                                <span class="rp-cluster" style="justify-content:flex-end;">
                                                                    <button type="button" class="rp-btn secondary sm" on:click=move |_| {
                                                                        let id = pid_ref.clone();
                                                                        let api = state.client();
                                                                        spawn_local(async move {
                                                                            match api.refresh_provider(&id).await {
                                                                                Ok(op) => {
                                                                                    state.poll_operation(op.operation_id, state.t("ui.refresh"));
                                                                                }
                                                                                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                                            }
                                                                        });
                                                                    }>{state.t("ui.refresh")}</button>
                                                                    <button type="button" class="rp-btn quiet sm" on:click=move |_| {
                                                                        del_target.set(Some((pid_del.clone(), pname_del.clone(), true)));
                                                                        del_confirm.set(true);
                                                                    }>"×"</button>
                                                                </span>
                                                            })}
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }}
                        </section>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div style="display:flex;flex-direction:column;gap:12px;">
                        <section class="rp-card">
                            <form class="rp-cluster" on:submit=add_node style="flex-wrap:wrap;">
                                <input class="rp-input" placeholder=move || state.t("ui.name") prop:value=move || new_name.get() on:input=move |ev| new_name.set(event_target_value(&ev)) />
                                <input class="rp-input" placeholder="vless://... / ss://..." style="flex:1;min-width:220px;" prop:value=move || new_link.get() on:input=move |ev| new_link.set(event_target_value(&ev)) />
                                <button type="submit" class="rp-btn accent sm">{move || state.t("ui.add")}</button>
                            </form>
                        </section>
                        <section class="rp-card" style="overflow:auto;">
                            {move || {
                                let q = filter.get().to_lowercase();
                                let nodes: Vec<_> = state
                                    .nodes
                                    .get()
                                    .map(|l| l.nodes)
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter(|n| q.is_empty() || n.name.to_lowercase().contains(&q) || n.protocol.as_deref().unwrap_or("").to_lowercase().contains(&q) || n.subscription_tag.as_deref().unwrap_or("").to_lowercase().contains(&q))
                                    .collect();
                                if nodes.is_empty() {
                                    return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                                }
                                view! {
                                    <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                        <thead>
                                            <tr>
                                                <th style="text-align:left;padding:6px 8px;">{state.t("ui.name")}</th>
                                                <th style="text-align:left;padding:6px 8px;">"Protocol"</th>
                                                <th style="text-align:left;padding:6px 8px;">"Source"</th>
                                                <th style="text-align:right;padding:6px 8px;">{state.t("act.latency")}</th>
                                                <th style="text-align:right;padding:6px 8px;">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {nodes.into_iter().take(200).map(|n| {
                                                let nid_i = n.id.clone();
                                                let nid_p = n.id.clone();
                                                let nid_d = n.id.clone();
                                                let nname_d = n.name.clone();
                                                let lat = format_latency(n.best_latency_ms());
                                                let health = n.health_state().to_string();
                                                view! {
                                                    <tr style="cursor:pointer;" on:click=move |_| {
                                                        let id = nid_i.clone();
                                                        let api = state.client();
                                                        spawn_local(async move {
                                                            if let Ok(det) = api.node_detail(&id).await {
                                                                inspected_node.set(Some(det));
                                                            }
                                                        });
                                                    }>
                                                        <td style="padding:6px 8px;">{n.name}</td>
                                                        <td style="padding:6px 8px;"><span class="rp-badge">{n.protocol.unwrap_or_else(|| "—".into())}</span></td>
                                                        <td style="padding:6px 8px;">{n.subscription_tag.unwrap_or_else(|| "inline".into())}</td>
                                                        <td style="text-align:right;padding:6px 8px;">
                                                            <span class=format!("rp-light {}", if health == "alive" { "ok" } else if health == "dead" { "err" } else { "muted" })>{lat}</span>
                                                        </td>
                                                        <td style="text-align:right;padding:6px 8px;" on:click=move |ev| ev.stop_propagation()>
                                                            <span class="rp-cluster" style="justify-content:flex-end;">
                                                                <button type="button" class="rp-btn secondary sm" on:click=move |_| {
                                                                    let id = nid_p.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        match api.probe_node(&id).await {
                                                                            Ok(op) => state.poll_operation(op.operation_id, state.t("pol.probeStarted")),
                                                                            Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                                        }
                                                                    });
                                                                }>{state.t("pol.probe")}</button>
                                                                <button type="button" class="rp-btn quiet sm" on:click=move |_| {
                                                                    del_target.set(Some((nid_d.clone(), nname_d.clone(), false)));
                                                                    del_confirm.set(true);
                                                                }>"×"</button>
                                                            </span>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }}
                        </section>
                    </div>
                }.into_any()
                };
                view! { <div class="rp-subtab-stage" data-parity=parity>{inner}</div> }
            }}

            <ConfirmSheet
                open=del_confirm
                title=Signal::derive(move || {
                    del_target
                        .get()
                        .map(|(_, n, p)| format!("删除{} “{n}”？", if p { "订阅" } else { "节点" }))
                        .unwrap_or_else(|| "确认删除".into())
                })
                message=Signal::derive(move || "删除后将立即从主配置移除并自动热重载核心。".into())
                confirm_label=Signal::derive(move || "确认删除".into())
                destructive=true
                on_confirm=confirm_delete
            />
        </div>
    }
}

#[component]
pub fn RulesPage() -> impl IntoView {
    let state = use_app_state();
    let tab = RwSignal::new("list".to_string());
    let tab_seq = RwSignal::new(0u64);
    let rule_filter = RwSignal::new(String::new());
    let rules = RwSignal::new(Vec::<Value>::new());
    let dns_rules = RwSignal::new(None::<Value>);
    let trace_domain = RwSignal::new("www.google.com".to_string());
    let trace_ip = RwSignal::new(String::new());
    let trace_port = RwSignal::new("443".to_string());
    let trace_net = RwSignal::new("tcp".to_string());
    let trace_pname = RwSignal::new(String::new());
    let trace_result = RwSignal::new(None::<Value>);
    let trace_busy = RwSignal::new(false);

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        let api = state.client();
        spawn_local(async move {
            if let Ok(raw) = api.rules().await {
                let items = raw
                    .get("rules")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                rules.set(items);
            }
            if let Ok(raw) = api.dns_rules().await {
                dns_rules.set(Some(raw));
            }
        });
    });

    let run_trace = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let domain = trace_domain.get_untracked().trim().to_string();
        let dst_ip = trace_ip.get_untracked().trim().to_string();
        let pname = trace_pname.get_untracked().trim().to_string();
        let port = trace_port.get_untracked().parse::<u16>().unwrap_or(443);
        let net = trace_net.get_untracked();
        if domain.is_empty() && dst_ip.is_empty() {
            return;
        }
        trace_busy.set(true);
        let api = state.client();
        spawn_local(async move {
            let mut input = serde_json::Map::new();
            input.insert("network".into(), json!(net));
            input.insert("dst_port".into(), json!(port));
            if !domain.is_empty() {
                input.insert("domain".into(), json!(domain));
            }
            if !dst_ip.is_empty() {
                input.insert("dst_ip".into(), json!(dst_ip));
            }
            if !pname.is_empty() {
                input.insert("pname".into(), json!(pname));
            }
            match api.routing_trace(Value::Object(input)).await {
                Ok(res) => trace_result.set(Some(res)),
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            trace_busy.set(false);
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-seg" data-size="M">
                    {[("list", "nav.rules"), ("dns", "nav.dns"), ("trace", "rules.tabTrace")].into_iter().map(|(id, key)| view! {
                        <button
                            type="button"
                            class="rp-btn"
                            data-size="M"
                            data-selected=move || (tab.get() == id).then_some("true")
                            on:click=move |_| {
                                if tab.get_untracked() != id {
                                    tab_seq.update(|s| *s += 1);
                                    tab.set(id.to_string());
                                }
                            }
                        >
                            {move || state.t(key)}
                        </button>
                    }).collect_view()}
                </div>
                <Show when=move || tab.get() == "list">
                    <input
                        class="rp-input"
                        type="search"
                        placeholder=move || state.t("ui.search")
                        prop:value=move || rule_filter.get()
                        on:input=move |ev| rule_filter.set(event_target_value(&ev))
                    />
                </Show>
            </div>

            {move || {
                let cur_tab = tab.get();
                let parity = (tab_seq.get_untracked() % 2).to_string();
                let inner = match cur_tab.as_str() {
                "trace" => view! {
                    <section class="rp-card" style="display:flex;flex-direction:column;gap:12px;">
                        <form class="rp-cluster" on:submit=run_trace style="flex-wrap:wrap;">
                            <input class="rp-input" placeholder="域名 (如 www.google.com)" style="flex:1;min-width:170px;" prop:value=move || trace_domain.get() on:input=move |ev| trace_domain.set(event_target_value(&ev)) />
                            <input class="rp-input" placeholder="目标 IP (可选)" style="width:140px;" prop:value=move || trace_ip.get() on:input=move |ev| trace_ip.set(event_target_value(&ev)) />
                            <input class="rp-input" placeholder="进程名 (可选)" style="width:130px;" prop:value=move || trace_pname.get() on:input=move |ev| trace_pname.set(event_target_value(&ev)) />
                            <input class="rp-input" placeholder="443" style="width:84px;" prop:value=move || trace_port.get() on:input=move |ev| trace_port.set(event_target_value(&ev)) />
                            <select class="rp-input" prop:value=move || trace_net.get() on:change=move |ev| trace_net.set(event_target_value(&ev))>
                                <option value="tcp">"TCP"</option>
                                <option value="udp">"UDP"</option>
                            </select>
                            <button type="submit" class="rp-btn accent" disabled=move || trace_busy.get()>
                                {move || state.t("rules.traceRun")}
                            </button>
                        </form>
                        {move || trace_result.get().map(|res| {
                            let eval0 = res
                                .get("evaluations")
                                .and_then(Value::as_array)
                                .and_then(|arr| arr.first());
                            let dec = eval0.or_else(|| res.get("decision")).unwrap_or(&res);
                            let outbound = dec
                                .get("outbound")
                                .or_else(|| res.get("outbound"))
                                .and_then(|o| o.get("name").and_then(Value::as_str).or_else(|| o.as_str()))
                                .unwrap_or("—")
                                .to_string();
                            let matched_expr = eval0
                                .and_then(|ev| ev.get("rules"))
                                .and_then(Value::as_array)
                                .and_then(|rules| {
                                    rules.iter().find(|r| {
                                        r.get("matched").and_then(Value::as_bool) == Some(true)
                                            || matches!(
                                                r.get("result").or_else(|| r.get("outcome")).and_then(Value::as_str),
                                                Some("matched" | "match")
                                            )
                                    })
                                })
                                .and_then(|r| r.get("expression").and_then(Value::as_str))
                                .or_else(|| {
                                    res.get("matched_rule")
                                        .or_else(|| dec.get("matched_rule"))
                                        .and_then(|r| r.get("expression").and_then(Value::as_str).or_else(|| r.as_str()))
                                })
                                .unwrap_or("fallback")
                                .to_string();
                            let decision_kind = dec
                                .get("decision")
                                .and_then(Value::as_str)
                                .or_else(|| res.get("mode").and_then(Value::as_str))
                                .unwrap_or("evaluated")
                                .to_string();
                            let dst_ip = dec
                                .get("dst_ip")
                                .and_then(Value::as_str)
                                .or_else(|| {
                                    res.get("dns")
                                        .and_then(Value::as_array)
                                        .and_then(|arr| arr.first())
                                        .and_then(|d| d.get("ip").or_else(|| d.get("data")).and_then(Value::as_str))
                                })
                                .unwrap_or("—")
                                .to_string();
                            view! {
                                <div class="rp-card" style="background:var(--rp-surface);">
                                    <div class="rp-between" style="margin-bottom:8px;">
                                        <StatusLight tone="ok" text=format!("出站决策: {outbound}") />
                                        <span class="rp-badge">{decision_kind}</span>
                                    </div>
                                    <div class="rp-kv">
                                        <div><span class="k">"命中规则"</span><span class="v"><code>{matched_expr}</code></span></div>
                                        <div><span class="k">"最终出站"</span><span class="v">{outbound}</span></div>
                                        <div><span class="k">"评估目标 IP"</span><span class="v"><code>{dst_ip}</code></span></div>
                                    </div>
                                </div>
                            }
                        })}
                    </section>
                }.into_any(),
                "dns" => view! {
                    <section class="rp-card" style="overflow:auto;">
                        {move || match dns_rules.get() {
                            None => view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any(),
                            Some(raw) => {
                                let req_rules = raw
                                    .get("request")
                                    .or_else(|| raw.get("rules"))
                                    .and_then(|r| r.as_array().cloned().or_else(|| r.get("rules").and_then(Value::as_array).cloned()))
                                    .unwrap_or_default();
                                let resp_rules = raw
                                    .get("response")
                                    .and_then(|r| r.as_array().cloned().or_else(|| r.get("rules").and_then(Value::as_array).cloned()))
                                    .unwrap_or_default();
                                view! {
                                    <div style="display:flex;flex-direction:column;gap:14px;">
                                        <div>
                                            <h3 class="rp-h3" style="margin-bottom:8px;">"DNS 请求分流规则 (Request)"</h3>
                                            {if req_rules.is_empty() {
                                                view! { <p class="rp-label">"使用默认 DNS 请求策略"</p> }.into_any()
                                            } else {
                                                view! {
                                                    <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                                        <thead>
                                                            <tr>
                                                                <th style="text-align:left;padding:6px 8px;">"#"</th>
                                                                <th style="text-align:left;padding:6px 8px;">"匹配表达式"</th>
                                                                <th style="text-align:left;padding:6px 8px;">"动作 / 上游"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            {req_rules.into_iter().enumerate().map(|(i, r)| {
                                                                let expr = r.get("expression").or_else(|| r.get("match")).and_then(Value::as_str).unwrap_or("fallback").to_string();
                                                                let act = r.get("action").or_else(|| r.get("upstream")).or_else(|| r.get("target")).and_then(Value::as_str).unwrap_or("—").to_string();
                                                                view! {
                                                                    <tr>
                                                                        <td style="padding:6px 8px;">{i + 1}</td>
                                                                        <td style="padding:6px 8px;"><code>{expr}</code></td>
                                                                        <td style="padding:6px 8px;"><span class="rp-badge">{act}</span></td>
                                                                    </tr>
                                                                }
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                }.into_any()
                                            }}
                                        </div>
                                        {(!resp_rules.is_empty()).then(|| view! {
                                            <div>
                                                <h3 class="rp-h3" style="margin-bottom:8px;">"DNS 响应规则 (Response)"</h3>
                                                <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                                    <thead>
                                                        <tr>
                                                            <th style="text-align:left;padding:6px 8px;">"#"</th>
                                                            <th style="text-align:left;padding:6px 8px;">"匹配表达式"</th>
                                                            <th style="text-align:left;padding:6px 8px;">"动作"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {resp_rules.into_iter().enumerate().map(|(i, r)| {
                                                            let expr = r.get("expression").or_else(|| r.get("match")).and_then(Value::as_str).unwrap_or("fallback").to_string();
                                                            let act = r.get("action").or_else(|| r.get("upstream")).or_else(|| r.get("target")).and_then(Value::as_str).unwrap_or("—").to_string();
                                                            view! {
                                                                <tr>
                                                                    <td style="padding:6px 8px;">{i + 1}</td>
                                                                    <td style="padding:6px 8px;"><code>{expr}</code></td>
                                                                    <td style="padding:6px 8px;"><span class="rp-badge">{act}</span></td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        })}
                                    </div>
                                }.into_any()
                            }
                        }}
                    </section>
                }.into_any(),
                _ => view! {
                    <section class="rp-card" style="overflow:auto;">
                        {move || {
                            let q = rule_filter.get().to_lowercase();
                            let items: Vec<_> = rules
                                .get()
                                .into_iter()
                                .enumerate()
                                .filter(|(_, r)| q.is_empty() || r.to_string().to_lowercase().contains(&q))
                                .collect();
                            if items.is_empty() {
                                return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                            }
                            view! {
                                <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                    <thead>
                                        <tr>
                                            <th style="text-align:left;padding:6px 8px;">"#"</th>
                                            <th style="text-align:left;padding:6px 8px;">"Expression"</th>
                                            <th style="text-align:left;padding:6px 8px;">"Outbound"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {items.into_iter().map(|(i, r)| {
                                            let expr = r.get("expression").and_then(Value::as_str).unwrap_or("fallback").to_string();
                                            let out = r.get("outbound").and_then(Value::as_str).unwrap_or("—").to_string();
                                            view! {
                                                <tr>
                                                    <td style="padding:6px 8px;">{i + 1}</td>
                                                    <td style="padding:6px 8px;"><code>{expr}</code></td>
                                                    <td style="padding:6px 8px;"><span class="rp-badge">{out}</span></td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }}
                    </section>
                }.into_any(),
            };
            view! { <div class="rp-subtab-stage" data-parity=parity>{inner}</div> }
            }}
        </div>
    }
}
