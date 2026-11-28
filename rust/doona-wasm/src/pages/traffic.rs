//! Traffic Hub pages: Connections, Flows, DNS (Query / Cache / Log / Rules), Live Logs SSE, Live Events SSE.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use doona_common::format::format_bytes;
use doona_common::i18n::Lang;
use doona_common::model::{ConnectionItem, value_u64};
#[cfg(target_arch = "wasm32")]
use doona_common::sse::SseParser;
use leptos::prelude::*;
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

use crate::state::use_app_state;
use crate::ui::{ConfirmSheet, StatusLight};

fn json_str<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for &k in keys {
        if let Some(s) = v.get(k).and_then(Value::as_str)
            && !s.is_empty()
        {
            return Some(s);
        }
    }
    None
}

fn json_u64_any(v: &Value, keys: &[&str]) -> Option<u64> {
    for &k in keys {
        if let Some(val) = v.get(k) {
            if let Some(n) = val.as_u64() {
                return Some(n);
            }
            if let Some(s) = val.as_str()
                && let Ok(n) = s.parse::<u64>()
            {
                return Some(n);
            }
        }
    }
    None
}

#[component]
pub fn ConnectionsPage() -> impl IntoView {
    let state = use_app_state();
    let filter = RwSignal::new(String::new());
    let rows = RwSignal::new(Vec::<(String, ConnectionItem)>::new());
    let totals = RwSignal::new((0u64, 0u64));
    let loading = RwSignal::new(true);
    let close_all_confirm = RwSignal::new(false);
    let selected = RwSignal::new(None::<(String, ConnectionItem)>);

    let load = move || {
        let api = state.client();
        spawn_local(async move {
            if let Ok(list) = api.connections().await {
                totals.set((list.total_tcp, list.total_udp));
                let mut combined = Vec::with_capacity(list.tcp.len() + list.udp.len());
                for c in list.tcp {
                    combined.push(("TCP".to_string(), c));
                }
                for c in list.udp {
                    combined.push(("UDP".to_string(), c));
                }
                rows.set(combined);
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        load();
    });

    let close_all = Callback::new(move |()| {
        let api = state.client();
        spawn_local(async move {
            match api.close_all_connections().await {
                Ok(_) => {
                    selected.set(None);
                    state.toast("positive", state.t("conn.closedAll"), None);
                    load();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-cluster" style="flex-wrap:wrap;">
                    <input
                        class="rp-input"
                        type="search"
                        placeholder=move || state.t("ui.search")
                        prop:value=move || filter.get()
                        on:input=move |ev| filter.set(event_target_value(&ev))
                    />
                    <span class="rp-badge">{move || format!("TCP {} · UDP {}", totals.get().0, totals.get().1)}</span>
                </div>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| load()>
                        {move || state.t("ui.refresh")}
                    </button>
                    <button
                        type="button"
                        class="rp-btn negative sm"
                        disabled=move || rows.get().is_empty()
                        on:click=move |_| close_all_confirm.set(true)
                    >
                        {move || state.t("conn.closeAll")}
                    </button>
                </div>
            </div>

            {move || selected.get().map(|(proto, c)| {
                let cid = c.id.clone();
                let target = c.domain.clone().or(c.dst.clone()).unwrap_or_else(|| "—".into());
                let src_addr = c.src.clone().unwrap_or_else(|| "—".into());
                let dst_addr = c.dst.clone().unwrap_or_else(|| "—".into());
                let chain = state.format_chain(&c.chain, c.outbound.as_deref());
                let rule = c.rule_expression.clone().unwrap_or_else(|| "fallback".into());
                let proc_name = c.pname.clone().unwrap_or_else(|| "—".into());
                let dl = value_u64(&c.download_bytes).map(format_bytes).unwrap_or_else(|| "0 B".into());
                let ul = value_u64(&c.upload_bytes).map(format_bytes).unwrap_or_else(|| "0 B".into());
                view! {
                    <div class="rp-ios-sheet-backdrop" on:click=move |_| selected.set(None)>
                        <section class="rp-card rp-ios-sheet" on:click=move |ev| ev.stop_propagation()>
                            <div class="rp-between" style="gap:8px;">
                                <div class="rp-cluster" style="min-width:0;">
                                    <span class="rp-badge">{proto}</span>
                                    <strong style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;"><code>{target}</code></strong>
                                </div>
                                <div class="rp-cluster">
                                    <button
                                        type="button"
                                        class="rp-btn negative sm"
                                        on:click=move |_| {
                                            let id = cid.clone();
                                            let api = state.client();
                                            spawn_local(async move {
                                                let _ = api.close_connection(&id).await;
                                                selected.set(None);
                                                load();
                                            });
                                        }
                                    >
                                        {move || match state.lang.get() {
                                            Lang::En => "Disconnect",
                                            Lang::ZhTw => "斷開連線",
                                            Lang::ZhCn => "断开连接",
                                        }}
                                    </button>
                                    <button type="button" class="rp-btn quiet sm" on:click=move |_| selected.set(None)>"×"</button>
                                </div>
                            </div>
                            <div class="rp-kv">
                                <div><span class="k">"五元组 / 地址"</span><span class="v"><code>{format!("{src_addr} → {dst_addr}")}</code></span></div>
                                <div><span class="k">{state.t("conn.outbound")}</span><span class="v">{chain}</span></div>
                                <div><span class="k">{state.t("nav.rules")}</span><span class="v"><code>{rule}</code></span></div>
                                <div><span class="k">{state.t("conn.process")}</span><span class="v">{proc_name}</span></div>
                                <div><span class="k">{state.t("ui.download")}</span><span class="v">{dl}</span></div>
                                <div><span class="k">{state.t("ui.upload")}</span><span class="v">{ul}</span></div>
                            </div>
                        </section>
                    </div>
                }
            })}

            <section class="rp-card" style="overflow-x:auto;">
                {move || {
                    if loading.get() {
                        return view! { <p class="rp-label">{state.t("ov.loading")}</p> }.into_any();
                    }
                    let q = filter.get().to_lowercase();
                    let list: Vec<_> = rows
                        .get()
                        .into_iter()
                        .filter(|(proto, c)| {
                            if q.is_empty() {
                                return true;
                            }
                            let resolved_chain = state.format_chain(&c.chain, c.outbound.as_deref());
                            let hay = format!(
                                "{} {} {} {} {} {} {} {}",
                                proto,
                                c.domain.as_deref().unwrap_or(""),
                                c.src.as_deref().unwrap_or(""),
                                c.dst.as_deref().unwrap_or(""),
                                c.outbound.as_deref().unwrap_or(""),
                                resolved_chain,
                                c.pname.as_deref().unwrap_or(""),
                                c.rule_expression.as_deref().unwrap_or(""),
                            )
                            .to_lowercase();
                            hay.contains(&q)
                        })
                        .collect();
                    if list.is_empty() {
                        return view! { <p class="rp-empty">{state.t("act.rankingEmpty")}</p> }.into_any();
                    }
                    view! {
                        <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                            <thead>
                                <tr>
                                    <th style="text-align:left;padding:6px 8px;white-space:nowrap;">"协议"</th>
                                    <th style="text-align:left;padding:6px 8px;min-width:130px;">{state.t("conn.target")}</th>
                                    <th style="text-align:left;padding:6px 8px;min-width:120px;">{state.t("conn.outbound")}</th>
                                    <th style="text-align:left;padding:6px 8px;white-space:nowrap;">{state.t("conn.process")}</th>
                                    <th style="text-align:right;padding:6px 8px;white-space:nowrap;">{state.t("ui.download")}</th>
                                    <th style="text-align:right;padding:6px 8px;white-space:nowrap;">{state.t("ui.upload")}</th>
                                    <th style="padding:6px 8px;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().take(200).map(|(proto, c)| {
                                    let row_pair = (proto.clone(), c.clone());
                                    let id = c.id.clone();
                                    let target = c.domain.clone().or(c.dst.clone()).unwrap_or_else(|| "—".into());
                                    let out = state.format_chain(&c.chain, c.outbound.as_deref());
                                    let dl = value_u64(&c.download_bytes).map(format_bytes).unwrap_or_else(|| "—".into());
                                    let ul = value_u64(&c.upload_bytes).map(format_bytes).unwrap_or_else(|| "—".into());
                                    view! {
                                        <tr style="cursor:pointer;border-bottom:0.5px solid var(--rp-hl-med);" on:click=move |_| selected.set(Some(row_pair.clone()))>
                                            <td style="padding:7px 8px;white-space:nowrap;"><span class="rp-badge">{proto}</span></td>
                                            <td style="padding:7px 8px;min-width:130px;overflow-wrap:anywhere;"><code>{target}</code></td>
                                            <td style="padding:7px 8px;min-width:120px;">{out}</td>
                                            <td style="padding:7px 8px;white-space:nowrap;">{c.pname.unwrap_or_else(|| "—".into())}</td>
                                            <td style="text-align:right;padding:7px 8px;white-space:nowrap;">{dl}</td>
                                            <td style="text-align:right;padding:7px 8px;white-space:nowrap;">{ul}</td>
                                            <td style="padding:6px 8px;" on:click=move |ev| ev.stop_propagation()>
                                                <button
                                                    type="button"
                                                    class="rp-btn quiet sm"
                                                    on:click=move |_| {
                                                        let cid = id.clone();
                                                        let api = state.client();
                                                        spawn_local(async move {
                                                            let _ = api.close_connection(&cid).await;
                                                            load();
                                                        });
                                                    }
                                                >
                                                    "×"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </section>

            <ConfirmSheet
                open=close_all_confirm
                title=Signal::derive(move || state.t("conn.closeAll"))
                message=Signal::derive(move || match state.lang.get() {
                    Lang::En => "Close all active TCP and UDP connections immediately?".into(),
                    Lang::ZhTw => "確定要立即斷開全部活躍的 TCP 與 UDP 連線嗎？".into(),
                    Lang::ZhCn => "确定要立即断开当前所有活跃的 TCP 与 UDP 连接吗？".into(),
                })
                confirm_label=Signal::derive(move || state.t("conn.closeAll"))
                destructive=true
                on_confirm=close_all
            />
        </div>
    }
}

#[component]
pub fn FlowsPage() -> impl IntoView {
    let state = use_app_state();
    let flows = RwSignal::new(Vec::<Value>::new());
    let selected = RwSignal::new(None::<Value>);
    let filter = RwSignal::new(String::new());

    let load = move || {
        let api = state.client();
        spawn_local(async move {
            if let Ok(raw) = api.flows().await {
                let items = raw
                    .get("flows")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                flows.set(items);
            }
        });
    };

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        load();
    });

    // Auto-open flow detail sheet when navigated from EventsPage (`#flows?flow=<id>`).
    Effect::new(move |_| {
        if let Some(fid) = crate::state::query_param(&state.query.get(), "flow")
            && !fid.is_empty()
        {
            let api = state.client();
            spawn_local(async move {
                if let Ok(det) = api.flow_detail(&fid).await {
                    selected.set(Some(det));
                }
            });
        }
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <input
                    class="rp-input"
                    type="search"
                    placeholder=move || state.t("ui.search")
                    prop:value=move || filter.get()
                    on:input=move |ev| filter.set(event_target_value(&ev))
                />
                <button type="button" class="rp-btn secondary sm" on:click=move |_| load()>
                    {move || state.t("ui.refresh")}
                </button>
            </div>

            {move || selected.get().map(|det| {
                let inp = det.get("input");
                let id = json_str(&det, &["id", "flow_id"]).unwrap_or("—");
                let short_fid: String = id.chars().take(12).collect();
                let net = json_str(&det, &["network", "transport", "protocol"]).unwrap_or("tcp").to_uppercase();
                let obs_by = json_str(&det, &["observed_by", "trace_status", "scope"]).unwrap_or("userspace").to_string();
                let domain = inp
                    .and_then(|i| json_str(i, &["domain", "sni", "host"]))
                    .or_else(|| json_str(&det, &["domain", "sni", "host"]))
                    .or_else(|| inp.and_then(|i| json_str(i, &["dst", "destination"])))
                    .unwrap_or("—")
                    .to_string();
                let src = inp
                    .and_then(|i| json_str(i, &["src", "source"]))
                    .or_else(|| json_str(&det, &["src", "source"]))
                    .unwrap_or("—")
                    .to_string();
                let dst = inp
                    .and_then(|i| json_str(i, &["dst", "destination"]))
                    .or_else(|| json_str(&det, &["dst", "destination"]))
                    .unwrap_or("—")
                    .to_string();
                let pname = inp
                    .and_then(|i| json_str(i, &["pname", "process"]))
                    .or_else(|| json_str(&det, &["pname", "process"]))
                    .unwrap_or("—")
                    .to_string();
                let chain_vec: Vec<String> = det
                    .get("chain")
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_string).collect())
                    .unwrap_or_default();
                let out_fb = json_str(&det, &["outbound", "selected_outbound"]);
                let out = state.format_chain(&chain_vec, out_fb);
                let rule = det
                    .get("rule")
                    .and_then(|r| r.get("expression").and_then(Value::as_str).or_else(|| r.as_str()))
                    .or_else(|| json_str(&det, &["rule_expression"]))
                    .unwrap_or("fallback")
                    .to_string();
                let st_str = json_str(&det, &["state", "status", "terminal_reason"]).unwrap_or("—").to_string();
                let steps = det
                    .pointer("/trace/steps")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let last_conn = steps
                    .iter()
                    .rev()
                    .find(|s| s.get("type").or_else(|| s.get("stage")).and_then(Value::as_str) == Some("connection"));
                let dl = json_u64_any(&det, &["download_bytes", "rx_bytes"])
                    .or_else(|| last_conn.and_then(|s| json_u64_any(s, &["bytes_received", "download_bytes", "rx_bytes"])))
                    .map(format_bytes)
                    .unwrap_or_else(|| "0 B".into());
                let ul = json_u64_any(&det, &["upload_bytes", "tx_bytes"])
                    .or_else(|| last_conn.and_then(|s| json_u64_any(s, &["bytes_sent", "upload_bytes", "tx_bytes"])))
                    .map(format_bytes)
                    .unwrap_or_else(|| "0 B".into());
                let started = json_str(&det, &["started_at", "created_at", "observed_at"]).unwrap_or("—").to_string();
                view! {
                    <div class="rp-ios-sheet-backdrop" on:click=move |_| selected.set(None)>
                        <section class="rp-card rp-ios-sheet" style="max-height:80vh;overflow:auto;" on:click=move |ev| ev.stop_propagation()>
                            <div class="rp-between" style="gap:8px;">
                                <div class="rp-cluster" style="min-width:0;">
                                    <span class="rp-badge">{net}</span>
                                    <strong style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;"><code>{domain}</code></strong>
                                    <span class="rp-badge">{st_str}</span>
                                </div>
                                <button type="button" class="rp-btn quiet sm" on:click=move |_| selected.set(None)>"×"</button>
                            </div>
                            <div class="rp-kv">
                                <div><span class="k">"流 ID"</span><span class="v"><code>{short_fid}</code></span></div>
                                <div><span class="k">"观测面"</span><span class="v">{obs_by}</span></div>
                                <div><span class="k">"五元组路径"</span><span class="v"><code>{format!("{src} → {dst}")}</code></span></div>
                                <div><span class="k">{state.t("conn.process")}</span><span class="v">{pname}</span></div>
                                <div><span class="k">{state.t("conn.outbound")}</span><span class="v">{out}</span></div>
                                <div><span class="k">{state.t("nav.rules")}</span><span class="v"><code>{rule}</code></span></div>
                                <div><span class="k">"流量统计"</span><span class="v">{format!("↓ {dl} · ↑ {ul}")}</span></div>
                                <div><span class="k">"开始时间"</span><span class="v">{started}</span></div>
                            </div>
                        </section>
                    </div>
                }
            })}

            <section class="rp-card" style="overflow-x:auto;">
                {move || {
                    let q = filter.get().to_lowercase();
                    let items: Vec<_> = flows
                        .get()
                        .into_iter()
                        .filter(|f| q.is_empty() || f.to_string().to_lowercase().contains(&q))
                        .collect();
                    if items.is_empty() {
                        return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                    }
                    view! {
                        <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                            <thead>
                                <tr>
                                    <th style="text-align:left;padding:6px 8px;white-space:nowrap;">"协议"</th>
                                    <th style="text-align:left;padding:6px 8px;min-width:130px;">{state.t("conn.target")}</th>
                                    <th style="text-align:left;padding:6px 8px;min-width:120px;">{state.t("conn.outbound")}</th>
                                    <th style="text-align:left;padding:6px 8px;white-space:nowrap;">"状态"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {items.into_iter().take(100).map(|f| {
                                    let fallback_val = f.clone();
                                    let fid = json_str(&f, &["id", "flow_id"]).unwrap_or("").to_string();
                                    let net = json_str(&f, &["network", "protocol"]).unwrap_or("tcp").to_uppercase();
                                    let inp = f.get("input");
                                    let target = inp
                                        .and_then(|i| json_str(i, &["domain", "dst"]))
                                        .or_else(|| json_str(&f, &["domain", "dst"]))
                                        .unwrap_or("—")
                                        .to_string();
                                    let chain_vec: Vec<String> = f
                                        .get("chain")
                                        .and_then(Value::as_array)
                                        .map(|arr| arr.iter().filter_map(Value::as_str).map(str::to_string).collect())
                                        .unwrap_or_default();
                                    let out = state.format_chain(&chain_vec, json_str(&f, &["outbound"]));
                                    let st = json_str(&f, &["state", "trace_status"]).unwrap_or("—").to_string();
                                    let fid_click = fid.clone();
                                    view! {
                                        <tr style="cursor:pointer;border-bottom:0.5px solid var(--rp-hl-med);" on:click=move |_| {
                                            let api = state.client();
                                            let id = fid_click.clone();
                                            let fb = fallback_val.clone();
                                            spawn_local(async move {
                                                match api.flow_detail(&id).await {
                                                    Ok(det) => selected.set(Some(det)),
                                                    Err(_) => selected.set(Some(fb)),
                                                }
                                            });
                                        }>
                                            <td style="padding:7px 8px;white-space:nowrap;"><span class="rp-badge">{net}</span></td>
                                            <td style="padding:7px 8px;min-width:130px;overflow-wrap:anywhere;"><code>{target}</code></td>
                                            <td style="padding:7px 8px;min-width:120px;">{out}</td>
                                            <td style="padding:7px 8px;white-space:nowrap;"><span class="rp-badge">{st}</span></td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </section>
        </div>
    }
}

#[component]
pub fn DnsPage() -> impl IntoView {
    let state = use_app_state();
    let tab = RwSignal::new("query".to_string());
    let tab_seq = RwSignal::new(0u64);
    let domain = RwSignal::new("example.com".to_string());
    let qtype = RwSignal::new("A".to_string());
    let cache_mode = RwSignal::new("normal".to_string());
    let query_result = RwSignal::new(None::<Value>);
    let query_busy = RwSignal::new(false);
    let cache_items = RwSignal::new(Vec::<Value>::new());
    let cache_observed_at = RwSignal::new(None::<String>);
    let cache_total = RwSignal::new(0usize);
    let cache_search = RwSignal::new(String::new());
    let log_items = RwSignal::new(Vec::<Value>::new());
    let log_search = RwSignal::new(String::new());
    let flush_confirm = RwSignal::new(false);

    let load_dns = move || {
        let api = state.client();
        spawn_local(async move {
            if let Ok(raw) = api.dns_cache().await {
                let obs = raw
                    .get("observed_at")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let list = raw
                    .get("entries")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let tot = raw
                    .get("total")
                    .and_then(Value::as_u64)
                    .map(|n| n as usize)
                    .unwrap_or(list.len());
                cache_observed_at.set(obs);
                cache_total.set(tot);
                cache_items.set(list);
            }
            if let Ok(raw) = api.dns_log().await {
                let list = raw
                    .get("records")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                log_items.set(list);
            }
        });
    };

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        load_dns();
    });

    let run_query = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let d = domain.get_untracked().trim().to_string();
        let t = qtype.get_untracked();
        let cm = cache_mode.get_untracked();
        if d.is_empty() {
            return;
        }
        query_busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.dns_query_mode(&d, &[t.as_str()], &cm).await {
                Ok(val) => {
                    query_result.set(Some(val));
                    load_dns();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            query_busy.set(false);
        });
    };

    let flush_cache = Callback::new(move |()| {
        let api = state.client();
        spawn_local(async move {
            match api.flush_dns_cache().await {
                Ok(_) => {
                    state.toast("positive", state.t("dns.flushed"), None);
                    load_dns();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-seg" data-size="M">
                    {[("query", "dns.tabQuery"), ("cache", "dns.tabCache"), ("log", "dns.tabLog")].into_iter().map(|(id, key)| view! {
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
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| load_dns()>{move || state.t("ui.refresh")}</button>
                    <button type="button" class="rp-btn negative sm" on:click=move |_| flush_confirm.set(true)>{move || state.t("dns.flush")}</button>
                </div>
            </div>

            {move || {
                let cur_tab = tab.get();
                let parity = (tab_seq.get_untracked() % 2).to_string();
                let inner = match cur_tab.as_str() {
                "cache" => view! {
                    <div style="display:flex;flex-direction:column;gap:12px;">
                        <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                            <input
                                class="rp-input"
                                type="search"
                                placeholder="搜索缓存域名 / 记录值…"
                                prop:value=move || cache_search.get()
                                on:input=move |ev| cache_search.set(event_target_value(&ev))
                                style="flex:1;min-width:200px;"
                            />
                            <span class="rp-badge">{move || format!("共 {} 条", cache_total.get())}</span>
                            <button
                                type="button"
                                class="rp-btn secondary sm"
                                disabled=move || cache_search.get().trim().is_empty()
                                on:click=move |_| {
                                    let target_name = cache_search.get_untracked().trim().to_string();
                                    if target_name.is_empty() {
                                        return;
                                    }
                                    let api = state.client();
                                    spawn_local(async move {
                                        match api.delete_dns_cache_name(&target_name).await {
                                            Ok(_) => {
                                                state.toast("positive", format!("已清除 {target_name}"), None);
                                                load_dns();
                                            }
                                            Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                        }
                                    });
                                }
                            >
                                "按域名删除"
                            </button>
                        </div>
                        <section class="rp-card" style="overflow-x:auto;">
                            {move || {
                                let q = cache_search.get().to_lowercase();
                                let obs = cache_observed_at.get();
                                let items: Vec<_> = cache_items
                                    .get()
                                    .into_iter()
                                    .filter(|e| q.is_empty() || e.to_string().to_lowercase().contains(&q))
                                    .collect();
                                if items.is_empty() {
                                    return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                                }
                                view! {
                                    <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                        <thead>
                                            <tr>
                                                <th style="text-align:left;padding:6px 8px;">"域名"</th>
                                                <th style="text-align:left;padding:6px 8px;white-space:nowrap;">"类型"</th>
                                                <th style="text-align:left;padding:6px 8px;">"解析结果"</th>
                                                <th style="text-align:right;padding:6px 8px;white-space:nowrap;">"TTL"</th>
                                                <th style="text-align:right;padding:6px 8px;white-space:nowrap;">"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.into_iter().take(200).map(|entry| {
                                                let eid = json_str(&entry, &["id", "entry_id"]).unwrap_or("").to_string();
                                                let q_obj = entry.get("question");
                                                let qname = q_obj
                                                    .and_then(|q| json_str(q, &["name", "domain"]))
                                                    .or_else(|| json_str(&entry, &["name", "domain"]))
                                                    .unwrap_or("—")
                                                    .trim_end_matches('.')
                                                    .to_string();
                                                let rtype = q_obj
                                                    .and_then(|q| json_str(q, &["type", "rtype"]))
                                                    .or_else(|| json_str(&entry, &["type", "rtype"]))
                                                    .unwrap_or("A")
                                                    .to_string();
                                                let answers = doona_common::format::format_dns_cache_answers(&entry, &rtype);
                                                let ttl = doona_common::format::format_dns_cache_ttl(&entry, obs.as_deref());
                                                view! {
                                                    <tr style="border-bottom:0.5px solid var(--rp-hl-med);">
                                                        <td style="padding:7px 8px;min-width:140px;overflow-wrap:anywhere;"><code>{qname}</code></td>
                                                        <td style="padding:7px 8px;white-space:nowrap;"><span class="rp-badge">{rtype}</span></td>
                                                        <td style="padding:7px 8px;min-width:130px;overflow-wrap:anywhere;"><code>{answers}</code></td>
                                                        <td style="text-align:right;padding:7px 8px;white-space:nowrap;font-variant-numeric:tabular-nums;">{ttl}</td>
                                                        <td style="text-align:right;padding:7px 8px;white-space:nowrap;">
                                                            <button
                                                                type="button"
                                                                class="rp-btn quiet sm"
                                                                disabled=eid.is_empty()
                                                                on:click=move |_| {
                                                                    let id = eid.clone();
                                                                    let api = state.client();
                                                                    spawn_local(async move {
                                                                        match api.delete_dns_cache_entry(&id).await {
                                                                            Ok(_) => load_dns(),
                                                                            Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                                        }
                                                                    });
                                                                }
                                                            >
                                                                "×"
                                                            </button>
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
                }.into_any(),
                "log" => view! {
                    <div style="display:flex;flex-direction:column;gap:12px;">
                        <input
                            class="rp-input"
                            type="search"
                            placeholder="搜索 DNS 查询日志…"
                            prop:value=move || log_search.get()
                            on:input=move |ev| log_search.set(event_target_value(&ev))
                        />
                        <section class="rp-card" style="overflow:auto;">
                            {move || {
                                let q = log_search.get().to_lowercase();
                                let items: Vec<_> = log_items
                                    .get()
                                    .into_iter()
                                    .filter(|e| q.is_empty() || e.to_string().to_lowercase().contains(&q))
                                    .collect();
                                if items.is_empty() {
                                    return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                                }
                                view! {
                                    <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                        <thead>
                                            <tr>
                                                <th style="text-align:left;padding:6px 8px;">"域名"</th>
                                                <th style="text-align:left;padding:6px 8px;">"类型"</th>
                                                <th style="text-align:left;padding:6px 8px;">"上游 / 来源"</th>
                                                <th style="text-align:left;padding:6px 8px;">"状态"</th>
                                                <th style="text-align:right;padding:6px 8px;">"耗时"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.into_iter().take(200).map(|rec| {
                                                let q_obj = rec.get("question");
                                                let name = q_obj
                                                    .and_then(|q| json_str(q, &["name", "domain"]))
                                                    .or_else(|| json_str(&rec, &["domain", "name"]))
                                                    .unwrap_or("—")
                                                    .to_string();
                                                let qtype_s = q_obj
                                                    .and_then(|q| json_str(q, &["type", "rtype"]))
                                                    .or_else(|| json_str(&rec, &["type", "qtype"]))
                                                    .unwrap_or("A")
                                                    .to_string();
                                                let cached = rec.get("cached").and_then(Value::as_bool).unwrap_or(false);
                                                let upstream = json_str(&rec, &["upstream", "source", "resolver", "action"])
                                                    .map(str::to_string)
                                                    .unwrap_or_else(|| if cached { "本地缓存".to_string() } else { "—".to_string() });
                                                let rcode = doona_common::format::format_dns_cache_answers(&rec, &qtype_s);
                                                let dur = json_u64_any(&rec, &["duration_ms", "latency_ms", "elapsed_ms"])
                                                    .map(|ms| format!("{ms} ms"))
                                                    .unwrap_or_else(|| "—".into());
                                                view! {
                                                    <tr style="border-bottom:0.5px solid var(--rp-hl-med);">
                                                        <td style="padding:6px 8px;min-width:140px;overflow-wrap:anywhere;"><code>{name.trim_end_matches('.').to_string()}</code></td>
                                                        <td style="padding:6px 8px;white-space:nowrap;"><span class="rp-badge">{qtype_s}</span></td>
                                                        <td style="padding:6px 8px;white-space:nowrap;">{upstream}</td>
                                                        <td style="padding:6px 8px;min-width:120px;overflow-wrap:anywhere;"><code>{rcode}</code></td>
                                                        <td style="text-align:right;padding:6px 8px;white-space:nowrap;">{dur}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }}
                        </section>
                    </div>
                }.into_any(),
                _ => view! {
                    <section class="rp-card" style="display:flex;flex-direction:column;gap:12px;">
                        <form class="rp-cluster" on:submit=run_query style="flex-wrap:wrap;">
                            <input
                                class="rp-input"
                                type="text"
                                prop:value=move || domain.get()
                                on:input=move |ev| domain.set(event_target_value(&ev))
                                placeholder="example.com"
                                style="flex:1;min-width:190px;"
                            />
                            <select
                                class="rp-input"
                                prop:value=move || qtype.get()
                                on:change=move |ev| qtype.set(event_target_value(&ev))
                            >
                                <option value="A">"A"</option>
                                <option value="AAAA">"AAAA"</option>
                                <option value="HTTPS">"HTTPS"</option>
                                <option value="CNAME">"CNAME"</option>
                                <option value="TXT">"TXT"</option>
                            </select>
                            <select
                                class="rp-input"
                                prop:value=move || cache_mode.get()
                                on:change=move |ev| cache_mode.set(event_target_value(&ev))
                            >
                                <option value="normal">"缓存优先"</option>
                                <option value="bypass">"绕过缓存"</option>
                            </select>
                            <button type="submit" class="rp-btn accent" disabled=move || query_busy.get()>
                                {move || state.t("dns.run")}
                            </button>
                        </form>
                        {move || query_result.get().map(|res| {
                            let queries = res
                                .get("queries")
                                .or_else(|| res.get("results"))
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_else(|| vec![res.clone()]);
                            view! {
                                <div style="display:flex;flex-direction:column;gap:10px;">
                                    {queries.into_iter().map(|qres| {
                                        let rcode = json_str(&qres, &["rcode", "status", "outcome"]).unwrap_or("NOERROR").to_string();
                                        let tone: &'static str = if rcode.eq_ignore_ascii_case("noerror") || rcode.eq_ignore_ascii_case("ok") {
                                            "ok"
                                        } else {
                                            "warn"
                                        };
                                        let cached = qres.get("cached").and_then(Value::as_bool).unwrap_or(false);
                                        let upstream = json_str(&qres, &["upstream", "resolver", "source"])
                                            .map(str::to_string)
                                            .unwrap_or_else(|| if cached { "本地缓存命中".to_string() } else { "—".to_string() });
                                        let latency = json_u64_any(&qres, &["duration_ms", "latency_ms", "elapsed_ms"])
                                            .map(|ms| format!("{ms} ms"))
                                            .unwrap_or_else(|| "—".into());
                                        let answers = qres
                                            .get("answers")
                                            .or_else(|| qres.get("records"))
                                            .and_then(Value::as_array)
                                            .cloned()
                                            .unwrap_or_default();
                                        let empty_hint = format!("无应答记录 ({rcode})");
                                        view! {
                                            <div class="rp-card" style="background:var(--rp-surface);">
                                                <div class="rp-between" style="margin-bottom:8px;flex-wrap:wrap;gap:8px;">
                                                    <StatusLight tone=tone text=rcode />
                                                    <span class="rp-label">{format!("上游: {upstream} · 耗时: {latency}")}</span>
                                                </div>
                                                {if answers.is_empty() {
                                                    view! { <p class="rp-label">{empty_hint}</p> }.into_any()
                                                } else {
                                                    view! {
                                                        <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                                            <thead>
                                                                <tr>
                                                                    <th style="text-align:left;padding:4px 6px;">"名称"</th>
                                                                    <th style="text-align:left;padding:4px 6px;">"类型"</th>
                                                                    <th style="text-align:right;padding:4px 6px;">"TTL"</th>
                                                                    <th style="text-align:left;padding:4px 6px;">"记录值"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {answers.into_iter().map(|ans| {
                                                                    let aname = json_str(&ans, &["name", "domain"]).unwrap_or("@").to_string();
                                                                    let atype = json_str(&ans, &["type", "rtype"]).unwrap_or("—").to_string();
                                                                    let attl = json_u64_any(&ans, &["ttl", "ttl_seconds"]).map(|v| format!("{v}s")).unwrap_or_else(|| "—".into());
                                                                    let adata = json_str(&ans, &["data", "value", "rdata", "ip", "target"])
                                                                        .map(str::to_string)
                                                                        .unwrap_or_else(|| ans.to_string());
                                                                    view! {
                                                                        <tr>
                                                                            <td style="padding:4px 6px;"><code>{aname}</code></td>
                                                                            <td style="padding:4px 6px;"><span class="rp-badge">{atype}</span></td>
                                                                            <td style="text-align:right;padding:4px 6px;">{attl}</td>
                                                                            <td style="padding:4px 6px;"><code>{adata}</code></td>
                                                                        </tr>
                                                                    }
                                                                }).collect_view()}
                                                            </tbody>
                                                        </table>
                                                    }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }
                        })}
                    </section>
                }.into_any(),
            };
            view! { <div class="rp-subtab-stage" data-parity=parity>{inner}</div> }
            }}

            <ConfirmSheet
                open=flush_confirm
                title=Signal::derive(move || state.t("dns.flush"))
                message=Signal::derive(move || "确定要清空全部本地 DNS 缓存条目吗？".into())
                confirm_label=Signal::derive(move || state.t("dns.flush"))
                destructive=true
                on_confirm=flush_cache
            />
        </div>
    }
}

/// Shared pure-Rust SSE streaming helper using browser `fetch()` + `AbortSignal` + `ReadableStreamDefaultReader` + `SseParser`.
#[cfg(target_arch = "wasm32")]
async fn stream_sse(
    url: String,
    token: Option<String>,
    signal: web_sys::AbortSignal,
    paused: RwSignal<bool>,
    connected: RwSignal<bool>,
    mut on_frame: impl FnMut(doona_common::sse::SseFrame),
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_cache(web_sys::RequestCache::NoStore);
    opts.set_signal(Some(&signal));
    let Ok(headers) = web_sys::Headers::new() else {
        return;
    };
    let _ = headers.set("Accept", "text/event-stream");
    if let Some(tok) = token {
        let _ = headers.set("Authorization", &format!("Bearer {tok}"));
    }
    opts.set_headers(&headers);
    let Ok(req) = web_sys::Request::new_with_str_and_init(&url, &opts) else {
        return;
    };
    let Ok(resp_val) = JsFuture::from(window.fetch_with_request(&req)).await else {
        connected.set(false);
        return;
    };
    let Ok(resp) = resp_val.dyn_into::<web_sys::Response>() else {
        connected.set(false);
        return;
    };
    if !resp.ok() {
        connected.set(false);
        return;
    }
    let Some(body) = resp.body() else {
        connected.set(false);
        return;
    };
    let Ok(reader) = body
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()
    else {
        connected.set(false);
        return;
    };
    let Ok(decoder) = web_sys::TextDecoder::new() else {
        connected.set(false);
        return;
    };
    connected.set(true);
    let mut parser = SseParser::new();
    loop {
        if signal.aborted() {
            break;
        }
        let Ok(chunk) = JsFuture::from(reader.read()).await else {
            break;
        };
        let done = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("done"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if done {
            break;
        }
        if let Ok(val) = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("value"))
            && let Ok(arr) = val.dyn_into::<js_sys::Uint8Array>()
        {
            let bytes = arr.to_vec();
            if let Ok(text) = decoder.decode_with_u8_array(&bytes) {
                for frame in parser.push(&text) {
                    if !paused.get_untracked() {
                        on_frame(frame);
                    }
                }
            }
        }
    }
    let _ = reader.cancel();
    connected.set(false);
}

type LogRow = (String, String, String, String);
type EventRow = (String, String, String);

#[component]
pub fn LogsPage() -> impl IntoView {
    let state = use_app_state();
    let entries = RwSignal::new(Vec::<LogRow>::new());
    let level = RwSignal::new(String::new());
    let filter = RwSignal::new(String::new());
    let paused = RwSignal::new(false);
    let connected = RwSignal::new(false);

    #[cfg(target_arch = "wasm32")]
    let active_abort = StoredValue::new_local(None::<web_sys::AbortController>);
    #[cfg(target_arch = "wasm32")]
    let pending_logs: Rc<RefCell<VecDeque<LogRow>>> =
        Rc::new(RefCell::new(VecDeque::with_capacity(64)));

    #[cfg(target_arch = "wasm32")]
    {
        let pending_flush = pending_logs.clone();
        let timer_id = web_sys::window().and_then(|w| {
            let cb = Closure::wrap(Box::new(move || {
                let mut q = pending_flush.borrow_mut();
                if q.is_empty() {
                    return;
                }
                let batch: Vec<_> = q.drain(..).collect();
                drop(q);
                entries.update(|list| {
                    list.extend(batch);
                    if list.len() > 400 {
                        let drop_n = list.len() - 400;
                        list.drain(0..drop_n);
                    }
                });
            }) as Box<dyn FnMut()>);
            let id = w
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    220,
                )
                .ok();
            cb.forget();
            id
        });
        on_cleanup(move || {
            active_abort.update_value(|slot| {
                if let Some(ctrl) = slot.take() {
                    ctrl.abort();
                }
            });
            if let (Some(w), Some(id)) = (web_sys::window(), timer_id) {
                w.clear_interval_with_handle(id);
            }
        });
    }

    Effect::new(move |_| {
        let lvl = level.get();
        let api = state.client();
        let mut path = "/api/v1/logs".to_string();
        if !lvl.is_empty() {
            path.push_str(&format!("?level={lvl}"));
        }
        let url = api.endpoint(&path);
        let tok = api.token.clone();
        #[cfg(target_arch = "wasm32")]
        {
            active_abort.update_value(|slot| {
                if let Some(prev) = slot.take() {
                    prev.abort();
                }
            });
            if let Ok(ctrl) = web_sys::AbortController::new() {
                let sig = ctrl.signal();
                active_abort.set_value(Some(ctrl));
                let queue = pending_logs.clone();
                spawn_local(async move {
                    stream_sse(url, tok, sig, paused, connected, move |frame| {
                        if frame.event == "log"
                            && let Ok(val) = serde_json::from_str::<Value>(&frame.data)
                        {
                            let ts = json_str(&val, &["ts", "observed_at", "timestamp"])
                                .and_then(|s| s.split('T').nth(1))
                                .map(|s| s.trim_end_matches('Z').chars().take(12).collect::<String>())
                                .unwrap_or_default();
                            let lv = val.get("level").and_then(Value::as_str).unwrap_or("info").to_string();
                            let tgt = val.get("target").and_then(Value::as_str).unwrap_or("").to_string();
                            let msg = val.get("message").and_then(Value::as_str).unwrap_or(&frame.data).to_string();
                            let mut q = queue.borrow_mut();
                            if q.len() >= 256 {
                                q.pop_front();
                            }
                            q.push_back((ts, lv, tgt, msg));
                        }
                    })
                    .await;
                });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (url, tok);
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-cluster">
                    <select class="rp-input" prop:value=move || level.get() on:change=move |ev| level.set(event_target_value(&ev))>
                        <option value="">"ALL"</option>
                        <option value="error">"ERROR"</option>
                        <option value="warn">"WARN"</option>
                        <option value="info">"INFO"</option>
                        <option value="debug">"DEBUG"</option>
                    </select>
                    <input
                        class="rp-input"
                        type="search"
                        placeholder=move || state.t("ui.search")
                        prop:value=move || filter.get()
                        on:input=move |ev| filter.set(event_target_value(&ev))
                    />
                    <span class=move || if connected.get() { "rp-light ok" } else { "rp-light muted" }>
                        <span class="rp-dot"></span>
                        {move || if connected.get() { state.t("act.live") } else { state.t("ov.loading") }}
                    </span>
                </div>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| paused.update(|p| *p = !*p)>
                        {move || if paused.get() { state.t("ui.resume") } else { state.t("ui.pause") }}
                    </button>
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| entries.set(Vec::new())>
                        {move || state.t("ui.clear")}
                    </button>
                </div>
            </div>
            <section class="rp-card" style="padding:14px 16px;">
                {move || {
                    let q = filter.get().to_lowercase();
                    let list: Vec<_> = entries
                        .get()
                        .into_iter()
                        .filter(|(_, l, t, m)| q.is_empty() || l.to_lowercase().contains(&q) || t.to_lowercase().contains(&q) || m.to_lowercase().contains(&q))
                        .collect();
                    if list.is_empty() {
                        return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                    }
                    view! {
                        <div style="display:flex;flex-direction:column;gap:8px;font-family:monospace;font-size:12px;">
                            {list.into_iter().rev().take(150).map(|(ts, l, t, m)| {
                                let badge_style = match l.as_str() {
                                    "error" => "background:rgba(235,111,146,0.20);color:var(--rp-love);",
                                    "warn" => "background:rgba(246,193,119,0.20);color:var(--rp-gold);",
                                    _ => "",
                                };
                                view! {
                                    <div style="padding:8px 0;border-bottom:0.5px solid var(--rp-hl-med);display:flex;flex-direction:column;gap:4px;min-width:0;flex-shrink:0;">
                                        <div class="rp-between" style="gap:6px;flex-wrap:wrap;">
                                            <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                                                <span class="rp-badge" style=badge_style>{l.to_uppercase()}</span>
                                                {(!t.is_empty()).then(|| view! { <span class="rp-label">{t}</span> })}
                                            </div>
                                            {(!ts.is_empty()).then(|| view! { <span class="rp-label" style="font-size:11px;font-variant-numeric:tabular-nums;">{ts}</span> })}
                                        </div>
                                        <div style="overflow-wrap:anywhere;word-break:break-word;line-height:1.45;color:var(--rp-text);">{m}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>
        </div>
    }
}

#[component]
pub fn EventsPage() -> impl IntoView {
    let state = use_app_state();
    let events = RwSignal::new(Vec::<(String, String, String)>::new());
    let event_filter = RwSignal::new(String::new());
    let paused = RwSignal::new(false);
    let connected = RwSignal::new(false);

    #[cfg(target_arch = "wasm32")]
    let active_abort = StoredValue::new_local(None::<web_sys::AbortController>);
    #[cfg(target_arch = "wasm32")]
    let pending_events: Rc<RefCell<VecDeque<EventRow>>> =
        Rc::new(RefCell::new(VecDeque::with_capacity(64)));

    #[cfg(target_arch = "wasm32")]
    {
        let pending_flush = pending_events.clone();
        let timer_id = web_sys::window().and_then(|w| {
            let cb = Closure::wrap(Box::new(move || {
                let mut q = pending_flush.borrow_mut();
                if q.is_empty() {
                    return;
                }
                let batch: Vec<_> = q.drain(..).collect();
                drop(q);
                events.update(|list| {
                    list.extend(batch);
                    if list.len() > 200 {
                        let drop_n = list.len() - 200;
                        list.drain(0..drop_n);
                    }
                });
            }) as Box<dyn FnMut()>);
            let id = w
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    220,
                )
                .ok();
            cb.forget();
            id
        });
        on_cleanup(move || {
            active_abort.update_value(|slot| {
                if let Some(ctrl) = slot.take() {
                    ctrl.abort();
                }
            });
            if let (Some(w), Some(id)) = (web_sys::window(), timer_id) {
                w.clear_interval_with_handle(id);
            }
        });
    }

    Effect::new(move |_| {
        let api = state.client();
        let url = api.endpoint("/api/v1/events");
        let tok = api.token.clone();
        #[cfg(target_arch = "wasm32")]
        {
            active_abort.update_value(|slot| {
                if let Some(prev) = slot.take() {
                    prev.abort();
                }
            });
            if let Ok(ctrl) = web_sys::AbortController::new() {
                let sig = ctrl.signal();
                active_abort.set_value(Some(ctrl));
                let queue = pending_events.clone();
                spawn_local(async move {
                    stream_sse(url, tok, sig, paused, connected, move |frame| {
                        let mut q = queue.borrow_mut();
                        if q.len() >= 180 {
                            q.pop_front();
                        }
                        q.push_back((frame.id.unwrap_or_default(), frame.event, frame.data));
                    })
                    .await;
                });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (url, tok);
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-cluster" style="flex-wrap:wrap;gap:8px;">
                    <span class=move || if connected.get() { "rp-light ok" } else { "rp-light muted" }>
                        <span class="rp-dot"></span>
                        {move || if connected.get() { state.t("act.live") } else { state.t("ov.loading") }}
                    </span>
                    <span class="rp-badge">{move || format!("{} 条", events.get().len())}</span>
                    <input
                        class="rp-input"
                        type="search"
                        placeholder="筛选事件类型 / ID…"
                        prop:value=move || event_filter.get()
                        on:input=move |ev| event_filter.set(event_target_value(&ev))
                        style="max-width:220px;"
                    />
                </div>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| paused.update(|p| *p = !*p)>
                        {move || if paused.get() { state.t("ui.resume") } else { state.t("ui.pause") }}
                    </button>
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| events.set(Vec::new())>
                        {move || state.t("ui.clear")}
                    </button>
                </div>
            </div>
            <section class="rp-card" style="padding:14px 16px;">
                {move || {
                    let q = event_filter.get().trim().to_lowercase();
                    let list: Vec<_> = events
                        .get()
                        .into_iter()
                        .filter(|(_, kind, data)| q.is_empty() || kind.to_lowercase().contains(&q) || data.to_lowercase().contains(&q))
                        .collect();
                    if list.is_empty() {
                        return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                    }
                    view! {
                        <div style="display:flex;flex-direction:column;">
                            {list.into_iter().rev().take(120).enumerate().map(|(idx, (_id, kind, data))| {
                                let (time_str, summary, flow_id) = doona_common::format::format_sse_event_row(&kind, &data);
                                let border = if idx > 0 { "border-top:0.5px solid var(--rp-hl-med);" } else { "" };
                                let row_style = format!("padding:10px 0;{border}display:flex;flex-direction:column;gap:5px;min-width:0;flex-shrink:0;");
                                let badge_tone = if kind.ends_with(".closed") {
                                    "background:color-mix(in srgb,var(--rp-subtle) 18%,transparent);color:var(--rp-text);"
                                } else if kind == "stream.ready" {
                                    "background:color-mix(in srgb,var(--rp-foam) 22%,transparent);color:var(--rp-text);"
                                } else {
                                    "background:color-mix(in srgb,var(--rp-accent) 18%,transparent);color:var(--rp-text);"
                                };
                                view! {
                                    <div style=row_style>
                                        <div class="rp-between" style="gap:8px;align-items:center;">
                                            <span class="rp-badge" style=badge_tone>{kind}</span>
                                            <span class="rp-label" style="font-family:monospace;font-size:11px;font-variant-numeric:tabular-nums;">{time_str}</span>
                                        </div>
                                        <div class="rp-between" style="gap:8px;align-items:center;">
                                            <span style="font-size:13px;color:var(--rp-text);overflow-wrap:anywhere;word-break:break-word;flex:1;min-width:0;">
                                                {summary}
                                            </span>
                                            {flow_id.map(|fid| {
                                                let href = format!("#flows?flow={fid}");
                                                view! {
                                                    <a class="rp-btn quiet sm" href=href style="flex:none;font-size:12px;">"查看流 →"</a>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>
        </div>
    }
}
