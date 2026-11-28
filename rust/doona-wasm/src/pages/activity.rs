//! Activity Hub page: Compact single-screen iOS dashboard showing Engine status, Outbound Mode,
//! Node Policy selector (via gated `policytarget-rs`), concrete live & cumulative throughput,
//! TCP/UDP connection breakdown, CPU/PID, RSS/eBPF memory, and live dual-area traffic chart.

use doona_common::format::{format_bytes, format_duration_secs, format_rate};
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

    // Fetch traffic history on load / window change / refresh.
    Effect::new(move |_| {
        let _nonce = state.reload_nonce.get();
        let win = history_window.get();
        let api = state.client();
        spawn_local(async move {
            if let Ok(hist) = api.traffic_history(win, 90).await {
                let mut d = Vec::with_capacity(hist.samples.len());
                let mut u = Vec::with_capacity(hist.samples.len());
                for pt in &hist.samples {
                    d.push(doona_common::model::value_f64(&pt.download_bytes_per_second).unwrap_or(0.0));
                    u.push(doona_common::model::value_f64(&pt.upload_bytes_per_second).unwrap_or(0.0));
                }
                history_down.set(d);
                history_up.set(u);
            }
        });
    });

    // Keep history_down / history_up rolling forward on every live runtime tick so the chart never freezes!
    Effect::new(move |_| {
        if let Some(rt) = state.runtime.get() {
            let dl = rt.download_rate();
            let ul = rt.upload_rate();
            history_down.update(|v| {
                if !v.is_empty() {
                    if v.len() >= 120 {
                        v.remove(0);
                    }
                    v.push(dl);
                }
            });
            history_up.update(|v| {
                if !v.is_empty() {
                    if v.len() >= 120 {
                        v.remove(0);
                    }
                    v.push(ul);
                }
            });
        }
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
        let main_writable = main_src
            .and_then(|s| s.writable)
            .unwrap_or_else(|| state.capabilities.get().is_none_or(|c| c.can_write_config()));
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
        let (_, fallback, active_target, options, uses_hub, hub_id, can_select_hub, _, main_id) =
            mode_model.get_untracked();
        let Some(source_id) = main_id else {
            state.toast("negative", state.t("act.modeNoRouting"), None);
            return;
        };
        let target_val = staged_target
            .get_untracked()
            .filter(|s| !s.is_empty())
            .or_else(|| (!active_target.is_empty()).then_some(active_target))
            .or_else(|| (!fallback.is_empty()).then_some(fallback))
            .or_else(|| options.first().map(|(id, _)| id.clone()))
            .unwrap_or_default();
        let submitted = match next_mode.as_str() {
            "direct" => OutboundMode::Direct,
            "global" => {
                if target_val.is_empty() {
                    state.toast("negative", state.t("act.globalMissing"), None);
                    return;
                }
                OutboundMode::Global(target_val)
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
                            Ok(op) => {
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
                                let ok_msg = state.t_args("act.modeApplied", &[("mode", &mode_lbl)]);
                                state.poll_operation(op.operation_id, ok_msg);
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
                                    Ok(op) => {
                                        if uses_hub
                                            && can_select_hub
                                            && let (Some(hid), Some(det)) = (hub_id, detail_now)
                                            && let Some(m) = det.members.iter().find(|m| m.name == target)
                                        {
                                            let _ = api.select_group(&hid, &m.id, "both").await;
                                        }
                                        state.set_saved_policy_target(&target);
                                        staged_target.set(None);
                                        let ok_msg = state.t_args("act.globalApplied", &[("target", &target)]);
                                        state.poll_operation(op.operation_id, ok_msg);
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
                                Ok(op) => {
                                    state.set_saved_policy_target(&target);
                                    staged_target.set(None);
                                    let ok_msg = state.t_args(
                                        "act.globalRuleApplied",
                                        &[("target", &target), ("mode", &mode_label)],
                                    );
                                    state.poll_operation(op.operation_id, ok_msg);
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
        <div class="rp-page" style="display:flex;flex-direction:column;gap:8px;">
            // 1. Unified iOS Control & Engine Status Card (Status + Outbound Mode + Node Policy in one inset-grouped card)
            <section class="rp-card" style="padding:10px 13px;display:flex;flex-direction:column;gap:8px;">
                // Row A: Engine status + uptime + datapath + Overview link
                <div class="rp-between" style="gap:8px;align-items:center;">
                    <div class="rp-cluster" style="gap:8px;min-width:0;">
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
                        <span class="rp-label" style="font-size:12px;font-variant-numeric:tabular-nums;white-space:nowrap;">
                            {move || {
                                let up = state
                                    .runtime
                                    .get()
                                    .and_then(|r| r.uptime_secs())
                                    .map(format_duration_secs)
                                    .unwrap_or_else(|| "—".into());
                                let dp = state
                                    .datapath
                                    .get()
                                    .and_then(|d| d.kind)
                                    .unwrap_or_else(|| "ebpf".into())
                                    .to_uppercase();
                                format!("{up} · {dp}")
                            }}
                        </span>
                    </div>
                    <a class="rp-btn quiet sm" href="#overview" style="padding:2px 8px;font-size:12px;">
                        {move || state.t("act.viewDetails")}
                    </a>
                </div>

                <div style="height:0.5px;background:var(--rp-hl-med);"></div>

                // Row B: Outbound mode (规则 / 直连 / 全局)
                <div class="rp-between" style="gap:8px;align-items:center;flex-wrap:wrap;">
                    <span style="font-size:13px;font-weight:600;color:var(--rp-text);">{move || state.t("act.mode")}</span>
                    <div class="rp-cluster" style="gap:6px;">
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
                                        class="rp-btn sm"
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
                    </div>
                </div>

                <div style="height:0.5px;background:var(--rp-hl-med);"></div>

                // Row C: Node Policy Selector (节点策略选择)
                <div class="rp-between" style="gap:8px;align-items:center;">
                    <span style="font-size:13px;font-weight:600;color:var(--rp-text);white-space:nowrap;">{move || state.t("act.global")}</span>
                    <div class="rp-cluster" style="gap:6px;flex:1;justify-content:flex-end;min-width:0;">
                        <select
                            class="rp-input"
                            style="max-width:190px;min-width:0;height:32px;padding:0 26px 0 10px;font-size:13px;"
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
                    </div>
                </div>
            </section>

            // 2. 2-Column Live Throughput & Cumulative Traffic Cards (Download | Upload)
            <div style="display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;">
                <a
                    class="rp-card rp-card-link"
                    href="#connections"
                    style="padding:11px 13px;text-decoration:none;color:inherit;display:flex;flex-direction:column;gap:4px;"
                >
                    <div class="rp-between" style="gap:4px;align-items:baseline;">
                        <span style="font-size:12px;color:var(--rp-c1);font-weight:600;">
                            {move || format!("↓ {}", state.t("ui.download"))}
                        </span>
                        <span class="rp-label" style="font-size:11px;font-variant-numeric:tabular-nums;">
                            {move || state.runtime.get().and_then(|r| r.total_download_bytes()).map(format_bytes).unwrap_or_else(|| "0 B".into())}
                        </span>
                    </div>
                    <div style="font-size:20px;font-weight:700;line-height:24px;font-variant-numeric:tabular-nums;">
                        {move || format_rate(state.runtime.get().map(|r| r.download_rate()).unwrap_or(0.0))}
                    </div>
                    {move || {
                        let vals: Vec<f64> = state.down_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c1)" height=22.0 /> }
                    }}
                </a>

                <a
                    class="rp-card rp-card-link"
                    href="#connections"
                    style="padding:11px 13px;text-decoration:none;color:inherit;display:flex;flex-direction:column;gap:4px;"
                >
                    <div class="rp-between" style="gap:4px;align-items:baseline;">
                        <span style="font-size:12px;color:var(--rp-c4);font-weight:600;">
                            {move || format!("↑ {}", state.t("ui.upload"))}
                        </span>
                        <span class="rp-label" style="font-size:11px;font-variant-numeric:tabular-nums;">
                            {move || state.runtime.get().and_then(|r| r.total_upload_bytes()).map(format_bytes).unwrap_or_else(|| "0 B".into())}
                        </span>
                    </div>
                    <div style="font-size:20px;font-weight:700;line-height:24px;font-variant-numeric:tabular-nums;">
                        {move || format_rate(state.runtime.get().map(|r| r.upload_rate()).unwrap_or(0.0))}
                    </div>
                    {move || {
                        let vals: Vec<f64> = state.up_ring.get().iter().copied().collect();
                        view! { <SparkSvg values=vals color="var(--rp-c4)" height=22.0 /> }
                    }}
                </a>
            </div>

            // 3. 3-Column Concrete System Telemetry Row (Connections TCP/UDP | CPU & PID | Memory RSS & eBPF)
            <div style="display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;">
                <a
                    class="rp-card rp-card-link"
                    href="#connections"
                    style="padding:10px 11px;text-decoration:none;color:inherit;display:flex;flex-direction:column;gap:2px;"
                >
                    <span class="rp-label" style="font-size:11px;white-space:nowrap;">{move || state.t("act.active")}</span>
                    <span style="font-size:18px;font-weight:700;line-height:22px;font-variant-numeric:tabular-nums;color:var(--rp-text);">
                        {move || state.runtime.get().and_then(|r| r.traffic.connections.total).unwrap_or(0).to_string()}
                    </span>
                    <span class="rp-label" style="font-size:10.5px;font-variant-numeric:tabular-nums;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                        {move || {
                            let rt = state.runtime.get();
                            let tcp = rt.as_ref().and_then(|r| r.traffic.connections.tcp).unwrap_or(0);
                            let udp = rt.as_ref().and_then(|r| r.traffic.connections.udp).unwrap_or(0);
                            format!("T:{tcp} · U:{udp}")
                        }}
                    </span>
                </a>

                <a
                    class="rp-card rp-card-link"
                    href="#overview"
                    style="padding:10px 11px;text-decoration:none;color:inherit;display:flex;flex-direction:column;gap:2px;"
                >
                    <span class="rp-label" style="font-size:11px;white-space:nowrap;">{move || state.t("act.cpu")}</span>
                    <span style="font-size:18px;font-weight:700;line-height:22px;font-variant-numeric:tabular-nums;color:var(--rp-text);">
                        {move || match state.runtime.get().and_then(|r| r.process.cpu_percent) {
                            Some(v) => format!("{v:.1}%"),
                            None => "—".to_string(),
                        }}
                    </span>
                    <span class="rp-label" style="font-size:10.5px;font-variant-numeric:tabular-nums;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                        {move || match state.runtime.get().and_then(|r| r.process.pid) {
                            Some(pid) => format!("PID {pid}"),
                            None => "honk-core".to_string(),
                        }}
                    </span>
                </a>

                <a
                    class="rp-card rp-card-link"
                    href="#overview"
                    style="padding:10px 11px;text-decoration:none;color:inherit;display:flex;flex-direction:column;gap:2px;"
                >
                    <span class="rp-label" style="font-size:11px;white-space:nowrap;">{move || state.t("act.memory")}</span>
                    <span style="font-size:18px;font-weight:700;line-height:22px;font-variant-numeric:tabular-nums;color:var(--rp-text);white-space:nowrap;">
                        {move || match state.memory.get().and_then(|m| m.rss().or_else(|| m.cgroup_current())) {
                            Some(b) => format_bytes(b),
                            None => "—".to_string(),
                        }}
                    </span>
                    <span class="rp-label" style="font-size:10.5px;font-variant-numeric:tabular-nums;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                        {move || match state.memory.get().and_then(|m| m.ebpf()) {
                            Some(b) => format!("BPF {}", format_bytes(b)),
                            None => "RSS".to_string(),
                        }}
                    </span>
                </a>
            </div>

            // 4. Compact Live Traffic Trend Card (fits 100% above dock even at scrollTop == 0!)
            <section class="rp-card" style="padding:9px 13px;display:flex;flex-direction:column;gap:5px;">
                <div class="rp-between" style="gap:6px;align-items:center;flex-wrap:wrap;">
                    <div class="rp-cluster" style="gap:8px;align-items:baseline;">
                        <h2 class="rp-h3" style="font-size:14px;margin:0;">{move || state.t("act.traffic")}</h2>
                        {move || {
                            let is_live_win = history_window.get() <= 60;
                            let down: Vec<f64> = if is_live_win && state.down_ring.get().len() >= 2 {
                                state.down_ring.get().iter().copied().collect()
                            } else if history_down.get().len() >= 2 {
                                history_down.get()
                            } else {
                                state.down_ring.get().iter().copied().collect()
                            };
                            let up: Vec<f64> = if is_live_win && state.up_ring.get().len() >= 2 {
                                state.up_ring.get().iter().copied().collect()
                            } else if history_up.get().len() >= 2 {
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
                                <span class="rp-label" style="font-size:11px;font-variant-numeric:tabular-nums;">
                                    {format!("↓ {} · ↑ {}", format_rate(avg_down), format_rate(avg_up))}
                                </span>
                            }
                        }}
                    </div>
                    <div class="rp-seg" data-size="M">
                        {[(60u64, "act.live"), (600u64, "act.m10")].into_iter().map(|(secs, key)| {
                            view! {
                                <button
                                    type="button"
                                    class="rp-btn sm"
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
                    let is_live_win = history_window.get() <= 60;
                    let down: Vec<f64> = if is_live_win && state.down_ring.get().len() >= 2 {
                        state.down_ring.get().iter().copied().collect()
                    } else if history_down.get().len() >= 2 {
                        history_down.get()
                    } else {
                        state.down_ring.get().iter().copied().collect()
                    };
                    let up: Vec<f64> = if is_live_win && state.up_ring.get().len() >= 2 {
                        state.up_ring.get().iter().copied().collect()
                    } else if history_up.get().len() >= 2 {
                        history_up.get()
                    } else {
                        state.up_ring.get().iter().copied().collect()
                    };
                    view! {
                        <DualAreaSvg primary=down secondary=up color_a="var(--rp-c1)" color_b="var(--rp-c4)" height=56.0 />
                    }
                }}
            </section>

            // Confirm dialog for direct/global mode switch
            <Show when=move || confirm_mode.get().is_some()>
                <div
                    class="rp-ios-sheet-backdrop"
                    on:click=move |_| confirm_mode.set(None)
                >
                    <div
                        class="rp-card rp-ios-sheet"
                        role="dialog"
                        aria-modal="true"
                        on:click=move |ev| ev.stop_propagation()
                    >
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
                        <div class="rp-ios-sheet-actions">
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
        </div>
    }
}
