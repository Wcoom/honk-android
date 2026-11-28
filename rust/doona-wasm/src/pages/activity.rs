//! Activity Hub page: Outbound Mode switcher, Node Policy selector (via gated `policytarget-rs`),
//! live download/upload/connections/CPU/memory sparkline tiles, traffic history chart, and outbounds.

use doona_common::format::{format_bytes, format_latency, format_rate};
use doona_common::model::GroupDetail;
use leptos::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen_futures::spawn_local;

use crate::mode_dae::{OutboundMode, read_mode, read_rule_fallback, write_mode, write_rule_fallback};
use crate::state::use_app_state;
use crate::ui::{DualAreaSvg, SparkSvg, StatusLight};

#[component]
pub fn ActivityPage() -> impl IntoView {
    let state = use_app_state();
    let hub_detail = RwSignal::new(None::<GroupDetail>);
    let staged_mode = RwSignal::new(None::<String>);
    let staged_target = RwSignal::new(None::<String>);
    let mode_busy = RwSignal::new(false);
    let confirm_mode = RwSignal::new(None::<String>);
    let history_window = RwSignal::new(600u64);
    let history_down = RwSignal::new(Vec::<f64>::new());
    let history_up = RwSignal::new(Vec::<f64>::new());
    let outbounds = RwSignal::new(Vec::<(String, u64, u64)>::new());

    // Fetch hub group detail whenever groups or main config change.
    Effect::new(move |_| {
        let groups = state.groups.get();
        let cfg = state.config.get();
        let main_content = cfg
            .as_ref()
            .and_then(|c| {
                c.sources
                    .iter()
                    .find(|s| s.kind.as_deref() == Some("main"))
                    .or_else(|| c.sources.first())
            })
            .and_then(|s| s.content.as_deref())
            .unwrap_or("");
        let fallback = read_rule_fallback(main_content);
        let groups_vec: Vec<Value> = groups
            .iter()
            .filter_map(|g| serde_json::to_value(g).ok())
            .collect();
        let found = doona_logic::policy_target::find_policy_hub(groups_vec, fallback)
            .ok()
            .flatten()
            .and_then(|v| v.get("id").and_then(Value::as_str).map(ToString::to_string));
        if let Some(hub_id) = found {
            let api = state.client();
            spawn_local(async move {
                if let Ok(detail) = api.group(&hub_id).await {
                    hub_detail.set(Some(detail));
                }
            });
        } else {
            hub_detail.set(None);
        }
    });

    // Fetch traffic history & outbounds on load / window change / refresh.
    Effect::new(move |_| {
        let _nonce = state.reload_nonce.get();
        let win = history_window.get();
        let api = state.client();
        spawn_local(async move {
            if let Ok(hist) = api.traffic_history(win, 120).await {
                let mut d = Vec::with_capacity(hist.samples.len());
                let mut u = Vec::with_capacity(hist.samples.len());
                for pt in &hist.samples {
                    d.push(doona_common::model::value_f64(&pt.download_bytes_per_second).unwrap_or(0.0));
                    u.push(doona_common::model::value_f64(&pt.upload_bytes_per_second).unwrap_or(0.0));
                }
                history_down.set(d);
                history_up.set(u);
            }
            if let Ok(raw) = api.runtime_outbounds().await {
                let mut rows = Vec::new();
                if let Some(arr) = raw
                    .get("outbounds")
                    .or_else(|| raw.get("items"))
                    .and_then(Value::as_array)
                {
                    for item in arr {
                        let name = item
                            .get("name")
                            .or_else(|| item.get("outbound"))
                            .and_then(Value::as_str)
                            .unwrap_or("—")
                            .to_string();
                        let dl = doona_common::model::value_u64(&item.get("download_bytes").cloned())
                            .unwrap_or(0);
                        let ul = doona_common::model::value_u64(&item.get("upload_bytes").cloned())
                            .unwrap_or(0);
                        rows.push((name, dl, ul));
                    }
                }
                rows.sort_by(|a, b| b.1.cmp(&a.1));
                outbounds.set(rows);
            }
        });
    });

    // Derived current outbound mode & policy target state via gated `doona_logic::policy_target`.
    let mode_model = Memo::new(move |_| {
        let cfg = state.config.get();
        let groups = state.groups.get();
        let detail = hub_detail.get();
        let saved_target = state.saved_policy_target.get();
        let main_src = cfg.as_ref().and_then(|c| {
            c.sources
                .iter()
                .find(|s| s.kind.as_deref() == Some("main"))
                .or_else(|| c.sources.first())
        });
        let content = main_src.and_then(|s| s.content.as_deref()).unwrap_or("");
        let current = read_mode(content);
        let fallback = read_rule_fallback(content);

        let groups_vec: Vec<Value> = groups
            .iter()
            .filter_map(|g| serde_json::to_value(g).ok())
            .collect();
        let groups_val = Value::Array(groups_vec.clone());
        let hub_val = doona_logic::policy_target::find_policy_hub(groups_vec, fallback.clone())
            .ok()
            .flatten()
            .unwrap_or(Value::Null);
        let current_val = match &current {
            OutboundMode::Rule => json!({ "mode": "rule" }),
            OutboundMode::Direct => json!({ "mode": "direct" }),
            OutboundMode::Global(t) => json!({ "mode": "global", "target": t }),
        };
        let detail_val = detail
            .as_ref()
            .and_then(|d| serde_json::to_value(d).ok())
            .unwrap_or(Value::Null);
        let resolved = doona_logic::policy_target::resolve_policy_state(json!({
            "current": current_val,
            "groups": groups_val,
            "hub": hub_val,
            "hubDetail": detail_val,
            "ruleFallback": fallback,
            "savedTarget": saved_target,
        }))
        .unwrap_or_else(|_| json!({}));

        let uses_hub = resolved.get("usesHub").and_then(Value::as_bool).unwrap_or(false);
        let active_target = resolved
            .get("activeTarget")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let options: Vec<(String, String)> = resolved
            .get("options")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|o| {
                        let id = o.get("id").and_then(Value::as_str)?;
                        let label = o.get("label").and_then(Value::as_str).unwrap_or(id);
                        Some((id.to_string(), label.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let hub_id = hub_val
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let can_select_hub = uses_hub
            && detail.as_ref().is_some_and(|d| d.capabilities.can_select);
        let main_writable = main_src.and_then(|s| s.writable).unwrap_or(false);
        let main_id = main_src.map(|s| s.id.clone());

        (
            current,
            fallback,
            active_target,
            options,
            uses_hub,
            hub_id,
            can_select_hub,
            main_writable,
            main_id,
        )
    });

    let apply_mode_now = move |next_mode: String| {
        let (_, _, active_target, _, uses_hub, hub_id, can_select_hub, _, main_id) =
            mode_model.get_untracked();
        let Some(source_id) = main_id else {
            state.toast("negative", state.t("act.modeNoRouting"), None);
            return;
        };
        let target_val = staged_target.get_untracked().unwrap_or(active_target);
        let submitted = match next_mode.as_str() {
            "direct" => OutboundMode::Direct,
            "global" => {
                if target_val.is_empty() {
                    state.toast("negative", state.t("act.globalMissing"), None);
                    return;
                }
                OutboundMode::Global(target_val.clone())
            }
            _ => OutboundMode::Rule,
        };
        mode_busy.set(true);
        let api = state.client();
        let detail_now = hub_detail.get_untracked();
        spawn_local(async move {
            match api.config_source(&source_id).await {
                Ok((src, etag)) => {
                    let old_content = src.content.unwrap_or_default();
                    let if_match = etag.unwrap_or_else(|| {
                        let sha = src.content_sha256.or(src.sha256).unwrap_or_default();
                        format!("\"{sha}\"")
                    });
                    match write_mode(&old_content, &submitted) {
                        Ok(updated) => match api.replace_config_source(&source_id, &updated, &if_match).await {
                            Ok(_) => {
                                if let OutboundMode::Global(ref tgt) = submitted {
                                    state.set_saved_policy_target(tgt);
                                    if uses_hub
                                        && can_select_hub
                                        && let (Some(hid), Some(det)) = (hub_id, detail_now)
                                        && let Some(member) = det.members.iter().find(|m| m.name == *tgt)
                                    {
                                        let _ = api.select_group(&hid, &member.id, "both").await;
                                    }
                                }
                                staged_mode.set(None);
                                let mode_lbl = state.t(match submitted {
                                    OutboundMode::Rule => "mode.rule",
                                    OutboundMode::Direct => "mode.direct",
                                    OutboundMode::Global(_) => "mode.global",
                                });
                                state.toast(
                                    "positive",
                                    state.t_args("act.modeApplied", &[("mode", &mode_lbl)]),
                                    None,
                                );
                                state.refresh_all();
                            }
                            Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                        },
                        Err(key) => state.toast("negative", state.t(&key), None),
                    }
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            mode_busy.set(false);
        });
    };

    let apply_target_now = move |_| {
        let (current, fallback, active_target, _, uses_hub, hub_id, can_select_hub, _, main_id) =
            mode_model.get_untracked();
        let Some(target) = staged_target.get_untracked() else {
            return;
        };
        if target.is_empty() || target == active_target {
            return;
        }
        mode_busy.set(true);
        let api = state.client();
        let detail_now = hub_detail.get_untracked();
        let mode_label = state.t(match current {
            OutboundMode::Rule => "mode.rule",
            OutboundMode::Direct => "mode.direct",
            OutboundMode::Global(_) => "mode.global",
        });

        spawn_local(async move {
            if matches!(current, OutboundMode::Global(_)) {
                if let Some(source_id) = main_id {
                    match api.config_source(&source_id).await {
                        Ok((src, etag)) => {
                            let old = src.content.unwrap_or_default();
                            let if_match = etag.unwrap_or_else(|| {
                                format!("\"{}\"", src.content_sha256.or(src.sha256).unwrap_or_default())
                            });
                            if let Ok(updated) = write_mode(&old, &OutboundMode::Global(target.clone())) {
                                match api.replace_config_source(&source_id, &updated, &if_match).await {
                                    Ok(_) => {
                                        if uses_hub
                                            && can_select_hub
                                            && let (Some(hid), Some(det)) = (hub_id, detail_now)
                                            && let Some(m) = det.members.iter().find(|m| m.name == target)
                                        {
                                            let _ = api.select_group(&hid, &m.id, "both").await;
                                        }
                                        state.set_saved_policy_target(&target);
                                        staged_target.set(None);
                                        state.toast(
                                            "positive",
                                            state.t_args("act.globalApplied", &[("target", &target)]),
                                            None,
                                        );
                                        state.refresh_all();
                                    }
                                    Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                                }
                            }
                        }
                        Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                    }
                }
            } else if uses_hub && can_select_hub {
                if let (Some(hid), Some(det)) = (hub_id, detail_now)
                    && let Some(member) = det.members.iter().find(|m| m.name == target)
                {
                    match api.select_group(&hid, &member.id, "both").await {
                        Ok(_) => {
                            state.set_saved_policy_target(&target);
                            staged_target.set(None);
                            if let Ok(refreshed) = api.group(&hid).await {
                                hub_detail.set(Some(refreshed));
                            }
                            if let Ok(gs) = api.groups().await {
                                state.groups.set(gs);
                            }
                            state.toast(
                                "positive",
                                state.t_args(
                                    "act.globalRuleApplied",
                                    &[("target", &target), ("mode", &mode_label)],
                                ),
                                None,
                            );
                        }
                        Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                    }
                }
            } else if !fallback.is_empty()
                && let Some(source_id) = main_id
            {
                match api.config_source(&source_id).await {
                    Ok((src, etag)) => {
                        let old = src.content.unwrap_or_default();
                        let if_match = etag.unwrap_or_else(|| {
                            format!("\"{}\"", src.content_sha256.or(src.sha256).unwrap_or_default())
                        });
                        if let Ok(updated) = write_rule_fallback(&old, &target) {
                            match api.replace_config_source(&source_id, &updated, &if_match).await {
                                Ok(_) => {
                                    state.set_saved_policy_target(&target);
                                    staged_target.set(None);
                                    state.toast(
                                        "positive",
                                        state.t_args(
                                            "act.globalRuleApplied",
                                            &[("target", &target), ("mode", &mode_label)],
                                        ),
                                        None,
                                    );
                                    state.refresh_all();
                                }
                                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                            }
                        }
                    }
                    Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                }
            } else {
                state.set_saved_policy_target(&target);
                staged_target.set(None);
                state.toast(
                    "positive",
                    state.t_args("act.globalApplied", &[("target", &target)]),
                    None,
                );
            }
            mode_busy.set(false);
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:14px;">
            // 1. Outbound mode card (规则 / 直连 / 全局)
            <section class="rp-card rp-control-card" aria-label=move || state.t("act.mode")>
                <div class="rp-row rp-between" style="flex-wrap:wrap;gap:10px;">
                    <span class="rp-qlabel"><strong>{move || state.t("act.mode")}</strong></span>
                    <span class="rp-cluster">
                        <div class="rp-seg" data-size="M" role="group">
                            {["rule", "direct", "global"].into_iter().map(|m| {
                                let m_str = m.to_string();
                                let label_key = match m {
                                    "rule" => "mode.rule",
                                    "direct" => "mode.direct",
                                    _ => "mode.global",
                                };
                                view! {
                                    <button
                                        type="button"
                                        class="rp-btn"
                                        data-size="M"
                                        data-selected=move || {
                                            let cur = staged_mode
                                                .get()
                                                .unwrap_or_else(|| mode_model.get().0.as_str().to_string());
                                            (cur == m_str).then_some("true")
                                        }
                                        disabled=move || mode_busy.get() || !mode_model.get().7
                                        on:click=move |_| {
                                            let cur = mode_model.get_untracked().0.as_str().to_string();
                                            if m == cur {
                                                staged_mode.set(None);
                                            } else {
                                                staged_mode.set(Some(m.to_string()));
                                            }
                                        }
                                    >
                                        {move || state.t(label_key)}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <button
                            type="button"
                            class="rp-btn accent sm"
                            disabled=move || mode_busy.get() || staged_mode.get().is_none() || !mode_model.get().7
                            on:click=move |_| {
                                if let Some(next) = staged_mode.get_untracked() {
                                    if next == "rule" {
                                        apply_mode_now(next);
                                    } else {
                                        confirm_mode.set(Some(next));
                                    }
                                }
                            }
                        >
                            {move || state.t("config.save")}
                        </button>
                    </span>
                </div>
            </section>

            // 2. Node Policy Selector card (节点策略选择 — preserves rule mode via hub group selection!)
            <section class="rp-card rp-control-card" aria-label=move || state.t("act.global")>
                <div class="rp-row rp-between" style="flex-wrap:wrap;gap:10px;">
                    <span class="rp-qlabel"><strong>{move || state.t("act.global")}</strong></span>
                    <span class="rp-cluster rp-global-controls">
                        <select
                            class="rp-input"
                            aria-label=move || state.t("act.global")
                            disabled=move || {
                                let (_, _, _, _, uses_hub, _, can_sel, main_w, _) = mode_model.get();
                                mode_busy.get() || !(if uses_hub { can_sel } else { main_w })
                            }
                            prop:value=move || {
                                staged_target.get().unwrap_or_else(|| mode_model.get().2)
                            }
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                let active = mode_model.get_untracked().2;
                                if val == active {
                                    staged_target.set(None);
                                } else {
                                    staged_target.set(Some(val));
                                }
                            }
                        >
                            {move || {
                                let opts = mode_model.get().3;
                                if opts.is_empty() {
                                    vec![view! { <option value="">"—"</option> }.into_any()]
                                } else {
                                    opts.into_iter()
                                        .map(|(id, label)| view! { <option value=id>{label}</option> }.into_any())
                                        .collect()
                                }
                            }}
                        </select>
                        <button
                            type="button"
                            class="rp-btn accent sm"
                            disabled=move || mode_busy.get() || staged_target.get().is_none()
                            on:click=apply_target_now
                        >
                            {move || state.t("config.save")}
                        </button>
                    </span>
                </div>
            </section>

            // Confirm dialog for direct/global mode switch
            <Show when=move || confirm_mode.get().is_some()>
                <div class="rp-modal" style="position:fixed;inset:0;z-index:900;background:rgba(0,0,0,0.45);display:flex;align-items:center;justify-content:center;">
                    <div class="rp-dialog rp-card" role="dialog" aria-modal="true" style="width:min(420px,92vw);padding:20px;display:flex;flex-direction:column;gap:12px;">
                        <h3 class="rp-h3">
                            {move || {
                                let m = confirm_mode.get().unwrap_or_default();
                                let lbl = state.t(if m == "direct" { "mode.direct" } else { "mode.global" });
                                state.t_args("act.modeConfirm", &[("mode", &lbl)])
                            }}
                        </h3>
                        <p class="rp-label">
                            {move || {
                                let m = confirm_mode.get().unwrap_or_default();
                                state.t(if m == "direct" { "act.modeDirectHelp" } else { "act.modeGlobalHelp" })
                            }}
                        </p>
                        <div class="rp-cluster" style="justify-content:flex-end;">
                            <button type="button" class="rp-btn secondary sm" on:click=move |_| confirm_mode.set(None)>
                                {move || state.t("ui.cancel")}
                            </button>
                            <button
                                type="button"
                                class="rp-btn accent sm"
                                on:click=move |_| {
                                    if let Some(m) = confirm_mode.get_untracked() {
                                        confirm_mode.set(None);
                                        apply_mode_now(m);
                                    }
                                }
                            >
                                {move || state.t("config.save")}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 3. Engine status bar
            <section class="rp-card rp-control-card">
                <div class="rp-row rp-between">
                    {move || {
                        let st = state
                            .runtime
                            .get()
                            .map(|r| r.state_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        let tone_str = doona_logic::selectors::lifecycle_tone(Some(Value::String(st.clone())))
                            .ok()
                            .and_then(|v| v.as_str().map(ToString::to_string))
                            .unwrap_or_else(|| "muted".to_string());
                        let tone: &'static str = match tone_str.as_str() {
                            "ok" => "ok",
                            "warn" => "warn",
                            "err" => "err",
                            _ => "muted",
                        };
                        let label_key = match st.as_str() {
                            "running" => "ov.running",
                            "degraded" => "ov.v.degraded",
                            "failed" => "ov.failed",
                            "suspended" => "ov.suspended",
                            _ => "ov.unknown",
                        };
                        view! { <StatusLight tone=tone text=state.t(label_key) /> }
                    }}
                    <a class="rp-btn quiet sm" href="#overview">{move || state.t("act.viewDetails")}</a>
                </div>
            </section>

            // 4. Metric tiles grid (Download, Upload, Active connections, CPU, Memory)
            <div class="rp-g3" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px;">
                <a class="rp-card rp-card-link" href="#connections?tab=traffic" style="text-decoration:none;color:inherit;">
                    <div class="rp-tile-head rp-tint-c1">
                        <span class="rp-tile-caption">{move || state.t("ui.download")}</span>
                    </div>
                    <div class="rp-big">{move || format_rate(state.runtime.get().map(|r| r.download_rate()).unwrap_or(0.0))}</div>
                    {move || {
                        let vals: Vec<f64> = state.down_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c1)" /> }
                    }}
                </a>
                <a class="rp-card rp-card-link" href="#connections?tab=traffic" style="text-decoration:none;color:inherit;">
                    <div class="rp-tile-head rp-tint-c4">
                        <span class="rp-tile-caption">{move || state.t("ui.upload")}</span>
                    </div>
                    <div class="rp-big">{move || format_rate(state.runtime.get().map(|r| r.upload_rate()).unwrap_or(0.0))}</div>
                    {move || {
                        let vals: Vec<f64> = state.up_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c4)" /> }
                    }}
                </a>
                <a class="rp-card rp-card-link" href="#connections" style="text-decoration:none;color:inherit;">
                    <div class="rp-tile-head rp-tint-c3">
                        <span class="rp-tile-caption">{move || state.t("act.active")}</span>
                    </div>
                    <div class="rp-big">
                        {move || state.runtime.get().and_then(|r| r.traffic.connections.total).unwrap_or(0).to_string()}
                    </div>
                    {move || {
                        let vals: Vec<f64> = state.conn_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c3)" /> }
                    }}
                </a>
                <a class="rp-card rp-card-link" href="#overview" style="text-decoration:none;color:inherit;">
                    <div class="rp-tile-head rp-tint-c2">
                        <span class="rp-tile-caption">{move || state.t("act.cpu")}</span>
                    </div>
                    <div class="rp-big">
                        {move || match state.runtime.get().and_then(|r| r.process.cpu_percent) {
                            Some(v) => format!("{v:.1}%"),
                            None => "—".to_string(),
                        }}
                    </div>
                    {move || {
                        let vals: Vec<f64> = state.cpu_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c2)" /> }
                    }}
                </a>
                <a class="rp-card rp-card-link" href="#overview" style="text-decoration:none;color:inherit;">
                    <div class="rp-tile-head rp-tint-c5">
                        <span class="rp-tile-caption">{move || state.t("act.memory")}</span>
                    </div>
                    <div class="rp-big">
                        {move || match state.memory.get().and_then(|m| m.rss().or_else(|| m.cgroup_current())) {
                            Some(b) => format_bytes(b),
                            None => "—".to_string(),
                        }}
                    </div>
                    {move || {
                        let vals: Vec<f64> = state.rss_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c5)" /> }
                    }}
                </a>
            </div>

            // 5. Traffic History Area Chart Card
            <section class="rp-card">
                <div class="rp-between" style="margin-bottom:10px;flex-wrap:wrap;gap:8px;">
                    <h2 class="rp-h3">{move || state.t("act.traffic")}</h2>
                    <div class="rp-seg" data-size="M">
                        {[(60u64, "act.live"), (600u64, "act.m10")].into_iter().map(|(secs, key)| {
                            view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    data-selected=move || (history_window.get() == secs).then_some("true")
                                    on:click=move |_| history_window.set(secs)
                                >
                                    {move || state.t(key)}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
                {move || {
                    let down = if history_down.get().len() >= 2 {
                        history_down.get()
                    } else {
                        state.down_ring.get().iter().copied().collect()
                    };
                    let up = if history_up.get().len() >= 2 {
                        history_up.get()
                    } else {
                        state.up_ring.get().iter().copied().collect()
                    };
                    let avg_down = doona_logic::rings::mean(down.iter().map(|&v| json!(v)).collect())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let avg_up = doona_logic::rings::mean(up.iter().map(|&v| json!(v)).collect())
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    view! {
                        <div>
                            <DualAreaSvg primary=down secondary=up color_a="var(--rp-c1)" color_b="var(--rp-c4)" />
                            <div class="rp-cluster" style="margin-top:8px;font-size:12px;">
                                <span class="rp-label">{move || format!("{}: {}", state.t("ui.download"), format_rate(avg_down))}</span>
                                <span class="rp-label">{move || format!("{}: {}", state.t("ui.upload"), format_rate(avg_up))}</span>
                            </div>
                        </div>
                    }
                }}
            </section>

            // 6. Active Groups & Outbounds summary
            <div class="rp-g21" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:12px;">
                <section class="rp-card">
                    <div class="rp-between" style="margin-bottom:8px;">
                        <h2 class="rp-h3">{move || state.t("nav.policies")}</h2>
                        <a class="rp-btn quiet sm" href="#policies">{move || state.t("act.viewAll")}</a>
                    </div>
                    <div class="rp-kv">
                        {move || {
                            let gs = state.groups.get();
                            let nodes = state.nodes.get().map(|n| n.nodes).unwrap_or_default();
                            gs.into_iter().take(6).map(|g| {
                                let sel = g.selection.tcp_member_id.clone().unwrap_or_default();
                                let resolved_name = nodes
                                    .iter()
                                    .find(|n| n.id == sel)
                                    .map(|n| format!("{} ({})", n.name, format_latency(n.best_latency_ms())))
                                    .unwrap_or_else(|| if sel.is_empty() { "—".to_string() } else { sel });
                                view! {
                                    <div>
                                        <span class="k">{g.name}</span>
                                        <span class="v">{resolved_name}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </section>

                <section class="rp-card">
                    <div class="rp-between" style="margin-bottom:8px;">
                        <h2 class="rp-h3">{move || state.t("act.outUsage")}</h2>
                        <a class="rp-btn quiet sm" href="#connections">{move || state.t("act.viewAll")}</a>
                    </div>
                    {move || {
                        let rows = outbounds.get();
                        if rows.is_empty() {
                            view! { <p class="rp-empty">{move || state.t("act.rankingEmpty")}</p> }.into_any()
                        } else {
                            view! {
                                <div class="rp-kv">
                                    {rows.into_iter().take(6).map(|(name, dl, ul)| view! {
                                        <div>
                                            <span class="k">{name}</span>
                                            <span class="v">{format!("↓ {} · ↑ {}", format_bytes(dl), format_bytes(ul))}</span>
                                        </div>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </section>
            </div>
        </div>
    }
}
