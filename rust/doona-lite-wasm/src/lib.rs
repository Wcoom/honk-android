//! Pure Rust WASM (Leptos 0.8 CSR) frontend for Doona Lite (`/ui/lite/`).
//! Covers Connect, Shell, all 5 screens (`groups`, `subscriptions`, `nodes`, `routing`, `dns`),
//! Command Palette (`Ctrl+K`), Settings & CoreUpdate dialog, Draft Review bar, RuleTester, and pure-Rust SVG Share QR.

pub mod qr;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use doona_common::client::ApiClient;
use doona_common::format::{format_bytes, format_latency};
use doona_common::i18n::{self, Lang};
use doona_common::model::{
    Capabilities, CoreUpdateStatus, EffectiveConfig, GroupDetail, GroupSummary, NodeList,
    ProviderItem, RuntimeSnapshot, VersionInfo,
};
use leptos::prelude::*;
use liquid_glass::{LiquidTabBarPhysics, LiquidTabFrame, NavDirection, build_svg_lens_assets};
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
    nav_dir: RwSignal<NavDirection>,
    nav_seq: RwSignal<u64>,
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
            nav_dir: RwSignal::new(NavDirection::CrossFade),
            nav_seq: RwSignal::new(0),
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
        let prev = self.screen.get_untracked();
        if prev != s {
            let fi = SCREENS.iter().position(|&x| x == prev).unwrap_or(0);
            let ti = SCREENS.iter().position(|&x| x == s).unwrap_or(0);
            self.nav_dir.set(if ti >= fi {
                NavDirection::TabForward
            } else {
                NavDirection::TabBackward
            });
            self.nav_seq.update(|n| *n += 1);
        }
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
                    let next = parse_screen(&h);
                    let prev = st.screen.get_untracked();
                    if prev != next {
                        let fi = SCREENS.iter().position(|&x| x == prev).unwrap_or(0);
                        let ti = SCREENS.iter().position(|&x| x == next).unwrap_or(0);
                        st.nav_dir.set(if ti >= fi {
                            NavDirection::TabForward
                        } else {
                            NavDirection::TabBackward
                        });
                        st.nav_seq.update(|n| *n += 1);
                    }
                    st.screen.set(next);
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
                {move || {
                    let current = st.screen.get();
                    let dir = st.nav_dir.get_untracked().css_attr();
                    let parity = (st.nav_seq.get_untracked() % 2).to_string();
                    let body = match current.as_str() {
                        "nodes" => view! { <NodesScreen /> }.into_any(),
                        "subscriptions" => view! { <SubscriptionsScreen /> }.into_any(),
                        "routing" => view! { <RoutingScreen /> }.into_any(),
                        "dns" => view! { <DnsScreen /> }.into_any(),
                        _ => view! { <GroupsScreen /> }.into_any(),
                    };
                    view! {
                        <div class="lite-route-stage" data-nav-dir=dir data-nav-parity=parity>
                            {body}
                        </div>
                    }
                }}
            </main>

            <LiteLiquidBottomBar />
        </div>
    }
}

#[component]
fn LiteLiquidBottomBar() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let lens = StoredValue::new(build_svg_lens_assets());
    let tabs_count = SCREENS.len();
    let init_idx = SCREENS
        .iter()
        .position(|&s| s == st.screen.get_untracked())
        .unwrap_or(0);

    let physics = Rc::new(RefCell::new(LiquidTabBarPhysics::new(tabs_count, init_idx)));
    let frame_sig = RwSignal::new({
        let mut p = physics.borrow_mut();
        p.step(1.0 / 60.0, 380.0, 1.0)
    });
    let raf_active = Rc::new(RefCell::new(false));
    let bar_width = Rc::new(RefCell::new(380.0f32));
    let drag_state = Rc::new(RefCell::new(None::<(i32, f32, bool)>));
    let bar_ref = NodeRef::<leptos::html::Div>::new();

    let wake_raf: Rc<dyn Fn()> = {
        let physics = physics.clone();
        let raf_active = raf_active.clone();
        let bar_width = bar_width.clone();
        Rc::new(move || {
            if *raf_active.borrow() {
                return;
            }
            *raf_active.borrow_mut() = true;
            let phys_c = physics.clone();
            let active_c = raf_active.clone();
            let width_c = bar_width.clone();
            let last_ms = Rc::new(RefCell::new(None::<f64>));
            type RafClosureSlot = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
            let cb_slot: RafClosureSlot = Rc::new(RefCell::new(None));
            let cb_slot_inner = cb_slot.clone();

            *cb_slot.borrow_mut() = Some(Closure::wrap(Box::new(move |now_ms: f64| {
                let dt = {
                    let mut prev = last_ms.borrow_mut();
                    let dt = match *prev {
                        Some(p) => ((now_ms - p) / 1000.0).clamp(0.001, 0.05) as f32,
                        None => 1.0 / 60.0,
                    };
                    *prev = Some(now_ms);
                    dt
                };
                let w = (*width_c.borrow()).max(180.0);
                let frame: LiquidTabFrame = phys_c.borrow_mut().step(dt, w, 1.0);
                frame_sig.set(frame);
                if frame.settled {
                    *active_c.borrow_mut() = false;
                    let _ = cb_slot_inner.borrow_mut().take();
                } else if let Some(win) = web_sys::window() {
                    if let Some(ref c) = *cb_slot_inner.borrow() {
                        let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
                    }
                } else {
                    *active_c.borrow_mut() = false;
                }
            }) as Box<dyn FnMut(f64)>));

            if let Some(win) = web_sys::window() {
                if let Some(ref c) = *cb_slot.borrow() {
                    let _ = win.request_animation_frame(c.as_ref().unchecked_ref());
                }
            } else {
                *raf_active.borrow_mut() = false;
            }
        })
    };

    {
        let physics = physics.clone();
        let wake = wake_raf.clone();
        Effect::new(move |_| {
            let cur = st.screen.get();
            let idx = SCREENS.iter().position(|&s| s == cur).unwrap_or(0);
            physics.borrow_mut().sync_external_tab(idx);
            wake();
        });
    }

    let on_down = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let bar_width = bar_width.clone();
        let wake = wake_raf.clone();
        move |ev: web_sys::PointerEvent| {
            if ev.button() != 0 {
                return;
            }
            if let Some(el) = bar_ref.get() {
                let rect = el.get_bounding_client_rect();
                if rect.width() > 40.0 {
                    *bar_width.borrow_mut() = rect.width() as f32;
                }
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
            *drag_state.borrow_mut() = Some((ev.pointer_id(), ev.client_x() as f32, false));
            physics.borrow_mut().on_drag_start();
            wake();
        }
    };

    let on_move = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let bar_width = bar_width.clone();
        let wake = wake_raf.clone();
        move |ev: web_sys::PointerEvent| {
            let mut ds = drag_state.borrow_mut();
            let Some((pid, ref mut last_x, ref mut moved)) = *ds else {
                return;
            };
            if ev.pointer_id() != pid {
                return;
            }
            let x = ev.client_x() as f32;
            let dx = x - *last_x;
            if dx.abs() > 1.5 || *moved {
                *moved = true;
                *last_x = x;
                let w = (*bar_width.borrow()).max(180.0);
                let tab_w = ((w - 8.0) / (tabs_count as f32)).max(24.0);
                physics.borrow_mut().on_drag_delta(dx, tab_w);
                wake();
            }
        }
    };

    let finish = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let wake = wake_raf.clone();
        Rc::new(move |ev: web_sys::PointerEvent| {
            let taken = {
                let mut ds = drag_state.borrow_mut();
                if ds.as_ref().is_some_and(|(pid, _, _)| *pid == ev.pointer_id()) {
                    ds.take()
                } else {
                    None
                }
            };
            let Some((_, _, moved)) = taken else {
                return;
            };
            if let Some(el) = bar_ref.get() {
                let _ = el.release_pointer_capture(ev.pointer_id());
            }
            let chosen = physics.borrow_mut().on_drag_end();
            wake();
            if moved
                && let Some(&scr) = SCREENS.get(chosen)
                && scr != st.screen.get_untracked()
            {
                st.go(scr);
            }
        })
    };
    let on_up = {
        let f = finish.clone();
        move |ev: web_sys::PointerEvent| f(ev)
    };
    let on_cancel = move |ev: web_sys::PointerEvent| finish(ev);

    let bar_style = move || {
        let f = frame_sig.get();
        format!(
            "transform:translate3d({:.2}px,0,0) scale({:.4});--lg-hl-x:{:.2}%;",
            f.panel_offset_x,
            f.panel_scale,
            f.highlight_center_frac * 100.0
        )
    };
    let droplet_style = move || {
        let f = frame_sig.get();
        let pct_w = 100.0 / (tabs_count as f32);
        let pct_x = f.value * 100.0;
        format!(
            "width:calc({pct_w:.4}% - 2px);transform:translate3d({pct_x:.2}%,0,0) scale({:.4},{:.4});",
            f.droplet_scale_x, f.droplet_scale_y
        )
    };

    view! {
        <svg aria-hidden="true" width="0" height="0" style="position:fixed;width:0;height:0;pointer-events:none;">
            <defs>
                {move || {
                    let a = lens.get_value();
                    let bs = format!("{:.2}", a.bar_scale);
                    let rs = format!("{:.2}", a.droplet_red_scale);
                    let gs = format!("{:.2}", a.droplet_scale);
                    let bbs = format!("{:.2}", a.droplet_blue_scale);
                    view! {
                        <filter id="lite-lg-bar" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
                            <feImage href=a.bar_map_uri.clone() result="barDisp" preserveAspectRatio="none" />
                            <feDisplacementMap in="SourceGraphic" in2="barDisp" scale=bs xChannelSelector="R" yChannelSelector="G" result="ref" />
                            <feColorMatrix in="ref" type="saturate" values="1.45" />
                        </filter>
                        <filter id="lite-lg-droplet" x="-10%" y="-10%" width="120%" height="120%" color-interpolation-filters="sRGB">
                            <feImage href=a.droplet_map_uri.clone() result="dropDisp" preserveAspectRatio="none" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=rs xChannelSelector="R" yChannelSelector="G" result="dR" />
                            <feColorMatrix in="dR" type="matrix" values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0" result="cR" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=gs xChannelSelector="R" yChannelSelector="G" result="dG" />
                            <feColorMatrix in="dG" type="matrix" values="0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0" result="cG" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=bbs xChannelSelector="R" yChannelSelector="G" result="dB" />
                            <feColorMatrix in="dB" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0" result="cB" />
                            <feBlend in="cR" in2="cG" mode="screen" result="rg" />
                            <feBlend in="rg" in2="cB" mode="screen" result="rgb" />
                            <feColorMatrix in="rgb" type="saturate" values="1.35" />
                        </filter>
                    }
                }}
            </defs>
        </svg>

        <nav class="lite-lg-dock-wrap" aria-label="Bottom navigation">
            <div
                node_ref=bar_ref
                class="lite-lg-dock"
                style=bar_style
                on:pointerdown=on_down
                on:pointermove=on_move
                on:pointerup=on_up
                on:pointercancel=on_cancel
            >
                <div class="lite-lg-surface" aria-hidden="true"></div>
                <div class="lite-lg-track">
                    <div class="lite-lg-droplet" style=droplet_style aria-hidden="true"></div>
                    {SCREENS.iter().enumerate().map(|(idx, &s)| {
                        let href = format!("#{s}");
                        let phys_tap = physics.clone();
                        let wake_tap = wake_raf.clone();
                        view! {
                            <a
                                class="lite-lg-tab"
                                href=href
                                aria-current=move || (st.screen.get() == s).then_some("page")
                                on:click=move |ev: web_sys::MouseEvent| {
                                    ev.prevent_default();
                                    phys_tap.borrow_mut().animate_to_tab(idx);
                                    wake_tap();
                                    st.go(s);
                                }
                            >
                                <span>{move || st.label_screen(s)}</span>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>
        </nav>
    }
}

#[component]
fn GroupsScreen() -> impl IntoView {
    let st = expect_context::<LiteState>();
    let collapsed = RwSignal::new(std::collections::HashSet::<String>::new());
    let seeded = RwSignal::new(false);

    Effect::new(move |_| {
        let gs = st.groups.get();
        if !gs.is_empty() && !seeded.get_untracked() {
            let mut set = std::collections::HashSet::new();
            for g in &gs {
                if g.member_count > 12 {
                    set.insert(g.id.clone());
                }
            }
            collapsed.set(set);
            seeded.set(true);
        }
    });

    view! {
        <div class="lite-page">
            <div class="lite-head">
                <h1>{move || st.label_screen("groups")}</h1>
                <button type="button" class="lite-btn sm" on:click=move |_| st.reload_data()>{move || st.t("ui.refresh")}</button>
            </div>
            <div class="lite-grid">
                {move || {
                    let all_gs = st.groups.get();
                    let dets = st.group_details.get();
                    let nodes = st.nodes.get().map(|n| n.nodes).unwrap_or_default();
                    let lang = st.lang.get();
                    all_gs.iter().map(|g| {
                        let gid = g.id.clone();
                        let gid_tog = g.id.clone();
                        let gid_chk = g.id.clone();
                        let gid_probe = g.id.clone();
                        let det = dets.get(&gid).cloned();
                        let summary = doona_common::model::resolve_group_effective_summary(
                            g,
                            det.as_ref(),
                            &all_gs,
                            &nodes,
                            lang,
                        );
                        let active_id = det
                            .as_ref()
                            .and_then(|d| d.runtime.selection.tcp.as_ref())
                            .and_then(|s| s.member_id.clone())
                            .or_else(|| g.selection.tcp_member_id.clone())
                            .unwrap_or_default();
                        let can_sel = det.as_ref().is_some_and(|d| d.capabilities.can_select || d.capabilities.can_override);
                        let nodes_c = nodes.clone();
                        view! {
                            <section class="lite-card">
                                <div class="lite-between" style="gap:8px;">
                                    <div
                                        style="cursor:pointer;flex:1;min-width:0;"
                                        on:click=move |_| {
                                            let id = gid_tog.clone();
                                            collapsed.update(|set| {
                                                if !set.remove(&id) {
                                                    set.insert(id);
                                                }
                                            });
                                        }
                                    >
                                        <strong>{move || if collapsed.get().contains(&gid_chk) { "▸ " } else { "▾ " }}</strong>
                                        <strong>{g.name.clone()}</strong>
                                        " "
                                        <span class="lite-badge">{g.policy.kind.clone()}</span>
                                        " "
                                        <span class="lite-muted" style="font-size:12px;">{summary}</span>
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
                                {move || (!collapsed.get().contains(&gid)).then(|| det.clone().map(|d| view! {
                                    <div class="lite-member-grid">
                                        {d.members.into_iter().map(|m| {
                                            let active = m.id == active_id;
                                            let mid = m.id.clone();
                                            let gid_c = gid.clone();
                                            let lat = nodes_c.iter().find(|n| n.id == m.id).and_then(|n| n.best_latency_ms());
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
    let pending_del = RwSignal::new(None::<(String, String)>);

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
                                    let del_name = n.name.clone();
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
                                                        match api.probe_node(&id).await {
                                                            Ok(_) => {
                                                                st.flash("ok", st.t("pol.probeStarted"));
                                                                st.reload_data();
                                                            }
                                                            Err(e) => st.flash("err", e),
                                                        }
                                                    });
                                                }>{st.t("pol.probe")}</button>
                                                <button type="button" class="lite-btn sm" on:click=move |_| {
                                                    st.share_payload.set(Some((share_name.clone(), share_val.clone())));
                                                }>"QR"</button>
                                                <button type="button" class="lite-btn sm" on:click=move |_| {
                                                    pending_del.set(Some((nid_d.clone(), del_name.clone())));
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
            <Show when=move || pending_del.get().is_some()>
                <div class="lite-modal-bg" on:click=move |_| pending_del.set(None)>
                    <div class="lite-card lite-modal" style="display:flex;flex-direction:column;gap:12px;" on:click=move |ev| ev.stop_propagation()>
                        <strong>{move || pending_del.get().map(|(_, n)| format!("删除节点 “{n}”？")).unwrap_or_default()}</strong>
                        <div style="display:flex;gap:8px;justify-content:flex-end;">
                            <button type="button" class="lite-btn sm" on:click=move |_| pending_del.set(None)>{move || st.t("ui.cancel")}</button>
                            <button type="button" class="lite-btn primary sm" on:click=move |_| {
                                if let Some((id, _)) = pending_del.get_untracked() {
                                    pending_del.set(None);
                                    let api = st.client();
                                    spawn_local(async move {
                                        let _ = api.delete_node(&id).await;
                                        st.reload_data();
                                    });
                                }
                            }>"确认删除"</button>
                        </div>
                    </div>
                </div>
            </Show>
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
                        let is_inline = p.id == "inline" || p.kind.as_deref() == Some("inline");
                        let pid_r = p.id.clone();
                        let pid_d = p.id.clone();
                        let can_share = p.url.as_deref().is_some_and(|u| !u.is_empty() && !u.contains("***"));
                        let share_u = p.url.clone().unwrap_or_default();
                        let share_n = p.name.clone();
                        let disp_name = if is_inline && p.name == "inline" { "内置节点 (Inline)".to_string() } else { p.name };
                        view! {
                            <div class="lite-between lite-subscription-row" style="padding:10px 0;">
                                <div>
                                    <strong>{disp_name}</strong>
                                    " "
                                    <span class="lite-badge">{format!("{} nodes", p.node_count.unwrap_or(0))}</span>
                                </div>
                                {(!is_inline).then(|| view! {
                                    <div style="display:flex;gap:6px;">
                                        <button type="button" class="lite-btn sm" on:click=move |_| {
                                            let id = pid_r.clone();
                                            let api = st.client();
                                            spawn_local(async move {
                                                match api.refresh_provider(&id).await {
                                                    Ok(_) => {
                                                        st.flash("ok", st.t("ui.refresh"));
                                                        st.reload_data();
                                                    }
                                                    Err(e) => st.flash("err", e),
                                                }
                                            });
                                        }>{st.t("ui.refresh")}</button>
                                        {can_share.then(|| view! {
                                            <button type="button" class="lite-btn sm" on:click=move |_| {
                                                st.share_payload.set(Some((share_n.clone(), share_u.clone())));
                                            }>"QR"</button>
                                        })}
                                        <button type="button" class="lite-btn sm" on:click=move |_| {
                                            let id = pid_d.clone();
                                            let api = st.client();
                                            spawn_local(async move {
                                                match api.delete_provider(&id).await {
                                                    Ok(_) => st.reload_data(),
                                                    Err(e) => st.flash("err", e),
                                                }
                                            });
                                        }>"×"</button>
                                    </div>
                                })}
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
                {move || trace_res.get().map(|v| {
                    let ev0 = v.get("evaluations").and_then(Value::as_array).and_then(|a| a.first());
                    let dec = ev0.or_else(|| v.get("decision")).unwrap_or(&v);
                    let out = dec.get("outbound").or_else(|| v.get("outbound")).and_then(|o| o.get("name").and_then(Value::as_str).or_else(|| o.as_str())).unwrap_or("—").to_string();
                    let rule = ev0
                        .and_then(|ev| ev.get("rules"))
                        .and_then(Value::as_array)
                        .and_then(|rs| rs.iter().find(|r| r.get("matched").and_then(Value::as_bool) == Some(true) || matches!(r.get("result").or_else(|| r.get("outcome")).and_then(Value::as_str), Some("matched" | "match"))))
                        .and_then(|r| r.get("expression").and_then(Value::as_str))
                        .or_else(|| v.get("matched_rule").or_else(|| dec.get("matched_rule")).and_then(|r| r.get("expression").and_then(Value::as_str).or_else(|| r.as_str())))
                        .unwrap_or("fallback")
                        .to_string();
                    view! {
                        <div style="margin-top:10px;padding:10px;border-radius:10px;border:1px solid var(--lite-border);display:flex;flex-direction:column;gap:4px;">
                            <div><strong>"出站: "</strong><span class="lite-badge">{out}</span></div>
                            <div><strong>"规则: "</strong><code>{rule}</code></div>
                        </div>
                    }
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
                {move || result.get().map(|v| {
                    let queries = v.get("queries").or_else(|| v.get("results")).and_then(Value::as_array).cloned().unwrap_or_else(|| vec![v.clone()]);
                    view! {
                        <div style="margin-top:10px;display:flex;flex-direction:column;gap:8px;">
                            {queries.into_iter().map(|q| {
                                let rcode = q.get("rcode").or_else(|| q.get("status")).and_then(Value::as_str).unwrap_or("NOERROR").to_string();
                                let answers = q.get("answers").or_else(|| q.get("records")).and_then(Value::as_array).map(|arr| {
                                    arr.iter().filter_map(|a| a.get("data").or_else(|| a.get("value")).and_then(Value::as_str)).collect::<Vec<_>>().join(", ")
                                }).filter(|s| !s.is_empty()).unwrap_or_else(|| "—".into());
                                view! {
                                    <div style="padding:8px 10px;border-radius:10px;border:1px solid var(--lite-border);display:flex;justify-content:space-between;gap:8px;">
                                        <span class="lite-badge">{rcode}</span>
                                        <code style="word-break:break-all;">{answers}</code>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }
                })}
            </section>
            {move || dns_rules.get().map(|r| {
                let upstreams = r.get("upstreams").and_then(Value::as_array).cloned().unwrap_or_default();
                view! {
                    <section class="lite-card">
                        <h2 style="margin-top:0;font-size:15px;">"DNS Upstreams"</h2>
                        <div style="display:flex;flex-direction:column;gap:6px;">
                            {upstreams.into_iter().map(|u| {
                                let n = u.get("name").and_then(Value::as_str).unwrap_or("upstream").to_string();
                                let addr = u.get("url").or_else(|| u.get("address")).and_then(Value::as_str).unwrap_or("—").to_string();
                                view! {
                                    <div class="lite-between">
                                        <strong>{n}</strong>
                                        <code>{addr}</code>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </section>
                }
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
