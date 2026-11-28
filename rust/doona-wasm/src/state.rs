//! Reactive global application state, polling loop, auth/discovery, and hash router for Doona Full (`/ui/`).

use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use doona_common::client::ApiClient;
use doona_common::i18n::{self, Lang};
use doona_common::model::{
    Capabilities, DatapathStatus, DiscoveryInfo, EffectiveConfig, GroupSummary, NodeList,
    ProviderItem, RuntimeMemory, RuntimeSnapshot, VersionInfo,
};
#[cfg(not(target_arch = "wasm32"))]
use doona_common::palettes::split_palette;
use doona_common::palettes::DEFAULT_PALETTE;
use leptos::prelude::*;
use liquid_glass::NavDirection;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::spawn_local;

pub const ROUTES: &[&str] = &[
    "activity",
    "overview",
    "connections",
    "flows",
    "dns",
    "logs",
    "events",
    "policies",
    "nodes",
    "rules",
    "config",
    "settings",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HubDef {
    pub id: &'static str,
    pub title_key: &'static str,
    pub pages: &'static [&'static str],
}

pub const HUBS: &[HubDef] = &[
    HubDef {
        id: "activity",
        title_key: "nav.activity",
        pages: &["activity", "overview"],
    },
    HubDef {
        id: "traffic",
        title_key: "hub.traffic",
        pages: &["connections", "flows", "dns", "logs", "events"],
    },
    HubDef {
        id: "routing",
        title_key: "hub.routing",
        pages: &["policies", "nodes", "rules"],
    },
    HubDef {
        id: "settings",
        title_key: "nav.settings",
        pages: &["config", "settings"],
    },
];

pub fn page_title_key(route: &str) -> &'static str {
    match route {
        "activity" => "nav.activity",
        "overview" => "nav.overview",
        "connections" => "nav.connections",
        "flows" => "nav.flows",
        "dns" => "nav.dns",
        "logs" => "nav.logs",
        "events" => "nav.events",
        "policies" => "nav.policies",
        "nodes" => "nav.nodes",
        "rules" => "nav.rules",
        "config" => "nav.config",
        "settings" => "nav.settings",
        _ => "nav.activity",
    }
}

pub fn parse_hash(hash: &str) -> (String, String) {
    let stripped = hash.trim_start_matches('#').trim_start_matches('/');
    let (path, query) = stripped.split_once('?').unwrap_or((stripped, ""));
    match path {
        "system" => ("overview".to_string(), query.to_string()),
        "subscriptions" => {
            let q = if query.is_empty() {
                "tab=subscriptions".to_string()
            } else {
                format!("tab=subscriptions&{query}")
            };
            ("nodes".to_string(), q)
        }
        p if ROUTES.contains(&p) => (p.to_string(), query.to_string()),
        _ => ("activity".to_string(), query.to_string()),
    }
}

pub fn query_param(query: &str, key: &str) -> Option<String> {
    for part in query.split('&') {
        if let Some((k, v)) = part.split_once('=')
            && k == key
        {
            return Some(v.to_string());
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct ToastItem {
    pub id: u64,
    pub tone: &'static str,
    pub text: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthGate {
    Checking,
    Ready,
    LoginRequired {
        mode: String,
        setup_required: bool,
        rejected: bool,
    },
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub lang: RwSignal<Lang>,
    pub scheme: RwSignal<String>,
    pub palette: RwSignal<String>,
    pub route: RwSignal<String>,
    pub nav_dir: RwSignal<NavDirection>,
    pub nav_seq: RwSignal<u64>,
    pub query: RwSignal<String>,
    pub api_base: RwSignal<String>,
    pub api_token: RwSignal<Option<String>>,
    pub auth_gate: RwSignal<AuthGate>,
    pub discovery: RwSignal<Option<DiscoveryInfo>>,
    pub version: RwSignal<Option<VersionInfo>>,
    pub capabilities: RwSignal<Option<Capabilities>>,
    pub runtime: RwSignal<Option<RuntimeSnapshot>>,
    pub memory: RwSignal<Option<RuntimeMemory>>,
    pub datapath: RwSignal<Option<DatapathStatus>>,
    pub groups: RwSignal<Vec<GroupSummary>>,
    pub nodes: RwSignal<Option<NodeList>>,
    pub providers: RwSignal<Vec<ProviderItem>>,
    pub config: RwSignal<Option<EffectiveConfig>>,
    pub down_ring: RwSignal<VecDeque<f64>>,
    pub up_ring: RwSignal<VecDeque<f64>>,
    pub conn_ring: RwSignal<VecDeque<f64>>,
    pub cpu_ring: RwSignal<VecDeque<f64>>,
    pub rss_ring: RwSignal<VecDeque<f64>>,
    pub toasts: RwSignal<Vec<ToastItem>>,
    pub next_toast_id: RwSignal<u64>,
    pub spinning: RwSignal<bool>,
    pub global_error: RwSignal<Option<String>>,
    pub search_open: RwSignal<bool>,
    pub about_open: RwSignal<bool>,
    pub saved_policy_target: RwSignal<String>,
    pub reload_nonce: RwSignal<u64>,
}

impl AppState {
    pub fn new() -> Self {
        let init_lang = read_storage("doona.lang")
            .or_else(|| {
                web_sys::window()
                    .map(|w| w.navigator().language().unwrap_or_default())
                    .filter(|s| !s.is_empty())
            })
            .map(|c| Lang::from_code(&c))
            .unwrap_or(Lang::ZhCn);
        let init_scheme = read_storage("doona.scheme").unwrap_or_else(|| "system".to_string());
        let init_palette =
            read_storage("doona.palette").unwrap_or_else(|| DEFAULT_PALETTE.to_string());
        let init_base = read_storage("doona.apiBase").unwrap_or_default();
        let init_token = read_storage("doona.token").filter(|s| !s.trim().is_empty());
        let init_target = read_storage("doona.globalTarget").unwrap_or_default();

        let (init_route, init_query) = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .map(|h| parse_hash(&h))
            .unwrap_or_else(|| ("activity".to_string(), String::new()));

        apply_dom_appearance(init_lang, &init_scheme, &init_palette);

        Self {
            lang: RwSignal::new(init_lang),
            scheme: RwSignal::new(init_scheme),
            palette: RwSignal::new(init_palette),
            route: RwSignal::new(init_route),
            nav_dir: RwSignal::new(NavDirection::CrossFade),
            nav_seq: RwSignal::new(0),
            query: RwSignal::new(init_query),
            api_base: RwSignal::new(init_base),
            api_token: RwSignal::new(init_token),
            auth_gate: RwSignal::new(AuthGate::Checking),
            discovery: RwSignal::new(None),
            version: RwSignal::new(None),
            capabilities: RwSignal::new(None),
            runtime: RwSignal::new(None),
            memory: RwSignal::new(None),
            datapath: RwSignal::new(None),
            groups: RwSignal::new(Vec::new()),
            nodes: RwSignal::new(None),
            providers: RwSignal::new(Vec::new()),
            config: RwSignal::new(None),
            down_ring: RwSignal::new(VecDeque::with_capacity(64)),
            up_ring: RwSignal::new(VecDeque::with_capacity(64)),
            conn_ring: RwSignal::new(VecDeque::with_capacity(64)),
            cpu_ring: RwSignal::new(VecDeque::with_capacity(64)),
            rss_ring: RwSignal::new(VecDeque::with_capacity(64)),
            toasts: RwSignal::new(Vec::new()),
            next_toast_id: RwSignal::new(1),
            spinning: RwSignal::new(false),
            global_error: RwSignal::new(None),
            search_open: RwSignal::new(false),
            about_open: RwSignal::new(false),
            saved_policy_target: RwSignal::new(init_target),
            reload_nonce: RwSignal::new(0),
        }
    }

    pub fn client(&self) -> ApiClient {
        ApiClient::new(self.api_base.get_untracked(), self.api_token.get_untracked())
    }

    pub fn t(&self, key: &str) -> String {
        i18n::tr(self.lang.get(), key)
    }

    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        i18n::tr_args(self.lang.get(), key, args)
    }

    pub fn set_lang(&self, next: Lang) {
        self.lang.set(next);
        write_storage("doona.lang", next.code());
        apply_dom_appearance(next, &self.scheme.get_untracked(), &self.palette.get_untracked());
    }

    pub fn set_scheme(&self, next: String) {
        self.scheme.set(next.clone());
        write_storage("doona.scheme", &next);
        apply_dom_appearance(self.lang.get_untracked(), &next, &self.palette.get_untracked());
    }

    pub fn toggle_scheme(&self) {
        let cur = self.scheme.get_untracked();
        let next = if cur == "dark" { "light" } else { "dark" };
        self.set_scheme(next.to_string());
    }

    pub fn set_palette(&self, next: String) {
        self.palette.set(next.clone());
        write_storage("doona.palette", &next);
        apply_dom_appearance(self.lang.get_untracked(), &self.scheme.get_untracked(), &next);
    }

    pub fn set_saved_policy_target(&self, target: &str) {
        self.saved_policy_target.set(target.to_string());
        write_storage("doona.globalTarget", target);
    }

    pub fn set_backend(&self, base: String, token: Option<String>) {
        let clean_base = base.trim().trim_end_matches('/').to_string();
        let clean_tok = token.map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
        self.api_base.set(clean_base.clone());
        self.api_token.set(clean_tok.clone());
        write_storage("doona.apiBase", &clean_base);
        match clean_tok {
            Some(ref tok) => write_storage("doona.token", tok),
            None => remove_storage("doona.token"),
        }
        self.bootstrap();
    }

    pub fn navigate(&self, target_route: &str, query: Option<&str>) {
        let prev = self.route.get_untracked();
        if prev != target_route {
            self.nav_dir.set(compute_nav_direction(&prev, target_route));
            self.nav_seq.update(|s| *s += 1);
        }
        let hash = match query {
            Some(q) if !q.is_empty() => format!("#{target_route}?{q}"),
            _ => format!("#{target_route}"),
        };
        if let Some(w) = web_sys::window() {
            let _ = w.location().set_hash(&hash);
        }
        self.route.set(target_route.to_string());
        self.query.set(query.unwrap_or_default().to_string());
    }

    pub fn toast(&self, tone: &'static str, text: impl Into<String>, detail: Option<String>) {
        let id = self.next_toast_id.get_untracked();
        self.next_toast_id.set(id + 1);
        let item = ToastItem {
            id,
            tone,
            text: text.into(),
            detail,
        };
        self.toasts.update(|list| {
            list.push(item);
            if list.len() > 4 {
                list.remove(0);
            }
        });
        let toasts = self.toasts;
        if let Some(w) = web_sys::window() {
            let cb = Closure::once(Box::new(move || {
                toasts.update(|list| list.retain(|t| t.id != id));
            }) as Box<dyn FnOnce()>);
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                4500,
            );
            cb.forget();
        }
    }

    pub fn dismiss_toast(&self, id: u64) {
        self.toasts.update(|list| list.retain(|t| t.id != id));
    }

    /// Resolves a Catalog Group UUID or Node UUID into its human-readable display name.
    pub fn resolve_catalog_name(&self, id: &str) -> Option<String> {
        if let Some(g) = self.groups.get_untracked().iter().find(|g| g.id == id) {
            return Some(g.name.clone());
        }
        if let Some(nl) = self.nodes.get_untracked()
            && let Some(n) = nl.nodes.iter().find(|n| n.id == id)
        {
            return Some(n.name.clone());
        }
        None
    }

    /// Formats a `chain` of Catalog UUIDs (or names) into a human-readable `A → B → C` path,
    /// falling back gracefully to `outbound` when `chain` is empty or unmapped.
    pub fn format_chain(&self, chain: &[String], outbound: Option<&str>) -> String {
        let groups = self.groups.get();
        let nodes = self.nodes.get();
        if !chain.is_empty() {
            let mut labels: Vec<String> = Vec::with_capacity(chain.len());
            for hop in chain {
                if let Some(g) = groups.iter().find(|g| g.id == *hop) {
                    labels.push(g.name.clone());
                } else if let Some(ref nl) = nodes
                    && let Some(n) = nl.nodes.iter().find(|n| n.id == *hop)
                {
                    labels.push(n.name.clone());
                } else if hop.len() == 36 && hop.as_bytes().get(8) == Some(&b'-') {
                    // Unresolved transient UUID: skip if outbound fallback exists, else shorten
                    if outbound.is_none_or(str::is_empty) {
                        labels.push(hop.chars().take(8).collect());
                    }
                } else if !hop.is_empty() {
                    labels.push(hop.clone());
                }
            }
            if !labels.is_empty() {
                return labels.join(" → ");
            }
        }
        outbound
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "—".to_string())
    }

    /// Polls an accepted async Native API operation (`/api/v1/operations/{id}`) until completion,
    /// then refreshes global state and surfaces final status feedback.
    pub fn poll_operation(&self, op_id: String, success_msg: String) {
        if op_id.trim().is_empty() {
            self.refresh_all();
            return;
        }
        let state = *self;
        spawn_local(async move {
            let api = state.client();
            for _ in 0..45 {
                sleep_ms(380).await;
                match api.operation(&op_id).await {
                    Ok(op) => match op.status.as_str() {
                        "succeeded" | "completed" => {
                            state.toast("positive", success_msg, None);
                            state.refresh_all();
                            return;
                        }
                        "failed" | "cancelled" | "canceled" => {
                            let detail = op.error.map(|e| format!("{}: {}", e.code, e.message));
                            state.toast("negative", state.t("ui.failed"), detail);
                            state.refresh_all();
                            return;
                        }
                        _ => {}
                    },
                    Err(_) => break,
                }
            }
            state.refresh_all();
        });
    }

    pub fn bootstrap(&self) {
        let state = *self;
        state.auth_gate.set(AuthGate::Checking);
        state.global_error.set(None);
        spawn_local(async move {
            let api = state.client();
            let disc = api.discovery().await.ok();
            state.discovery.set(disc.clone());
            match api.capabilities().await {
                Ok(caps) => {
                    state.capabilities.set(Some(caps));
                    state.auth_gate.set(AuthGate::Ready);
                    state.refresh_all();
                }
                Err(err) => {
                    if err.contains("HTTP 401") || err.contains("authentication_required") {
                        let mode = disc
                            .as_ref()
                            .and_then(|d| d.auth.as_ref())
                            .and_then(|a| a.mode.clone())
                            .unwrap_or_else(|| "token".to_string());
                        let setup_required = disc
                            .as_ref()
                            .and_then(|d| d.auth.as_ref())
                            .is_some_and(|a| a.setup_required);
                        state.auth_gate.set(AuthGate::LoginRequired {
                            mode,
                            setup_required,
                            rejected: state.api_token.get_untracked().is_some(),
                        });
                    } else {
                        state.global_error.set(Some(err));
                        state.auth_gate.set(AuthGate::Ready);
                    }
                }
            }
        });
    }

    pub fn refresh_all(&self) {
        let state = *self;
        state.spinning.set(true);
        state.reload_nonce.update(|n| *n += 1);
        spawn_local(async move {
            let api = state.client();
            if let Ok(v) = api.version().await {
                state.version.set(Some(v));
            }
            if let Ok(c) = api.capabilities().await {
                state.capabilities.set(Some(c));
                state.global_error.set(None);
            }
            state.poll_telemetry_once().await;
            if let Ok(g) = api.groups().await {
                state.groups.set(g);
            }
            if let Ok(n) = api.nodes().await {
                state.nodes.set(Some(n));
            }
            if let Ok(p) = api.providers().await {
                state.providers.set(p.providers);
            }
            if let Ok(cfg) = api.config().await {
                state.config.set(Some(cfg));
            }
            state.spinning.set(false);
        });
    }

    pub async fn poll_telemetry_once(&self) {
        let api = self.client();
        let cur_route = self.route.get_untracked();
        let need_heavy = matches!(cur_route.as_str(), "activity" | "overview");
        if let Ok(rt) = api.runtime().await {
            let down = rt.download_rate();
            let up = rt.upload_rate();
            let conns = rt.traffic.connections.total.unwrap_or(0) as f64;
            let cpu = rt.process.cpu_percent.unwrap_or(0.0);
            push_ring(&self.down_ring, down);
            push_ring(&self.up_ring, up);
            push_ring(&self.conn_ring, conns);
            push_ring(&self.cpu_ring, cpu);
            self.runtime.set(Some(rt));
            self.global_error.set(None);
        }
        if (need_heavy || self.memory.get_untracked().is_none())
            && let Ok(mem) = api.runtime_memory().await
        {
            let rss = mem.rss().or_else(|| mem.cgroup_current()).unwrap_or(0) as f64;
            push_ring(&self.rss_ring, rss);
            self.memory.set(Some(mem));
        }
        if (need_heavy || self.datapath.get_untracked().is_none())
            && let Ok(dp) = api.datapath().await
        {
            self.datapath.set(Some(dp));
        }
    }

    pub fn start_background_tasks(&self) {
        let state = *self;
        if let Some(w) = web_sys::window() {
            let on_hash = Closure::wrap(Box::new(move || {
                if let Some(win) = web_sys::window()
                    && let Ok(h) = win.location().hash()
                {
                    let (r, q) = parse_hash(&h);
                    let prev = state.route.get_untracked();
                    if prev != r {
                        state.nav_dir.set(compute_nav_direction(&prev, &r));
                        state.nav_seq.update(|s| *s += 1);
                    }
                    state.route.set(r);
                    state.query.set(q);
                }
            }) as Box<dyn FnMut()>);
            let _ = w.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());
            on_hash.forget();

            // Ctrl+K / Cmd+K search modal shortcut
            let on_key = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
                if (ev.ctrl_key() || ev.meta_key())
                    && ev.key().eq_ignore_ascii_case("k")
                    && !ev.alt_key()
                {
                    ev.prevent_default();
                    state.search_open.update(|v| *v = !*v);
                } else if ev.key() == "Escape" {
                    state.search_open.set(false);
                    state.about_open.set(false);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            let _ = w.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
            on_key.forget();

            // Battery-smart 2-second telemetry poll timer with `document.hidden()` pause + single-flight guard
            let in_flight = Rc::new(Cell::new(false));
            let in_flight_tick = in_flight.clone();
            let on_tick = Closure::wrap(Box::new(move || {
                let hidden = web_sys::window()
                    .and_then(|win| win.document())
                    .is_some_and(|doc| doc.hidden());
                if hidden || in_flight_tick.get() {
                    return;
                }
                if state.auth_gate.get_untracked() == AuthGate::Ready {
                    in_flight_tick.set(true);
                    let flag = in_flight_tick.clone();
                    spawn_local(async move {
                        state.poll_telemetry_once().await;
                        flag.set(false);
                    });
                }
            }) as Box<dyn FnMut()>);
            let _ = w.set_interval_with_callback_and_timeout_and_arguments_0(
                on_tick.as_ref().unchecked_ref(),
                2000,
            );
            on_tick.forget();

            // Instant refresh upon returning from background / screen-off (`visibilitychange`)
            if let Some(doc) = w.document() {
                let in_flight_vis = in_flight;
                let on_vis = Closure::wrap(Box::new(move || {
                    let visible = web_sys::window()
                        .and_then(|win| win.document())
                        .is_some_and(|d| !d.hidden());
                    if visible
                        && !in_flight_vis.get()
                        && state.auth_gate.get_untracked() == AuthGate::Ready
                    {
                        in_flight_vis.set(true);
                        let flag = in_flight_vis.clone();
                        spawn_local(async move {
                            state.poll_telemetry_once().await;
                            flag.set(false);
                        });
                    }
                }) as Box<dyn FnMut()>);
                let _ = doc.add_event_listener_with_callback(
                    "visibilitychange",
                    on_vis.as_ref().unchecked_ref(),
                );
                on_vis.forget();
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_nav_direction(from: &str, to: &str) -> NavDirection {
    if from == to {
        return NavDirection::CrossFade;
    }
    let locate = |r: &str| -> (usize, usize) {
        for (hi, hub) in HUBS.iter().enumerate() {
            if let Some(pi) = hub.pages.iter().position(|&p| p == r) {
                return (hi, pi);
            }
        }
        (0, 0)
    };
    let (fh, fp) = locate(from);
    let (th, tp) = locate(to);
    if fh != th {
        if th > fh {
            NavDirection::TabForward
        } else {
            NavDirection::TabBackward
        }
    } else if tp > fp {
        NavDirection::Push
    } else {
        NavDirection::Pop
    }
}

fn push_ring(sig: &RwSignal<VecDeque<f64>>, val: f64) {
    sig.update(|ring| {
        if ring.len() >= 60 {
            ring.pop_front();
        }
        ring.push_back(val);
    });
}

pub fn apply_dom_appearance(lang: Lang, scheme: &str, palette: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        doona_common::palettes::browser::apply_appearance(scheme, palette);
        if let Some(doc) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = doc.document_element()
        {
            let _ = root.set_attribute("lang", lang.code());
            let _ = root.remove_attribute("data-warm");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (lang, scheme, split_palette(palette));
    }
}

pub fn read_storage(key: &str) -> Option<String> {
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

pub fn write_storage(key: &str, val: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        doona_common::palettes::browser::write_storage(key, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, val);
    }
}

pub fn remove_storage(key: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        doona_common::palettes::browser::remove_storage(key);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
    }
}

pub fn use_app_state() -> AppState {
    expect_context::<AppState>()
}

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        } else {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(_ms: i32) {}
