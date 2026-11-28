//! Traffic Hub pages: Connections, Flows, DNS (Query / Cache / Log / Rules), Live Logs SSE, Live Events SSE.

use doona_common::format::format_bytes;
use doona_common::model::{ConnectionItem, value_u64};
#[cfg(target_arch = "wasm32")]
use doona_common::sse::SseParser;
use leptos::prelude::*;
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

use crate::state::use_app_state;

#[component]
pub fn ConnectionsPage() -> impl IntoView {
    let state = use_app_state();
    let filter = RwSignal::new(String::new());
    let rows = RwSignal::new(Vec::<(String, ConnectionItem)>::new());
    let totals = RwSignal::new((0u64, 0u64));
    let loading = RwSignal::new(true);

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

    let close_all = move |_| {
        let api = state.client();
        spawn_local(async move {
            match api.close_all_connections().await {
                Ok(_) => {
                    state.toast("positive", state.t("conn.closedAll"), None);
                    load();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-cluster">
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
                    <button type="button" class="rp-btn negative sm" on:click=close_all>
                        {move || state.t("conn.closeAll")}
                    </button>
                </div>
            </div>

            <section class="rp-card" style="overflow:auto;">
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
                            let hay = format!(
                                "{} {} {} {} {} {}",
                                proto,
                                c.domain.as_deref().unwrap_or(""),
                                c.dst.as_deref().unwrap_or(""),
                                c.outbound.as_deref().unwrap_or(""),
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
                                    <th style="text-align:left;padding:6px 8px;">"Proto"</th>
                                    <th style="text-align:left;padding:6px 8px;">{state.t("conn.target")}</th>
                                    <th style="text-align:left;padding:6px 8px;">{state.t("conn.outbound")}</th>
                                    <th style="text-align:left;padding:6px 8px;">{state.t("conn.process")}</th>
                                    <th style="text-align:right;padding:6px 8px;">{state.t("ui.download")}</th>
                                    <th style="text-align:right;padding:6px 8px;">{state.t("ui.upload")}</th>
                                    <th style="padding:6px 8px;"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().take(200).map(|(proto, c)| {
                                    let id = c.id.clone();
                                    let target = c.domain.clone().or(c.dst.clone()).unwrap_or_else(|| "—".into());
                                    let out = if !c.chain.is_empty() {
                                        c.chain.join(" → ")
                                    } else {
                                        c.outbound.clone().unwrap_or_else(|| "—".into())
                                    };
                                    let dl = value_u64(&c.download_bytes).map(format_bytes).unwrap_or_else(|| "—".into());
                                    let ul = value_u64(&c.upload_bytes).map(format_bytes).unwrap_or_else(|| "—".into());
                                    view! {
                                        <tr>
                                            <td style="padding:6px 8px;"><span class="rp-badge">{proto}</span></td>
                                            <td style="padding:6px 8px;"><code>{target}</code></td>
                                            <td style="padding:6px 8px;">{out}</td>
                                            <td style="padding:6px 8px;">{c.pname.unwrap_or_else(|| "—".into())}</td>
                                            <td style="text-align:right;padding:6px 8px;">{dl}</td>
                                            <td style="text-align:right;padding:6px 8px;">{ul}</td>
                                            <td style="padding:6px 8px;">
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

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between">
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
            <section class="rp-card" style="overflow:auto;">
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
                                    <th style="text-align:left;padding:6px 8px;">"ID"</th>
                                    <th style="text-align:left;padding:6px 8px;">"Scope"</th>
                                    <th style="text-align:left;padding:6px 8px;">"Target"</th>
                                    <th style="text-align:left;padding:6px 8px;">"Outbound"</th>
                                    <th style="text-align:left;padding:6px 8px;">"State"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {items.into_iter().take(100).map(|f| {
                                    let fid = f.get("id").and_then(Value::as_str).unwrap_or("").to_string();
                                    let short = fid.chars().take(10).collect::<String>();
                                    let scope = f.get("scope").and_then(Value::as_str).unwrap_or("—").to_string();
                                    let target = f.get("domain").or_else(|| f.get("dst")).and_then(Value::as_str).unwrap_or("—").to_string();
                                    let out = f.get("outbound").and_then(Value::as_str).unwrap_or("—").to_string();
                                    let st = f.get("state").and_then(Value::as_str).unwrap_or("—").to_string();
                                    let fid_click = fid.clone();
                                    view! {
                                        <tr style="cursor:pointer;" on:click=move |_| {
                                            let api = state.client();
                                            let id = fid_click.clone();
                                            spawn_local(async move {
                                                if let Ok(det) = api.flow_detail(&id).await {
                                                    selected.set(Some(det));
                                                }
                                            });
                                        }>
                                            <td style="padding:6px 8px;"><code>{short}</code></td>
                                            <td style="padding:6px 8px;">{scope}</td>
                                            <td style="padding:6px 8px;"><code>{target}</code></td>
                                            <td style="padding:6px 8px;">{out}</td>
                                            <td style="padding:6px 8px;"><span class="rp-badge">{st}</span></td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </section>
            {move || selected.get().map(|det| view! {
                <section class="rp-card">
                    <div class="rp-between" style="margin-bottom:8px;">
                        <h3 class="rp-h3">"Flow Detail"</h3>
                        <button type="button" class="rp-btn quiet sm" on:click=move |_| selected.set(None)>"×"</button>
                    </div>
                    <pre class="rp-limit-snippet" style="overflow:auto;max-height:300px;">{serde_json::to_string_pretty(&det).unwrap_or_default()}</pre>
                </section>
            })}
        </div>
    }
}

#[component]
pub fn DnsPage() -> impl IntoView {
    let state = use_app_state();
    let tab = RwSignal::new("query".to_string());
    let domain = RwSignal::new("example.com".to_string());
    let qtype = RwSignal::new("A".to_string());
    let query_result = RwSignal::new(None::<Value>);
    let query_busy = RwSignal::new(false);
    let cache_items = RwSignal::new(Vec::<Value>::new());
    let log_items = RwSignal::new(Vec::<Value>::new());

    let load_dns = move || {
        let api = state.client();
        spawn_local(async move {
            if let Ok(raw) = api.dns_cache().await {
                let list = raw
                    .get("entries")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
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
        if d.is_empty() {
            return;
        }
        query_busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.dns_query(&d, &[t.as_str()]).await {
                Ok(val) => query_result.set(Some(val)),
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            query_busy.set(false);
        });
    };

    let flush_cache = move |_| {
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
    };

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
                            on:click=move |_| tab.set(id.to_string())
                        >
                            {move || state.t(key)}
                        </button>
                    }).collect_view()}
                </div>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| load_dns()>{move || state.t("ui.refresh")}</button>
                    <button type="button" class="rp-btn negative sm" on:click=flush_cache>{move || state.t("dns.flush")}</button>
                </div>
            </div>

            {move || match tab.get().as_str() {
                "cache" => view! {
                    <section class="rp-card" style="overflow:auto;">
                        <pre class="rp-limit-snippet" style="max-height:480px;overflow:auto;">
                            {serde_json::to_string_pretty(&cache_items.get()).unwrap_or_default()}
                        </pre>
                    </section>
                }.into_any(),
                "log" => view! {
                    <section class="rp-card" style="overflow:auto;">
                        <pre class="rp-limit-snippet" style="max-height:480px;overflow:auto;">
                            {serde_json::to_string_pretty(&log_items.get()).unwrap_or_default()}
                        </pre>
                    </section>
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
                                style="flex:1;min-width:200px;"
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
                            <button type="submit" class="rp-btn accent" disabled=move || query_busy.get()>
                                {move || state.t("dns.run")}
                            </button>
                        </form>
                        {move || query_result.get().map(|res| view! {
                            <pre class="rp-limit-snippet" style="overflow:auto;max-height:420px;">{serde_json::to_string_pretty(&res).unwrap_or_default()}</pre>
                        })}
                    </section>
                }.into_any(),
            }}
        </div>
    }
}

/// Shared pure-Rust SSE streaming helper using browser `fetch()` + `ReadableStreamDefaultReader` + `SseParser`.
#[cfg(target_arch = "wasm32")]
async fn stream_sse(
    url: String,
    token: Option<String>,
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
        return;
    };
    let Ok(resp) = resp_val.dyn_into::<web_sys::Response>() else {
        return;
    };
    if !resp.ok() {
        return;
    }
    let Some(body) = resp.body() else {
        return;
    };
    let Ok(reader) = body
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()
    else {
        return;
    };
    let Ok(decoder) = web_sys::TextDecoder::new() else {
        return;
    };
    connected.set(true);
    let mut parser = SseParser::new();
    loop {
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
    connected.set(false);
}

#[component]
pub fn LogsPage() -> impl IntoView {
    let state = use_app_state();
    let entries = RwSignal::new(Vec::<(String, String, String)>::new());
    let level = RwSignal::new(String::new());
    let filter = RwSignal::new(String::new());
    let paused = RwSignal::new(false);
    let connected = RwSignal::new(false);

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
        spawn_local(async move {
            stream_sse(url, tok, paused, connected, move |frame| {
                if frame.event == "log"
                    && let Ok(val) = serde_json::from_str::<Value>(&frame.data)
                {
                    let lv = val.get("level").and_then(Value::as_str).unwrap_or("info").to_string();
                    let tgt = val.get("target").and_then(Value::as_str).unwrap_or("").to_string();
                    let msg = val.get("message").and_then(Value::as_str).unwrap_or(&frame.data).to_string();
                    entries.update(|list| {
                        list.push((lv, tgt, msg));
                        if list.len() > 500 {
                            list.remove(0);
                        }
                    });
                }
            })
            .await;
        });
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
            <section class="rp-card" style="font-family:monospace;font-size:12px;max-height:65vh;overflow:auto;">
                {move || {
                    let q = filter.get().to_lowercase();
                    let list: Vec<_> = entries
                        .get()
                        .into_iter()
                        .filter(|(l, t, m)| q.is_empty() || l.to_lowercase().contains(&q) || t.to_lowercase().contains(&q) || m.to_lowercase().contains(&q))
                        .collect();
                    if list.is_empty() {
                        return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                    }
                    list.into_iter().rev().map(|(l, t, m)| view! {
                        <div style="padding:3px 0;border-bottom:1px solid var(--rp-line);">
                            <span class="rp-badge" style="margin-right:6px;">{l}</span>
                            <span class="rp-label" style="margin-right:6px;">{t}</span>
                            <span>{m}</span>
                        </div>
                    }).collect_view().into_any()
                }}
            </section>
        </div>
    }
}

#[component]
pub fn EventsPage() -> impl IntoView {
    let state = use_app_state();
    let events = RwSignal::new(Vec::<(String, String, String)>::new());
    let paused = RwSignal::new(false);
    let connected = RwSignal::new(false);

    Effect::new(move |_| {
        let api = state.client();
        let url = api.endpoint("/api/v1/events");
        let tok = api.token.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            stream_sse(url, tok, paused, connected, move |frame| {
                events.update(|list| {
                    list.push((frame.id.unwrap_or_default(), frame.event, frame.data));
                    if list.len() > 200 {
                        list.remove(0);
                    }
                });
            })
            .await;
        });
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (url, tok);
    });

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between">
                <span class=move || if connected.get() { "rp-light ok" } else { "rp-light muted" }>
                    <span class="rp-dot"></span>
                    {move || if connected.get() { state.t("act.live") } else { state.t("ov.loading") }}
                </span>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| paused.update(|p| *p = !*p)>
                        {move || if paused.get() { state.t("ui.resume") } else { state.t("ui.pause") }}
                    </button>
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| events.set(Vec::new())>
                        {move || state.t("ui.clear")}
                    </button>
                </div>
            </div>
            <section class="rp-card" style="font-family:monospace;font-size:12px;max-height:65vh;overflow:auto;">
                {move || {
                    let list = events.get();
                    if list.is_empty() {
                        return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                    }
                    list.into_iter().rev().map(|(id, kind, data)| view! {
                        <div style="padding:4px 0;border-bottom:1px solid var(--rp-line);">
                            <span class="rp-badge" style="margin-right:6px;">{kind}</span>
                            <span class="rp-label" style="margin-right:6px;">{id}</span>
                            <span>{data}</span>
                        </div>
                    }).collect_view().into_any()
                }}
            </section>
        </div>
    }
}
