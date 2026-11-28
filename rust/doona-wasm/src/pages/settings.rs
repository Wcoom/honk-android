//! Settings Hub pages: Multi-source `.dae` Config Editor (with pure-Rust syntax highlighting,
//! validate, save, import/export, revisions) & Settings Page (CoreUpdateCard, Appearance 16 palettes,
//! Language, Backend profile, Geodata, Runtime Settings).

use doona_common::format::format_bytes;
use doona_common::i18n::Lang;
use doona_common::model::{ConfigRevisionList, CoreUpdateStatus, value_u64};
use doona_common::palettes::PALETTES;
use leptos::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen_futures::spawn_local;

use crate::mode_dae::highlight_dae;
use crate::state::use_app_state;
use crate::ui::StatusLight;

#[component]
pub fn ConfigPage() -> impl IntoView {
    let state = use_app_state();
    let selected_id = RwSignal::new(String::new());
    let draft = RwSignal::new(String::new());
    let preview_mode = RwSignal::new(false);
    let validation = RwSignal::new(None::<Value>);
    let revisions = RwSignal::new(None::<ConfigRevisionList>);
    let busy = RwSignal::new(false);

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
                if cur != src.id {
                    selected_id.set(src.id.clone());
                }
                if let Some(ref text) = src.content {
                    draft.set(text.clone());
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
                        .map(|s| {
                            let content = if s.id == sid {
                                text.clone()
                            } else {
                                s.content.unwrap_or_default()
                            };
                            json!({
                                "path": s.path.unwrap_or_else(|| "config.dae".into()),
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
                        Ok(_) => {
                            state.toast("positive", state.t("ui.saved"), None);
                            state.refresh_all();
                        }
                        Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
                    }
                }
                Err(e) => state.toast("negative", state.t("ui.writeFailed"), Some(e)),
            }
            busy.set(false);
        });
    };

    let on_import = move |_| {
        busy.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.import_config().await {
                Ok(_) => {
                    state.toast("positive", state.t("config.backup.imported"), None);
                    state.refresh_all();
                }
                Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
            }
            busy.set(false);
        });
    };

    view! {
        <div class="rp-page" style="display:flex;flex-direction:column;gap:12px;">
            <div class="rp-between" style="flex-wrap:wrap;gap:8px;">
                <select
                    class="rp-input"
                    prop:value=move || selected_id.get()
                    on:change=move |ev| {
                        let id = event_target_value(&ev);
                        selected_id.set(id.clone());
                        if let Some(cfg) = state.config.get_untracked()
                            && let Some(src) = cfg.sources.into_iter().find(|s| s.id == id)
                        {
                            draft.set(src.content.unwrap_or_default());
                        }
                    }
                >
                    {move || state.config.get().map(|c| c.sources.into_iter().map(|s| {
                        let label = s.path.clone().or(s.name.clone()).unwrap_or_else(|| s.id.clone());
                        view! { <option value=s.id>{label}</option> }
                    }).collect_view())}
                </select>
                <div class="rp-cluster">
                    <button type="button" class="rp-btn secondary sm" on:click=move |_| preview_mode.update(|v| *v = !*v)>
                        {move || if preview_mode.get() { "Edit" } else { "Highlight" }}
                    </button>
                    <button type="button" class="rp-btn secondary sm" disabled=move || busy.get() on:click=on_validate>
                        {move || state.t("config.validate")}
                    </button>
                    <button type="button" class="rp-btn secondary sm" disabled=move || busy.get() on:click=on_import>
                        {move || state.t("config.backup.import")}
                    </button>
                    <button type="button" class="rp-btn accent sm" disabled=move || busy.get() on:click=on_save>
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

            {move || validation.get().map(|v| view! {
                <section class="rp-card">
                    <h3 class="rp-h3" style="margin-bottom:8px;">{state.t("config.validate")}</h3>
                    <pre class="rp-limit-snippet" style="overflow:auto;max-height:240px;">{serde_json::to_string_pretty(&v).unwrap_or_default()}</pre>
                </section>
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
                                                    Ok(_) => {
                                                        state.toast("positive", state.t("ui.saved"), None);
                                                        state.refresh_all();
                                                    }
                                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                                }
                                            });
                                        }
                                    >
                                        {if is_active { "Active" } else { "Activate" }}
                                    </button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </section>
            })}
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let state = use_app_state();
    let base_input = RwSignal::new(state.api_base.get_untracked());
    let token_input = RwSignal::new(state.api_token.get_untracked().unwrap_or_default());
    let geodata = RwSignal::new(None::<Value>);
    let rt_settings = RwSignal::new(None::<Value>);

    Effect::new(move |_| {
        let _ = state.reload_nonce.get();
        let api = state.client();
        spawn_local(async move {
            if let Ok(g) = api.geodata().await {
                geodata.set(Some(g));
            }
            if let Ok(rs) = api.runtime_settings().await {
                rt_settings.set(Some(rs));
            }
        });
    });

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

            // Geodata Card
            <section class="rp-card" id="geodata">
                <div class="rp-between" style="margin-bottom:8px;">
                    <h2 class="rp-h3">{move || state.t("settings.geodata")}</h2>
                    <button
                        type="button"
                        class="rp-btn secondary sm"
                        on:click=move |_| {
                            let api = state.client();
                            spawn_local(async move {
                                match api.update_geodata().await {
                                    Ok(_) => state.toast("positive", state.t("ov.succeeded"), None),
                                    Err(e) => state.toast("negative", state.t("ui.failed"), Some(e)),
                                }
                            });
                        }
                    >
                        {move || state.t("ui.refresh")}
                    </button>
                </div>
                {move || geodata.get().map(|g| view! {
                    <pre class="rp-limit-snippet" style="overflow:auto;max-height:200px;">{serde_json::to_string_pretty(&g).unwrap_or_default()}</pre>
                })}
            </section>

            // Runtime Settings Card
            {move || rt_settings.get().map(|rs| view! {
                <section class="rp-card" id="runtime">
                    <h2 class="rp-h3" style="margin-bottom:8px;">{move || state.t("settings.runtime")}</h2>
                    <pre class="rp-limit-snippet" style="overflow:auto;max-height:220px;">{serde_json::to_string_pretty(&rs).unwrap_or_default()}</pre>
                </section>
            })}
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
                    <div class="rp-modal" style="position:fixed;inset:0;z-index:900;background:rgba(0,0,0,0.45);display:flex;align-items:center;justify-content:center;">
                        <div class="rp-dialog rp-card" role="dialog" aria-modal="true" style="width:min(440px,92vw);padding:20px;display:flex;flex-direction:column;gap:12px;">
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
                            <div class="rp-cluster" style="justify-content:flex-end;">
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
