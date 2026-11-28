//! Pure Rust WASM (Leptos 0.8 CSR) frontend for Doona Lite (`/ui/lite/`).
//! Covers Connect, Shell, all 5 screens (`groups`, `subscriptions`, `nodes`, `routing`, `dns`),
//! Command Palette (`Ctrl+K`), Settings & CoreUpdate dialog, Draft Review bar, RuleTester, and pure-Rust SVG Share QR.

pub mod qr;

use std::collections::HashMap;

use doona_common::client::ApiClient;
use doona_common::format::{format_bytes, format_latency};
use doona_common::i18n::{self, Lang};
use doona_common::model::{
    Capabilities, CoreUpdateStatus, EffectiveConfig, GroupDetail, GroupSummary, NodeList,
    ProviderItem, RuntimeSnapshot, VersionInfo,
};
use leptos::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

const SCREENS: &[&str] = &["groups", "nodes", "subscriptions", "routing", "dns"];

fn parse_screen(hash: &str) -> String {
    let s = hash.trim_start_matches('#').trim_start_matches('/');
    if SCREENS.contains(&s) {
        s.to_string()
    } else {
        "groups".to_string()
    }
}

fn read_ls(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        doona_common::palettes::browser::read_storage(key)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        None
    }
}

fn write_ls(key: &str, val: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        doona_common::palettes::browser::write_storage(key, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, val);
    }
}

fn apply_lite_theme(lang: Lang, mode: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window()
            && let Some(doc) = w.document()
            && let Some(root) = doc.document_element()
        {
            let _ = root.set_attribute("lang", lang.code());
            let _ = root.set_attribute("data-theme", mode);
            let prefers_dark = w
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .is_some_and(|m| m.matches());
            let scheme = match mode {
                "dark" => "dark",
                "light" => "light",
                _ if prefers_dark => "dark",
                _ => "light",
            };
            let _ = root.set_attribute("data-color-mode", scheme);
            let _ = root.set_attribute("data-scheme", scheme);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (lang, mode);
    }
}

#[derive(Clone, Copy)]
struct LiteState {
    lang: RwSignal<Lang>,
    theme: RwSignal<String>,
    screen: RwSignal<String>,
    api_base: RwSignal<String>,
    api_token: RwSignal<Option<String>>,
    connected: RwSignal<bool>,
    checking: RwSignal<bool>,
    version: RwSignal<Option<VersionInfo>>,
    capabilities: RwSignal<Option<Capabilities>>,
    runtime: RwSignal<Option<RuntimeSnapshot>>,
    groups: RwSignal<Vec<GroupSummary>>,
    group_details: RwSignal<HashMap<String, GroupDetail>>,
    nodes: RwSignal<Option<NodeList>>,
    providers: RwSignal<Vec<ProviderItem>>,
    config: RwSignal<Option<EffectiveConfig>>,
    notice: RwSignal<Option<(String, String)>>,
    commands_open: RwSignal<bool>,
    settings_open: RwSignal<bool>,
    share_payload: RwSignal<Option<(String, String)>>,
    draft_source_id: RwSignal<Option<String>>,
    draft_text: RwSignal<Option<String>>,
    review_open: RwSignal<bool>,
}

impl LiteState {
    fn new() -> Self {
        let lang = read_ls("doona-lite.locale")
            .map(|c| Lang::from_code(&c))
            .unwrap_or(Lang::ZhCn);
        let theme = read_ls("doona-lite.theme").unwrap_or_else(|| "system".to_string());
        let base = read_ls("doona-lite.apiBase").unwrap_or_default();
        let token = read_ls("doona-lite.token").filter(|s| !s.trim().is_empty());
        let screen = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .map(|h| parse_screen(&h))
            .unwrap_or_else(|| "groups".to_string());
        apply_lite_theme(lang, &theme);
        Self {
            lang: RwSignal::new(lang),
            theme: RwSignal::new(theme),
            screen: RwSignal::new(screen),
            api_base: RwSignal::new(base),
            api_token: RwSignal::new(token),
            connected: RwSignal::new(false),
            checking: RwSignal::new(true),
            version: RwSignal::new(None),
            capabilities: RwSignal::new(None),
            runtime: RwSignal::new(None),
            groups: RwSignal::new(Vec::new()),
            group_details: RwSignal::new(HashMap::new()),
            nodes: RwSignal::new(None),
            providers: RwSignal::new(Vec::new()),
            config: RwSignal::new(None),
            notice: RwSignal::new(None),
            commands_open: RwSignal::new(false),
            settings_open: RwSignal::new(false),
            share_payload: RwSignal::new(None),
            draft_source_id: RwSignal::new(None),
            draft_text: RwSignal::new(None),
            review_open: RwSignal::new(false),
        }
    }

    fn client(&self) -> ApiClient {
        ApiClient::new(self.api_base.get_untracked(), self.api_token.get_untracked())
    }

    fn t(&self, key: &str) -> String {
        i18n::tr(self.lang.get(), key)
    }

    fn label_screen(&self, s: &str) -> String {
        match (self.lang.get(), s) {
            (Lang::En, "groups") => "Groups".into(),
            (Lang::En, "nodes") => "Nodes".into(),
            (Lang::En, "subscriptions") => "Subscriptions".into(),
            (Lang::En, "routing") => "Routing".into(),
            (Lang::En, "dns") => "DNS".into(),
            (Lang::ZhTw, "groups") => "策略組".into(),
            (Lang::ZhTw, "nodes") => "節點".into(),
            (Lang::ZhTw, "subscriptions") => "訂閱".into(),
            (Lang::ZhTw, "routing") => "路由".into(),
            (Lang::ZhTw, "dns") => "DNS".into(),
            (_, "groups") => "策略组".into(),
            (_, "nodes") => "节点".into(),
            (_, "subscriptions") => "订阅".into(),
            (_, "routing") => "路由".into(),
            _ => "DNS".into(),
        }
    }

    fn flash(&self, tone: &str, msg: impl Into<String>) {
        self.notice.set(Some((tone.to_string(), msg.into())));
    }

    fn set_lang(&self, next: Lang) {
        self.lang.set(next);
        write_ls("doona-lite.locale", next.code());
        apply_lite_theme(next, &self.theme.get_untracked());
    }

    fn set_theme(&self, next: String) {
        self.theme.set(next.clone());
        write_ls("doona-lite.theme", &next);
        apply_lite_theme(self.lang.get_untracked(), &next);
    }

    fn go(&self, s: &str) {
        if let Some(w) = web_sys::window() {
            let _ = w.location().set_hash(&format!("#{s}"));
        }
        self.screen.set(s.to_string());
    }

    fn try_connect(&self) {
        let st = *self;
        st.checking.set(true);
        spawn_local(async move {
            let api = st.client();
            match api.capabilities().await {
                Ok(caps) => {
                    st.capabilities.set(Some(caps));
                    st.connected.set(true);
                    st.checking.set(false);
                    st.reload_data();
                }
                Err(_) => {
                    st.connected.set(false);
                    st.checking.set(false);
                }
            }
        });
    }

    fn reload_data(&self) {
        let st = *self;
        spawn_local(async move {
            let api = st.client();
            if let Ok(v) = api.version().await {
                st.version.set(Some(v));
            }
            if let Ok(rt) = api.runtime().await {
                st.runtime.set(Some(rt));
            }
            if let Ok(gs) = api.groups().await {
                let ids: Vec<String> = gs.iter().map(|g| g.id.clone()).collect();
                st.groups.set(gs);
                for gid in ids {
                    let api_c = api.clone();
                    spawn_local(async move {
                        if let Ok(det) = api_c.group(&gid).await {
                            st.group_details.update(|m| {
                                m.insert(gid, det);
                            });
                        }
                    });
                }
            }
            if let Ok(nl) = api.nodes().await {
                st.nodes.set(Some(nl));
            }
            if let Ok(pl) = api.providers().await {
                st.providers.set(pl.providers);
            }
            if let Ok(cfg) = api.config().await {
                st.config.set(Some(cfg));
            }
        });
    }

    fn start_listeners(&self) {
        let st = *self;
        if let Some(w) = web_sys::window() {
            let on_hash = Closure::wrap(Box::new(move || {
                if let Some(win) = web_sys::window()
                    && let Ok(h) = win.location().hash()
                {
                    st.screen.set(parse_screen(&h));
                }
            }) as Box<dyn FnMut()>);
            let _ = w.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());
            on_hash.forget();

            let on_key = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
                if (ev.ctrl_key() || ev.meta_key()) && ev.key().eq_ignore_ascii_case("k") {
                    ev.prevent_default();
                    st.commands_open.update(|v| *v = !*v);
                } else if ev.key() == "Escape" {
                    st.commands_open.set(false);
                    st.settings_open.set(false);
                    st.share_payload.set(None);
                    st.review_open.set(false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            let _ = w.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
            on_key.forget();
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let mount = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("root"));
    if let Some(el) = mount
        && let Ok(html) = el.dyn_into::<web_sys::HtmlElement>()
    {
        leptos::mount::mount_to(html, LiteApp).forget();
        return;
    }
    leptos::mount::mount_to_body(LiteApp);
}

#[component]
pub fn LiteApp() -> impl IntoView {
    let st = LiteState::new();
    provide_context(st);
    st.start_listeners();
    st.try_connect();

    view! {
        <div class="lite-app-root">
            {move || if st.checking.get() {
                view! { <div class="lite-center"><p>{move || st.t("ov.loading")}</p></div> }.into_any()
            } else if !st.connected.get() {
                view! { <ConnectScreen /> }.into_any()
            } else {
                view! { <LiteShell /> }.into_any()
            }}
            <CommandPalette />
            <SettingsModal />
            <ShareQrModal />
            <DraftBar />
        </div>
    }
}

#[component]
fn ConnectScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let addr = RwSignal::new(st.api_base.get_untracked());
    let token = RwSignal::new(st.api_token.get_untracked().unwrap_or_default());
    let err = RwSignal::new(None::<String>);

    let on_connect = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let base = addr.get_untracked().trim().trim_end_matches('/').to_string();
        let tok = token.get_untracked().trim().to_string();
        st.api_base.set(base.clone());
        st.api_token.set((!tok.is_empty()).then_some(tok.clone()));
        write_ls("doona-lite.apiBase", &base);
        write_ls("doona-lite.token", &tok);
        err.set(None);
        st.try_connect();
    };

    view! {
        <div class="lite-center">
            <section class="lite-card" style="width:min(400px,92vw);">
                <div style="display:flex;align-items:center;gap:10px;margin-bottom:14px;">
                    <img src="./logo.svg" alt="doona lite" style="width:36px;height:36px;" />
                    <h1 style="margin:0;font-size:20px;">"doona lite"</h1>
                </div>
                {move || err.get().map(|m| view! { <div class="lite-alert">{m}</div> })}
                <form on:submit=on_connect style="display:flex;flex-direction:column;gap:10px;">
                    <label class="lite-field">
                        <span>"Backend URL"</span>
                        <input class="lite-input" placeholder="(same origin)" prop:value=move || addr.get() on:input=move |ev| addr.set(event_target_value(&ev)) />
                    </label>
                    <label class="lite-field">
                        <span>{move || st.t("login.token")}</span>
                        <input class="lite-input" type="password" prop:value=move || token.get() on:input=move |ev| token.set(event_target_value(&ev)) />
                    </label>
                    <button type="submit" class="lite-btn primary">{move || st.t("login.signIn")}</button>
                </form>
            </section>
        </div>
    }
}

#[component]
fn LiteShell() -> impl IntoView {
    let st = expect_context::<LiteState>();
    view! {
        <div class="lite-shell">
            <aside class="lite-sidenav" aria-label="Navigation">
                <div class="lite-brand">
                    <img src="./logo.svg" alt="doona lite" style="width:28px;height:28px;" />
                    <strong>"doona lite"</strong>
                </div>
                <button type="button" class="lite-nav-btn" on:click=move |_| st.commands_open.set(true)>
                    <span>{move || st.t("shell.search")}</span>
                    <kbd>"Ctrl+K"</kbd>
                </button>
                <nav class="lite-nav-list">
                    {SCREENS.iter().map(|&s| {
                        let href = format!("#{s}");
                        view! {
                            <a
                                class="lite-nav-link"
                                href=href
                                aria-current=move || (st.screen.get() == s).then_some("page")
                            >
                                {move || st.label_screen(s)}
                            </a>
                        }
                    }).collect_view()}
                </nav>
                <div class="lite-sidenav-footer">
                    <div class="lite-status-row">
                        <span class="lite-dot"></span>
                        <span>{move || st.runtime.get().map(|r| r.state_str().to_string()).unwrap_or_else(|| "ready".into())}</span>
                    </div>
                    <div style="display:flex;gap:6px;flex-wrap:wrap;">
                        <button type="button" class="lite-btn sm" on:click=move |_| st.reload_data()>{move || st.t("ui.refresh")}</button>
                        <button type="button" class="lite-btn sm" on:click=move |_| st.settings_open.set(true)>{move || st.t("nav.settings")}</button>
                        <a class="lite-btn sm" href="../">"Full UI"</a>
                    </div>
                </div>
            </aside>

            <main class="lite-main">
                {move || st.notice.get().map(|(tone, msg)| view! {
                    <div class=format!("lite-toast {tone}") role="status">
                        <span>{msg}</span>
                        <button type="button" class="lite-btn sm" on:click=move |_| st.notice.set(None)>"×"</button>
                    </div>
                })}
                {move || match st.screen.get().as_str() {
                    "nodes" => view! { <NodesScreen /> }.into_any(),
                    "subscriptions" => view! { <SubscriptionsScreen /> }.into_any(),
                    "routing" => view! { <RoutingScreen /> }.into_any(),
                    "dns" => view! { <DnsScreen /> }.into_any(),
                    _ => view! { <GroupsScreen /> }.into_any(),
                }}
            </main>
        </div>
    }
}

#[component]
fn GroupsScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let selected_group = RwSignal::new(String::new());

    view! {
        <div class="lite-page">
            <div class="lite-head">
                <h1>{move || st.label_screen("groups")}</h1>
                <button type="button" class="lite-btn sm" on:click=move |_| st.reload_data()>{move || st.t("ui.refresh")}</button>
            </div>
            <div class="lite-grid">
                {move || {
                    let gs = st.groups.get();
                    let dets = st.group_details.get();
                    let nodes = st.nodes.get().map(|n| n.nodes).unwrap_or_default();
                    gs.into_iter().map(|g| {
                        let gid = g.id.clone();
                        let gid_probe = g.id.clone();
                        let det = dets.get(&gid).cloned();
                        let active_id = det
                            .as_ref()
                            .and_then(|d| d.runtime.selection.tcp.as_ref())
                            .and_then(|s| s.member_id.clone())
                            .or_else(|| g.selection.tcp_member_id.clone())
                            .unwrap_or_default();
                        let can_sel = det.as_ref().is_some_and(|d| d.capabilities.can_select || d.capabilities.can_override);
                        let is_open = selected_group.get().is_empty() || selected_group.get() == gid;
                        view! {
                            <section class="lite-card">
                                <div class="lite-between">
                                    <div>
                                        <strong>{g.name.clone()}</strong>
                                        " "
                                        <span class="lite-badge">{g.policy.kind.clone()}</span>
                                    </div>
                                    <button type="button" class="lite-btn sm" on:click=move |_| {
                                        let id = gid_probe.clone();
                                        let api = st.client();
                                        spawn_local(async move {
                                            match api.probe_group(&id).await {
                                                Ok(_) => {
                                                    st.flash("ok", st.t("pol.probeStarted"));
                                                    st.reload_data();
                                                }
                                                Err(e) => st.flash("err", e),
                                            }
                                        });
                                    }>{move || st.t("pol.probe")}</button>
                                </div>
                                {is_open.then(|| det.map(|d| view! {
                                    <div class="lite-member-grid">
                                        {d.members.into_iter().map(|m| {
                                            let active = m.id == active_id;
                                            let mid = m.id.clone();
                                            let gid_c = gid.clone();
                                            let lat = nodes.iter().find(|n| n.id == m.id).and_then(|n| n.best_latency_ms());
                                            view! {
                                                <button
                                                    type="button"
                                                    class=if active { "lite-member active" } else { "lite-member" }
                                                    disabled=!can_sel
                                                    on:click=move |_| {
                                                        let api = st.client();
                                                        let g_c = gid_c.clone();
                                                        let m_c = mid.clone();
                                                        spawn_local(async move {
                                                            match api.select_group(&g_c, &m_c, "both").await {
                                                                Ok(_) => st.reload_data(),
                                                                Err(e) => st.flash("err", e),
                                                            }
                                                        });
                                                    }
                                                >
                                                    <span>{m.name}</span>
                                                    <span class="lite-muted">{format_latency(lat)}</span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }))}
                            </section>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

#[component]
fn NodesScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let search = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let link = RwSignal::new(String::new());

    let add_node = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let n = name.get_untracked().trim().to_string();
        let l = link.get_untracked().trim().to_string();
        if n.is_empty() || l.is_empty() {
            return;
        }
        let api = st.client();
        spawn_local(async move {
            match api.create_node(&n, &l).await {
                Ok(_) => {
                    name.set(String::new());
                    link.set(String::new());
                    st.flash("ok", st.t("ui.saved"));
                    st.reload_data();
                }
                Err(e) => st.flash("err", e),
            }
        });
    };

    view! {
        <div class="lite-page">
            <div class="lite-head">
                <h1>{move || st.label_screen("nodes")}</h1>
                <input class="lite-input" type="search" placeholder=move || st.t("ui.search") prop:value=move || search.get() on:input=move |ev| search.set(event_target_value(&ev)) />
            </div>
            <section class="lite-card">
                <form on:submit=add_node style="display:flex;gap:8px;flex-wrap:wrap;">
                    <input class="lite-input" placeholder=move || st.t("ui.name") prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) />
                    <input class="lite-input" placeholder="vless://... / ss://..." style="flex:1;min-width:200px;" prop:value=move || link.get() on:input=move |ev| link.set(event_target_value(&ev)) />
                    <button type="submit" class="lite-btn primary sm">{move || st.t("ui.add")}</button>
                </form>
            </section>
            <section class="lite-card" style="overflow:auto;">
                <table class="lite-table">
                    <thead>
                        <tr>
                            <th>{move || st.t("ui.name")}</th>
                            <th>"Protocol"</th>
                            <th>{move || st.t("act.latency")}</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let q = search.get().to_lowercase();
                            st.nodes
                                .get()
                                .map(|l| l.nodes)
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|n| q.is_empty() || n.name.to_lowercase().contains(&q))
                                .take(200)
                                .map(|n| {
                                    let nid_p = n.id.clone();
                                    let nid_d = n.id.clone();
                                    let share_name = n.name.clone();
                                    let share_val = format!("honk://node/{}", n.id);
                                    let lat = format_latency(n.best_latency_ms());
                                    view! {
                                        <tr>
                                            <td>{n.name}</td>
                                            <td><span class="lite-badge">{n.protocol.unwrap_or_else(|| "—".into())}</span></td>
                                            <td>{lat}</td>
                                            <td style="display:flex;gap:6px;">
                                                <button type="button" class="lite-btn sm" on:click=move |_| {
                                                    let id = nid_p.clone();
                                                    let api = st.client();
                                                    spawn_local(async move {
                                                        let _ = api.probe_node(&id).await;
                                                        st.reload_data();
                                                    });
                                                }>{st.t("pol.probe")}</button>
                                                <button type="button" class="lite-btn sm" on:click=move |_| {
                                                    st.share_payload.set(Some((share_name.clone(), share_val.clone())));
                                                }>"QR"</button>
                                                <button type="button" class="lite-btn sm" on:click=move |_| {
                                                    let id = nid_d.clone();
                                                    let api = st.client();
                                                    spawn_local(async move {
                                                        let _ = api.delete_node(&id).await;
                                                        st.reload_data();
                                                    });
                                                }>"×"</button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </section>
        </div>
    }
}

#[component]
fn SubscriptionsScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let name = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());

    let add_sub = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let n = name.get_untracked().trim().to_string();
        let u = url.get_untracked().trim().to_string();
        if n.is_empty() || u.is_empty() {
            return;
        }
        let api = st.client();
        spawn_local(async move {
            match api.create_provider(&n, &u).await {
                Ok(_) => {
                    name.set(String::new());
                    url.set(String::new());
                    st.flash("ok", st.t("ui.saved"));
                    st.reload_data();
                }
                Err(e) => st.flash("err", e),
            }
        });
    };

    view! {
        <div class="lite-page">
            <div class="lite-head"><h1>{move || st.label_screen("subscriptions")}</h1></div>
            <section class="lite-card">
                <form on:submit=add_sub style="display:flex;gap:8px;flex-wrap:wrap;">
                    <input class="lite-input" placeholder=move || st.t("ui.name") prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) />
                    <input class="lite-input" placeholder="https://..." style="flex:1;min-width:200px;" prop:value=move || url.get() on:input=move |ev| url.set(event_target_value(&ev)) />
                    <button type="submit" class="lite-btn primary sm">{move || st.t("ui.add")}</button>
                </form>
            </section>
            <section class="lite-card">
                {move || {
                    let list = st.providers.get();
                    if list.is_empty() {
                        return view! { <p class="lite-muted">{st.t("act.noSubscriptions")}</p> }.into_any();
                    }
                    list.into_iter().map(|p| {
                        let pid_r = p.id.clone();
                        let pid_d = p.id.clone();
                        let share_u = p.url.clone().unwrap_or_else(|| p.name.clone());
                        let share_n = p.name.clone();
                        view! {
                            <div class="lite-between lite-subscription-row" style="padding:10px 0;">
                                <div>
                                    <strong>{p.name}</strong>
                                    " "
                                    <span class="lite-badge">{format!("{} nodes", p.node_count.unwrap_or(0))}</span>
                                </div>
                                <div style="display:flex;gap:6px;">
                                    <button type="button" class="lite-btn sm" on:click=move |_| {
                                        let id = pid_r.clone();
                                        let api = st.client();
                                        spawn_local(async move {
                                            let _ = api.refresh_provider(&id).await;
                                            st.reload_data();
                                        });
                                    }>{st.t("ui.refresh")}</button>
                                    <button type="button" class="lite-btn sm" on:click=move |_| {
                                        st.share_payload.set(Some((share_n.clone(), share_u.clone())));
                                    }>"QR"</button>
                                    <button type="button" class="lite-btn sm" on:click=move |_| {
                                        let id = pid_d.clone();
                                        let api = st.client();
                                        spawn_local(async move {
                                            let _ = api.delete_provider(&id).await;
                                            st.reload_data();
                                        });
                                    }>"×"</button>
                                </div>
                            </div>
                        }
                    }).collect_view().into_any()
                }}
            </section>
        </div>
    }
}

#[component]
fn RoutingScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let trace_domain = RwSignal::new("www.google.com".to_string());
    let trace_res = RwSignal::new(None::<Value>);
    let text_draft = RwSignal::new(String::new());

    Effect::new(move |_| {
        if let Some(cfg) = st.config.get()
            && let Some(main) = cfg
                .sources
                .iter()
                .find(|s| s.kind.as_deref() == Some("main"))
                .or_else(|| cfg.sources.first())
        {
            text_draft.set(main.content.clone().unwrap_or_default());
        }
    });

    let run_trace = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let d = trace_domain.get_untracked().trim().to_string();
        if d.is_empty() {
            return;
        }
        let api = st.client();
        spawn_local(async move {
            match api
                .routing_trace(json!({ "network": "tcp", "domain": d, "dst_port": 443 }))
                .await
            {
                Ok(v) => trace_res.set(Some(v)),
                Err(e) => st.flash("err", e),
            }
        });
    };

    view! {
        <div class="lite-page">
            <div class="lite-head"><h1>{move || st.label_screen("routing")}</h1></div>
            <section class="lite-card">
                <h2 style="margin-top:0;font-size:15px;">"Rule Tester"</h2>
                <form on:submit=run_trace style="display:flex;gap:8px;flex-wrap:wrap;">
                    <input class="lite-input" style="flex:1;" prop:value=move || trace_domain.get() on:input=move |ev| trace_domain.set(event_target_value(&ev)) />
                    <button type="submit" class="lite-btn primary sm">{move || st.t("rules.traceRun")}</button>
                </form>
                {move || trace_res.get().map(|v| view! {
                    <pre class="lite-pre">{serde_json::to_string_pretty(&v).unwrap_or_default()}</pre>
                })}
            </section>
            <section class="lite-card">
                <h2 style="margin-top:0;font-size:15px;">"Main Config (.dae)"</h2>
                <textarea
                    class="lite-input"
                    rows="16"
                    spellcheck="false"
                    style="width:100%;font-family:monospace;font-size:12px;"
                    prop:value=move || text_draft.get()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        text_draft.set(val.clone());
                        if let Some(cfg) = st.config.get_untracked()
                            && let Some(main) = cfg.sources.iter().find(|s| s.kind.as_deref() == Some("main")).or_else(|| cfg.sources.first())
                        {
                            st.draft_source_id.set(Some(main.id.clone()));
                            st.draft_text.set(Some(val));
                        }
                    }
                />
            </section>
        </div>
    }
}

#[component]
fn DnsScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let domain = RwSignal::new("example.com".to_string());
    let result = RwSignal::new(None::<Value>);
    let dns_rules = RwSignal::new(None::<Value>);

    Effect::new(move |_| {
        let api = st.client();
        spawn_local(async move {
            if let Ok(r) = api.dns_rules().await {
                dns_rules.set(Some(r));
            }
        });
    });

    let query = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let d = domain.get_untracked().trim().to_string();
        if d.is_empty() {
            return;
        }
        let api = st.client();
        spawn_local(async move {
            match api.dns_query(&d, &["A", "AAAA", "HTTPS"]).await {
                Ok(v) => result.set(Some(v)),
                Err(e) => st.flash("err", e),
            }
        });
    };

    view! {
        <div class="lite-page">
            <div class="lite-head">
                <h1>{move || st.label_screen("dns")}</h1>
                <button type="button" class="lite-btn sm" on:click=move |_| {
                    let api = st.client();
                    spawn_local(async move {
                        match api.flush_dns_cache().await {
                            Ok(_) => st.flash("ok", st.t("dns.flushed")),
                            Err(e) => st.flash("err", e),
                        }
                    });
                }>{move || st.t("dns.flush")}</button>
            </div>
            <section class="lite-card">
                <form on:submit=query style="display:flex;gap:8px;flex-wrap:wrap;">
                    <input class="lite-input" style="flex:1;" prop:value=move || domain.get() on:input=move |ev| domain.set(event_target_value(&ev)) />
                    <button type="submit" class="lite-btn primary sm">{move || st.t("dns.run")}</button>
                </form>
                {move || result.get().map(|v| view! {
                    <pre class="lite-pre">{serde_json::to_string_pretty(&v).unwrap_or_default()}</pre>
                })}
            </section>
            {move || dns_rules.get().map(|r| view! {
                <section class="lite-card">
                    <h2 style="margin-top:0;font-size:15px;">"DNS Rules"</h2>
                    <pre class="lite-pre">{serde_json::to_string_pretty(&r).unwrap_or_default()}</pre>
                </section>
            })}
        </div>
    }
}

#[component]
fn DraftBar() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let busy = RwSignal::new(false);

    let apply_draft = move |_| {
        let (Some(sid), Some(text)) = (st.draft_source_id.get_untracked(), st.draft_text.get_untracked()) else {
            return;
        };
        busy.set(true);
        let api = st.client();
        spawn_local(async move {
            match api.config_source(&sid).await {
                Ok((src, etag)) => {
                    let if_match = etag.unwrap_or_else(|| {
                        format!("\"{}\"", src.content_sha256.or(src.sha256).unwrap_or_default())
                    });
                    match api.replace_config_source(&sid, &text, &if_match).await {
                        Ok(_) => {
                            st.draft_source_id.set(None);
                            st.draft_text.set(None);
                            st.review_open.set(false);
                            st.flash("ok", st.t("ui.saved"));
                            st.reload_data();
                        }
                        Err(e) => st.flash("err", e),
                    }
                }
                Err(e) => st.flash("err", e),
            }
            busy.set(false);
        });
    };

    view! {
        <Show when=move || st.draft_text.get().is_some()>
            <div class="lite-draft-bar">
                <span>"Unsaved configuration draft"</span>
                <div style="display:flex;gap:8px;">
                    <button type="button" class="lite-btn sm" on:click=move |_| {
                        st.draft_source_id.set(None);
                        st.draft_text.set(None);
                    }>{move || st.t("ui.cancel")}</button>
                    <button type="button" class="lite-btn primary sm" disabled=move || busy.get() on:click=apply_draft>
                        {move || st.t("config.save")}
                    </button>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn CommandPalette() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let q = RwSignal::new(String::new());
    view! {
        <Show when=move || st.commands_open.get()>
            <div class="lite-modal-bg" on:click=move |_| st.commands_open.set(false)>
                <div class="lite-card lite-modal" on:click=move |ev| ev.stop_propagation()>
                    <input class="lite-input" type="search" placeholder="Jump to..." style="width:100%;margin-bottom:10px;" prop:value=move || q.get() on:input=move |ev| q.set(event_target_value(&ev)) />
                    <div style="display:flex;flex-direction:column;gap:4px;">
                        {move || {
                            let needle = q.get().to_lowercase();
                            SCREENS.iter().filter(|s| needle.is_empty() || s.contains(&needle) || st.label_screen(s).to_lowercase().contains(&needle)).map(|&s| view! {
                                <button type="button" class="lite-nav-btn" on:click=move |_| {
                                    st.go(s);
                                    st.commands_open.set(false);
                                }>{st.label_screen(s)}</button>
                            }).collect_view()
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn SettingsModal() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let update_status = RwSignal::new(None::<CoreUpdateStatus>);
    let checking = RwSignal::new(false);

    Effect::new(move |_| {
        if st.settings_open.get() && st.capabilities.get().is_some_and(|c| c.core_update_available()) {
            let api = st.client();
            spawn_local(async move {
                if let Ok(u) = api.core_update_status(false).await {
                    update_status.set(Some(u));
                }
            });
        }
    });

    view! {
        <Show when=move || st.settings_open.get()>
            <div class="lite-modal-bg" on:click=move |_| st.settings_open.set(false)>
                <div class="lite-card lite-modal" style="width:min(480px,92vw);display:flex;flex-direction:column;gap:12px;" on:click=move |ev| ev.stop_propagation()>
                    <div class="lite-between">
                        <h2 style="margin:0;">{move || st.t("nav.settings")}</h2>
                        <button type="button" class="lite-btn sm" on:click=move |_| st.settings_open.set(false)>"×"</button>
                    </div>
                    <div class="lite-between">
                        <span>{move || st.t("ui.lang")}</span>
                        <select class="lite-input" prop:value=move || st.lang.get().code() on:change=move |ev| st.set_lang(Lang::from_code(&event_target_value(&ev)))>
                            {Lang::ALL.iter().map(|&l| view! { <option value=l.code()>{l.label()}</option> }).collect_view()}
                        </select>
                    </div>
                    <div class="lite-between">
                        <span>{move || st.t("ui.theme")}</span>
                        <select class="lite-input" prop:value=move || st.theme.get() on:change=move |ev| st.set_theme(event_target_value(&ev))>
                            <option value="system">"System"</option>
                            <option value="light">"Light"</option>
                            <option value="dark">"Dark"</option>
                        </select>
                    </div>
                    {move || update_status.get().map(|u| view! {
                        <div style="border-top:1px solid var(--color-border,#ccc);padding-top:10px;display:flex;flex-direction:column;gap:6px;">
                            <strong>{st.t("settings.coreUpdate")}</strong>
                            <div class="lite-muted">{format!("{} · {}", u.current_version, u.latest_version.as_deref().unwrap_or("—"))}</div>
                            {doona_common::model::value_u64(&u.asset_size_bytes).map(|b| view! { <div class="lite-muted">{format_bytes(b)}</div> })}
                            <button type="button" class="lite-btn sm" disabled=move || checking.get() on:click=move |_| {
                                checking.set(true);
                                let api = st.client();
                                spawn_local(async move {
                                    if let Ok(next) = api.core_update_status(true).await {
                                        update_status.set(Some(next));
                                    }
                                    checking.set(false);
                                });
                            }>{st.t("settings.coreUpdateCheck")}</button>
                        </div>
                    })}
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ShareQrModal() -> impl IntoView {
    let st = expect_context::<LiteState>();
    view! {
        <Show when=move || st.share_payload.get().is_some()>
            <div class="lite-modal-bg" on:click=move |_| st.share_payload.set(None)>
                <div class="lite-card lite-modal" style="text-align:center;" on:click=move |ev| ev.stop_propagation()>
                    {move || st.share_payload.get().map(|(title, payload)| {
                        let svg = qr::render_svg_qr(&payload).unwrap_or_default();
                        view! {
                            <div style="display:flex;flex-direction:column;gap:10px;align-items:center;">
                                <strong>{title}</strong>
                                <div inner_html=svg />
                                <code style="word-break:break-all;font-size:11px;">{payload}</code>
                                <button type="button" class="lite-btn sm" on:click=move |_| st.share_payload.set(None)>{st.t("ui.close")}</button>
                            </div>
                        }
                    })}
                </div>
            </div>
        </Show>
    }
}
