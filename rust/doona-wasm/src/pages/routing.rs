//! Routing Hub pages: Policy Groups selector & probe, Nodes & Subscriptions manager, Rules & Routing Trace.

use std::collections::HashMap;

use doona_common::format::format_latency;
use doona_common::model::GroupDetail;
use leptos::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen_futures::spawn_local;

use crate::state::use_app_state;

#[component]
pub fn PoliciesPage() -> impl IntoView {
    let state = use_app_state();
    let details = RwSignal::new(HashMap::<String, GroupDetail>::new());
    let busy_group = RwSignal::new(None::<String>);

    // Load full details for all groups when group list updates.
    Effect::new(move |_| {
        let gs = state.groups.get();
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

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            {move || {
                let gs = state.groups.get();
                let det_map = details.get();
                let nodes = state.nodes.get().map(|n| n.nodes).unwrap_or_default();
                if gs.is_empty() {
                    return view! { <section class="rp-card"><p class="rp-empty">{state.t("ui.empty")}</p></section> }.into_any();
                }
                gs.into_iter().map(|g| {
                    let gid = g.id.clone();
                    let gid_probe_dis = g.id.clone();
                    let gid_probe = g.id.clone();
                    let gid_clear = g.id.clone();
                    let det = det_map.get(&gid).cloned();
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
                            <div class="rp-between" style="margin-bottom:10px;flex-wrap:wrap;gap:8px;">
                                <div class="rp-cluster">
                                    <h2 class="rp-h3">{g.name.clone()}</h2>
                                    <span class="rp-badge">{g.policy.kind.clone()}</span>
                                    <span class="rp-label">{format!("{} members", g.member_count)}</span>
                                </div>
                                <div class="rp-cluster">
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
                                                    Ok(_) => {
                                                        state.toast("positive", state.t("pol.probeStarted"), None);
                                                        if let Ok(d) = api.group(&id).await {
                                                            details.update(|m| { m.insert(id, d); });
                                                        }
                                                    }
                                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                }
                                                busy_group.set(None);
                                            });
                                        }
                                    >
                                        {move || state.t("pol.probe")}
                                    </button>
                                    {det.as_ref().is_some_and(|d| d.capabilities.can_override).then(|| view! {
                                        <button
                                            type="button"
                                            class="rp-btn quiet sm"
                                            on:click=move |_| {
                                                let id = gid_clear.clone();
                                                let api = state.client();
                                                spawn_local(async move {
                                                    let _ = api.clear_group_override(&id, "both").await;
                                                    if let Ok(d) = api.group(&id).await {
                                                        details.update(|m| { m.insert(id, d); });
                                                    }
                                                });
                                            }
                                        >
                                            {move || state.t("pol.clearOverride")}
                                        </button>
                                    })}
                                </div>
                            </div>
                            <div class="rp-nodes" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(190px,1fr));gap:8px;">
                                {det.map(|d| d.members.into_iter().map(|m| {
                                    let is_sel = m.id == selected_id;
                                    let mid = m.id.clone();
                                    let gid_sel = gid.clone();
                                    let lat = nodes
                                        .iter()
                                        .find(|n| n.id == m.id)
                                        .and_then(|n| n.best_latency_ms());
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
                        </section>
                    }
                }).collect_view().into_any()
            }}
        </div>
    }
}

#[component]
pub fn NodesPage() -> impl IntoView {
    let state = use_app_state();
    let tab = RwSignal::new("nodes".to_string());
    let filter = RwSignal::new(String::new());
    let new_name = RwSignal::new(String::new());
    let new_link = RwSignal::new(String::new());
    let sub_name = RwSignal::new(String::new());
    let sub_url = RwSignal::new(String::new());

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

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-seg" data-size="M">
                    <button
                        type="button"
                        class="rp-btn"
                        data-size="M"
                        data-selected=move || (tab.get() == "nodes").then_some("true")
                        on:click=move |_| tab.set("nodes".to_string())
                    >
                        {move || state.t("nav.nodes")}
                    </button>
                    <button
                        type="button"
                        class="rp-btn"
                        data-size="M"
                        data-selected=move || (tab.get() == "subscriptions").then_some("true")
                        on:click=move |_| tab.set("subscriptions".to_string())
                    >
                        {move || state.t("nodes.subscriptions")}
                    </button>
                </div>
                <input
                    class="rp-input"
                    type="search"
                    placeholder=move || state.t("ui.search")
                    prop:value=move || filter.get()
                    on:input=move |ev| filter.set(event_target_value(&ev))
                />
            </div>

            {move || if tab.get() == "subscriptions" {
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
                                                let pid_ref = p.id.clone();
                                                let pid_del = p.id.clone();
                                                view! {
                                                    <tr>
                                                        <td style="padding:6px 8px;"><strong>{p.name}</strong></td>
                                                        <td style="padding:6px 8px;"><span class="rp-badge">{p.kind.unwrap_or_else(|| "subscription".into())}</span></td>
                                                        <td style="text-align:right;padding:6px 8px;">{p.node_count.map(|c| c.to_string()).unwrap_or_else(|| "—".into())}</td>
                                                        <td style="text-align:right;padding:6px 8px;">
                                                            <span class="rp-cluster" style="justify-content:flex-end;">
                                                                <button type="button" class="rp-btn secondary sm" on:click=move |_| {
                                                                    let id = pid_ref.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        match api.refresh_provider(&id).await {
                                                                            Ok(_) => {
                                                                                state.toast("positive", state.t("ui.refresh"), None);
                                                                                state.refresh_all();
                                                                            }
                                                                            Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                                        }
                                                                    });
                                                                }>{state.t("ui.refresh")}</button>
                                                                <button type="button" class="rp-btn quiet sm" on:click=move |_| {
                                                                    let id = pid_del.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        let _ = api.delete_provider(&id).await;
                                                                        state.refresh_all();
                                                                    });
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
                                                let nid_p = n.id.clone();
                                                let nid_d = n.id.clone();
                                                let lat = format_latency(n.best_latency_ms());
                                                let health = n.health_state().to_string();
                                                view! {
                                                    <tr>
                                                        <td style="padding:6px 8px;">{n.name}</td>
                                                        <td style="padding:6px 8px;"><span class="rp-badge">{n.protocol.unwrap_or_else(|| "—".into())}</span></td>
                                                        <td style="padding:6px 8px;">{n.subscription_tag.unwrap_or_else(|| "inline".into())}</td>
                                                        <td style="text-align:right;padding:6px 8px;">
                                                            <span class=format!("rp-light {}", if health == "alive" { "ok" } else if health == "dead" { "err" } else { "muted" })>{lat}</span>
                                                        </td>
                                                        <td style="text-align:right;padding:6px 8px;">
                                                            <span class="rp-cluster" style="justify-content:flex-end;">
                                                                <button type="button" class="rp-btn secondary sm" on:click=move |_| {
                                                                    let id = nid_p.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        let _ = api.probe_node(&id).await;
                                                                        state.refresh_all();
                                                                    });
                                                                }>{state.t("pol.probe")}</button>
                                                                <button type="button" class="rp-btn quiet sm" on:click=move |_| {
                                                                    let id = nid_d.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        let _ = api.delete_node(&id).await;
                                                                        state.refresh_all();
                                                                    });
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
            }}
        </div>
    }
}

#[component]
pub fn RulesPage() -> impl IntoView {
    let state = use_app_state();
    let tab = RwSignal::new("list".to_string());
    let rules = RwSignal::new(Vec::<Value>::new());
    let dns_rules = RwSignal::new(None::<Value>);
    let trace_domain = RwSignal::new("www.google.com".to_string());
    let trace_port = RwSignal::new("443".to_string());
    let trace_net = RwSignal::new("tcp".to_string());
    let trace_result = RwSignal::new(None::<Value>);

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
        let port = trace_port.get_untracked().parse::<u16>().unwrap_or(443);
        let net = trace_net.get_untracked();
        if domain.is_empty() {
            return;
        }
        let api = state.client();
        spawn_local(async move {
            let input = json!({
                "network": net,
                "domain": domain,
                "dst_port": port,
            });
            match api.routing_trace(input).await {
                Ok(res) => trace_result.set(Some(res)),
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-seg" data-size="M">
                {[("list", "nav.rules"), ("dns", "nav.dns"), ("trace", "rules.tabTrace")].into_iter().map(|(id, key)| view! {
                    <button
                        type="button"
                        class="rp-btn"
                        data-size="M"
                        data-selected=move || (tab.get() == id).then_some("true")
                        on:click=move |_| tab.set(id.to_string())
                    >
                        {move || state.t(key)}
                    </button>
                }).collect_view()}
            </div>

            {move || match tab.get().as_str() {
                "trace" => view! {
                    <section class="rp-card" style="display:flex;flex-direction:column;gap:12px;">
                        <form class="rp-cluster" on:submit=run_trace style="flex-wrap:wrap;">
                            <input class="rp-input" placeholder="domain" style="flex:1;min-width:180px;" prop:value=move || trace_domain.get() on:input=move |ev| trace_domain.set(event_target_value(&ev)) />
                            <input class="rp-input" placeholder="443" style="width:90px;" prop:value=move || trace_port.get() on:input=move |ev| trace_port.set(event_target_value(&ev)) />
                            <select class="rp-input" prop:value=move || trace_net.get() on:change=move |ev| trace_net.set(event_target_value(&ev))>
                                <option value="tcp">"TCP"</option>
                                <option value="udp">"UDP"</option>
                            </select>
                            <button type="submit" class="rp-btn accent">{move || state.t("rules.traceRun")}</button>
                        </form>
                        {move || trace_result.get().map(|res| view! {
                            <pre class="rp-limit-snippet" style="overflow:auto;max-height:380px;">{serde_json::to_string_pretty(&res).unwrap_or_default()}</pre>
                        })}
                    </section>
                }.into_any(),
                "dns" => view! {
                    <section class="rp-card">
                        <pre class="rp-limit-snippet" style="overflow:auto;max-height:480px;">
                            {serde_json::to_string_pretty(&dns_rules.get()).unwrap_or_default()}
                        </pre>
                    </section>
                }.into_any(),
                _ => view! {
                    <section class="rp-card" style="overflow:auto;">
                        {move || {
                            let items = rules.get();
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
                                        {items.into_iter().enumerate().map(|(i, r)| {
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
            }}
        </div>
    }
}
