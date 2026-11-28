//! Settings Hub pages: Multi-source `.dae` Config Editor (create source, dirty tracking, upload/download,
//! structured validation diagnostics, revisions) & Settings Page (CoreUpdateCard, Appearance 16 palettes,
//! Language, Backend profile, structured Geodata cards, interactive Runtime Settings PATCH form).

use doona_common::format::format_bytes;
use doona_common::i18n::Lang;
use doona_common::model::{ConfigRevisionList, CoreUpdateStatus, value_u64};
use doona_common::palettes::PALETTES;
use leptos::prelude::*;
use serde_json::{Value, json};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::mode_dae::highlight_dae;
use crate::state::use_app_state;
use crate::ui::{ConfirmSheet, StatusLight, download_text_file};

#[component]
pub fn ConfigPage() -> impl IntoView {
    let state = use_app_state();
    let selected_id = RwSignal::new(String::new());
    let draft = RwSignal::new(String::new());
    let baseline = RwSignal::new(String::new());
    let preview_mode = RwSignal::new(false);
    let validation = RwSignal::new(None::<Value>);
    let revisions = RwSignal::new(None::<ConfigRevisionList>);
    let busy = RwSignal::new(false);
    let import_confirm = RwSignal::new(false);
    let new_src_open = RwSignal::new(false);
    let new_src_path = RwSignal::new("custom.dae".to_string());
    let new_src_content = RwSignal::new("# custom config fragment\n".to_string());
    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if let Some(cfg) = state.config.get() {
            let cur = selected_id.get_untracked();
            let chosen = cfg
                .sources
                .iter()
                .find(|s| s.id == cur)
                .or_else(|| cfg.sources.iter().find(|s| s.kind.as_deref() == Some("main")))
                .or_else(|| cfg.sources.first());
            if let Some(src) = chosen {
                let text = src.content.clone().unwrap_or_default();
                let was_clean = draft.get_untracked() == baseline.get_untracked();
                if cur != src.id {
                    selected_id.set(src.id.clone());
                    baseline.set(text.clone());
                    draft.set(text);
                } else if was_clean {
                    baseline.set(text.clone());
                    draft.set(text);
                }
            }
        }
        let api = state.client();
        spawn_local(async move {
            if let Ok(revs) = api.config_revisions().await {
                revisions.set(Some(revs));
            }
        });
    });

    let is_dirty = Signal::derive(move || draft.get() != baseline.get());

    let on_validate = move |_| {
        let sid = selected_id.get_untracked();
        let text = draft.get_untracked();
        let cfg = state.config.get_untracked();
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            let sources_arr: Vec<Value> = cfg
                .map(|c| {
                    c.sources
                        .into_iter()
                        .enumerate()
                        .map(|(idx, s)| {
                            let content = if s.id == sid {
                                text.clone()
                            } else {
                                s.content.unwrap_or_default()
                            };
                            json!({
                                "path": s.path.unwrap_or_else(|| if idx == 0 { "config.dae".into() } else { format!("fragment-{idx}.dae") }),
                                "content": content,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            match api
                .validate_config(json!({ "mode": "full", "sources": sources_arr }))
                .await
            {
                Ok(res) => validation.set(Some(res)),
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            busy.set(false);
        });
    };

    let on_save = move |_| {
        let sid = selected_id.get_untracked();
        let text = draft.get_untracked();
        if sid.is_empty() {
            return;
        }
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.config_source(&sid).await {
                Ok((src, etag)) => {
                    let if_match = etag.unwrap_or_else(|| {
                        format!("\"{}\"", src.content_sha256.or(src.sha256).unwrap_or_default())
                    });
                    match api.replace_config_source(&sid, &text, &if_match).await {
                        Ok(op) => {
                            baseline.set(text);
                            state.poll_operation(op.operation_id, state.t("ui.saved"));
                        }
                        Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                    }
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            busy.set(false);
        });
    };

    let on_import_disk = Callback::new(move |()| {
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.import_config_replace(true).await {
                Ok(op) => {
                    state.poll_operation(op.operation_id, state.t("config.backup.imported"));
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            busy.set(false);
        });
    });

    let on_export_file = move |_| {
        let api = state.client();
        spawn_local(async move {
            match api.export_config_text().await {
                Ok(text) => {
                    let _ = download_text_file("honk-config.dae", &text, "text/plain;charset=utf-8");
                    state.toast("positive", state.t("config.backup.export"), Some("honk-config.dae".into()));
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
        });
    };

    let on_file_selected = move |ev: web_sys::Event| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(target) = ev.target()
                && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>()
                && let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                let promise = file.text();
                spawn_local(async move {
                    if let Ok(val) = wasm_bindgen_futures::JsFuture::from(promise).await
                        && let Some(txt) = val.as_string()
                    {
                        draft.set(txt);
                        state.toast("neutral", "已载入文件内容至编辑器，点击保存生效", None);
                    }
                });
                input.set_value("");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = ev;
    };

    let create_source = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let path = new_src_path.get_untracked().trim().to_string();
        let content = new_src_content.get_untracked();
        if path.is_empty() {
            return;
        }
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.create_config_source(&path, &content).await {
                Ok(op) => {
                    new_src_open.set(false);
                    state.poll_operation(op.operation_id, state.t("ui.saved"));
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            busy.set(false);
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <div class="rp-cluster" style="flex-wrap:wrap;">
                    <select
                        class="rp-input"
                        prop:value=move || selected_id.get()
                        on:change=move |ev| {
                            let id = event_target_value(&ev);
                            selected_id.set(id.clone());
                            if let Some(cfg) = state.config.get_untracked()
                                && let Some(src) = cfg.sources.into_iter().find(|s| s.id == id)
                            {
                                let content = src.content.unwrap_or_default();
                                baseline.set(content.clone());
                                draft.set(content);
                                validation.set(None);
                            }
                        }
                    >
                        {move || state.config.get().map(|c| c.sources.into_iter().map(|s| {
                            let label = s.path.clone().or(s.name.clone()).unwrap_or_else(|| s.id.clone());
                            view! { <option value=s.id>{label}</option> }
                        }).collect_view())}
                    </select>
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| new_src_open.set(true)>
                        "+ 新建片段"
                    </button>
                    <Show when=move || is_dirty.get()>
                        <span class="rp-light warn"><span class="rp-dot"></span>"未保存更改"</span>
                        <button
                            type="button"
                            class="rp-btn quiet sm"
                            on:click=move |_| draft.set(baseline.get_untracked())
                        >
                            "还原"
                        </button>
                    </Show>
                </div>

                <div class="rp-cluster" style="flex-wrap:wrap;">
                    <input
                        node_ref=file_input_ref
                        type="file"
                        accept=".dae,.conf,.txt"
                        style="display:none;"
                        on:change=on_file_selected
                    />
                    <button
                        type="button"
                        class="rp-btn secondary sm"
                        on:click=move |_| {
                            if let Some(el) = file_input_ref.get() {
                                el.click();
                            }
                        }
                    >
                        "打开本地文件"
                    </button>
                    <button type="button" class="rp-btn secondary sm" on:click=on_export_file>
                        {move || state.t("ov.export")}
                    </button>
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| preview_mode.update(|v| *v = !*v)>
                        {move || if preview_mode.get() { "编辑" } else { "高亮预览" }}
                    </button>
                    <button type="button" class="rp-btn secondary sm" disabled=move || busy.get() on:click=on_validate>
                        {move || state.t("config.validate")}
                    </button>
                    <button type="button" class="rp-btn secondary sm" disabled=move || busy.get() on:click=move |_| import_confirm.set(true)>
                        {move || state.t("config.backup.import")}
                    </button>
                    <button type="button" class="rp-btn accent sm" disabled=move || busy.get() || !is_dirty.get() on:click=on_save>
                        {move || state.t("config.save")}
                    </button>
                </div>
            </div>

            <section class="rp-card">
                {move || if preview_mode.get() {
                    let tokens = highlight_dae(&draft.get());
                    view! {
                        <pre class="rp-code" style="overflow:auto;max-height:55vh;white-space:pre;font-family:monospace;font-size:13px;line-height:1.5;">
                            {tokens.into_iter().map(|(cls, tok)| {
                                if cls.is_empty() {
                                    view! { <span>{tok}</span> }.into_any()
                                } else {
                                    view! { <span class=cls>{tok}</span> }.into_any()
                                }
                            }).collect_view()}
                        </pre>
                    }.into_any()
                } else {
                    view! {
                        <textarea
                            class="rp-input"
                            rows="22"
                            spellcheck="false"
                            style="width:100%;font-family:monospace;font-size:13px;line-height:1.5;min-height:360px;"
                            prop:value=move || draft.get()
                            on:input=move |ev| draft.set(event_target_value(&ev))
                        />
                    }.into_any()
                }}
            </section>

            {move || validation.get().map(|v| {
                let valid = v.get("valid").and_then(Value::as_bool).unwrap_or_else(|| {
                    v.get("diagnostics")
                        .and_then(Value::as_array)
                        .is_none_or(|d| d.is_empty())
                });
                let diags = v
                    .get("diagnostics")
                    .or_else(|| v.get("errors"))
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                view! {
                    <section class="rp-card">
                        <div class="rp-between" style="margin-bottom:8px;">
                            <StatusLight
                                tone=if valid { "ok" } else { "err" }
                                text=if valid { "语法与拓扑校验通过".into() } else { "配置存在校验问题".into() }
                            />
                            <button type="button" class="rp-btn quiet sm" on:click=move |_| validation.set(None)>"×"</button>
                        </div>
                        {(!diags.is_empty()).then(|| view! {
                            <div style="display:flex;flex-direction:column;gap:6px;">
                                {diags.into_iter().map(|d| {
                                    let sev = d.get("severity").and_then(Value::as_str).unwrap_or("error").to_uppercase();
                                    let path = d.get("path").or_else(|| d.get("file")).and_then(Value::as_str).unwrap_or("config.dae");
                                    let line = d.get("line").and_then(Value::as_u64).unwrap_or(1);
                                    let msg = d.get("message").and_then(Value::as_str).unwrap_or("校验错误").to_string();
                                    let loc = format!("{path}:{line}");
                                    view! {
                                        <div style="padding:6px 8px;border-radius:8px;background:var(--rp-surface);">
                                            <span class="rp-badge" style="margin-right:8px;">{sev}</span>
                                            <code style="margin-right:8px;">{loc}</code>
                                            <span>{msg}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        })}
                    </section>
                }
            })}

            {move || revisions.get().filter(|r| !r.revisions.is_empty()).map(|revs| view! {
                <section class="rp-card">
                    <h3 class="rp-h3" style="margin-bottom:8px;">{state.t("config.history")}</h3>
                    <div style="display:flex;flex-direction:column;gap:6px;">
                        {revs.revisions.into_iter().take(10).map(|rev| {
                            let rnum = rev.revision;
                            let is_active = revs.active == Some(rnum);
                            view! {
                                <div class="rp-between" style="padding:4px 0;border-bottom:1px solid var(--rp-line);">
                                    <span>
                                        <strong>{format!("#{rnum}")}</strong>
                                        " · "
                                        <span class="rp-label">{rev.created_at}</span>
                                        " · "
                                        <span class="rp-badge">{rev.origin}</span>
                                    </span>
                                    <button
                                        type="button"
                                        class="rp-btn secondary sm"
                                        disabled=is_active
                                        on:click=move |_| {
                                            let api = state.client();
                                            spawn_local(async move {
                                                match api.activate_config_revision(rnum).await {
                                                    Ok(op) => {
                                                        state.poll_operation(op.operation_id, state.t("ui.saved"));
                                                    }
                                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                }
                                            });
                                        }
                                    >
                                        {if is_active { "当前生效" } else { "回滚至此版本" }}
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </section>
            })}

            <Show when=move || new_src_open.get()>
                <div class="rp-ios-sheet-backdrop" on:click=move |_| new_src_open.set(false)>
                    <form
                        class="rp-dialog rp-card rp-ios-sheet"
                        style="width:min(460px,92vw);display:flex;flex-direction:column;gap:12px;"
                        on:click=move |ev| ev.stop_propagation()
                        on:submit=create_source
                    >
                        <h3 class="rp-h3">"新建配置片段"</h3>
                        <label class="rp-field">
                            <span class="rp-label">"相对文件名 (.dae)"</span>
                            <input
                                class="rp-input"
                                placeholder="rules.d/custom.dae"
                                prop:value=move || new_src_path.get()
                                on:input=move |ev| new_src_path.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="rp-field">
                            <span class="rp-label">"初始内容"</span>
                            <textarea
                                class="rp-input"
                                rows="6"
                                style="font-family:monospace;font-size:12px;"
                                prop:value=move || new_src_content.get()
                                on:input=move |ev| new_src_content.set(event_target_value(&ev))
                            />
                        </label>
                        <div class="rp-ios-sheet-actions">
                            <button type="button" class="rp-btn secondary" on:click=move |_| new_src_open.set(false)>
                                {move || state.t("ui.cancel")}
                            </button>
                            <button type="submit" class="rp-btn accent" disabled=move || busy.get()>
                                {move || state.t("ui.add")}
                            </button>
                        </div>
                    </form>
                </div>
            </Show>

            <ConfirmSheet
                open=import_confirm
                title=Signal::derive(move || state.t("config.backup.import"))
                message=Signal::derive(move || "从磁盘重新导入 config.dae 将生成新版本并覆盖当前数据库草稿，确定继续？".into())
                confirm_label=Signal::derive(move || state.t("config.backup.import"))
                destructive=false
                on_confirm=on_import_disk
            />
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let state = use_app_state();
    let base_input = RwSignal::new(state.api_base.get_untracked());
    let token_input = RwSignal::new(state.api_token.get_untracked().unwrap_or_default());
    let geodata = RwSignal::new(None::<Value>);
    let geo_busy = RwSignal::new(false);
    let rt_settings = RwSignal::new(None::<Value>);
    let rt_saving = RwSignal::new(false);

    // Editable Runtime Settings signals
    let rt_level = RwSignal::new("info".to_string());
    let rt_rec_flows = RwSignal::new("auto".to_string());
    let rt_rec_logs = RwSignal::new("auto".to_string());
    let rt_rec_dns = RwSignal::new("auto".to_string());
    let rt_log_buf = RwSignal::new("512".to_string());
    let rt_dns_max = RwSignal::new("512".to_string());
    let rt_flow_max = RwSignal::new("512".to_string());
    let rt_flow_ret = RwSignal::new("300".to_string());

    let sync_rt_signals = move |rs: &Value| {
        if let Some(lv) = rs.get("log").and_then(|l| l.get("level")).and_then(Value::as_str) {
            rt_level.set(lv.to_string());
        }
        if let Some(n) = rs.get("log").and_then(|l| l.get("buffered_records")).and_then(Value::as_u64) {
            rt_log_buf.set(n.to_string());
        }
        if let Some(n) = rs.get("dns_log").and_then(|l| l.get("max_records")).and_then(Value::as_u64) {
            rt_dns_max.set(n.to_string());
        }
        if let Some(n) = rs.get("flows").and_then(|f| f.get("max_flows")).and_then(Value::as_u64) {
            rt_flow_max.set(n.to_string());
        }
        if let Some(n) = rs.get("flows").and_then(|f| f.get("retention_seconds")).and_then(Value::as_u64) {
            rt_flow_ret.set(n.to_string());
        }
        if let Some(rec) = rs.get("recording") {
            if let Some(m) = rec.get("flows").and_then(|x| x.get("mode")).and_then(Value::as_str) {
                rt_rec_flows.set(m.to_string());
            }
            if let Some(m) = rec.get("logs").and_then(|x| x.get("mode")).and_then(Value::as_str) {
                rt_rec_logs.set(m.to_string());
            }
            if let Some(m) = rec.get("dns_log").and_then(|x| x.get("mode")).and_then(Value::as_str) {
                rt_rec_dns.set(m.to_string());
            }
        }
    };

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        let api = state.client();
        spawn_local(async move {
            if let Ok(g) = api.geodata().await {
                geodata.set(Some(g));
            }
            if let Ok(rs) = api.runtime_settings().await {
                sync_rt_signals(&rs);
                rt_settings.set(Some(rs));
            }
        });
    });

    let apply_rt_patch = move |patch: Value| {
        rt_saving.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.patch_runtime_settings(patch).await {
                Ok(updated) => {
                    sync_rt_signals(&updated);
                    rt_settings.set(Some(updated));
                    state.toast("positive", state.t("ui.saved"), None);
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            rt_saving.set(false);
        });
    };

    let save_runtime_settings = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let Some(cur) = rt_settings.get_untracked() else {
            return;
        };
        let rec = cur.get("recording").cloned().unwrap_or_else(|| json!({}));
        let allow_flows = rec
            .get("flows")
            .and_then(|x| x.get("allowed"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let allow_logs = rec
            .get("logs")
            .and_then(|x| x.get("allowed"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let allow_dns = rec
            .get("dns_log")
            .and_then(|x| x.get("allowed"))
            .and_then(Value::as_bool)
            .unwrap_or(true);

        let mut patch = serde_json::Map::new();
        let mf = rt_rec_flows.get_untracked();
        if allow_flows || mf != "on" {
            patch.insert("record_flows".into(), json!(mf));
        }
        let ml = rt_rec_logs.get_untracked();
        if allow_logs || ml != "on" {
            patch.insert("record_logs".into(), json!(ml));
        }
        let md = rt_rec_dns.get_untracked();
        if allow_dns || md != "on" {
            patch.insert("record_dns_log".into(), json!(md));
        }
        if allow_logs {
            let buf = rt_log_buf
                .get_untracked()
                .parse::<usize>()
                .unwrap_or(512)
                .clamp(64, 512);
            patch.insert(
                "log".into(),
                json!({
                    "level": rt_level.get_untracked(),
                    "buffered_records": buf,
                }),
            );
        }
        if allow_dns {
            let max_r = rt_dns_max
                .get_untracked()
                .parse::<usize>()
                .unwrap_or(512)
                .clamp(64, 512);
            patch.insert("dns_log".into(), json!({ "max_records": max_r }));
        }
        if allow_flows {
            let max_f = rt_flow_max
                .get_untracked()
                .parse::<usize>()
                .unwrap_or(1024)
                .clamp(64, 1024);
            let ret_s = rt_flow_ret
                .get_untracked()
                .parse::<u64>()
                .unwrap_or(300)
                .clamp(1, 300);
            patch.insert(
                "flows".into(),
                json!({
                    "max_flows": max_f,
                    "retention_seconds": ret_s,
                }),
            );
        }

        apply_rt_patch(Value::Object(patch));
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:14px;">
            // Core Update Card (`/api/v1/x-honk/core/update`)
            <CoreUpdateCard />

            // Appearance & Language Card
            <section class="rp-card" id="appearance">
                <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("ui.appearanceSettings")}</h2>
                <div style="display:flex;flex-direction:column;gap:12px;">
                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">{move || state.t("ui.lang")}</span>
                        <div class="rp-seg" data-size="M">
                            {Lang::ALL.iter().map(|&l| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    data-selected=move || (state.lang.get() == l).then_some("true")
                                    on:click=move |_| state.set_lang(l)
                                >
                                    {l.label()}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">{move || state.t("ui.theme")}</span>
                        <div class="rp-seg" data-size="M">
                            {[("system", "Auto"), ("light", "Light"), ("dark", "Dark")].into_iter().map(|(id, lbl)| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    data-selected=move || (state.scheme.get() == id).then_some("true")
                                    on:click=move |_| state.set_scheme(id.to_string())
                                >
                                    {lbl}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div>
                        <div class="rp-label" style="margin-bottom:8px;">{move || state.t("ui.palette")}</div>
                        <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:8px;">
                            {PALETTES.iter().map(|&(id, label)| view! {
                                <button
                                    type="button"
                                    class=move || if state.palette.get() == id { "rp-btn accent sm" } else { "rp-btn secondary sm" }
                                    on:click=move |_| state.set_palette(id.to_string())
                                >
                                    {label}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </section>

            // Interactive Runtime Settings Card (`PATCH /api/v1/runtime/settings`) — instant click-to-apply + ring buffer controls!
            <section class="rp-card" id="runtime">
                <div class="rp-between" style="margin-bottom:10px;flex-wrap:wrap;gap:8px;">
                    <div class="rp-cluster">
                        <h2 class="rp-h3" style="margin:0;">{move || state.t("settings.runtime")}</h2>
                        <span
                            class="rp-badge"
                            style=move || {
                                let is_rt = rt_settings
                                    .get()
                                    .and_then(|rs| rs.get("source").and_then(Value::as_str).map(|s| s == "runtime"))
                                    .unwrap_or(false);
                                if is_rt {
                                    "background:color-mix(in srgb,var(--rp-accent) 22%,transparent);color:var(--rp-text);"
                                } else {
                                    ""
                                }
                            }
                        >
                            {move || {
                                match rt_settings
                                    .get()
                                    .and_then(|rs| rs.get("source").and_then(Value::as_str).map(ToString::to_string))
                                    .as_deref()
                                {
                                    Some("runtime") => "临时覆盖生效中",
                                    Some(_) => "配置文件默认",
                                    None => "加载中…",
                                }
                            }}
                        </span>
                    </div>
                    <button
                        type="button"
                        class="rp-btn quiet sm"
                        disabled=move || rt_saving.get()
                        on:click=move |_| {
                            let cur = rt_settings.get_untracked();
                            let allow = |k: &str| {
                                cur.as_ref()
                                    .and_then(|rs| rs.get("recording")?.get(k)?.get("allowed")?.as_bool())
                                    .unwrap_or(true)
                            };
                            let mf = if allow("flows") { "auto" } else { "off" };
                            let ml = if allow("logs") { "auto" } else { "off" };
                            let md = if allow("dns_log") { "auto" } else { "off" };
                            rt_level.set("info".to_string());
                            rt_rec_flows.set(mf.to_string());
                            rt_rec_logs.set(ml.to_string());
                            rt_rec_dns.set(md.to_string());
                            rt_log_buf.set("512".to_string());
                            rt_dns_max.set("512".to_string());
                            rt_flow_max.set("1024".to_string());
                            rt_flow_ret.set("300".to_string());
                            let mut patch = serde_json::Map::new();
                            patch.insert("record_flows".into(), json!(mf));
                            patch.insert("record_logs".into(), json!(ml));
                            patch.insert("record_dns_log".into(), json!(md));
                            if allow("logs") {
                                patch.insert("log".into(), json!({ "level": "info", "buffered_records": 512 }));
                            }
                            if allow("dns_log") {
                                patch.insert("dns_log".into(), json!({ "max_records": 512 }));
                            }
                            if allow("flows") {
                                patch.insert("flows".into(), json!({ "max_flows": 1024, "retention_seconds": 300 }));
                            }
                            apply_rt_patch(Value::Object(patch));
                        }
                    >
                        "恢复默认"
                    </button>
                </div>

                <form on:submit=save_runtime_settings style="display:flex;flex-direction:column;gap:12px;">
                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">"实时日志级别"</span>
                        <div class="rp-seg" data-size="M" style="flex-wrap:wrap;">
                            {["trace", "debug", "info", "warn", "error"].into_iter().map(|lv| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    disabled=move || rt_saving.get()
                                    data-selected=move || (rt_level.get() == lv).then_some("true")
                                    on:click=move |_| {
                                        rt_level.set(lv.to_string());
                                        apply_rt_patch(json!({ "log": { "level": lv } }));
                                    }
                                >
                                    {lv.to_uppercase()}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">"连接流记录策略 (Flows)"</span>
                        <div class="rp-seg" data-size="M">
                            {[("auto", "按需 Auto"), ("on", "常开 On"), ("off", "关闭 Off")].into_iter().map(|(m, lbl)| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    disabled=move || {
                                        let allowed = rt_settings
                                            .get()
                                            .and_then(|rs| rs.get("recording")?.get("flows")?.get("allowed")?.as_bool())
                                            .unwrap_or(true);
                                        rt_saving.get() || (m == "on" && !allowed)
                                    }
                                    data-selected=move || (rt_rec_flows.get() == m).then_some("true")
                                    on:click=move |_| {
                                        rt_rec_flows.set(m.to_string());
                                        apply_rt_patch(json!({ "record_flows": m }));
                                    }
                                >
                                    {lbl}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">"运行日志记录策略 (Logs)"</span>
                        <div class="rp-seg" data-size="M">
                            {[("auto", "按需 Auto"), ("on", "常开 On"), ("off", "关闭 Off")].into_iter().map(|(m, lbl)| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    disabled=move || {
                                        let allowed = rt_settings
                                            .get()
                                            .and_then(|rs| rs.get("recording")?.get("logs")?.get("allowed")?.as_bool())
                                            .unwrap_or(true);
                                        rt_saving.get() || (m == "on" && !allowed)
                                    }
                                    data-selected=move || (rt_rec_logs.get() == m).then_some("true")
                                    on:click=move |_| {
                                        rt_rec_logs.set(m.to_string());
                                        apply_rt_patch(json!({ "record_logs": m }));
                                    }
                                >
                                    {lbl}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                        <span class="rp-label">"DNS 日志记录策略 (DNS Log)"</span>
                        <div class="rp-seg" data-size="M">
                            {[("auto", "按需 Auto"), ("on", "常开 On"), ("off", "关闭 Off")].into_iter().map(|(m, lbl)| view! {
                                <button
                                    type="button"
                                    class="rp-btn"
                                    data-size="M"
                                    disabled=move || {
                                        let allowed = rt_settings
                                            .get()
                                            .and_then(|rs| rs.get("recording")?.get("dns_log")?.get("allowed")?.as_bool())
                                            .unwrap_or(true);
                                        rt_saving.get() || (m == "on" && !allowed)
                                    }
                                    data-selected=move || (rt_rec_dns.get() == m).then_some("true")
                                    on:click=move |_| {
                                        rt_rec_dns.set(m.to_string());
                                        apply_rt_patch(json!({ "record_dns_log": m }));
                                    }
                                >
                                    {lbl}
                                </button>
                            }).collect_view()}
                        </div>
                    </div>

                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(148px,1fr));gap:10px;">
                        <label class="rp-field">
                            <span class="rp-label">"日志缓冲条数 (64–512)"</span>
                            <input class="rp-input" type="number" min="64" max="512" prop:value=move || rt_log_buf.get() on:input=move |ev| rt_log_buf.set(event_target_value(&ev)) />
                        </label>
                        <label class="rp-field">
                            <span class="rp-label">"DNS 日志条数 (64–512)"</span>
                            <input class="rp-input" type="number" min="64" max="512" prop:value=move || rt_dns_max.get() on:input=move |ev| rt_dns_max.set(event_target_value(&ev)) />
                        </label>
                        <label class="rp-field">
                            <span class="rp-label">"流记录上限 (64–1024)"</span>
                            <input class="rp-input" type="number" min="64" max="1024" prop:value=move || rt_flow_max.get() on:input=move |ev| rt_flow_max.set(event_target_value(&ev)) />
                        </label>
                        <label class="rp-field">
                            <span class="rp-label">"流保留秒数 (1–300)"</span>
                            <input class="rp-input" type="number" min="1" max="300" prop:value=move || rt_flow_ret.get() on:input=move |ev| rt_flow_ret.set(event_target_value(&ev)) />
                        </label>
                    </div>

                    <div class="rp-cluster">
                        <button type="submit" class="rp-btn accent sm" disabled=move || rt_saving.get()>
                            "应用缓冲参数"
                        </button>
                    </div>
                </form>
            </section>

            // Structured Geodata Card (`/api/v1/geodata` + `/api/v1/geodata/update`)
            <section class="rp-card" id="geodata">
                <div class="rp-between" style="margin-bottom:10px;">
                    <h2 class="rp-h3">{move || state.t("settings.geodata")}</h2>
                    <button
                        type="button"
                        class="rp-btn accent sm"
                        disabled=move || geo_busy.get()
                        on:click=move |_| {
                            geo_busy.set(true);
                            let api = state.client();
                            spawn_local(async move {
                                match api.update_geodata().await {
                                    Ok(op) => state.poll_operation(op.operation_id, state.t("ov.succeeded")),
                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                }
                                geo_busy.set(false);
                            });
                        }
                    >
                        "立即更新数据库"
                    </button>
                </div>
                {move || match geodata.get() {
                    None => view! { <p class="rp-label">{state.t("ov.loading")}</p> }.into_any(),
                    Some(g) => {
                        let assets: Vec<(String, Value)> = if let Some(arr) = g.get("assets").or_else(|| g.get("files")).and_then(Value::as_array) {
                            arr.iter()
                                .enumerate()
                                .map(|(i, item)| {
                                    let k = item.get("kind").or_else(|| item.get("name")).and_then(Value::as_str).unwrap_or(if i == 0 { "geoip" } else { "geosite" }).to_string();
                                    (k, item.clone())
                                })
                                .collect()
                        } else {
                            ["geoip", "geosite"]
                                .into_iter()
                                .filter_map(|k| g.get(k).map(|v| (k.to_uppercase(), v.clone())))
                                .collect()
                        };
                        if assets.is_empty() {
                            return view! { <p class="rp-empty">{state.t("ui.empty")}</p> }.into_any();
                        }
                        view! {
                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:10px;">
                                {assets.into_iter().map(|(title, item)| {
                                    let present = item.get("present").or_else(|| item.get("exists")).and_then(Value::as_bool).unwrap_or(true);
                                    let size = value_u64(&item.get("size_bytes").cloned())
                                        .or_else(|| value_u64(&item.get("bytes").cloned()))
                                        .map(format_bytes)
                                        .unwrap_or_else(|| "—".into());
                                    let sha = item.get("sha256").and_then(Value::as_str).map(|s| s.chars().take(12).collect::<String>()).unwrap_or_else(|| "—".into());
                                    let mtime = item.get("modified_at").or_else(|| item.get("updated_at")).and_then(Value::as_str).unwrap_or("—").to_string();
                                    view! {
                                        <div class="rp-card" style="background:var(--rp-surface);">
                                            <div class="rp-between" style="margin-bottom:6px;">
                                                <strong>{title.to_uppercase()}</strong>
                                                <StatusLight tone=if present { "ok" } else { "warn" } text=if present { "已就绪".into() } else { "缺失".into() } />
                                            </div>
                                            <div class="rp-kv">
                                                <div><span class="k">"文件大小"</span><span class="v">{size}</span></div>
                                                <div><span class="k">"SHA-256"</span><span class="v"><code>{sha}</code></span></div>
                                                <div><span class="k">"更新时间"</span><span class="v">{mtime}</span></div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </section>

            // Backend Connection Card
            <section class="rp-card" id="backend">
                <h2 class="rp-h3" style="margin-bottom:10px;">{move || state.t("settings.backend")}</h2>
                <form
                    class="rp-form"
                    style="display:flex;flex-direction:column;gap:10px;"
                    on:submit=move |ev: web_sys::SubmitEvent| {
                        ev.prevent_default();
                        state.set_backend(base_input.get_untracked(), Some(token_input.get_untracked()));
                        state.toast("positive", state.t("ui.saved"), None);
                    }
                >
                    <label class="rp-field">
                        <span class="rp-label">"API Base URL"</span>
                        <input
                            class="rp-input"
                            placeholder="(same origin)"
                            prop:value=move || base_input.get()
                            on:input=move |ev| base_input.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="rp-field">
                        <span class="rp-label">{move || state.t("login.token")}</span>
                        <input
                            class="rp-input"
                            type="password"
                            prop:value=move || token_input.get()
                            on:input=move |ev| token_input.set(event_target_value(&ev))
                        />
                    </label>
                    <div class="rp-cluster">
                        <button type="submit" class="rp-btn accent sm">{move || state.t("config.save")}</button>
                        <button
                            type="button"
                            class="rp-btn negative sm"
                            on:click=move |_| {
                                let api = state.client();
                                spawn_local(async move {
                                    let _ = api.logout().await;
                                    state.set_backend(state.api_base.get_untracked(), None);
                                });
                            }
                        >
                            {move || state.t("settings.signOut")}
                        </button>
                    </div>
                </form>
            </section>
        </div>
    }
}

#[component]
fn CoreUpdateCard() -> impl IntoView {
    let state = use_app_state();
    let status = RwSignal::new(None::<CoreUpdateStatus>);
    let checking = RwSignal::new(false);
    let applying = RwSignal::new(false);
    let confirm_open = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let load = move |refresh: bool, announce: bool| {
        if refresh {
            checking.set(true);
        }
        error.set(None);
        let api = state.client();
        spawn_local(async move {
            match api.core_update_status(refresh).await {
                Ok(next) => {
                    if announce
                        && let Some(ref latest) = next.latest_version
                    {
                        if next.has_update {
                            state.toast(
                                "neutral",
                                state.t_args("settings.coreUpdateAvailable", &[("latest", latest)]),
                                None,
                            );
                        } else {
                            state.toast(
                                "positive",
                                state.t_args(
                                    "settings.coreUpdateUpToDateToast",
                                    &[("version", &next.current_version)],
                                ),
                                None,
                            );
                        }
                    }
                    status.set(Some(next));
                }
                Err(e) => {
                    error.set(Some(e.clone()));
                    if refresh {
                        state.toast("negative", state.t("settings.coreUpdateCheckFailed"), Some(e));
                    }
                }
            }
            checking.set(false);
        });
    };

    Effect::new(move |_| {
        if state
            .capabilities
            .get()
            .is_some_and(|c| c.core_update_available())
        {
            load(false, false);
        }
    });

    let apply_update = move |_| {
        confirm_open.set(false);
        let cur = status.get_untracked();
        let force = cur
            .as_ref()
            .is_some_and(|s| !s.has_update && s.latest_version.is_some());
        applying.set(true);
        error.set(None);
        let api = state.client();
        spawn_local(async move {
            match api.apply_core_update(force).await {
                Ok(res) => {
                    let ver = res
                        .installed_version
                        .or(res.current_version)
                        .unwrap_or_default();
                    state.toast(
                        "positive",
                        state.t_args("settings.coreUpdateDone", &[("version", &ver)]),
                        None,
                    );
                    state.refresh_all();
                    load(false, false);
                }
                Err(e) => {
                    error.set(Some(e.clone()));
                    state.toast("negative", state.t("settings.coreUpdateApplyFailed"), Some(e));
                }
            }
            applying.set(false);
        });
    };

    view! {
        <Show when=move || state.capabilities.get().is_some_and(|c| c.core_update_available())>
            <section class="rp-card" id="coreUpdate">
                <h2 class="rp-h3" style="margin-bottom:6px;">{move || state.t("settings.coreUpdate")}</h2>
                <p class="rp-label" style="margin-bottom:10px;">
                    {move || {
                        let repo = status
                            .get()
                            .map(|s| s.repo)
                            .unwrap_or_else(|| "Wcoom/honk-android".to_string());
                        state.t_args("settings.coreUpdateNote", &[("repo", &repo)])
                    }}
                </p>
                {move || error.get().map(|e| view! { <div class="rp-alert negative">{e}</div> })}
                {move || status.get().map(|s| {
                    let short_rev: String = s.current_revision.chars().take(12).collect();
                    let cur_display = if short_rev.is_empty() {
                        s.current_version.clone()
                    } else {
                        format!("{} · {short_rev}", s.current_version)
                    };
                    let size_str = value_u64(&s.asset_size_bytes).map(format_bytes);
                    let sha_short = s.asset_sha256.as_ref().map(|h| h.chars().take(12).collect::<String>());
                    let status_tone: &'static str = if s.latest_version.is_none() {
                        "muted"
                    } else if s.has_update {
                        "warn"
                    } else {
                        "ok"
                    };
                    let status_text = match s.latest_version.as_deref() {
                        Some(lat) if s.has_update => state.t_args("settings.coreUpdateAvailable", &[("latest", lat)]),
                        Some(_) => state.t_args("settings.coreUpdateUpToDate", &[("version", &s.current_version)]),
                        None => state.t("settings.coreUpdateUnchecked"),
                    };
                    view! {
                        <div style="display:flex;flex-direction:column;gap:10px;margin-bottom:12px;">
                            <div class="rp-cluster">
                                <StatusLight tone=status_tone text=status_text />
                                {s.release_url.clone().map(|url| view! {
                                    <a class="rp-link" href=url target="_blank" rel="noopener noreferrer">
                                        {state.t("settings.coreUpdateOpenRelease")}
                                    </a>
                                })}
                            </div>
                            <dl class="rp-kv detail">
                                <div><dt class="k">{state.t("settings.coreUpdateCurrent")}</dt><dd class="v">{cur_display}</dd></div>
                                <div><dt class="k">{state.t("settings.coreUpdateTarget")}</dt><dd class="v">{s.target.clone()}</dd></div>
                                <div><dt class="k">{state.t("settings.coreUpdateAsset")}</dt><dd class="v">{s.asset_name.clone()}</dd></div>
                                <div><dt class="k">{state.t("settings.coreUpdateLatest")}</dt><dd class="v">{s.latest_version.clone().unwrap_or_else(|| state.t("settings.coreUpdateUnchecked"))}</dd></div>
                                <div><dt class="k">{state.t("settings.coreUpdateCheckedAt")}</dt><dd class="v">{s.checked_at.clone().unwrap_or_else(|| state.t("settings.coreUpdateUnchecked"))}</dd></div>
                                {size_str.map(|sz| view! { <div><dt class="k">{state.t("settings.coreUpdateSize")}</dt><dd class="v">{sz}</dd></div> })}
                                {sha_short.map(|h| view! { <div><dt class="k">{state.t("settings.coreUpdateSha256")}</dt><dd class="v"><code>{h}</code></dd></div> })}
                            </dl>
                            {s.last_error.map(|le| view! {
                                <div class="rp-geodata-note negative">{le.message}</div>
                            })}
                        </div>
                    }
                })}
                <div class="rp-cluster">
                    <button
                        type="button"
                        class="rp-btn secondary sm"
                        disabled=move || checking.get() || applying.get()
                        on:click=move |_| load(true, true)
                    >
                        {move || if checking.get() { state.t("settings.coreUpdateChecking") } else { state.t("settings.coreUpdateCheck") }}
                    </button>
                    <button
                        type="button"
                        class="rp-btn accent sm"
                        disabled=move || {
                            let st = status.get();
                            checking.get()
                                || applying.get()
                                || !st.as_ref().is_some_and(|s| s.can_apply && s.latest_version.is_some())
                        }
                        on:click=move |_| confirm_open.set(true)
                    >
                        {move || {
                            let st = status.get();
                            if applying.get() {
                                state.t("settings.coreUpdateApplying")
                            } else if st.as_ref().is_some_and(|s| !s.has_update && s.latest_version.is_some()) {
                                state.t("settings.coreUpdateReinstall")
                            } else {
                                state.t("settings.coreUpdateApply")
                            }
                        }}
                    </button>
                </div>

                <Show when=move || confirm_open.get()>
                    <div class="rp-ios-sheet-backdrop" on:click=move |_| confirm_open.set(false)>
                        <div class="rp-dialog rp-card rp-ios-sheet" role="dialog" aria-modal="true" style="width:min(440px,92vw);padding:20px;display:flex;flex-direction:column;gap:12px;" on:click=move |ev| ev.stop_propagation()>
                            <h3 class="rp-h3">{move || state.t("settings.coreUpdateConfirmTitle")}</h3>
                            <p class="rp-label">
                                {move || {
                                    let s = status.get().unwrap_or_default();
                                    let lat = s.latest_version.unwrap_or_else(|| s.current_version.clone());
                                    state.t_args(
                                        "settings.coreUpdateConfirmHelp",
                                        &[("latest", &lat), ("asset", &s.asset_name), ("path", &s.executable_path)],
                                    )
                                }}
                            </p>
                            <div class="rp-ios-sheet-actions">
                                <button type="button" class="rp-btn secondary sm" on:click=move |_| confirm_open.set(false)>
                                    {move || state.t("ui.cancel")}
                                </button>
                                <button type="button" class="rp-btn accent sm" on:click=apply_update>
                                    {move || state.t("settings.coreUpdateApply")}
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </section>
        </Show>
    }
}
