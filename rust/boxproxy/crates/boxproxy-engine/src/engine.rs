//! Core MVI runtime hosting `BoxDbRepo`, `PrefsStore`, `RuntimeFileStore`, `BackupService`,
//! `RootExecutor`, `HonkNativeClient`/`ClashApiClient`, and all 58 module reducers/executors.

use crate::action::AppAction;
use crate::effect::AppEffect;
use crate::i18n::tr;
use crate::state::{
    AppRoute, AppState, NotificationButtonSnapshot, NotificationRenderSnapshot, ServiceStatus,
    SpeedPoint, TileStateSnapshot,
};
use crate::EngineResult;
use boxproxy_api::{discover_endpoint, ApiKind, ClashApiClient, HonkNativeClient};
use boxproxy_core_mgr::{
    apply_download_prefix, builtin_core_sources, extract_webui_zip, install_core_binary_atomic,
    parse_substore_env, PackageFormat, UpdateCheckResult,
};
use boxproxy_root::hooks::{run_shell_hook, HookContext};
use boxproxy_root::shell::{check_root_granted, SuShellPool};
use boxproxy_root::{
    install_boot_script, parse_arp_table, remove_boot_script, sample_process, sync_binary_atomic,
    BoxctlClient, RootExecutor,
};
use boxproxy_schema::{match_first_rule_group, CoreKind, NetworkMode};
use boxproxy_store::{BackupManifest, BackupScope, BackupService, BoxDbRepo, PrefsStore, RuntimeFileStore};
use boxproxy_tools::{
    classify_smart_select, decode_save_file_payload, generate_monet_colors_css,
    probe_all_platforms, probe_ip_quality, probe_latency_targets, probe_wan_geo,
    render_notification_template, run_speed_test, validate_cidr_v4, validate_cidr_v6,
    validate_config_text, InstalledAppInfo, MonetPaletteInput, NotificationRenderVars,
    SpeedTestSource, WebMessageCommand,
};
use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 7-primitive reverse callback implemented by the ultra-thin Android DEX shell (Section 5.3).
pub trait PlatformBridge: Send + Sync {
    fn on_effect(&self, effect: &AppEffect);
    fn set_component_alias(&self, alias_name: &str, enabled: bool);
    fn query_installed_apps(&self) -> Vec<InstalledAppInfo>;
    fn query_network_snapshot(&self) -> boxproxy_schema::NetworkObservation;
    fn has_permission(&self, permission: &str) -> bool;
    fn read_asset_bytes(&self, asset_path: &str) -> Option<Vec<u8>>;
    fn open_external_url(&self, url: &str);
}

#[derive(Default)]
pub struct NoopPlatformBridge;

impl PlatformBridge for NoopPlatformBridge {
    fn on_effect(&self, _effect: &AppEffect) {}
    fn set_component_alias(&self, _alias_name: &str, _enabled: bool) {}
    fn query_installed_apps(&self) -> Vec<InstalledAppInfo> {
        Vec::new()
    }
    fn query_network_snapshot(&self) -> boxproxy_schema::NetworkObservation {
        boxproxy_schema::NetworkObservation::default()
    }
    fn has_permission(&self, _permission: &str) -> bool {
        true
    }
    fn read_asset_bytes(&self, _asset_path: &str) -> Option<Vec<u8>> {
        None
    }
    fn open_external_url(&self, _url: &str) {}
}

pub trait StateListener: Send + Sync {
    fn on_state_snapshot(&self, version: u64, state_json: &[u8]);
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub files_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub package_name: String,
    pub app_version: String,
    pub sdk_int: i32,
    pub initial_locale: String,
}

type ListenerMap = BTreeMap<u64, (String, Arc<dyn StateListener>)>;

pub struct BoxProxyEngine {
    pub box_home: PathBuf,
    pub config: EngineConfig,
    pub db: Arc<BoxDbRepo>,
    pub prefs_store: Arc<PrefsStore>,
    pub file_store: Arc<RuntimeFileStore>,
    pub backup: Arc<BackupService>,
    pub root: Arc<dyn RootExecutor>,
    pub platform: Arc<dyn PlatformBridge>,
    pub state: Arc<RwLock<AppState>>,
    assets: Arc<RwLock<BTreeMap<String, Vec<u8>>>>,
    listeners: Arc<RwLock<ListenerMap>>,
    next_sub_id: AtomicU64,
    version_counter: AtomicU64,
    runtime: tokio::runtime::Runtime,
}

impl BoxProxyEngine {
    pub fn new(
        config: EngineConfig,
        platform: Arc<dyn PlatformBridge>,
        root_override: Option<Arc<dyn RootExecutor>>,
    ) -> EngineResult<Self> {
        let box_home = config.files_dir.join("box");
        let file_store = Arc::new(RuntimeFileStore::new(&box_home));
        file_store.ensure_layout()?;

        let db = Arc::new(BoxDbRepo::open(box_home.join("box.db"), &box_home)?);
        let prefs_path = box_home.join("app_prefs.json");
        let prefs_store = Arc::new(PrefsStore::new(&prefs_path));
        let backup = Arc::new(BackupService::new(&box_home, &prefs_path));
        let root: Arc<dyn RootExecutor> =
            root_override.unwrap_or_else(|| Arc::new(SuShellPool::new()));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| format!("build tokio runtime failed: {e}"))?;

        let mut st = AppState::initial(config.package_name.clone(), config.app_version.clone());
        st.prefs = prefs_store.load().unwrap_or_default();
        if st.prefs.boxapp_settings.app_language == "system" && !config.initial_locale.is_empty() {
            // Keep "system" persisted while resolving display locale via initial_locale.
        }
        st.runtime_profile = db.get_runtime_profile().unwrap_or_default();
        st.core_config_files = list_core_config_files(&box_home, &st.runtime_profile.core_name);
        st.config_overrides = db
            .get_config_overrides(&st.runtime_profile.core_name)
            .unwrap_or_default();
        st.selected_uids = db.get_app_selection().unwrap_or_default();
        st.gid_list = db.get_gid_list().unwrap_or_default();
        st.cnip_settings = db.get_cnip_settings().unwrap_or_default();
        st.cnip_force_apps = db.get_cnip_force_apps().unwrap_or_default();
        st.cnip_force_uids = db.get_cnip_force_uids().unwrap_or_default();
        st.intranet_ipv4 = db.get_intranet_ipv4_cidrs().unwrap_or_default();
        st.intranet_ipv6 = db.get_intranet_ipv6_cidrs().unwrap_or_default();
        st.blocked_interfaces = db.get_blocked_interfaces().unwrap_or_default();
        st.shared_network_settings = db.get_shared_network_settings().unwrap_or_default();
        st.shared_network_interfaces = db.get_shared_network_interfaces().unwrap_or_default();
        st.shared_network_macs = db.get_shared_network_macs().unwrap_or_default();
        st.network_match_settings = db.get_network_match_settings().unwrap_or_default();
        st.network_match_ssids = db.get_network_match_ssids().unwrap_or_default();
        st.network_match_bssids = db.get_network_match_bssids().unwrap_or_default();
        st.network_match_conditions = db.get_network_match_conditions().unwrap_or_default();
        st.network_rule_groups = db.list_network_rule_groups().unwrap_or_default();
        st.mihomo_proxy_providers = file_store.load_proxy_providers().unwrap_or_default();
        st.builtin_update_sources = builtin_core_sources(
            st.prefs.core_channels.mihomo_stable,
            st.prefs.core_channels.sing_box_stable,
        );
        st.webui_one_time_token = format!("tok-{}", now_millis());
        refresh_api_and_webui_url(&mut st, &box_home);

        if !st.prefs.boxapp_settings.welcome_onboarding_shown {
            st.current_route = AppRoute::Onboarding;
        } else if st.prefs.boxapp_settings.open_panel_on_launch {
            st.current_route = panel_route_from_entry_mode(&st.prefs.boxapp_settings.home_panel_entry_mode);
        }

        let engine = Self {
            box_home,
            config,
            db,
            prefs_store,
            file_store,
            backup,
            root,
            platform,
            state: Arc::new(RwLock::new(st)),
            assets: Arc::new(RwLock::new(BTreeMap::new())),
            listeners: Arc::new(RwLock::new(BTreeMap::new())),
            next_sub_id: AtomicU64::new(1),
            version_counter: AtomicU64::new(1),
            runtime,
        };

        engine.sync_aliases();
        Ok(engine)
    }

    pub fn register_asset(&self, asset_path: &str, bytes: Vec<u8>) {
        if let Ok(mut guard) = self.assets.write() {
            guard.insert(asset_path.to_string(), bytes);
        }
    }

    pub fn subscribe(&self, view_id: String, listener: Arc<dyn StateListener>) -> u64 {
        let id = self.next_sub_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut guard) = self.listeners.write() {
            guard.insert(id, (view_id, listener.clone()));
        }
        let (ver, bytes) = self.snapshot_bytes();
        listener.on_state_snapshot(ver, &bytes);
        id
    }

    pub fn unsubscribe(&self, sub_id: u64) {
        if let Ok(mut guard) = self.listeners.write() {
            guard.remove(&sub_id);
        }
    }

    pub fn snapshot_bytes(&self) -> (u64, Vec<u8>) {
        let guard = self.state.read().expect("state lock poisoned");
        let bytes = serde_json::to_vec(&*guard).unwrap_or_else(|_| b"{}".to_vec());
        (guard.version, bytes)
    }

    fn mutate_and_broadcast<F: FnOnce(&mut AppState)>(&self, f: F) -> Vec<u8> {
        let (ver, bytes) = {
            let mut guard = self.state.write().expect("state lock poisoned");
            f(&mut guard);
            let next_v = self.version_counter.fetch_add(1, Ordering::SeqCst) + 1;
            guard.version = next_v;
            let bytes = serde_json::to_vec(&*guard).unwrap_or_else(|_| b"{}".to_vec());
            (next_v, bytes)
        };
        let listeners: Vec<Arc<dyn StateListener>> = self
            .listeners
            .read()
            .map(|m| m.values().map(|(_, l)| l.clone()).collect())
            .unwrap_or_default();
        for l in listeners {
            l.on_state_snapshot(ver, &bytes);
        }
        bytes
    }

    fn sync_aliases(&self) {
        let (icon_variant, tile_target) = {
            let st = self.state.read().expect("state lock");
            (
                st.prefs.app_icon.variant.clone(),
                st.prefs.boxapp_settings.tile_long_click_target.clone(),
            )
        };
        let alt = icon_variant == "alternate";
        self.platform
            .set_component_alias("MainLauncherDefault", !alt);
        self.platform
            .set_component_alias("MainLauncherAlternate", alt);
        self.platform.set_component_alias(
            "BoxTilePreferencesDefaultActivity",
            tile_target == "panel" && !alt,
        );
        self.platform.set_component_alias(
            "BoxTilePreferencesAlternateActivity",
            tile_target == "panel" && alt,
        );
        self.platform.set_component_alias(
            "BoxTilePreferencesWebUiActivity",
            tile_target == "web_ui",
        );
        self.platform
            .set_component_alias("BoxTilePreferencesAppActivity", tile_target == "app");
    }

    /// Dispatch a JSON-encoded `AppAction` (used directly by JNI `nativeDispatch` and host tests).
    pub fn dispatch_json(&self, action_bytes: &[u8]) -> EngineResult<Vec<u8>> {
        let action: AppAction = serde_json::from_slice(action_bytes)
            .map_err(|e| format!("invalid AppAction JSON: {e}"))?;
        self.dispatch(action)
    }

    pub fn dispatch(&self, action: AppAction) -> EngineResult<Vec<u8>> {
        match action {
            // -----------------------------------------------------------------
            // Navigation & OS launch
            // -----------------------------------------------------------------
            AppAction::Navigate { route } => Ok(self.mutate_and_broadcast(|st| {
                if st.current_route != route {
                    st.back_stack.push(st.current_route.clone());
                    st.current_route = route;
                }
            })),
            AppAction::PopBack => Ok(self.mutate_and_broadcast(|st| {
                if let Some(prev) = st.back_stack.pop() {
                    st.current_route = prev;
                } else {
                    st.current_route = AppRoute::Main;
                }
            })),
            AppAction::SwitchTab { tab } => Ok(self.mutate_and_broadcast(|st| {
                st.active_tab = tab.min(3);
                st.back_stack.clear();
                st.current_route = AppRoute::Main;
            })),
            AppAction::HandleLaunchTarget { target } => {
                match target.as_str() {
                    "panel" => self.platform.on_effect(&AppEffect::OpenPanelSheet),
                    "web_ui" => {
                        let url = self.state.read().expect("state").webui_resolved_url.clone();
                        self.mutate_and_broadcast(|st| st.current_route = AppRoute::WebUi);
                        self.platform.on_effect(&AppEffect::OpenWebUi { url });
                    }
                    _ => {
                        self.mutate_and_broadcast(|st| {
                            st.active_tab = 0;
                            st.current_route = AppRoute::Main;
                        });
                    }
                }
                Ok(self.snapshot_bytes().1)
            }

            // -----------------------------------------------------------------
            // M01–M08 Home & Service Control
            // -----------------------------------------------------------------
            AppAction::RefreshStatus => {
                let box_home = self.box_home.clone();
                let root = self.root.clone();
                let (api_kind, base_url, secret, clash_url, core_name, need_history) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                        st.runtime_profile.core_name.clone(),
                        st.net_speed_history.len() < 3,
                    )
                };
                let granted = self.runtime.block_on(check_root_granted(root.as_ref()));
                let mut proc_metrics = sample_process(&box_home, None, 1.0);
                let cfg_files = list_core_config_files(&box_home, &core_name);

                // Enrich live metrics from Honk Native API / Clash API if reachable.
                let live_snap = self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        if let Ok(client) = HonkNativeClient::new(&base_url, &secret) {
                            if let Ok(rt) = client.runtime().await {
                                let mem = client.runtime_memory().await.ok();
                                let groups = client.groups().await.ok();
                                let rules = client.rules().await.ok();
                                let conns = client.connections().await.ok();
                                let hist = if need_history {
                                    client.traffic_history(60, 60).await.ok()
                                } else {
                                    None
                                };
                                let clash_cfg = if let Some(ref cu) = clash_url {
                                    if let Ok(cc) = ClashApiClient::new(cu, &secret) {
                                        cc.get_configs().await.ok()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                return Some((rt, mem, groups, rules, conns, hist, clash_cfg));
                            }
                        }
                    }
                    None
                });

                let out = self.mutate_and_broadcast(|st| {
                    st.root_granted = granted;
                    st.core_config_files = cfg_files;
                    if let Some((rt, mem, groups, rules, conns, hist, clash_cfg)) = live_snap {
                        if !proc_metrics.running {
                            let running = rt
                                .get("lifecycle")
                                .and_then(|l| l.get("state"))
                                .and_then(|s| s.as_str())
                                .map(|s| s.eq_ignore_ascii_case("running") || s.eq_ignore_ascii_case("active"))
                                .unwrap_or(true);
                            proc_metrics.running = running;
                        }
                        if let Some(up) = rt
                            .get("lifecycle")
                            .and_then(|l| l.get("uptime_seconds"))
                            .and_then(json_u64)
                        {
                            proc_metrics.uptime_secs = up;
                        }
                        if let Some(p) = rt.get("process") {
                            if let Some(pid) = p.get("pid").and_then(json_u64) {
                                proc_metrics.pid = Some(pid as u32);
                            }
                            if let Some(up) = p.get("uptime_seconds").and_then(json_u64) {
                                if proc_metrics.uptime_secs == 0 {
                                    proc_metrics.uptime_secs = up;
                                }
                            }
                            if let Some(cpu) = p.get("cpu_percent").and_then(json_f64) {
                                proc_metrics.cpu_percent = cpu as f32;
                            }
                            if let Some(rss) = p.get("rss_bytes").and_then(json_u64) {
                                proc_metrics.pss_mb = rss as f64 / (1024.0 * 1024.0);
                                proc_metrics.rss_mb = proc_metrics.pss_mb;
                            }
                        }
                        if let Some(dp) = rt.get("datapath") {
                            st.overview_datapath = dp.clone();
                        }
                        if let Some(m) = mem {
                            let rss_bytes = m
                                .get("process")
                                .and_then(|p| p.get("rss_bytes"))
                                .and_then(json_u64);
                            let cgroup_bytes = m
                                .get("cgroup")
                                .and_then(|c| c.get("current_bytes"))
                                .and_then(json_u64);
                            if let Some(b) = rss_bytes.or(cgroup_bytes) {
                                proc_metrics.rss_mb = b as f64 / (1024.0 * 1024.0);
                                proc_metrics.pss_mb = cgroup_bytes
                                    .unwrap_or(b) as f64
                                    / (1024.0 * 1024.0);
                            }
                            st.overview_memory = m;
                        }
                        if let Some(t) = rt.get("traffic") {
                            let up_bps = t
                                .get("rates")
                                .and_then(|r| r.get("upload_bytes_per_second"))
                                .and_then(json_u64)
                                .or_else(|| t.get("upload_bps").and_then(json_u64))
                                .unwrap_or(0);
                            let down_bps = t
                                .get("rates")
                                .and_then(|r| r.get("download_bytes_per_second"))
                                .and_then(json_u64)
                                .or_else(|| t.get("download_bps").and_then(json_u64))
                                .unwrap_or(0);
                            let tot_up = t
                                .get("bytes")
                                .and_then(|b| b.get("upload"))
                                .and_then(json_u64)
                                .or_else(|| t.get("upload_bytes").and_then(json_u64))
                                .unwrap_or(st.total_upload);
                            let tot_down = t
                                .get("bytes")
                                .and_then(|b| b.get("download"))
                                .and_then(json_u64)
                                .or_else(|| t.get("download_bytes").and_then(json_u64))
                                .unwrap_or(st.total_download);
                            if let Some(conn_tot) = t
                                .get("connections")
                                .and_then(|c| c.get("total"))
                                .and_then(json_u64)
                            {
                                st.active_connections_count = conn_tot as usize;
                            }
                            st.upload_bps = up_bps;
                            st.download_bps = down_bps;
                            st.total_upload = tot_up;
                            st.total_download = tot_down;
                            if let Some(h) = hist {
                                if let Some(pts) = h.get("points").and_then(|p| p.as_array()) {
                                    let seeded: Vec<SpeedPoint> = pts
                                        .iter()
                                        .rev()
                                        .take(60)
                                        .rev()
                                        .map(|pt| SpeedPoint {
                                            timestamp_ms: now_millis(),
                                            upload_bps: pt
                                                .get("upload_bytes_per_second")
                                                .and_then(json_u64)
                                                .unwrap_or(0),
                                            download_bps: pt
                                                .get("download_bytes_per_second")
                                                .and_then(json_u64)
                                                .unwrap_or(0),
                                        })
                                        .collect();
                                    if !seeded.is_empty() {
                                        st.net_speed_history = seeded;
                                    }
                                }
                                st.overview_traffic_history = h;
                            }
                            if st.net_speed_history.len() >= 60 {
                                st.net_speed_history.remove(0);
                            }
                            st.net_speed_history.push(SpeedPoint {
                                timestamp_ms: now_millis(),
                                upload_bps: up_bps,
                                download_bps: down_bps,
                            });
                        }
                        if let Some(g) = groups {
                            if let Some(arr) = g.as_array() {
                                st.groups_count = arr.len();
                            }
                            st.honk_groups_data = g;
                        }
                        if let Some(r) = rules {
                            let count = r
                                .as_array()
                                .map(|a| a.len())
                                .or_else(|| r.get("rules").and_then(|x| x.as_array()).map(|a| a.len()))
                                .unwrap_or(0);
                            if count > 0 {
                                st.rules_count = count;
                            }
                            st.rules_data = r;
                        }
                        if let Some(c) = conns {
                            let tcp_len = c.get("tcp").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                            let udp_len = c.get("udp").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                            let flat_len = c
                                .as_array()
                                .map(|a| a.len())
                                .or_else(|| c.get("connections").and_then(|x| x.as_array()).map(|a| a.len()))
                                .unwrap_or(0);
                            let count = if tcp_len + udp_len > 0 {
                                tcp_len + udp_len
                            } else {
                                flat_len
                            };
                            if count > 0 {
                                st.active_connections_count = count;
                            }
                        }
                        if let Some(cc) = clash_cfg {
                            if !cc.mode.is_empty() {
                                st.outbound_mode = cc.mode.to_ascii_lowercase();
                            }
                            if !cc.mode_list.is_empty() {
                                st.outbound_mode_list = cc
                                    .mode_list
                                    .into_iter()
                                    .map(|m| m.to_ascii_lowercase())
                                    .collect();
                            }
                        }
                    }
                    st.process = proc_metrics.clone();
                    st.service_status = if proc_metrics.running {
                        ServiceStatus::Running
                    } else {
                        ServiceStatus::Stopped
                    };
                    refresh_api_and_webui_url(st, &box_home);
                });
                Ok(out)
            }
            AppAction::ServiceUp => self.run_service_transition("up"),
            AppAction::ServiceDown => self.run_service_transition("down"),
            AppAction::ServiceRestart => self.run_service_transition("restart"),
            AppAction::ServiceReload => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let res = self.runtime.block_on(async {
                    match api_kind {
                        ApiKind::HonkNative => {
                            HonkNativeClient::new(&base_url, &secret)?
                                .reload()
                                .await
                                .map(|_| ())
                        }
                        ApiKind::ClashApi => ClashApiClient::new(&base_url, &secret)?.reload_config().await,
                        ApiKind::Unsupported => {
                            if let Some(cu) = clash_url {
                                ClashApiClient::new(&cu, &secret)?.reload_config().await
                            } else {
                                Err("core does not support hot reload".to_string())
                            }
                        }
                    }
                });
                match res {
                    Ok(()) => {
                        self.platform.on_effect(&AppEffect::Toast {
                            message: "Reloaded".into(),
                            is_error: false,
                        });
                        Ok(self.snapshot_bytes().1)
                    }
                    Err(e) => {
                        self.record_error(&e);
                        Err(e)
                    }
                }
            }
            AppAction::SwitchOutboundMode { mode } => {
                let normalized = mode.trim().to_ascii_lowercase();
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let target_url = clash_url.unwrap_or(base_url);
                self.runtime.block_on(async {
                    if let Ok(client) = ClashApiClient::new(&target_url, &secret) {
                        let _ = client.set_mode(&normalized).await;
                    }
                });
                Ok(self.mutate_and_broadcast(|st| st.outbound_mode = normalized))
            }
            AppAction::ProbeLatencyTargets => {
                let targets: Vec<(String, String)> = {
                    let st = self.state.read().expect("state");
                    st.prefs
                        .latency_targets
                        .targets
                        .iter()
                        .map(|t| (t.name.clone(), t.url.clone()))
                        .collect()
                };
                let results = self.runtime.block_on(probe_latency_targets(&targets));
                Ok(self.mutate_and_broadcast(|st| st.latency_results = results))
            }
            AppAction::RefreshIpGeo => {
                let geo = self.runtime.block_on(probe_wan_geo())?;
                Ok(self.mutate_and_broadcast(|st| st.ip_geo = geo))
            }
            AppAction::ToggleHomeIpMode => {
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.home_ui.home_ip_mode = if st.prefs.home_ui.home_ip_mode == "wan" {
                        "lan".into()
                    } else {
                        "wan".into()
                    };
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::RefreshSubscriptions => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let data = self.runtime.block_on(async {
                    match api_kind {
                        ApiKind::HonkNative => HonkNativeClient::new(&base_url, &secret)?.providers().await,
                        _ => {
                            let url = clash_url.unwrap_or(base_url);
                            ClashApiClient::new(&url, &secret)?.get_proxy_providers().await
                        }
                    }
                })?;
                Ok(self.mutate_and_broadcast(|st| st.proxy_providers_data = data))
            }
            AppAction::IgnoreCoreUpdateVersion { core, version } => {
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs
                        .core_update_monitor
                        .ignored_versions
                        .insert(core, version);
                    st.pending_core_update = None;
                });
                self.persist_prefs()?;
                Ok(out)
            }

            // -----------------------------------------------------------------
            // M09–M16 Panel & WebUI
            // -----------------------------------------------------------------
            AppAction::RefreshOverview => {
                let (api_kind, base_url, secret, window) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.prefs.overview.top_traffic_window.clone(),
                    )
                };
                if api_kind == ApiKind::HonkNative {
                    let secs = parse_window_seconds(&window);
                    let (hist, dp, mem) = self.runtime.block_on(async {
                        let client = HonkNativeClient::new(&base_url, &secret)?;
                        let h = client.traffic_history(secs, 60).await.unwrap_or(serde_json::Value::Null);
                        let d = client.datapath().await.unwrap_or(serde_json::Value::Null);
                        let m = client.runtime_memory().await.unwrap_or(serde_json::Value::Null);
                        Ok::<_, String>((h, d, m))
                    })?;
                    return Ok(self.mutate_and_broadcast(|st| {
                        st.overview_traffic_history = hist;
                        st.overview_datapath = dp;
                        st.overview_memory = mem;
                    }));
                }
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateOverviewPrefs { prefs } => {
                let out = self.mutate_and_broadcast(|st| st.prefs.overview = prefs);
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::ClearClashApiHistory => {
                let client = BoxctlClient::new(&self.box_home);
                let _ = self.runtime.block_on(client.clashapi_clear(self.root.as_ref()));
                Ok(self.mutate_and_broadcast(|st| st.overview_traffic_history = serde_json::Value::Null))
            }
            AppAction::RefreshConnections => {
                let paused = self.state.read().expect("state").connections_paused;
                if paused {
                    return Ok(self.snapshot_bytes().1);
                }
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let url = clash_url.unwrap_or(base_url);
                let snap = self.runtime.block_on(async {
                    ClashApiClient::new(&url, &secret)?.get_connections().await
                })?;
                Ok(self.mutate_and_broadcast(|st| {
                    let prev_up = st.total_upload;
                    let prev_down = st.total_download;
                    st.upload_bps = snap.upload_total.saturating_sub(prev_up);
                    st.download_bps = snap.download_total.saturating_sub(prev_down);
                    st.total_upload = snap.upload_total;
                    st.total_download = snap.download_total;
                    st.active_connections_count = snap.connections.len();
                    if st.net_speed_history.len() >= 60 {
                        st.net_speed_history.remove(0);
                    }
                    st.net_speed_history.push(SpeedPoint {
                        timestamp_ms: now_millis(),
                        upload_bps: st.upload_bps,
                        download_bps: st.download_bps,
                    });
                    st.connections_snapshot = snap;
                }))
            }
            AppAction::SetConnectionsPaused { paused } => {
                Ok(self.mutate_and_broadcast(|st| st.connections_paused = paused))
            }
            AppAction::CloseConnection { id } => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        HonkNativeClient::new(&base_url, &secret)?.close_connection(&id).await.map(|_| ())
                    } else {
                        ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                            .close_connection(&id)
                            .await
                    }
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::CloseAllConnections => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        HonkNativeClient::new(&base_url, &secret)?.close_all_connections().await.map(|_| ())
                    } else {
                        ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                            .close_all_connections()
                            .await
                    }
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateConnectionPrefs { prefs } => {
                let out = self.mutate_and_broadcast(|st| st.prefs.connections = prefs);
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::RefreshRules => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let data = self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        HonkNativeClient::new(&base_url, &secret)?.rules().await
                    } else {
                        ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?.get_rules().await
                    }
                })?;
                Ok(self.mutate_and_broadcast(|st| st.rules_data = data))
            }
            AppAction::TraceRoute { request } => {
                let (base_url, secret) = {
                    let st = self.state.read().expect("state");
                    (st.active_api.base_url.clone(), st.active_api.secret.clone())
                };
                let res = self.runtime.block_on(async {
                    HonkNativeClient::new(&base_url, &secret)?
                        .routing_trace(&request)
                        .await
                })?;
                Ok(self.mutate_and_broadcast(|st| st.route_trace_result = res))
            }
            AppAction::RefreshRuleProviders => {
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let data = self.runtime.block_on(async {
                    ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                        .get_rule_providers()
                        .await
                })?;
                Ok(self.mutate_and_broadcast(|st| st.rule_providers_data = data))
            }
            AppAction::UpdateRuleProvider { name } => {
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                self.runtime.block_on(async {
                    ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                        .update_rule_provider(&name)
                        .await
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateAllRuleProviders => {
                let names: Vec<String> = {
                    let st = self.state.read().expect("state");
                    st.rule_providers_data
                        .get("providers")
                        .and_then(|p| p.as_object())
                        .map(|o| o.keys().cloned().collect())
                        .unwrap_or_default()
                };
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                let url = clash_url.unwrap_or(base_url);
                self.runtime.block_on(async {
                    let client = ClashApiClient::new(&url, &secret)?;
                    for name in names {
                        let _ = client.update_rule_provider(&name).await;
                    }
                    Ok::<_, String>(())
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateProxyProvider { name } => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        HonkNativeClient::new(&base_url, &secret)?
                            .refresh_provider(&name)
                            .await
                            .map(|_| ())
                    } else {
                        ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                            .update_proxy_provider(&name)
                            .await
                    }
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::HealthcheckProxyProvider { name } => {
                let (base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                self.runtime.block_on(async {
                    ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                        .healthcheck_proxy_provider(&name)
                        .await
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::SelectWebPanel { panel_id } => {
                let box_home = self.box_home.clone();
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.web_panel.webui_selected_panel = panel_id;
                    refresh_api_and_webui_url(st, &box_home);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::SetWebUiLocalMode { mode } => {
                let box_home = self.box_home.clone();
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.web_panel.webui_local_panel_mode = mode;
                    refresh_api_and_webui_url(st, &box_home);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::AddCustomWebPanel { panel, is_substore } => {
                let out = self.mutate_and_broadcast(|st| {
                    let list = if is_substore {
                        &mut st.prefs.web_panel.substore_custom_panels
                    } else {
                        &mut st.prefs.web_panel.webui_custom_panels
                    };
                    list.retain(|p| p.id != panel.id);
                    list.push(panel);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::DeleteCustomWebPanel { panel_id, is_substore } => {
                let out = self.mutate_and_broadcast(|st| {
                    let list = if is_substore {
                        &mut st.prefs.web_panel.substore_custom_panels
                    } else {
                        &mut st.prefs.web_panel.webui_custom_panels
                    };
                    list.retain(|p| p.id != panel_id);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::HandleWebMessage { raw_json } => {
                let cmd: WebMessageCommand =
                    serde_json::from_str(&raw_json).map_err(|e| format!("bad WebMessage: {e}"))?;
                match cmd {
                    WebMessageCommand::ZashboardDetected { page_url, token } => {
                        let expected = self.state.read().expect("state").webui_one_time_token.clone();
                        if token == expected {
                            let out = self.mutate_and_broadcast(|st| {
                                st.prefs.zashboard_monet_panels.insert(page_url);
                            });
                            self.persist_prefs()?;
                            return Ok(out);
                        }
                    }
                    WebMessageCommand::SaveFile {
                        base64,
                        file_name,
                        mime_type,
                    } => {
                        let _decoded = decode_save_file_payload(&base64)?;
                        self.platform.on_effect(&AppEffect::SaveFile {
                            file_name,
                            mime_type,
                            base64,
                        });
                    }
                }
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateCustomApi {
                enabled,
                endpoint,
                secret,
            } => {
                let box_home = self.box_home.clone();
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.proxy_selector.custom_api_enabled = enabled;
                    st.prefs.proxy_selector.custom_api_endpoint = endpoint;
                    st.prefs.proxy_selector.custom_api_secret = secret;
                    refresh_api_and_webui_url(st, &box_home);
                });
                self.persist_prefs()?;
                Ok(out)
            }

            // -----------------------------------------------------------------
            // M17–M20 Proxy Selector
            // -----------------------------------------------------------------
            AppAction::RefreshProxies => {
                let (api_kind, base_url, secret, clash_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                    )
                };
                if api_kind == ApiKind::HonkNative {
                    let (groups, proxies) = self.runtime.block_on(async {
                        let g = HonkNativeClient::new(&base_url, &secret)?
                            .groups()
                            .await?;
                        let p = if let Some(ref cu) = clash_url {
                            if let Ok(cc) = ClashApiClient::new(cu, &secret) {
                                cc.get_proxies().await.unwrap_or_default()
                            } else {
                                BTreeMap::new()
                            }
                        } else {
                            BTreeMap::new()
                        };
                        Ok::<_, String>((g, p))
                    })?;
                    return Ok(self.mutate_and_broadcast(|st| {
                        if let Some(arr) = groups.as_array() {
                            st.groups_count = arr.len();
                        }
                        st.honk_groups_data = groups;
                        if !proxies.is_empty() {
                            st.proxies_data = proxies;
                        }
                    }));
                }
                let proxies = self.runtime.block_on(async {
                    ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?
                        .get_proxies()
                        .await
                })?;
                Ok(self.mutate_and_broadcast(|st| st.proxies_data = proxies))
            }
            AppAction::SelectProxy { group, node } => {
                let (api_kind, base_url, secret, clash_url, disconnect) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.kind,
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                        st.prefs.proxy_selector.disconnect_on_select,
                    )
                };
                self.runtime.block_on(async {
                    if api_kind == ApiKind::HonkNative {
                        let c = HonkNativeClient::new(&base_url, &secret)?;
                        c.select_group_member(&group, &node).await?;
                        if disconnect {
                            let _ = c.close_all_connections().await;
                        }
                    } else {
                        let c = ClashApiClient::new(clash_url.unwrap_or(base_url), &secret)?;
                        c.select_proxy(&group, &node).await?;
                        if disconnect {
                            let _ = c.close_all_connections().await;
                        }
                    }
                    Ok::<_, String>(())
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::TestProxyDelay { node } => {
                let (base_url, secret, clash_url, test_url, detect_v6) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                        st.prefs.proxy_selector.custom_latency_test_url.clone(),
                        st.prefs.proxy_selector.detect_ipv6_on_latency_test,
                    )
                };
                let url = clash_url.unwrap_or(base_url);
                let v6_ok = self.runtime.block_on(async {
                    let c = ClashApiClient::new(&url, &secret)?;
                    let _ = c.test_proxy_delay(&node, &test_url, 3000).await?;
                    let v6 = if detect_v6 {
                        c.test_proxy_delay(&node, "https://ipv6.google.com/generate_204", 2000)
                            .await
                            .is_ok()
                    } else {
                        false
                    };
                    Ok::<_, String>(v6)
                })?;
                Ok(self.mutate_and_broadcast(|st| {
                    if v6_ok {
                        st.ipv6_capable_nodes.insert(node);
                    }
                }))
            }
            AppAction::TestGroupDelay { group } => {
                let (base_url, secret, clash_url, test_url) = {
                    let st = self.state.read().expect("state");
                    (
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        st.active_api.clash_fallback_url.clone(),
                        st.prefs.proxy_selector.custom_latency_test_url.clone(),
                    )
                };
                let url = clash_url.unwrap_or(base_url);
                self.runtime.block_on(async {
                    ClashApiClient::new(&url, &secret)?
                        .test_group_delay(&group, &test_url, 3000)
                        .await
                })?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::UpdateProxySelectorPrefs { prefs } => {
                let box_home = self.box_home.clone();
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.proxy_selector = prefs;
                    refresh_api_and_webui_url(st, &box_home);
                });
                self.persist_prefs()?;
                Ok(out)
            }

            // -----------------------------------------------------------------
            // M21–M29 Core, App, CNIP, Bypass, Shared Network, Network Match
            // -----------------------------------------------------------------
            AppAction::SaveRuntimeProfile {
                profile,
                apply_live,
            } => {
                let _ = CoreKind::parse(&profile.core_name)?;
                let _ = NetworkMode::parse(&profile.mode)?;
                self.db.save_runtime_profile(&profile)?;
                let overrides = self
                    .db
                    .get_config_overrides(&profile.core_name)
                    .unwrap_or_default();
                let box_home = self.box_home.clone();
                let is_ebpf = profile.mode == "ebpf";
                let running = self.state.read().expect("state").service_status.is_running();
                let out = self.mutate_and_broadcast(|st| {
                    st.runtime_profile = profile;
                    st.core_config_files = list_core_config_files(&box_home, &st.runtime_profile.core_name);
                    st.config_overrides = overrides;
                    refresh_api_and_webui_url(st, &box_home);
                });
                if apply_live && running {
                    let client = BoxctlClient::new(&self.box_home);
                    if is_ebpf {
                        self.runtime.block_on(client.restart(self.root.as_ref()))?;
                    } else {
                        self.runtime.block_on(client.mode_renew(self.root.as_ref()))?;
                    }
                }
                Ok(out)
            }
            AppAction::SetConfigOverride {
                core,
                option,
                value,
            } => {
                self.db.set_config_override(&core, &option, &value)?;
                let overrides = self.db.get_config_overrides(&core)?;
                Ok(self.mutate_and_broadcast(|st| st.config_overrides = overrides))
            }
            AppAction::ConfigSync { manual } => {
                let client = BoxctlClient::new(&self.box_home);
                self.runtime
                    .block_on(client.config_sync(self.root.as_ref(), manual))?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::ApplyResources => {
                let client = BoxctlClient::new(&self.box_home);
                self.runtime
                    .block_on(client.resource_apply(self.root.as_ref()))?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::RefreshInstalledApps => {
                let apps = self.platform.query_installed_apps();
                Ok(self.mutate_and_broadcast(|st| st.installed_apps = apps))
            }
            AppAction::SaveAppSelection {
                uids,
                gids,
                proxy_mode,
            } => {
                self.db.set_app_selection(&uids)?;
                self.db.set_gid_list(&gids)?;
                let mut profile = self.db.get_runtime_profile()?;
                profile.proxy_mode = proxy_mode;
                self.db.save_runtime_profile(&profile)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.selected_uids = uids;
                    st.gid_list = gids;
                    st.runtime_profile = profile;
                });
                self.hot_renew_if_running();
                Ok(out)
            }
            AppAction::RunSmartSelect { mode } => {
                let apps = {
                    let existing = self.state.read().expect("state").installed_apps.clone();
                    if existing.is_empty() {
                        self.platform.query_installed_apps()
                    } else {
                        existing
                    }
                };
                let picked = classify_smart_select(&apps, mode);
                self.db.set_app_selection(&picked)?;
                Ok(self.mutate_and_broadcast(|st| {
                    st.installed_apps = apps;
                    st.selected_uids = picked;
                }))
            }
            AppAction::SaveCnipSettings { settings } => {
                for line in settings.ipv4_additions.lines().filter(|l| !l.trim().is_empty()) {
                    validate_cidr_v4(line)?;
                }
                for line in settings.ipv6_additions.lines().filter(|l| !l.trim().is_empty()) {
                    validate_cidr_v6(line)?;
                }
                self.db.save_cnip_settings(&settings)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.runtime_profile.bypass_cn = settings.bypass_cnip;
                    st.cnip_settings = settings;
                });
                self.hot_renew_if_running();
                Ok(out)
            }
            AppAction::SaveCnipForceApps {
                apps,
                resolved_uids,
            } => {
                self.db.set_cnip_force_apps(&apps, &resolved_uids)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.cnip_force_apps = apps;
                    st.cnip_force_uids = resolved_uids;
                });
                self.hot_renew_if_running();
                Ok(out)
            }
            AppAction::SaveBypassRules {
                bypass_intranet: _,
                ipv4,
                ipv6,
                blocked_interfaces,
            } => {
                for cidr in &ipv4 {
                    validate_cidr_v4(cidr)?;
                }
                for cidr in &ipv6 {
                    validate_cidr_v6(cidr)?;
                }
                self.db.set_intranet_ipv4_cidrs(&ipv4)?;
                self.db.set_intranet_ipv6_cidrs(&ipv6)?;
                self.db.set_blocked_interfaces(&blocked_interfaces)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.intranet_ipv4 = ipv4;
                    st.intranet_ipv6 = ipv6;
                    st.blocked_interfaces = blocked_interfaces;
                });
                self.hot_renew_if_running();
                Ok(out)
            }
            AppAction::ResetDefaultIntranetCidrs => {
                let v4 = self.db.reset_intranet_ipv4_cidrs()?;
                let v6 = self.db.reset_intranet_ipv6_cidrs()?;
                Ok(self.mutate_and_broadcast(|st| {
                    st.intranet_ipv4 = v4;
                    st.intranet_ipv6 = v6;
                }))
            }
            AppAction::SaveSharedNetwork {
                settings,
                interfaces,
                macs,
            } => {
                self.db.save_shared_network_settings(&settings)?;
                self.db.set_shared_network_interfaces(&interfaces)?;
                self.db.set_shared_network_macs(&macs)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.shared_network_settings = settings;
                    st.shared_network_interfaces = interfaces;
                    st.shared_network_macs = macs;
                });
                self.hot_renew_if_running();
                Ok(out)
            }
            AppAction::ScanArpClients => {
                let text = std::fs::read_to_string("/proc/net/arp").unwrap_or_default();
                let clients = parse_arp_table(&text);
                Ok(self.mutate_and_broadcast(|st| st.arp_clients = clients))
            }
            AppAction::SaveNetworkMatchSettings {
                settings,
                ssids,
                bssids,
                conditions,
            } => {
                self.db.save_network_match_settings(&settings)?;
                self.db.set_network_match_ssids(&ssids)?;
                self.db.set_network_match_bssids(&bssids)?;
                self.db.set_network_match_conditions(&conditions)?;
                Ok(self.mutate_and_broadcast(|st| {
                    st.network_match_settings = settings;
                    st.network_match_ssids = ssids;
                    st.network_match_bssids = bssids;
                    st.network_match_conditions = conditions;
                }))
            }
            AppAction::UpsertNetworkRuleGroup { group } => {
                self.db.upsert_network_rule_group(&group)?;
                let all = self.db.list_network_rule_groups()?;
                Ok(self.mutate_and_broadcast(|st| st.network_rule_groups = all))
            }
            AppAction::DeleteNetworkRuleGroup { group_id } => {
                self.db.delete_network_rule_group(group_id)?;
                let all = self.db.list_network_rule_groups()?;
                Ok(self.mutate_and_broadcast(|st| st.network_rule_groups = all))
            }
            AppAction::ApplyNetworkMatch => {
                let client = BoxctlClient::new(&self.box_home);
                self.runtime
                    .block_on(client.network_match_apply(self.root.as_ref()))?;
                let _ = self.runtime.block_on(client.monitor(self.root.as_ref()));
                Ok(self.snapshot_bytes().1)
            }
            AppAction::EvaluateNetworkObservation { observation } => {
                let matched_action: Option<String> = {
                    let st = self.state.read().expect("state");
                    match_first_rule_group(&st.network_rule_groups, &observation)
                        .and_then(|g| g.actions.first().map(|a| a.action_type.clone()))
                };
                self.mutate_and_broadcast(|st| st.last_network_observation = observation);
                if let Some(act) = matched_action {
                    match act.as_str() {
                        "start" | "enable" | "up" => {
                            let _ = self.run_service_transition("up");
                        }
                        "stop" | "disable" | "down" => {
                            let _ = self.run_service_transition("down");
                        }
                        "restart" => {
                            let _ = self.run_service_transition("restart");
                        }
                        _ => {}
                    }
                }
                Ok(self.snapshot_bytes().1)
            }

            // -----------------------------------------------------------------
            // M30–M35 Subscriptions YAML, Scripts, Runtime Files, Editor, Logs
            // -----------------------------------------------------------------
            AppAction::SaveProxyProviders { entries } => {
                self.file_store.save_proxy_providers(&entries)?;
                Ok(self.mutate_and_broadcast(|st| st.mihomo_proxy_providers = entries))
            }
            AppAction::ListJsScripts { core } => {
                let scripts = self.file_store.list_js_scripts(&core)?;
                Ok(self.mutate_and_broadcast(|st| st.js_scripts = scripts))
            }
            AppAction::SaveJsScript {
                core,
                name,
                content,
            } => {
                self.file_store.save_js_script(&core, &name, &content)?;
                let scripts = self.file_store.list_js_scripts(&core)?;
                Ok(self.mutate_and_broadcast(|st| st.js_scripts = scripts))
            }
            AppAction::SetEnabledJsScripts { core, ordered } => {
                self.file_store.set_enabled_js_scripts(&core, &ordered)?;
                let scripts = self.file_store.list_js_scripts(&core)?;
                Ok(self.mutate_and_broadcast(|st| st.js_scripts = scripts))
            }
            AppAction::ListShellScripts => {
                let (pre, post) = {
                    let st = self.state.read().expect("state");
                    (
                        st.prefs.shell_scripts.pre_start_script.clone(),
                        st.prefs.shell_scripts.post_stop_script.clone(),
                    )
                };
                let scripts = self.file_store.list_shell_scripts(&pre, &post)?;
                Ok(self.mutate_and_broadcast(|st| st.shell_scripts = scripts))
            }
            AppAction::SaveShellScript { name, content } => {
                self.file_store.save_shell_script(&name, content.as_bytes())?;
                self.dispatch(AppAction::ListShellScripts)
            }
            AppAction::SetShellHooks {
                pre_start,
                post_stop,
            } => {
                self.db.set_app_setting("shell_scripts_pre_start", &pre_start)?;
                self.db.set_app_setting("shell_scripts_post_stop", &post_stop)?;
                self.mutate_and_broadcast(|st| {
                    st.prefs.shell_scripts.pre_start_script = pre_start;
                    st.prefs.shell_scripts.post_stop_script = post_stop;
                });
                self.persist_prefs()?;
                self.dispatch(AppAction::ListShellScripts)
            }
            AppAction::RunShellScript { name, event } => {
                let path = self.file_store.resolve_safe(&format!("scripts/{name}"))?;
                let profile = self.state.read().expect("state").runtime_profile.clone();
                let ctx = HookContext {
                    event,
                    box_home: self.box_home.clone(),
                    core: profile.core_name,
                    config_name: profile.config_name,
                    mode: profile.mode,
                    proxy_mode: profile.proxy_mode,
                };
                let res = self
                    .runtime
                    .block_on(run_shell_hook(self.root.as_ref(), &path, &ctx))?;
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.shell_scripts.last_runs.insert(
                        name.clone(),
                        boxproxy_store::prefs::ScriptLastRun {
                            exit_code: res.exit_code,
                            at_millis: res.at_millis,
                        },
                    );
                    st.last_hook_result = Some(res);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::ListRuntimeFiles { dir } => {
                let files = self.file_store.list_dir(&dir)?;
                Ok(self.mutate_and_broadcast(|st| {
                    st.runtime_files_dir = dir;
                    st.runtime_files = files;
                }))
            }
            AppAction::OpenEditorFile { relative_path } => {
                let content = self.file_store.read_text(&relative_path)?;
                let diags = validate_config_text(&relative_path, &content);
                Ok(self.mutate_and_broadcast(|st| {
                    st.editor_path = relative_path.clone();
                    st.editor_content = content;
                    st.editor_diagnostics = diags;
                    st.current_route = AppRoute::RuntimeFileEditor { relative_path };
                }))
            }
            AppAction::SaveEditorFile {
                relative_path,
                content,
            } => {
                self.file_store.write_text(&relative_path, &content)?;
                let diags = validate_config_text(&relative_path, &content);
                Ok(self.mutate_and_broadcast(|st| {
                    st.editor_path = relative_path;
                    st.editor_content = content;
                    st.editor_diagnostics = diags;
                }))
            }
            AppAction::ValidateEditorText { filename, content } => {
                let diags = validate_config_text(&filename, &content);
                Ok(self.mutate_and_broadcast(|st| st.editor_diagnostics = diags))
            }
            AppAction::RenameRuntimeFile { from, to } => {
                self.file_store.rename(&from, &to)?;
                let dir = self.state.read().expect("state").runtime_files_dir.clone();
                self.dispatch(AppAction::ListRuntimeFiles { dir })
            }
            AppAction::DeleteRuntimeFile { relative_path } => {
                self.file_store.delete(&relative_path)?;
                let dir = self.state.read().expect("state").runtime_files_dir.clone();
                self.dispatch(AppAction::ListRuntimeFiles { dir })
            }
            AppAction::ListLogs => {
                let logs = self.file_store.list_logs()?;
                Ok(self.mutate_and_broadcast(|st| st.log_files = logs))
            }
            AppAction::ReadLogTail {
                filename,
                max_lines,
            } => {
                let tail = self.file_store.read_log_tail(&filename, max_lines.max(50))?;
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.tools_logs.selected_file = filename;
                    st.log_tail = tail;
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::ClearLog { filename } => {
                self.file_store.clear_log(&filename)?;
                Ok(self.mutate_and_broadcast(|st| st.log_tail.clear()))
            }

            // -----------------------------------------------------------------
            // M36–M42 Updaters, Sub-Store, Diagnostics
            // -----------------------------------------------------------------
            AppAction::CheckCoreUpdate { source_id, refresh } => {
                // ADR-009: When checking `honk` while Honk Native API is live, prefer `/api/v1/x-honk/core/update`.
                let (is_honk_native, base_url, secret, source, local_ver, token) = {
                    let st = self.state.read().expect("state");
                    let src = st
                        .builtin_update_sources
                        .iter()
                        .find(|s| s.id == source_id)
                        .cloned()
                        .ok_or_else(|| format!("unknown update source: {source_id}"))?;
                    let tok = if st.prefs.boxapp_settings.github_token_enabled {
                        Some(st.prefs.boxapp_settings.github_token.clone())
                    } else {
                        None
                    };
                    (
                        source_id == "honk"
                            && st.active_api.kind == ApiKind::HonkNative
                            && st.service_status.is_running(),
                        st.active_api.base_url.clone(),
                        st.active_api.secret.clone(),
                        src,
                        st.local_core_version.clone(),
                        tok,
                    )
                };

                let check = self.runtime.block_on(async {
                    if is_honk_native {
                        if let Ok(client) = HonkNativeClient::new(&base_url, &secret) {
                            if let Ok(v) = client.check_core_update(refresh).await {
                                return Ok(UpdateCheckResult {
                                    source_id: "honk".into(),
                                    target_core: "honk".into(),
                                    local_version: v
                                        .get("current_version")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or(&local_ver)
                                        .to_string(),
                                    remote_version: v
                                        .get("latest_version")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    asset_name: "honk-android-arm64".into(),
                                    download_url: String::new(),
                                    sha256_expected: None,
                                    update_available: v
                                        .get("update_available")
                                        .and_then(|x| x.as_bool())
                                        .unwrap_or(false),
                                });
                            }
                        }
                    }
                    boxproxy_core_mgr::sources::check_github_source(
                        &source,
                        &local_ver,
                        token.as_deref(),
                    )
                    .await
                })?;

                Ok(self.mutate_and_broadcast(|st| {
                    if check.update_available {
                        st.pending_core_update = Some(check.clone());
                    }
                    st.update_check_results.insert(source_id, check);
                }))
            }
            AppAction::InstallCoreUpdate { source_id } => {
                let (check, source, accel_on, accel_prefix, running_same) = {
                    let st = self.state.read().expect("state");
                    let src = st
                        .builtin_update_sources
                        .iter()
                        .find(|s| s.id == source_id)
                        .cloned()
                        .ok_or_else(|| format!("unknown update source: {source_id}"))?;
                    let chk = st
                        .update_check_results
                        .get(&source_id)
                        .cloned()
                        .ok_or_else(|| "run CheckCoreUpdate first".to_string())?;
                    let running_same = st.service_status.is_running()
                        && st.runtime_profile.core_name == src.target_core;
                    (
                        chk,
                        src,
                        st.prefs.boxapp_settings.accelerated_download_enabled,
                        st.prefs.boxapp_settings.accelerated_download_prefix.clone(),
                        running_same,
                    )
                };
                if source_id == "honk" && check.download_url.is_empty() {
                    let (base_url, secret) = {
                        let st = self.state.read().expect("state");
                        (st.active_api.base_url.clone(), st.active_api.secret.clone())
                    };
                    self.runtime.block_on(async {
                        HonkNativeClient::new(&base_url, &secret)?
                            .apply_core_update(false)
                            .await
                            .map(|_| ())
                    })?;
                    return Ok(self.snapshot_bytes().1);
                }

                let dl_url = apply_download_prefix(&check.download_url, accel_on, &accel_prefix);
                let bytes = self.runtime.block_on(async {
                    let resp = reqwest::Client::builder()
                        .timeout(Duration::from_secs(120))
                        .build()
                        .map_err(|e| e.to_string())?
                        .get(&dl_url)
                        .send()
                        .await
                        .map_err(|e| format!("download core failed: {e}"))?;
                    if !resp.status().is_success() {
                        return Err(format!("download core HTTP {}", resp.status()));
                    }
                    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())
                })?;

                self.runtime.block_on(install_core_binary_atomic(
                    self.root.as_ref(),
                    &self.box_home,
                    &source.target_core,
                    &bytes,
                    source.package_format,
                    &source.binary_name,
                    check.sha256_expected.as_deref(),
                    running_same,
                ))?;
                Ok(self.snapshot_bytes().1)
            }
            AppAction::SaveCustomUpdateSources { sources } => {
                let out = self.mutate_and_broadcast(|st| st.prefs.core_update.custom_sources = sources);
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::SetCoreChannels {
                mihomo_stable,
                sing_box_stable,
            } => {
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.core_channels.mihomo_stable = mihomo_stable;
                    st.prefs.core_channels.sing_box_stable = sing_box_stable;
                    st.builtin_update_sources =
                        builtin_core_sources(mihomo_stable, sing_box_stable);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::UpdateLocalWebUi {
                target_path,
                download_url,
            } => {
                let (accel_on, accel_prefix) = {
                    let st = self.state.read().expect("state");
                    (
                        st.prefs.boxapp_settings.accelerated_download_enabled,
                        st.prefs.boxapp_settings.accelerated_download_prefix.clone(),
                    )
                };
                let url = apply_download_prefix(&download_url, accel_on, &accel_prefix);
                let zip_bytes = self.runtime.block_on(async {
                    reqwest::Client::builder()
                        .timeout(Duration::from_secs(90))
                        .build()
                        .map_err(|e| e.to_string())?
                        .get(&url)
                        .send()
                        .await
                        .map_err(|e| e.to_string())?
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| e.to_string())
                })?;
                let dst = self.file_store.resolve_safe(&target_path)?;
                let res = extract_webui_zip(&zip_bytes, &dst)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.webui_update.target_path = target_path;
                    st.prefs.webui_update.download_url = download_url;
                    st.webui_update_status =
                        format!("Extracted {} files to {}", res.files_extracted, res.target_dir);
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::RefreshSubStore => {
                let out = self.runtime.block_on(self.root.exec(
                    "test -d /data/adb/modules/sub_store && echo INSTALLED; cat /data/local/sub_store/scripts/sub_store.env 2>/dev/null || true",
                    Duration::from_secs(5),
                ))?;
                let installed = out.stdout.lines().next().map(|l| l.trim() == "INSTALLED").unwrap_or(false);
                let env = parse_substore_env(&out.stdout, installed);
                Ok(self.mutate_and_broadcast(|st| st.substore_env = env))
            }
            AppAction::RunPlatformProbes => {
                let results = self.runtime.block_on(probe_all_platforms());
                Ok(self.mutate_and_broadcast(|st| st.platform_probes = results))
            }
            AppAction::RunIpQuality => {
                let quality = self.runtime.block_on(probe_ip_quality())?;
                Ok(self.mutate_and_broadcast(|st| st.ip_quality = Some(quality)))
            }
            AppAction::RunSpeedTest { source } => {
                self.mutate_and_broadcast(|st| st.speed_test_running = true);
                let parsed = SpeedTestSource::parse(&source);
                let result = self.runtime.block_on(run_speed_test(parsed));
                match result {
                    Ok(res) => {
                        let out = self.mutate_and_broadcast(|st| {
                            st.speed_test_running = false;
                            st.last_speed_test = Some(res.clone());
                            st.prefs.network_speed_history.insert(
                                0,
                                boxproxy_store::prefs::SpeedHistoryEntry {
                                    id: format!("spd-{}", now_millis()),
                                    timestamp_ms: now_millis(),
                                    source: res.source,
                                    server: res.server,
                                    download_mbps: res.download_mbps,
                                    upload_mbps: res.upload_mbps,
                                    latency_ms: res.latency_ms,
                                    jitter_ms: res.jitter_ms,
                                    exit_ip: res.exit_ip,
                                },
                            );
                            st.prefs.network_speed_history.truncate(50);
                        });
                        self.persist_prefs()?;
                        Ok(out)
                    }
                    Err(e) => {
                        self.mutate_and_broadcast(|st| {
                            st.speed_test_running = false;
                            st.last_error = Some(e.clone());
                        });
                        Err(e)
                    }
                }
            }
            AppAction::ClearSpeedHistory => {
                let out = self.mutate_and_broadcast(|st| st.prefs.network_speed_history.clear());
                self.persist_prefs()?;
                Ok(out)
            }

            // -----------------------------------------------------------------
            // M43–M58 Settings, Onboarding, Tile, Notification
            // -----------------------------------------------------------------
            AppAction::SetBootAutoStart { enabled } => {
                let mut profile = self.db.get_runtime_profile()?;
                let prev = profile.boot_auto_start;
                profile.boot_auto_start = enabled;
                self.db.save_runtime_profile(&profile)?;
                let op = if enabled {
                    self.runtime
                        .block_on(install_boot_script(self.root.as_ref(), &self.box_home))
                } else {
                    self.runtime
                        .block_on(remove_boot_script(self.root.as_ref()))
                };
                if let Err(err) = op {
                    profile.boot_auto_start = prev;
                    let _ = self.db.save_runtime_profile(&profile);
                    return Err(err);
                }
                let out = self.mutate_and_broadcast(|st| {
                    st.runtime_profile.boot_auto_start = enabled;
                    st.prefs.boxapp_settings.auto_start_enabled = enabled;
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::SetLanguage { language } => {
                self.db.set_app_setting("app_language", &language)?;
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.boxapp_settings.app_language = language;
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::UpdateBoxAppSettings { settings } => {
                let notif_enabled = settings.notifications_enabled;
                let out = self.mutate_and_broadcast(|st| st.prefs.boxapp_settings = settings);
                self.persist_prefs()?;
                self.sync_aliases();
                self.platform
                    .on_effect(&AppEffect::SyncNotificationService { enabled: notif_enabled });
                Ok(out)
            }
            AppAction::SaveLatencyTargets { targets } => {
                let out = self.mutate_and_broadcast(|st| st.prefs.latency_targets.targets = targets);
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::SetAppIconVariant { variant } => {
                let normalized = if variant == "alternate" {
                    "alternate".to_string()
                } else {
                    "default".to_string()
                };
                let out = self.mutate_and_broadcast(|st| st.prefs.app_icon.variant = normalized);
                self.persist_prefs()?;
                self.sync_aliases();
                Ok(out)
            }
            AppAction::CompleteOnboarding => {
                let _ = self.sync_embedded_binaries();
                let out = self.mutate_and_broadcast(|st| {
                    st.prefs.boxapp_settings.welcome_onboarding_shown = true;
                    st.current_route = AppRoute::Main;
                    st.active_tab = 0;
                });
                self.persist_prefs()?;
                Ok(out)
            }
            AppAction::TileToggle => {
                let running = self.state.read().expect("state").service_status.is_running();
                if running {
                    self.run_service_transition("down")
                } else {
                    self.run_service_transition("up")
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Direct OS entry-point helpers (Tile, Notification, Monet CSS, CoreImport, Backup)
    // -------------------------------------------------------------------------

    pub fn query_tile_state(&self) -> TileStateSnapshot {
        let st = self.state.read().expect("state");
        let loc = effective_locale(&st, &self.config.initial_locale);
        let active = st.service_status.is_running();
        let working = st.service_status.is_busy();
        let subtitle = if working {
            tr(loc, "status_working")
        } else if active {
            tr(loc, "status_running")
        } else {
            tr(loc, "status_stopped")
        };
        TileStateSnapshot {
            active,
            working,
            label: "Box Honk".into(),
            subtitle: subtitle.to_string(),
        }
    }

    pub fn render_notification(&self, force_refresh: bool) -> NotificationRenderSnapshot {
        if force_refresh {
            let _ = self.dispatch(AppAction::RefreshStatus);
        }
        let st = self.state.read().expect("state");
        let cfg = &st.prefs.boxapp_settings;
        let loc = effective_locale(&st, &self.config.initial_locale);
        let should_show = cfg.notifications_enabled && st.service_status.is_running();
        let vars = NotificationRenderVars {
            status: tr(
                loc,
                if st.service_status.is_running() {
                    "status_running"
                } else {
                    "status_stopped"
                },
            )
            .to_string(),
            pid: st
                .process
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into()),
            core: st.runtime_profile.core_name.clone(),
            mode: st.runtime_profile.mode.clone(),
            config: st.runtime_profile.config_name.clone(),
            uptime: format_uptime(st.process.uptime_secs),
            cpu: format!("{:.1}%", st.process.cpu_percent),
            memory: format!("{:.0} MB", st.process.pss_mb),
            upload: format_rate(st.upload_bps),
            download: format_rate(st.download_bps),
            total_upload: format_bytes(st.total_upload),
            total_download: format_bytes(st.total_download),
        };

        let mut buttons = Vec::new();
        for btn in [
            &cfg.notification_action_1,
            &cfg.notification_action_2,
            &cfg.notification_action_3,
        ] {
            if btn.enabled {
                let label = if !btn.label.trim().is_empty() {
                    btn.label.clone()
                } else {
                    tr(loc, &format!("btn_{}", btn.target)).to_string()
                };
                buttons.push(NotificationButtonSnapshot {
                    target: btn.target.clone(),
                    label,
                });
            }
        }

        NotificationRenderSnapshot {
            should_show,
            title: render_notification_template(&cfg.notification_title_template, &vars),
            content: render_notification_template(&cfg.notification_content_template, &vars),
            click_target: cfg.notification_click_target.clone(),
            icon_variant: st.prefs.app_icon.variant.clone(),
            refresh_interval_seconds: cfg.notification_refresh_interval_seconds.max(1),
            buttons,
        }
    }

    pub fn resolve_monet_css(&self, palette_json: &[u8]) -> String {
        let palette: MonetPaletteInput =
            serde_json::from_slice(palette_json).unwrap_or_default();
        generate_monet_colors_css(&palette)
    }

    pub fn import_core_from_bytes(
        &self,
        target_core: &str,
        bytes: &[u8],
        display_name: &str,
    ) -> EngineResult<usize> {
        let core = CoreKind::parse(target_core)?;
        let format = PackageFormat::detect_from_name(display_name);
        let running_same = {
            let st = self.state.read().expect("state");
            st.service_status.is_running() && st.runtime_profile.core_name == core.as_str()
        };
        let expected_bin = if core == CoreKind::V2fly {
            "v2ray"
        } else {
            core.as_str()
        };
        self.runtime.block_on(install_core_binary_atomic(
            self.root.as_ref(),
            &self.box_home,
            core.as_str(),
            bytes,
            format,
            expected_bin,
            None,
            running_same,
        ))
    }

    pub fn export_backup_to_writer<W: Write + Seek>(
        &self,
        writer: W,
        scope: BackupScope,
    ) -> EngineResult<BackupManifest> {
        self.backup
            .export_to_writer(writer, scope, &self.config.package_name)
    }

    pub fn restore_backup_from_reader<R: Read + Seek>(
        &self,
        reader: R,
        scope: BackupScope,
    ) -> EngineResult<BackupManifest> {
        let manifest = self.backup.restore_from_reader(reader, scope)?;
        let prefs = self.prefs_store.load().unwrap_or_default();
        let profile = self.db.get_runtime_profile().unwrap_or_default();
        self.mutate_and_broadcast(|st| {
            st.prefs = prefs;
            st.runtime_profile = profile;
        });
        Ok(manifest)
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn persist_prefs(&self) -> EngineResult<()> {
        let snapshot = self.state.read().expect("state").prefs.clone();
        self.prefs_store.save(&snapshot)
    }

    fn record_error(&self, err: &str) {
        self.mutate_and_broadcast(|st| st.last_error = Some(err.to_string()));
        self.platform.on_effect(&AppEffect::Toast {
            message: err.to_string(),
            is_error: true,
        });
    }

    fn sync_embedded_binaries(&self) -> EngineResult<()> {
        for (asset, dst_rel) in [
            ("boxctl/arm64-v8a/boxctl", "bin/boxctl"),
            ("boxctl/arm64-v8a/boxbpf", "bin/boxbpf"),
        ] {
            let bytes = self
                .assets
                .read()
                .ok()
                .and_then(|m| m.get(asset).cloned())
                .or_else(|| self.platform.read_asset_bytes(asset));
            if let Some(b) = bytes {
                let dst = self.box_home.join(dst_rel);
                self.runtime
                    .block_on(sync_binary_atomic(self.root.as_ref(), &b, &dst))?;
            }
        }
        Ok(())
    }

    fn run_service_transition(&self, verb: &str) -> EngineResult<Vec<u8>> {
        let _ = self.sync_embedded_binaries();
        let busy_status = match verb {
            "up" => ServiceStatus::Starting,
            "down" => ServiceStatus::Stopping,
            _ => ServiceStatus::Restarting,
        };
        self.mutate_and_broadcast(|st| {
            st.service_status = busy_status;
            st.last_error = None;
        });
        self.platform.on_effect(&AppEffect::RefreshTile);

        let client = BoxctlClient::new(&self.box_home);
        let res = self.runtime.block_on(async {
            match verb {
                "up" => client.up(self.root.as_ref()).await,
                "down" => client.down(self.root.as_ref()).await,
                _ => client.restart(self.root.as_ref()).await,
            }
        });

        match res {
            Ok(_) => {
                if verb == "up" || verb == "restart" {
                    let _ = self.runtime.block_on(client.monitor(self.root.as_ref()));
                }
                let notif_enabled = self
                    .state
                    .read()
                    .expect("state")
                    .prefs
                    .boxapp_settings
                    .notifications_enabled;
                let out = self.mutate_and_broadcast(|st| {
                    st.service_status = if verb == "down" {
                        ServiceStatus::Stopped
                    } else {
                        ServiceStatus::Running
                    };
                });
                self.platform.on_effect(&AppEffect::RefreshTile);
                self.platform.on_effect(&AppEffect::SyncNotificationService {
                    enabled: notif_enabled && verb != "down",
                });
                Ok(out)
            }
            Err(err) => {
                self.mutate_and_broadcast(|st| {
                    st.service_status = ServiceStatus::Error(err.clone());
                    st.last_error = Some(err.clone());
                });
                self.platform.on_effect(&AppEffect::RefreshTile);
                self.platform.on_effect(&AppEffect::Toast {
                    message: err.clone(),
                    is_error: true,
                });
                Err(err)
            }
        }
    }

    fn hot_renew_if_running(&self) {
        let (running, is_ebpf) = {
            let st = self.state.read().expect("state");
            (st.service_status.is_running(), st.runtime_profile.mode == "ebpf")
        };
        if !running {
            return;
        }
        let client = BoxctlClient::new(&self.box_home);
        let _ = self.runtime.block_on(async {
            if is_ebpf {
                client.restart(self.root.as_ref()).await
            } else {
                client.mode_renew(self.root.as_ref()).await
            }
        });
    }
}

fn refresh_api_and_webui_url(st: &mut AppState, box_home: &Path) {
    let core = CoreKind::parse(&st.runtime_profile.core_name).unwrap_or(CoreKind::Honk);
    st.active_api = discover_endpoint(
        core,
        box_home,
        st.prefs.proxy_selector.custom_api_enabled,
        &st.prefs.proxy_selector.custom_api_endpoint,
        &st.prefs.proxy_selector.custom_api_secret,
    );
    st.webui_resolved_url = match st.prefs.web_panel.webui_selected_panel.as_str() {
        "zashboard" => "http://board.zash.run.place".into(),
        "metacubexd" => "http://metacubex.github.io/metacubexd".into(),
        "sing_box" => "http://sing-box-dashboard.sagernet.org".into(),
        "local" => {
            if core == CoreKind::Honk {
                format!("{}/ui/", st.active_api.base_url.trim_end_matches('/'))
            } else {
                format!(
                    "{}/ui/",
                    st.active_api
                        .clash_fallback_url
                        .as_deref()
                        .unwrap_or(&st.active_api.base_url)
                        .trim_end_matches('/')
                )
            }
        }
        custom_id => st
            .prefs
            .web_panel
            .webui_custom_panels
            .iter()
            .find(|p| p.id == custom_id)
            .map(|p| p.url.clone())
            .unwrap_or_else(|| "http://127.0.0.1:9527/ui/".into()),
    };
}

fn panel_route_from_entry_mode(mode: &str) -> AppRoute {
    match mode {
        "proxy_selector" => AppRoute::ProxySelector,
        "connections" => AppRoute::ConnectionsDetails,
        "subscription" => AppRoute::SubscriptionDetails,
        "rules" => AppRoute::Rules,
        "rule_providers" => AppRoute::RuleProviders,
        "web_ui" => AppRoute::WebUi,
        _ => AppRoute::Overview,
    }
}

fn effective_locale<'a>(st: &'a AppState, initial_locale: &'a str) -> &'a str {
    let configured = st.prefs.boxapp_settings.app_language.as_str();
    if configured == "system" || configured.is_empty() {
        initial_locale
    } else {
        configured
    }
}

fn parse_window_seconds(win: &str) -> u64 {
    match win {
        "1h" => 3600,
        "6h" => 21_600,
        "24h" => 86_400,
        "7d" => 604_800,
        "30d" => 2_592_000,
        _ => 1800,
    }
}

fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn format_bytes(b: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    let f = b as f64;
    if f >= GB {
        format!("{:.2} GB", f / GB)
    } else if f >= MB {
        format!("{:.1} MB", f / MB)
    } else if f >= KB {
        format!("{:.1} KB", f / KB)
    } else {
        format!("{b} B")
    }
}

fn format_rate(bps: u64) -> String {
    format!("{}/s", format_bytes(bps))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn json_u64(v: &serde_json::Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
}

fn json_f64(v: &serde_json::Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
}

fn list_core_config_files(box_home: &Path, core_name: &str) -> Vec<String> {
    let core = CoreKind::parse(core_name).unwrap_or(CoreKind::Honk);
    let dir = box_home.join(core.as_str());
    let exts = core.config_extensions();
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for entry in rd.flatten() {
            if entry.path().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                if exts.iter().any(|ext| lower.ends_with(ext)) {
                    out.push(name);
                }
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::all_58_modules;
    use boxproxy_root::MockRootExecutor;
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn engine_bootstraps_all_58_modules_and_dispatches_mvi_flows() {
        assert_eq!(all_58_modules().len(), 58);

        let dir = tempdir().unwrap();
        let mock_root = Arc::new(MockRootExecutor::default());
        let engine = BoxProxyEngine::new(
            EngineConfig {
                files_dir: dir.path().join("files"),
                cache_dir: dir.path().join("cache"),
                package_name: "io.honk.android".into(),
                app_version: "1.0.10-rust".into(),
                sdk_int: 35,
                initial_locale: "zh-CN".into(),
            },
            Arc::new(NoopPlatformBridge),
            Some(mock_root.clone()),
        )
        .unwrap();

        // Onboarding -> Main
        engine.dispatch(AppAction::CompleteOnboarding).unwrap();
        assert_eq!(engine.state.read().unwrap().current_route, AppRoute::Main);

        // Service lifecycle (M01 + M55 Tile)
        engine.dispatch(AppAction::ServiceUp).unwrap();
        assert!(engine.query_tile_state().active);
        assert_eq!(engine.query_tile_state().subtitle, "运行中");

        // Persistent Notification rendering (M57/M58)
        let mut settings = engine.state.read().unwrap().prefs.boxapp_settings.clone();
        settings.notifications_enabled = true;
        engine
            .dispatch(AppAction::UpdateBoxAppSettings { settings })
            .unwrap();
        let notif = engine.render_notification(false);
        assert!(notif.should_show);
        assert!(notif.content.contains("运行中"));
        assert_eq!(notif.buttons.len(), 3);

        // M49 Backup & Restore roundtrip
        let mut zip_buf = Cursor::new(Vec::new());
        let manifest = engine
            .export_backup_to_writer(&mut zip_buf, BackupScope::default())
            .unwrap();
        assert_eq!(manifest.app_package, "io.honk.android");
        zip_buf.set_position(0);
        engine
            .restore_backup_from_reader(zip_buf, BackupScope::default())
            .unwrap();

        // M15 Monet CSS
        let css = engine.resolve_monet_css(b"{}");
        assert!(css.contains("--monet-primary:"));
    }

    #[test]
    fn exercises_routing_tools_scripts_boot_and_alias_mvi_paths() {
        let dir = tempdir().unwrap();
        let mock_root = Arc::new(MockRootExecutor::default());
        let engine = BoxProxyEngine::new(
            EngineConfig {
                files_dir: dir.path().join("files"),
                cache_dir: dir.path().join("cache"),
                package_name: "io.honk.android".into(),
                app_version: "26.11.28.pure-rust".into(),
                sdk_int: 35,
                initial_locale: "zh-CN".into(),
            },
            Arc::new(NoopPlatformBridge),
            Some(mock_root),
        )
        .unwrap();

        // M21/M22 Core + config override
        let mut profile = engine.state.read().unwrap().runtime_profile.clone();
        profile.core_name = "meow".into();
        engine
            .dispatch(AppAction::SaveRuntimeProfile {
                profile,
                apply_live: false,
            })
            .unwrap();
        engine
            .dispatch(AppAction::SetConfigOverride {
                core: "meow".into(),
                option: "override_enabled".into(),
                value: "1".into(),
            })
            .unwrap();
        assert_eq!(
            engine
                .state
                .read()
                .unwrap()
                .config_overrides
                .get("override_enabled")
                .map(String::as_str),
            Some("1")
        );

        // M24/M26/M27/M28/M29 Routing rules & AST
        engine
            .dispatch(AppAction::SaveAppSelection {
                uids: vec![10001, 10002],
                gids: vec!["3003".into()],
                proxy_mode: "blacklist".into(),
            })
            .unwrap();
        engine
            .dispatch(AppAction::ResetDefaultIntranetCidrs)
            .unwrap();
        assert_eq!(engine.state.read().unwrap().intranet_ipv4.len(), 16);
        assert_eq!(engine.state.read().unwrap().intranet_ipv6.len(), 12);

        // M31/M32/M33/M34/M35 Scripts, files, editor, logs
        engine
            .dispatch(AppAction::SaveJsScript {
                core: "mihomo".into(),
                name: "override".into(),
                content: "function main(c) { return c; }".into(),
            })
            .unwrap();
        engine
            .dispatch(AppAction::SaveShellScript {
                name: "pre".into(),
                content: "#!/system/bin/sh\necho hi\n".into(),
            })
            .unwrap();
        engine
            .dispatch(AppAction::SetShellHooks {
                pre_start: "pre.sh".into(),
                post_stop: String::new(),
            })
            .unwrap();
        engine
            .dispatch(AppAction::SaveEditorFile {
                relative_path: "honk/test.dae".into(),
                content: "global { log_level: info }\n".into(),
            })
            .unwrap();
        assert!(engine.state.read().unwrap().editor_diagnostics.is_empty());

        // M43/M44/M50 Boot autostart, language, icon variant
        engine
            .dispatch(AppAction::SetBootAutoStart { enabled: true })
            .unwrap();
        assert!(engine.state.read().unwrap().runtime_profile.boot_auto_start);
        engine
            .dispatch(AppAction::SetLanguage {
                language: "en".into(),
            })
            .unwrap();
        engine
            .dispatch(AppAction::SetAppIconVariant {
                variant: "alternate".into(),
            })
            .unwrap();
        assert_eq!(
            engine.state.read().unwrap().prefs.app_icon.variant,
            "alternate"
        );

        // M15 WebMessage zashboard token validation
        let tok = engine.state.read().unwrap().webui_one_time_token.clone();
        let msg = serde_json::json!({
            "type": "zashboard-detected",
            "pageUrl": "http://board.zash.run.place",
            "token": tok
        });
        engine
            .dispatch(AppAction::HandleWebMessage {
                raw_json: msg.to_string(),
            })
            .unwrap();
        assert!(engine
            .state
            .read()
            .unwrap()
            .prefs
            .zashboard_monet_panels
            .contains("http://board.zash.run.place"));
    }
}
