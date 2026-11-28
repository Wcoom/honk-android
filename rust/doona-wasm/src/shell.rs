//! Doona shell frame: Login gate, TopBar, SideNav, PageHead + HubNav, Main content router, and mobile HubBar.

use doona_common::i18n::Lang;
use doona_common::palettes::PALETTES;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::pages;
use crate::state::{AppState, AuthGate, HUBS, page_title_key, use_app_state};
use crate::ui::{AboutModal, SearchModal, ToastViewport, icons};

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);
    state.start_background_tasks();
    state.bootstrap();

    view! {
        <div class="rp-app-root">
            {move || match state.auth_gate.get() {
                AuthGate::Checking => view! {
                    <div class="rp-login-page">
                        <main class="rp-login-main">
                            <section class="rp-card rp-login-card">
                                <div class="rp-login-brand">
                                    <img src="./logo.svg" alt="" style="width:36px;height:36px;" />
                                    <span class="rp-brand-text"><span>"doona"</span></span>
                                </div>
                                <p class="rp-label">{move || state.t("ov.loading")}</p>
                            </section>
                        </main>
                    </div>
                }.into_any(),
                AuthGate::LoginRequired { mode, setup_required, rejected } => view! {
                    <LoginScreen mode=mode setup_required=setup_required rejected=rejected />
                }.into_any(),
                AuthGate::Ready => view! {
                    <MainShell />
                }.into_any(),
            }}
            <SearchModal />
            <AboutModal />
            <ToastViewport />
        </div>
    }
}

#[component]
fn LoginScreen(mode: String, setup_required: bool, rejected: bool) -> impl IntoView {
    let state = use_app_state();
    let token = RwSignal::new(state.api_token.get_untracked().unwrap_or_default());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let err_msg = RwSignal::new(if rejected {
        Some(state.t("login.sessionEnded"))
    } else {
        None
    });

    let is_password = mode == "password";
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        busy.set(true);
        err_msg.set(None);
        if !is_password {
            let tok = token.get_untracked().trim().to_string();
            busy.set(false);
            state.set_backend(state.api_base.get_untracked(), Some(tok));
            return;
        }
        let u = username.get_untracked();
        let p = password.get_untracked();
        let api = state.client();
        spawn_local(async move {
            let res = if setup_required {
                api.setup_auth(&u, &p).await
            } else {
                api.login(&u, &p).await
            };
            busy.set(false);
            match res {
                Ok(session) => state.set_backend(state.api_base.get_untracked(), Some(session.token)),
                Err(e) => err_msg.set(Some(e)),
            }
        });
    };

    view! {
        <div class="rp-login-page">
            <header class="rp-login-controls">
                <select
                    class="rp-input"
                    aria-label=move || state.t("ui.lang")
                    prop:value=move || state.lang.get().code()
                    on:change=move |ev| state.set_lang(Lang::from_code(&event_target_value(&ev)))
                >
                    {Lang::ALL.iter().map(|&l| view! {
                        <option value=l.code()>{l.label()}</option>
                    }).collect_view()}
                </select>
                <button type="button" class="rp-btn quiet icon" on:click=move |_| state.toggle_scheme()>
                    {icons::theme_icon()}
                </button>
            </header>
            <main class="rp-login-main">
                <section class="rp-card rp-login-card">
                    <div class="rp-login-brand">
                        <img src="./logo.svg" alt="" style="width:36px;height:36px;" />
                        <span class="rp-brand-text"><span>"doona"</span></span>
                    </div>
                    <div class="rp-login-head">
                        <h1 class="rp-h1">{move || if setup_required { state.t("login.setupTitle") } else { state.t("login.signIn") }}</h1>
                    </div>
                    {move || err_msg.get().map(|m| view! { <div class="rp-alert negative" role="alert">{m}</div> })}
                    <form class="rp-login rp-form" on:submit=on_submit>
                        {if !is_password {
                            view! {
                                <label class="rp-field">
                                    <span class="rp-label">{move || state.t("login.token")}</span>
                                    <input
                                        class="rp-input"
                                        type="password"
                                        prop:value=move || token.get()
                                        on:input=move |ev| token.set(event_target_value(&ev))
                                    />
                                </label>
                            }.into_any()
                        } else {
                            view! {
                                <label class="rp-field">
                                    <span class="rp-label">{move || state.t("login.username")}</span>
                                    <input
                                        class="rp-input"
                                        type="text"
                                        prop:value=move || username.get()
                                        on:input=move |ev| username.set(event_target_value(&ev))
                                    />
                                </label>
                                <label class="rp-field">
                                    <span class="rp-label">{move || state.t("login.password")}</span>
                                    <input
                                        class="rp-input"
                                        type="password"
                                        prop:value=move || password.get()
                                        on:input=move |ev| password.set(event_target_value(&ev))
                                    />
                                </label>
                            }.into_any()
                        }}
                        <button type="submit" class="rp-btn accent rp-login-submit" disabled=move || busy.get()>
                            {move || if setup_required { state.t("login.create") } else { state.t("login.signIn") }}
                        </button>
                    </form>
                </section>
            </main>
        </div>
    }
}

#[component]
fn MainShell() -> impl IntoView {
    let state = use_app_state();
    let reloading = RwSignal::new(false);

    let on_reload = move |_| {
        reloading.set(true);
        let api = state.client();
        spawn_local(async move {
            match api.start_reload().await {
                Ok(_) => {
                    state.toast("positive", state.t("ov.succeeded"), None);
                    state.refresh_all();
                }
                Err(e) => state.toast("negative", state.t("ov.failed"), Some(e)),
            }
            reloading.set(false);
        });
    };

    view! {
        <div class="rp-shell">
            <header class="rp-top">
                <button
                    type="button"
                    class="rp-brand"
                    title=move || state.t("about.title")
                    on:click=move |_| state.about_open.set(true)
                >
                    <img src="./logo.svg" alt="doona" />
                    <span class="rp-brand-text">
                        <span>"doona"</span>
                        <span class="rp-brand-version">
                            {move || state.version.get().and_then(|v| v.engine).and_then(|e| e.version).unwrap_or_default()}
                        </span>
                    </span>
                </button>
                <div class="rp-search-wrap">
                    <button
                        type="button"
                        class="rp-search"
                        on:click=move |_| state.search_open.set(true)
                    >
                        {icons::search_icon()}
                        <span class="grow">{move || state.t("shell.search")}</span>
                        <span class="rp-kbd">"Ctrl+K"</span>
                    </button>
                </div>
                <div class="rp-actions">
                    <span class="rp-search-compact">
                        <button
                            type="button"
                            class="rp-btn quiet icon"
                            aria-label=move || state.t("shell.search")
                            on:click=move |_| state.search_open.set(true)
                        >
                            {icons::search_icon()}
                        </button>
                    </span>
                    <button
                        type="button"
                        class="rp-btn quiet icon"
                        aria-label=move || state.t("ui.refresh")
                        disabled=move || state.spinning.get()
                        on:click=move |_| state.refresh_all()
                    >
                        {icons::refresh_icon()}
                    </button>
                    <button
                        type="button"
                        class="rp-btn quiet sm"
                        disabled=move || reloading.get()
                        on:click=on_reload
                        title=move || state.t("ov.reload")
                    >
                        {move || state.t("ov.reload")}
                    </button>
                    <span class="rp-wide-only rp-cluster">
                        <select
                            class="rp-input"
                            aria-label=move || state.t("ui.lang")
                            style="height:30px;padding:0 8px;font-size:12px;"
                            prop:value=move || state.lang.get().code()
                            on:change=move |ev| state.set_lang(Lang::from_code(&event_target_value(&ev)))
                        >
                            {Lang::ALL.iter().map(|&l| view! {
                                <option value=l.code()>{l.label()}</option>
                            }).collect_view()}
                        </select>
                        <select
                            class="rp-input"
                            aria-label=move || state.t("ui.palette")
                            style="height:30px;padding:0 8px;font-size:12px;"
                            prop:value=move || state.palette.get()
                            on:change=move |ev| state.set_palette(event_target_value(&ev))
                        >
                            {PALETTES.iter().map(|&(id, label)| view! {
                                <option value=id>{label}</option>
                            }).collect_view()}
                        </select>
                        <button
                            type="button"
                            class="rp-btn quiet icon"
                            aria-label=move || state.t("ui.theme")
                            on:click=move |_| state.toggle_scheme()
                        >
                            {icons::theme_icon()}
                        </button>
                    </span>
                </div>
            </header>

            <nav class="rp-side" aria-label="Navigation">
                <div class="rp-side-links rp-overlay-scroll">
                    {HUBS.iter().map(|hub| {
                        view! {
                            <div data-group=hub.id>
                                <div class="rp-label" style="padding:8px 12px 4px;font-size:11px;font-weight:600;text-transform:uppercase;opacity:0.75;">
                                    {move || state.t(hub.title_key)}
                                </div>
                                {hub.pages.iter().map(|&p| {
                                    let href = format!("#{p}");
                                    view! {
                                        <a
                                            class="rp-nav"
                                            href=href
                                            aria-current=move || (state.route.get() == p).then_some("page")
                                        >
                                            {icons::hub_icon(hub.id)}
                                            <span>{move || state.t(page_title_key(p))}</span>
                                        </a>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    }).collect_view()}
                </div>
                <div style="padding:10px 12px;border-top:1px solid var(--rp-line);font-size:12px;">
                    <a class="rp-link" href="./lite/">"Doona Lite"</a>
                </div>
            </nav>

            <main class="rp-main">
                <div class="rp-content">
                    <div class="rp-head">
                        <div class="rp-page-heading">
                            <div class="rp-title">
                                <h1 class="rp-h1">{move || state.t(page_title_key(&state.route.get()))}</h1>
                            </div>
                        </div>
                        {move || {
                            let cur = state.route.get();
                            HUBS.iter().find(|h| h.pages.contains(&cur.as_str())).map(|hub| {
                                view! {
                                    <div class="rp-hubnav">
                                        <nav class="rp-page-links" aria-label=move || state.t(hub.title_key)>
                                            {hub.pages.iter().map(|&p| {
                                                let href = format!("#{p}");
                                                view! {
                                                    <a
                                                        class="rp-btn quiet"
                                                        href=href
                                                        aria-current=move || (state.route.get() == p).then_some("page")
                                                    >
                                                        {move || state.t(page_title_key(p))}
                                                    </a>
                                                }
                                            }).collect_view()}
                                        </nav>
                                    </div>
                                }
                            })
                        }}
                    </div>

                    {move || state.global_error.get().map(|err| view! {
                        <div class="rp-alert negative" role="alert" style="margin-bottom:12px;">
                            <span>{err}</span>
                            <button type="button" class="rp-btn sm secondary" on:click=move |_| state.refresh_all()>
                                {move || state.t("ui.retry")}
                            </button>
                        </div>
                    })}

                    {move || match state.route.get().as_str() {
                        "overview" => view! { <pages::overview::OverviewPage /> }.into_any(),
                        "connections" => view! { <pages::traffic::ConnectionsPage /> }.into_any(),
                        "flows" => view! { <pages::traffic::FlowsPage /> }.into_any(),
                        "dns" => view! { <pages::traffic::DnsPage /> }.into_any(),
                        "logs" => view! { <pages::traffic::LogsPage /> }.into_any(),
                        "events" => view! { <pages::traffic::EventsPage /> }.into_any(),
                        "policies" => view! { <pages::routing::PoliciesPage /> }.into_any(),
                        "nodes" => view! { <pages::routing::NodesPage /> }.into_any(),
                        "rules" => view! { <pages::routing::RulesPage /> }.into_any(),
                        "config" => view! { <pages::settings::ConfigPage /> }.into_any(),
                        "settings" => view! { <pages::settings::SettingsPage /> }.into_any(),
                        _ => view! { <pages::activity::ActivityPage /> }.into_any(),
                    }}
                </div>
            </main>

            <nav class="rp-hubbar" aria-label=move || state.t("shell.hubs")>
                {HUBS.iter().map(|hub| {
                    let first = hub.pages[0];
                    let href = format!("#{first}");
                    view! {
                        <a
                            href=href
                            aria-current=move || {
                                let cur = state.route.get();
                                hub.pages.contains(&cur.as_str()).then_some("page")
                            }
                        >
                            <span class="rp-hubbar-pill">{icons::hub_icon(hub.id)}</span>
                            <span>{move || state.t(hub.title_key)}</span>
                        </a>
                    }
                }).collect_view()}
            </nav>
        </div>
    }
}
