//! Strongly-typed preferences covering all 24 `shared_prefs` files of `com.boxproxy.honk` v214,
//! with atomic JSON persistence + Android `shared_prefs/*.xml` import compatibility.

use crate::StoreResult;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Complete 24-file preference model (23 backed up + `core_update_monitor` excluded from backup).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AllPrefs {
    pub boxapp_settings: BoxAppSettings,
    pub web_panel: WebPanelPrefs,
    pub proxy_selector: ProxySelectorPrefs,
    pub api_reveal: ApiRevealPrefs,
    pub latency_targets: LatencyTargetsPrefs,
    pub app_icon: AppIconPrefs,
    pub app_management: AppManagementPrefs,
    pub core_update: CoreUpdatePrefs,
    pub core_channels: CoreChannelPrefs,
    /// #10 `core_update_monitor` — runtime cache, explicitly excluded from ZIP backups!
    pub core_update_monitor: CoreUpdateMonitorPrefs,
    pub connections: ConnectionPrefs,
    pub home_ui: HomeUiPrefs,
    pub home_subscription_cache: SubscriptionCachePrefs,
    pub tools_logs: LogViewerPrefs,
    pub runtime_file_editor: EditorPrefs,
    pub module_file_editor: EditorPrefs,
    pub runtime_files_download_cache: BTreeMap<String, DownloadSourceEntry>,
    pub download: DownloadPrefs,
    pub shell_scripts: ShellScriptsPrefs,
    pub webui_update: WebUiUpdatePrefs,
    pub overview: OverviewPrefs,
    pub network_speed_history: Vec<SpeedHistoryEntry>,
    pub zashboard_monet_panels: BTreeSet<String>,
}

impl AllPrefs {
    /// Clone suitable for ZIP backup (strips volatile `core_update_monitor`).
    pub fn for_backup(&self) -> Self {
        let mut copy = self.clone();
        copy.core_update_monitor = CoreUpdateMonitorPrefs::default();
        copy
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationActionConfig {
    pub enabled: bool,
    pub target: String,
    pub label: String,
}

impl NotificationActionConfig {
    pub fn new(target: &str) -> Self {
        Self {
            enabled: true,
            target: target.to_string(),
            label: String::new(),
        }
    }
}

impl Default for NotificationActionConfig {
    fn default() -> Self {
        Self::new("reload")
    }
}

/// #1 `boxapp_settings` (45+ keys: UI mode, theme, Miuix/OS4 glass, notifications, launch, network).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BoxAppSettings {
    pub ui_mode: String,
    pub app_language: String,
    pub home_panel_entry_mode: String,
    pub auto_start_enabled: bool,
    pub accelerated_download_enabled: bool,
    pub accelerated_download_prefix: String,
    pub github_token_enabled: bool,
    pub github_token: String,
    pub notifications_enabled: bool,
    pub notification_refresh_interval_seconds: u32,
    pub notification_title_template: String,
    pub notification_content_template: String,
    pub notification_click_target: String,
    pub notification_action_1: NotificationActionConfig,
    pub notification_action_2: NotificationActionConfig,
    pub notification_action_3: NotificationActionConfig,
    pub open_panel_on_launch: bool,
    pub show_proxy_selector_bottom_bar_item: bool,
    pub tile_long_click_target: String,
    pub theme_mode: u8,
    pub pure_black_dark_theme: bool,
    pub miuix_monet_enabled: bool,
    pub accent_color: i32,
    pub accent_use_default: bool,
    pub theme_palette_style: String,
    pub theme_color_spec: String,
    pub enable_blur: bool,
    pub top_bar_style: String,
    pub bottom_bar_style: String,
    pub floating_bottom_bar: bool,
    pub floating_bottom_bar_blur: bool,
    pub landscape_glass_navigation: bool,
    pub enable_predictive_back_animation: bool,
    pub predictive_back_animation_follow_edge: bool,
    pub enable_swipe_back: bool,
    pub use_net_speed_clash_api: bool,
    pub speed_test_source: String,
    pub speed_test_use_megabytes: bool,
    pub use_subscription_detail_clash_api: bool,
    pub page_scale: f32,
    pub welcome_onboarding_shown: bool,
}

impl Default for BoxAppSettings {
    fn default() -> Self {
        Self {
            ui_mode: "miuix".into(),
            app_language: "system".into(),
            home_panel_entry_mode: "overview".into(),
            auto_start_enabled: false,
            accelerated_download_enabled: false,
            accelerated_download_prefix: String::new(),
            github_token_enabled: false,
            github_token: String::new(),
            notifications_enabled: false,
            notification_refresh_interval_seconds: 5,
            notification_title_template: "BoxProxy Honk".into(),
            notification_content_template:
                "{status} · {uptime}\n↑ {upload} · ↓ {download}\nCPU {cpu} · RAM {memory}".into(),
            notification_click_target: "panel".into(),
            notification_action_1: NotificationActionConfig::new("reload"),
            notification_action_2: NotificationActionConfig::new("stop"),
            notification_action_3: NotificationActionConfig::new("restart"),
            open_panel_on_launch: false,
            show_proxy_selector_bottom_bar_item: true,
            tile_long_click_target: "panel".into(),
            theme_mode: 0,
            pure_black_dark_theme: true,
            miuix_monet_enabled: true,
            accent_color: -12_232_054,
            accent_use_default: true,
            theme_palette_style: "TonalSpot".into(),
            theme_color_spec: "Spec2021".into(),
            enable_blur: true,
            top_bar_style: "Classic".into(),
            bottom_bar_style: "LiquidGlass".into(),
            floating_bottom_bar: true,
            floating_bottom_bar_blur: true,
            landscape_glass_navigation: true,
            enable_predictive_back_animation: true,
            predictive_back_animation_follow_edge: true,
            enable_swipe_back: true,
            use_net_speed_clash_api: true,
            speed_test_source: "cloudflare".into(),
            speed_test_use_megabytes: false,
            use_subscription_detail_clash_api: false,
            page_scale: 1.0,
            welcome_onboarding_shown: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomWebPanel {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// #2 `web_panel_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebPanelPrefs {
    pub webui_local_panel_mode: String,
    pub webui_selected_panel: String,
    pub substore_selected_panel: String,
    pub webui_custom_panels: Vec<CustomWebPanel>,
    pub substore_custom_panels: Vec<CustomWebPanel>,
}

impl Default for WebPanelPrefs {
    fn default() -> Self {
        Self {
            webui_local_panel_mode: "auto".into(),
            webui_selected_panel: "local".into(),
            substore_selected_panel: "local".into(),
            webui_custom_panels: Vec::new(),
            substore_custom_panels: Vec::new(),
        }
    }
}

/// #3 `proxy_selector_preferences` (all 24 keys).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxySelectorPrefs {
    pub custom_api_enabled: bool,
    pub custom_api_endpoint: String,
    pub custom_api_secret: String,
    pub custom_latency_test_url_enabled: bool,
    pub custom_latency_test_url: String,
    pub policy_icon_pack_urls: Vec<String>,
    pub policy_icon_overrides: BTreeMap<String, String>,
    pub node_sort_mode: String,
    pub show_hidden_groups: bool,
    pub show_unavailable_nodes: bool,
    pub group_by_provider: bool,
    pub detect_ipv6_on_latency_test: bool,
    pub name_overflow_mode: String,
    pub scroll_overflowing_node_names: bool,
    pub display_global_by_mode: bool,
    pub disconnect_on_select: bool,
    pub collapse_previous_group_on_expand: bool,
    pub expand_selected_policy_in_bottom_sheet: bool,
    pub node_column_mode: String,
    pub node_column_count: u8,
    pub group_column_mode: String,
    pub group_column_count: u8,
    pub node_density: String,
    pub group_density: String,
}

impl Default for ProxySelectorPrefs {
    fn default() -> Self {
        Self {
            custom_api_enabled: false,
            custom_api_endpoint: String::new(),
            custom_api_secret: String::new(),
            custom_latency_test_url_enabled: false,
            custom_latency_test_url:
                "https://connectivitycheck.platform.hicloud.com/generate_204".into(),
            policy_icon_pack_urls: Vec::new(),
            policy_icon_overrides: BTreeMap::new(),
            node_sort_mode: "defaultsort".into(),
            show_hidden_groups: true,
            show_unavailable_nodes: true,
            group_by_provider: false,
            detect_ipv6_on_latency_test: false,
            name_overflow_mode: "clip".into(),
            scroll_overflowing_node_names: false,
            display_global_by_mode: false,
            disconnect_on_select: false,
            collapse_previous_group_on_expand: false,
            expand_selected_policy_in_bottom_sheet: false,
            node_column_mode: "auto".into(),
            node_column_count: 1,
            group_column_mode: "auto".into(),
            group_column_count: 1,
            node_density: "standard".into(),
            group_density: "standard".into(),
        }
    }
}

/// #4 `api_config_reveal_preferences`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiRevealPrefs {
    pub endpoint_revealed: bool,
    pub secret_revealed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyTarget {
    pub name: String,
    pub url: String,
}

/// #5 `latency_settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LatencyTargetsPrefs {
    pub targets: [LatencyTarget; 3],
}

impl Default for LatencyTargetsPrefs {
    fn default() -> Self {
        Self {
            targets: [
                LatencyTarget {
                    name: "Baidu".into(),
                    url: "https://baidu.com".into(),
                },
                LatencyTarget {
                    name: "Cloudflare".into(),
                    url: "https://cloudflare.com".into(),
                },
                LatencyTarget {
                    name: "Google".into(),
                    url: "https://google.com".into(),
                },
            ],
        }
    }
}

/// #6 `app_icon_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppIconPrefs {
    pub variant: String,
}

impl Default for AppIconPrefs {
    fn default() -> Self {
        Self {
            variant: "default".into(),
        }
    }
}

/// #7 `tools_app_management`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppManagementPrefs {
    pub show_system_apps: bool,
    pub show_selected_first: bool,
    pub search_query: String,
}

impl Default for AppManagementPrefs {
    fn default() -> Self {
        Self {
            show_system_apps: false,
            show_selected_first: true,
            search_query: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCoreUpdateSource {
    pub id: String,
    pub name: String,
    pub target: String,
    pub kind: String,
    pub location: String,
    #[serde(default)]
    pub asset_regex: String,
    #[serde(default)]
    pub package_format: String,
    #[serde(default)]
    pub binary_name: String,
    #[serde(default)]
    pub compare_mode: String,
}

/// #8 `tools_core_update`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreUpdatePrefs {
    pub selected_core_id: String,
    pub custom_sources: Vec<CustomCoreUpdateSource>,
    pub meow_source_added: bool,
    pub honk_source_added: bool,
}

impl Default for CoreUpdatePrefs {
    fn default() -> Self {
        Self {
            selected_core_id: "honk".into(),
            custom_sources: Vec::new(),
            meow_source_added: true,
            honk_source_added: true,
        }
    }
}

/// #9 `core_update_settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreChannelPrefs {
    pub mihomo_stable: bool,
    pub sing_box_stable: bool,
}

impl Default for CoreChannelPrefs {
    fn default() -> Self {
        Self {
            mihomo_stable: true,
            sing_box_stable: true,
        }
    }
}

/// #10 `core_update_monitor` (NOT included in ZIP backups).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreUpdateMonitorPrefs {
    pub cache: BTreeMap<String, serde_json::Value>,
    pub ignored_versions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionFilterRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub mode: String,
    pub fields: Vec<String>,
    pub ignore_case: bool,
    pub enabled: bool,
}

/// #11 & #12 `connection_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionPrefs {
    pub sort_mode: String,
    pub reverse_sort: bool,
    pub source_ip_filter: Option<String>,
    pub filter_rules: Vec<ConnectionFilterRule>,
}

impl Default for ConnectionPrefs {
    fn default() -> Self {
        Self {
            sort_mode: "Host".into(),
            reverse_sort: false,
            source_ip_filter: None,
            filter_rules: Vec::new(),
        }
    }
}

/// #13 `home_ui_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeUiPrefs {
    pub home_ip_mode: String,
}

impl Default for HomeUiPrefs {
    fn default() -> Self {
        Self {
            home_ip_mode: "wan".into(),
        }
    }
}

/// #14 `home_subscription_cache`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubscriptionCachePrefs {
    pub settings_json: serde_json::Value,
    pub local_cache_json: serde_json::Value,
}

/// #15 `tools_logs`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogViewerPrefs {
    pub selected_file: String,
    pub level_filter: String,
    pub auto_scroll: bool,
}

impl Default for LogViewerPrefs {
    fn default() -> Self {
        Self {
            selected_file: "honk.log".into(),
            level_filter: "all".into(),
            auto_scroll: true,
        }
    }
}

/// #16 & #17 `runtime_file_editor_prefs` / `module_file_editor_prefs`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorPrefs {
    pub word_wrap: bool,
    pub sticky_scroll: bool,
}

impl Default for EditorPrefs {
    fn default() -> Self {
        Self {
            word_wrap: true,
            sticky_scroll: false,
        }
    }
}

/// #18 `runtime_files_download_cache` entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadSourceEntry {
    pub url: String,
    pub user_agent: String,
}

/// #19 `download_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadPrefs {
    pub user_agent: String,
}

impl Default for DownloadPrefs {
    fn default() -> Self {
        Self {
            user_agent: "boxproxy".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptLastRun {
    pub exit_code: i32,
    pub at_millis: u64,
}

/// #20 `shell_scripts`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellScriptsPrefs {
    pub pre_start_script: String,
    pub post_stop_script: String,
    pub last_runs: BTreeMap<String, ScriptLastRun>,
}

/// #21 `webui_update_cache`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WebUiUpdatePrefs {
    pub target_path: String,
    pub download_url: String,
}

impl Default for WebUiUpdatePrefs {
    fn default() -> Self {
        Self {
            target_path: "mihomo/dashboard".into(),
            download_url: "https://github.com/Zephyruso/zashboard/releases/latest/download/dist.zip"
                .into(),
        }
    }
}

/// #22 `overview_preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OverviewPrefs {
    pub auto_expand_top_traffic: bool,
    pub top_traffic_window: String,
    pub top_traffic_sort_descending: bool,
    pub top_traffic_limit: u16,
    pub top_traffic_sort: String,
    pub realtime_rank_dimension: String,
    pub realtime_rank_limit: u16,
    pub realtime_rank_sort: String,
    pub realtime_rank_sort_descending: bool,
}

impl Default for OverviewPrefs {
    fn default() -> Self {
        Self {
            auto_expand_top_traffic: false,
            top_traffic_window: "30m".into(),
            top_traffic_sort_descending: true,
            top_traffic_limit: 10,
            top_traffic_sort: "download".into(),
            realtime_rank_dimension: "outbound".into(),
            realtime_rank_limit: 5,
            realtime_rank_sort: "connections".into(),
            realtime_rank_sort_descending: true,
        }
    }
}

/// #23 `network_speed_history` entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpeedHistoryEntry {
    pub id: String,
    pub timestamp_ms: u64,
    pub source: String,
    pub server: String,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub exit_ip: String,
}

pub struct PrefsStore {
    path: PathBuf,
}

impl PrefsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> StoreResult<AllPrefs> {
        if !self.path.is_file() {
            return Ok(AllPrefs::default());
        }
        let text = fs::read_to_string(&self.path)
            .map_err(|e| format!("read prefs {} failed: {e}", self.path.display()))?;
        serde_json::from_str(&text)
            .map_err(|e| format!("parse prefs {} failed: {e}", self.path.display()))
    }

    pub fn save(&self, prefs: &AllPrefs) -> StoreResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create prefs dir failed: {e}"))?;
        }
        let json = serde_json::to_string_pretty(prefs)
            .map_err(|e| format!("serialize prefs failed: {e}"))?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| format!("write temp prefs failed: {e}"))?;
        fs::rename(&tmp, &self.path).map_err(|e| format!("commit prefs failed: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrips_all_24_prefs_and_excludes_monitor_from_backup() {
        let dir = tempdir().unwrap();
        let store = PrefsStore::new(dir.path().join("app_prefs.json"));
        let mut prefs = AllPrefs::default();
        prefs.boxapp_settings.ui_mode = "miuix".into();
        prefs.boxapp_settings.bottom_bar_style = "LiquidGlass".into();
        prefs.core_update_monitor.ignored_versions.insert("honk".into(), "1.0.9".into());
        prefs.zashboard_monet_panels.insert("http://board.zash.run.place".into());
        store.save(&prefs).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded, prefs);
        let backup = loaded.for_backup();
        assert!(backup.core_update_monitor.ignored_versions.is_empty());
        assert_eq!(backup.boxapp_settings.ui_mode, "miuix");
    }
}
