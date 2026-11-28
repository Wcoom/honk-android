//! System Status (Overview) page: Engine info, Counters, ALL 8 Memory metrics + meter,
//! Datapath + eBPF attachments table, and Capabilities overview.

use doona_common::format::{format_bytes, format_duration_secs};
use doona_common::i18n::Lang;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::state::use_app_state;
use crate::ui::{ConfirmSheet, StatusLight, copy_to_clipboard, download_text_file};

fn datapath_label(state: &crate::state::AppState, value: &str) -> String {
    let key = match value {
        "ebpf" => "ov.v.ebpf",
        "userspace" => "ov.v.userspace",
        "mock" => "ov.v.mock",
        "active" => "ov.v.active",
        "degraded" => "ov.v.degraded",
        "detached" => "ov.v.detached",
        "failed" => "ov.v.failed",
        "disabled" => "ov.v.disabled",
        "full" => "ov.v.full",
        "partial" => "ov.v.partial",
        "none" => "ov.v.none",
        "real" => "ov.v.real",
        "loaded" => "ov.v.loaded",
        "not_loaded" => "ov.v.notLoaded",
        "attached" => "ov.v.attached",
        "partially_attached" => "ov.v.partiallyAttached",
        "published" => "ov.v.published",
        "not_published" => "ov.v.notPublished",
        "healthy" => "ov.v.healthy",
        "ready" => "ov.v.ready",
        "error" => "ov.v.error",
        "unknown" => "ov.v.unknown",
        "ingress" => "ov.v.ingress",
        "egress" => "ov.v.egress",
        _ => return value.to_string(),
    };
    state.t(key)
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    let state = use_app_state();
    let busy = RwSignal::new(false);
    let reload_confirm = RwSignal::new(false);

    let run_reload = Callback::new(move |()| {
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.start_reload().await {
                Ok(op) => {
                    state.poll_operation(op.operation_id, state.t("ov.succeeded"));
                }
                Err(e) => state.toast("negative", state.t("ov.failed"), Some(e)),
            }
            busy.set(false);
        });
    });

    let on_export = move |_| {
        let api = state.client();
        spawn_local(async move {
            match api.export_config_text().await {
                Ok(text) => {
                    let _ = download_text_file("honk-config.dae", &text, "text/plain;charset=utf-8");
                    copy_to_clipboard(&text);
                    state.toast(
                        "positive",
                        state.t("config.backup.export"),
                        Some(format!("honk-config.dae · {} bytes", text.len())),
                    );
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:14px;">
            // Top status strip
            <div class="rp-between" style="flex-wrap:wrap;gap:10px;">
                <div class="rp-cluster" style="flex-wrap:wrap;gap:12px;">
                    {move || {
                        let st = state.runtime.get().map(|r| r.state_str().to_string()).unwrap_or_else(|| "unknown".into());
                        let tone: &'static str = match st.as_str() {
                            "running" => "ok",
                            "degraded" | "reloading" | "starting" => "warn",
                            "failed" => "err",
                            _ => "muted",
                        };
                        let key = match st.as_str() {
                            "running" => "ov.running",
                            "degraded" => "ov.v.degraded",
                            "failed" => "ov.failed",
                            "suspended" => "ov.suspended",
                            "reloading" => "ov.reloading",
                            "starting" => "ov.starting",
                            _ => "ov.unknown",
                        };
                        view! { <StatusLight tone=tone text=state.t(key) /> }
                    }}
                    <div class="rp-kv inline row">
                        <div>
                            <span class="k">{move || state.t("ov.config")}</span>
                            <span class="v">
                                {move || {
                                    let rev = state
                                        .runtime
                                        .get()
                                        .and_then(|r| r.generation.config_revision)
                                        .unwrap_or_else(|| "—".into());
                                    doona_logic::selectors::short_id(rev.clone())
                                        .ok()
                                        .and_then(|v| v.as_str().map(ToString::to_string))
                                        .unwrap_or(rev)
                                }}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.uptime")}</span>
                            <span class="v">
                                {move || state.runtime.get().and_then(|r| r.uptime_secs()).map(format_duration_secs).unwrap_or_else(|| "—".into())}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.cpu")}</span>
                            <span class="v">
                                {move || match state.runtime.get().and_then(|r| r.process.cpu_percent) {
                                    Some(v) => format!("{v:.1}%"),
                                    None => "—".into(),
                                }}
                            </span>
                        </div>
                    </div>
                </div>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=on_export>
                        {move || state.t("ov.export")}
                    </button>
                    <button type="button" class="rp-btn accent sm" disabled=move || busy.get() on:click=move |_| reload_confirm.set(true)>
                        {move || state.t("ov.reload")}
                    </button>
                </div>
            </div>

            <ConfirmSheet
                open=reload_confirm
                title=Signal::derive(move || state.t("ov.reload"))
                message=Signal::derive(move || match state.lang.get() {
                    Lang::En => "Hot-reload active configuration? Existing connections stay intact when possible.".into(),
                    Lang::ZhTw => "確定要熱重載目前生效的核心設定嗎？".into(),
                    Lang::ZhCn => "确定要热重载当前生效的核心配置吗？现有连接将尽可能保持不断开。".into(),
                })
                confirm_label=Signal::derive(move || state.t("ov.reload"))
                destructive=false
                on_confirm=run_reload
            />

            // 3-card row: Engine | Counters | Memory (ALL 8 METRICS!)
            <div class="rp-g3" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px;">
                // Engine Card
                <section class="rp-card">
                    <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("ov.f.engine")}</h2>
                    <div class="rp-kv">
                        <div>
                            <span class="k">{move || state.t("ov.f.engine")}</span>
                            <span class="v">
                                {move || state.version.get().and_then(|v| v.engine).map(|e| format!("{} {}", e.name.unwrap_or_default(), e.version.unwrap_or_default())).unwrap_or_else(|| "—".into())}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.api")}</span>
                            <span class="v">
                                {move || state.version.get().and_then(|v| v.api).map(|a| format!("{} v{} ({})", a.name.unwrap_or_default(), a.major.unwrap_or(1), a.status.unwrap_or_default())).unwrap_or_else(|| "—".into())}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.build")}</span>
                            <span class="v">
                                {move || state.version.get().and_then(|v| v.build).map(|b| {
                                    let parts: Vec<String> = [b.revision, b.target].into_iter().flatten().filter(|s| !s.is_empty()).collect();
                                    if parts.is_empty() { "—".into() } else { parts.join(" · ") }
                                }).unwrap_or_else(|| "—".into())}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.instance")}</span>
                            <span class="v">
                                {move || {
                                    let inst = state.runtime.get().and_then(|r| r.instance_id).unwrap_or_else(|| "—".into());
                                    doona_logic::selectors::short_id(inst.clone())
                                        .ok()
                                        .and_then(|v| v.as_str().map(ToString::to_string))
                                        .unwrap_or(inst)
                                }}
                            </span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.started")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.lifecycle.started_at).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.activated")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.generation.activated_at).unwrap_or_else(|| "—".into())}</span>
                        </div>
                    </div>
                    <Show when=move || state.capabilities.get().is_some_and(|c| c.core_update_available())>
                        <div class="rp-cluster" style="margin-top:10px;">
                            <a class="rp-link" href="#settings?card=coreUpdate">{move || state.t("ov.coreUpdateLink")}</a>
                        </div>
                    </Show>
                </section>

                // Counters Card
                <section class="rp-card">
                    <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("ov.counters")}</h2>
                    <div class="rp-kv">
                        <div>
                            <span class="k">{move || state.t("ov.f.tcp")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.traffic.connections.tcp).map(|v| v.to_string()).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.udp")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.traffic.connections.udp).map(|v| v.to_string()).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.total")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.traffic.connections.total).map(|v| v.to_string()).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ui.upload")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.total_upload_bytes()).map(format_bytes).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ui.download")}</span>
                            <span class="v">{move || state.runtime.get().and_then(|r| r.total_download_bytes()).map(format_bytes).unwrap_or_else(|| "—".into())}</span>
                        </div>
                        <div>
                            <span class="k">{move || state.t("ov.f.rateWindow")}</span>
                            <span class="v">
                                {move || state.runtime.get().and_then(|r| r.traffic.rates).and_then(|r| r.window_seconds).map(|w| format!("{w:.0} s")).unwrap_or_else(|| "—".into())}
                            </span>
                        </div>
                    </div>
                </section>

                // Memory Card — ALL 8 METRICS
                <section class="rp-card" id="memory-card">
                    <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("act.memory")}</h2>
                    {move || {
                        let mem = state.memory.get();
                        let limit_reported = state.capabilities.get().is_some_and(|c| c.memory_limit_reported());
                        let rss = mem.as_ref().and_then(|m| m.rss()).map(format_bytes).unwrap_or_else(|| "—".into());
                        let cg_cur_raw = mem.as_ref().and_then(|m| m.cgroup_current());
                        let cg_lim_raw = mem.as_ref().and_then(|m| m.cgroup_limit());
                        let cg_cur = cg_cur_raw.map(format_bytes).unwrap_or_else(|| "—".into());
                        let cg_lim = match cg_lim_raw {
                            Some(b) => format_bytes(b),
                            None if limit_reported => state.t("ov.noLimit"),
                            None => "—".into(),
                        };
                        let scope = match mem.as_ref().and_then(|m| m.cgroup_scope()) {
                            Some("service") => state.t("ov.v.cgroupService"),
                            Some("shared") => state.t("ov.v.cgroupShared"),
                            _ => "—".into(),
                        };
                        let high = mem.as_ref().and_then(|m| m.high()).map(|v| v.to_string()).unwrap_or_else(|| "—".into());
                        let oom = mem.as_ref().and_then(|m| m.oom()).map(|v| v.to_string()).unwrap_or_else(|| "—".into());
                        let oom_kill = mem.as_ref().and_then(|m| m.oom_kill()).map(|v| v.to_string()).unwrap_or_else(|| "—".into());
                        let ebpf = mem.as_ref().and_then(|m| m.ebpf()).map(format_bytes).unwrap_or_else(|| "—".into());

                        let meter = match (cg_cur_raw, cg_lim_raw) {
                            (Some(cur), Some(lim)) if lim > 0 => {
                                let pct = ((cur as f64 / lim as f64) * 100.0).clamp(0.0, 100.0);
                                let tone = doona_logic::selectors::memory_tone(pct).unwrap_or_else(|_| "ok".into());
                                Some((pct, tone, format!("{} / {}", format_bytes(cur), format_bytes(lim))))
                            }
                            _ => None,
                        };

                        view! {
                            <div class="rp-kv">
                                {meter.map(|(pct, tone, text)| {
                                    let cls = format!("rp-kv-meter {tone} wide");
                                    let bar_style = format!("width:{pct:.1}%;height:6px;border-radius:3px;background:var(--rp-accent);");
                                    view! {
                                        <div class=cls style="grid-column:1/-1;">
                                            <span class="k">{state.t("ov.f.cgroupPercent")}</span>
                                            <span class="v">{text}</span>
                                            <div class="rp-meter" style="width:100%;height:6px;background:var(--rp-hl-med);border-radius:3px;overflow:hidden;margin-top:4px;">
                                                <div style=bar_style></div>
                                            </div>
                                        </div>
                                    }
                                })}
                                <div><span class="k">{state.t("ov.f.rss")}</span><span class="v">{rss}</span></div>
                                <div><span class="k">{state.t("ov.f.cgroupCurrent")}</span><span class="v">{cg_cur}</span></div>
                                <div><span class="k">{state.t("ov.f.cgroupLimit")}</span><span class="v">{cg_lim}</span></div>
                                <div><span class="k">{state.t("ov.f.cgroupScope")}</span><span class="v">{scope}</span></div>
                                <div><span class="k">{state.t("ov.f.oomHigh")}</span><span class="v">{high}</span></div>
                                <div><span class="k">{state.t("ov.f.oom")}</span><span class="v">{oom}</span></div>
                                <div><span class="k">{state.t("ov.f.oomKill")}</span><span class="v">{oom_kill}</span></div>
                                <div><span class="k">{state.t("ov.f.ebpfBytes")}</span><span class="v">{ebpf}</span></div>
                            </div>
                        }
                    }}
                </section>
            </div>

            // Lower row: Datapath + Attachments | Capabilities
            <div class="rp-g21 rp-overview-lower" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:12px;">
                <section class="rp-card">
                    <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("ov.datapath")}</h2>
                    {move || match state.datapath.get() {
                        None => view! { <p class="rp-empty">{state.t("ov.unavailable")}</p> }.into_any(),
                        Some(dp) => {
                            let kind = datapath_label(&state, dp.kind.as_deref().unwrap_or("unknown"));
                            let st = datapath_label(&state, dp.state.as_deref().unwrap_or("unknown"));
                            let ebpf = dp.ebpf.clone();
                            view! {
                                <div style="display:flex;flex-direction:column;gap:12px;">
                                    <div class="rp-kv">
                                        <div><span class="k">{state.t("ov.f.kind")}</span><span class="v">{kind}</span></div>
                                        <div><span class="k">{state.t("ov.f.state")}</span><span class="v">{st}</span></div>
                                        {ebpf.as_ref().map(|e| {
                                            let backend = datapath_label(&state, e.backend.as_deref().unwrap_or("unknown"));
                                            let progs = datapath_label(&state, e.programs.as_deref().unwrap_or("unknown"));
                                            let hooks = datapath_label(&state, e.hooks.as_deref().unwrap_or("unknown"));
                                            let routing = datapath_label(&state, e.routing.as_ref().and_then(|r| r.state.as_deref()).unwrap_or("unknown"));
                                            let health = datapath_label(&state, e.health.as_deref().unwrap_or("unknown"));
                                            let maps = e.maps.as_ref().and_then(|m| m.conn_state.as_ref()).map(|c| {
                                                if c.occupancy_known && let Some(occ) = c.occupancy {
                                                    format!("{occ} / {}", c.capacity)
                                                } else {
                                                    state.t_args("ov.mapCapacity", &[("capacity", &c.capacity.to_string())])
                                                }
                                            }).unwrap_or_else(|| "—".into());
                                            view! {
                                                <>
                                                    <div><span class="k">{state.t("ov.f.backend")}</span><span class="v">{backend}</span></div>
                                                    <div><span class="k">{state.t("ov.f.programs")}</span><span class="v">{progs}</span></div>
                                                    <div><span class="k">{state.t("ov.f.hooks")}</span><span class="v">{hooks}</span></div>
                                                    <div><span class="k">{state.t("ov.f.routing")}</span><span class="v">{routing}</span></div>
                                                    <div><span class="k">{state.t("ov.f.health")}</span><span class="v">{health}</span></div>
                                                    <div><span class="k">{state.t("ov.f.maps")}</span><span class="v">{maps}</span></div>
                                                </>
                                            }
                                        })}
                                    </div>
                                    {ebpf.filter(|e| !e.attachments.is_empty()).map(|e| view! {
                                        <div style="overflow:auto;">
                                            <table class="rp-table" style="width:100%;border-collapse:collapse;font-size:13px;">
                                                <thead>
                                                    <tr>
                                                        <th style="text-align:left;padding:6px 8px;">{state.t("ui.name")}</th>
                                                        <th style="text-align:left;padding:6px 8px;">{state.t("ov.interface")}</th>
                                                        <th style="text-align:left;padding:6px 8px;">{state.t("ov.direction")}</th>
                                                        <th style="text-align:left;padding:6px 8px;">{state.t("ui.state")}</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {e.attachments.into_iter().map(|a| view! {
                                                        <tr>
                                                            <td style="padding:6px 8px;"><code>{a.name}</code></td>
                                                            <td style="padding:6px 8px;">{a.interface}</td>
                                                            <td style="padding:6px 8px;">{datapath_label(&state, &a.direction)}</td>
                                                            <td style="padding:6px 8px;">{datapath_label(&state, &a.state)}</td>
                                                        </tr>
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

                <section class="rp-card">
                    <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("ov.resources")}</h2>
                    <div class="rp-capabilities">
                        {move || {
                            let caps = state.capabilities.get();
                            let entries: Vec<(&str, &str)> = vec![
                                ("connections", "#connections"),
                                ("flows", "#flows"),
                                ("dns_query", "#dns"),
                                ("dns_cache", "#dns"),
                                ("dns_log", "#dns"),
                                ("logs", "#logs"),
                                ("events", "#events"),
                                ("groups", "#policies"),
                                ("nodes", "#nodes"),
                                ("providers", "#nodes"),
                                ("rules", "#rules"),
                                ("routing_trace", "#rules"),
                                ("config", "#config"),
                                ("geodata", "#settings"),
                                ("runtime_settings", "#settings"),
                            ];
                            entries
                                .into_iter()
                                .filter(|(key, _)| caps.as_ref().is_some_and(|c| c.resource_available(key)))
                                .map(|(key, href)| view! {
                                    <div class="rp-capability">
                                        <span class="rp-light ok">
                                            <span class="rp-dot" aria-hidden="true"></span>
                                            <a class="rp-link" href=href>{key}</a>
                                        </span>
                                    </div>
                                })
                                .collect_view()
                        }}
                    </div>
                </section>
            </div>
        </div>
    }
}
