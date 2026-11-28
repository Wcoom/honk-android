//! Complete MVI state model covering all 32 routes and 58 modules (M01–M58).

use boxproxy_api::{ApiEndpoint, ClashConnectionsSnapshot, SubscriptionUserInfo};
use boxproxy_core_mgr::{CoreUpdateSource, SubStoreEnv, UpdateCheckResult};
use boxproxy_root::{ArpClient, HookRunResult, ProcessMetrics};
use boxproxy_schema::{
    CnipSettings, NetworkMatchCondition, NetworkMatchSettings, NetworkObservation,
    NetworkRuleGroup, RuntimeProfile, SharedNetworkSettings,
};
use boxproxy_store::{
    AllPrefs, FileEntry, JsScriptItem, LogFileInfo, ProxyProviderEntry, ShellScriptItem,
};
use boxproxy_tools::{
    InstalledAppInfo, IpGeoInfo, IpQualityResult, LatencyProbeResult, PlatformProbeResult,
    SpeedTestResult, ValidationDiagnostic,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// All 32 navigation routes (`BoxAppRoute` / `kb0` parity + PanelSheet/Onboarding).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "route", rename_all = "snake_case")]
pub enum AppRoute {
    #[default]
    Main,
    Overview,
    ProxySelector,
    ConnectionsDetails,
    Rules,
    RuleProviders,
    SubscriptionDetails,
    SubscriptionLocalDetails,
    WebUi,
    CustomPolicyIcons,
    ProxyConfig,
    OtherProxyConfig,
    AppManagement,
    CnipSettings,
    BypassRules,
    SharedNetworkConfig,
    NetworkMatch,
    SubscriptionManagement,
    JsScripts { core: String },
    ShellScripts,
    RuntimeFiles,
    RuntimeFileEditor { relative_path: String },
    LogViewer,
    CoreUpdateSources,
    SubStore,
    NetworkTest,
    NetworkSpeedHistory,
    ThemeSettings,
    LatencyTargets,
    NotificationSettings,
    About,
    AboutLibraries,
    AboutLibraryDetail { library_id: String },
    AboutSponsorship,
    Onboarding,
    PanelSheet,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "message", rename_all = "snake_case")]
pub enum ServiceStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Stopping,
    Restarting,
    Error(String),
}

impl ServiceStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Starting | Self::Stopping | Self::Restarting)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeedPoint {
    pub timestamp_ms: u64,
    pub upload_bps: u64,
    pub download_bps: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    pub id: &'static str,
    pub domain: &'static str,
    pub title_zh: &'static str,
    pub title_en: &'static str,
    pub route_name: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileStateSnapshot {
    pub active: bool,
    pub working: bool,
    pub label: String,
    pub subtitle: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationButtonSnapshot {
    pub target: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationRenderSnapshot {
    pub should_show: bool,
    pub title: String,
    pub content: String,
    pub click_target: String,
    pub icon_variant: String,
    pub refresh_interval_seconds: u32,
    pub buttons: Vec<NotificationButtonSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub version: u64,
    pub app_package: String,
    pub app_version: String,
    pub active_tab: u8,
    pub current_route: AppRoute,
    pub back_stack: Vec<AppRoute>,
    pub root_granted: bool,

    // M01–M08 Home & Service
    pub service_status: ServiceStatus,
    pub process: ProcessMetrics,
    pub local_core_version: String,
    pub active_api: ApiEndpoint,
    pub outbound_mode: String,
    pub outbound_mode_list: Vec<String>,
    pub upload_bps: u64,
    pub download_bps: u64,
    pub total_upload: u64,
    pub total_download: u64,
    pub active_connections_count: usize,
    pub groups_count: usize,
    pub rules_count: usize,
    pub net_speed_history: Vec<SpeedPoint>,
    pub latency_results: Vec<LatencyProbeResult>,
    pub ip_geo: IpGeoInfo,
    pub lan_ips: Vec<String>,
    pub local_subscriptions: BTreeMap<String, SubscriptionUserInfo>,
    pub pending_core_update: Option<UpdateCheckResult>,

    // M09–M16 Panel & WebUI
    pub overview_traffic_history: Value,
    pub overview_datapath: Value,
    pub overview_memory: Value,
    pub connections_snapshot: ClashConnectionsSnapshot,
    pub connections_paused: bool,
    pub rules_data: Value,
    pub route_trace_result: Value,
    pub rule_providers_data: Value,
    pub proxy_providers_data: Value,
    pub webui_resolved_url: String,
    pub webui_one_time_token: String,

    // M17–M20 Proxy Selector
    pub proxies_data: BTreeMap<String, Value>,
    pub honk_groups_data: Value,
    pub ipv6_capable_nodes: BTreeSet<String>,

    // M21–M29 Core, App, CNIP, Bypass, Shared Network, Network Match
    pub runtime_profile: RuntimeProfile,
    pub core_config_files: Vec<String>,
    pub config_overrides: BTreeMap<String, String>,
    pub selected_uids: Vec<u32>,
    pub gid_list: Vec<String>,
    pub installed_apps: Vec<InstalledAppInfo>,
    pub cnip_settings: CnipSettings,
    pub cnip_force_apps: Vec<String>,
    pub cnip_force_uids: Vec<u32>,
    pub intranet_ipv4: Vec<String>,
    pub intranet_ipv6: Vec<String>,
    pub blocked_interfaces: Vec<String>,
    pub shared_network_settings: SharedNetworkSettings,
    pub shared_network_interfaces: Vec<String>,
    pub shared_network_macs: Vec<String>,
    pub arp_clients: Vec<ArpClient>,
    pub network_match_settings: NetworkMatchSettings,
    pub network_match_ssids: Vec<String>,
    pub network_match_bssids: Vec<String>,
    pub network_match_conditions: Vec<NetworkMatchCondition>,
    pub network_rule_groups: Vec<NetworkRuleGroup>,
    pub last_network_observation: NetworkObservation,

    // M30–M35 Subscriptions YAML, JS/Shell Scripts, Runtime Files, Editor, Logs
    pub mihomo_proxy_providers: Vec<ProxyProviderEntry>,
    pub js_scripts: Vec<JsScriptItem>,
    pub shell_scripts: Vec<ShellScriptItem>,
    pub last_hook_result: Option<HookRunResult>,
    pub runtime_files_dir: String,
    pub runtime_files: Vec<FileEntry>,
    pub editor_path: String,
    pub editor_content: String,
    pub editor_diagnostics: Vec<ValidationDiagnostic>,
    pub log_files: Vec<LogFileInfo>,
    pub log_tail: String,

    // M36–M42 Updaters, Sub-Store, Diagnostics
    pub builtin_update_sources: Vec<CoreUpdateSource>,
    pub update_check_results: BTreeMap<String, UpdateCheckResult>,
    pub webui_update_status: String,
    pub substore_env: SubStoreEnv,
    pub platform_probes: Vec<PlatformProbeResult>,
    pub ip_quality: Option<IpQualityResult>,
    pub speed_test_running: bool,
    pub last_speed_test: Option<SpeedTestResult>,

    // M43–M58 Settings, About, Onboarding, Tile, Notification
    pub prefs: AllPrefs,
    pub last_error: Option<String>,
    pub module_count: usize,
}

impl AppState {
    pub fn initial(app_package: String, app_version: String) -> Self {
        Self {
            version: 1,
            app_package,
            app_version,
            active_tab: 0,
            current_route: AppRoute::Main,
            back_stack: Vec::new(),
            root_granted: false,
            service_status: ServiceStatus::Stopped,
            process: ProcessMetrics::default(),
            local_core_version: String::new(),
            active_api: ApiEndpoint {
                kind: boxproxy_api::ApiKind::HonkNative,
                base_url: "http://127.0.0.1:9527".into(),
                secret: String::new(),
                clash_fallback_url: Some("http://127.0.0.1:9090".into()),
            },
            outbound_mode: "rule".into(),
            outbound_mode_list: vec!["rule".into(), "direct".into(), "global".into()],
            upload_bps: 0,
            download_bps: 0,
            total_upload: 0,
            total_download: 0,
            active_connections_count: 0,
            groups_count: 0,
            rules_count: 0,
            net_speed_history: Vec::with_capacity(60),
            latency_results: Vec::new(),
            ip_geo: IpGeoInfo::default(),
            lan_ips: Vec::new(),
            local_subscriptions: BTreeMap::new(),
            pending_core_update: None,
            overview_traffic_history: Value::Null,
            overview_datapath: Value::Null,
            overview_memory: Value::Null,
            connections_snapshot: ClashConnectionsSnapshot::default(),
            connections_paused: false,
            rules_data: Value::Null,
            route_trace_result: Value::Null,
            rule_providers_data: Value::Null,
            proxy_providers_data: Value::Null,
            webui_resolved_url: "http://127.0.0.1:9527/ui/".into(),
            webui_one_time_token: String::new(),
            proxies_data: BTreeMap::new(),
            honk_groups_data: Value::Null,
            ipv6_capable_nodes: BTreeSet::new(),
            runtime_profile: RuntimeProfile::default(),
            core_config_files: Vec::new(),
            config_overrides: BTreeMap::new(),
            selected_uids: Vec::new(),
            gid_list: Vec::new(),
            installed_apps: Vec::new(),
            cnip_settings: CnipSettings::default(),
            cnip_force_apps: Vec::new(),
            cnip_force_uids: Vec::new(),
            intranet_ipv4: Vec::new(),
            intranet_ipv6: Vec::new(),
            blocked_interfaces: Vec::new(),
            shared_network_settings: SharedNetworkSettings::default(),
            shared_network_interfaces: Vec::new(),
            shared_network_macs: Vec::new(),
            arp_clients: Vec::new(),
            network_match_settings: NetworkMatchSettings::default(),
            network_match_ssids: Vec::new(),
            network_match_bssids: Vec::new(),
            network_match_conditions: Vec::new(),
            network_rule_groups: Vec::new(),
            last_network_observation: NetworkObservation::default(),
            mihomo_proxy_providers: Vec::new(),
            js_scripts: Vec::new(),
            shell_scripts: Vec::new(),
            last_hook_result: None,
            runtime_files_dir: String::new(),
            runtime_files: Vec::new(),
            editor_path: String::new(),
            editor_content: String::new(),
            editor_diagnostics: Vec::new(),
            log_files: Vec::new(),
            log_tail: String::new(),
            builtin_update_sources: boxproxy_core_mgr::builtin_core_sources(true, true),
            update_check_results: BTreeMap::new(),
            webui_update_status: String::new(),
            substore_env: SubStoreEnv::default(),
            platform_probes: Vec::new(),
            ip_quality: None,
            speed_test_running: false,
            last_speed_test: None,
            prefs: AllPrefs::default(),
            last_error: None,
            module_count: all_58_modules().len(),
        }
    }
}

/// Canonical registry of all 58 functional modules (M01–M58) matching `boxproxy-honk-spec.md`.
pub fn all_58_modules() -> &'static [ModuleDescriptor] {
    &[
        // Domain A: Home (M01–M08)
        ModuleDescriptor { id: "M01", domain: "Home", title_zh: "服务状态主控卡片", title_en: "Service Control", route_name: "main" },
        ModuleDescriptor { id: "M02", domain: "Home", title_zh: "运行系统详情弹窗", title_en: "System Details", route_name: "main" },
        ModuleDescriptor { id: "M03", domain: "Home", title_zh: "实时网速与流量卡片", title_en: "Net Speed Card", route_name: "main" },
        ModuleDescriptor { id: "M04", domain: "Home", title_zh: "出站模式快速切换器", title_en: "Outbound Mode Switch", route_name: "main" },
        ModuleDescriptor { id: "M05", domain: "Home", title_zh: "三目标延迟测速卡片", title_en: "Latency Card", route_name: "main" },
        ModuleDescriptor { id: "M06", domain: "Home", title_zh: "LAN/WAN IP 归属地卡片", title_en: "IP Geo Card", route_name: "main" },
        ModuleDescriptor { id: "M07", domain: "Home", title_zh: "订阅流量汇总卡片", title_en: "Subscription Summary", route_name: "main" },
        ModuleDescriptor { id: "M08", domain: "Home", title_zh: "快捷面板入口与更新横幅", title_en: "Panel & Update Banner", route_name: "main" },
        // Domain B: Panel (M09–M16)
        ModuleDescriptor { id: "M09", domain: "Panel", title_zh: "概览页", title_en: "Overview", route_name: "overview" },
        ModuleDescriptor { id: "M10", domain: "Panel", title_zh: "实时连接管理页", title_en: "Connections", route_name: "connections_details" },
        ModuleDescriptor { id: "M11", domain: "Panel", title_zh: "分流规则与路由追踪页", title_en: "Rules & Trace", route_name: "rules" },
        ModuleDescriptor { id: "M12", domain: "Panel", title_zh: "规则集管理页", title_en: "Rule Providers", route_name: "rule_providers" },
        ModuleDescriptor { id: "M13", domain: "Panel", title_zh: "订阅详情（API）", title_en: "Subscriptions API", route_name: "subscription_details" },
        ModuleDescriptor { id: "M14", domain: "Panel", title_zh: "订阅详情（本地）", title_en: "Subscriptions Local", route_name: "subscription_local_details" },
        ModuleDescriptor { id: "M15", domain: "Panel", title_zh: "内置 WebUI 浏览器", title_en: "WebUI Browser", route_name: "web_ui" },
        ModuleDescriptor { id: "M16", domain: "Panel", title_zh: "API 配置与联动抽屉", title_en: "API Config", route_name: "overview" },
        // Domain C: Proxy Selector (M17–M20)
        ModuleDescriptor { id: "M17", domain: "Proxy", title_zh: "策略组与节点选择器", title_en: "Proxy Selector", route_name: "proxy_selector" },
        ModuleDescriptor { id: "M18", domain: "Proxy", title_zh: "延迟测速与 IPv6 探测", title_en: "Delay & IPv6 Probe", route_name: "proxy_selector" },
        ModuleDescriptor { id: "M19", domain: "Proxy", title_zh: "策略页布局与排序定制", title_en: "Selector Layout", route_name: "proxy_selector" },
        ModuleDescriptor { id: "M20", domain: "Proxy", title_zh: "自定义策略组图标", title_en: "Custom Policy Icons", route_name: "custom_policy_icons" },
        // Domain D: Tools (M21–M42)
        ModuleDescriptor { id: "M21", domain: "Tools", title_zh: "核心切换与主代理配置", title_en: "Proxy Config", route_name: "proxy_config" },
        ModuleDescriptor { id: "M22", domain: "Tools", title_zh: "核心配置自动覆写", title_en: "Config Override", route_name: "proxy_config" },
        ModuleDescriptor { id: "M23", domain: "Tools", title_zh: "高级代理与资源控制", title_en: "Other Proxy Config", route_name: "other_proxy_config" },
        ModuleDescriptor { id: "M24", domain: "Tools", title_zh: "应用分流管理", title_en: "App Management", route_name: "app_management" },
        ModuleDescriptor { id: "M25", domain: "Tools", title_zh: "智能分流选应用", title_en: "Smart Select", route_name: "app_management" },
        ModuleDescriptor { id: "M26", domain: "Tools", title_zh: "CNIP 绕过与强制代理", title_en: "CNIP Settings", route_name: "cnip_settings" },
        ModuleDescriptor { id: "M27", domain: "Tools", title_zh: "绕过规则与内网 CIDR", title_en: "Bypass Rules", route_name: "bypass_rules" },
        ModuleDescriptor { id: "M28", domain: "Tools", title_zh: "共享网络与 MAC 过滤", title_en: "Shared Network", route_name: "shared_network_config" },
        ModuleDescriptor { id: "M29", domain: "Tools", title_zh: "网络匹配与布尔规则组", title_en: "Network Match", route_name: "network_match" },
        ModuleDescriptor { id: "M30", domain: "Tools", title_zh: "订阅配置覆写管理", title_en: "Subscription Mgmt", route_name: "subscription_management" },
        ModuleDescriptor { id: "M31", domain: "Tools", title_zh: "JS 覆写脚本管理", title_en: "JS Scripts", route_name: "js_scripts" },
        ModuleDescriptor { id: "M32", domain: "Tools", title_zh: "Shell 钩子脚本管理", title_en: "Shell Scripts", route_name: "shell_scripts" },
        ModuleDescriptor { id: "M33", domain: "Tools", title_zh: "运行时文件管理器", title_en: "Runtime Files", route_name: "runtime_files" },
        ModuleDescriptor { id: "M34", domain: "Tools", title_zh: "代码编辑器与 Schema 校验", title_en: "Code Editor", route_name: "runtime_file_editor" },
        ModuleDescriptor { id: "M35", domain: "Tools", title_zh: "运行日志查看器", title_en: "Log Viewer", route_name: "log_viewer" },
        ModuleDescriptor { id: "M36", domain: "Tools", title_zh: "核心在线更新器", title_en: "Core Updater", route_name: "core_update_sources" },
        ModuleDescriptor { id: "M37", domain: "Tools", title_zh: "核心自定义更新源", title_en: "Core Update Sources", route_name: "core_update_sources" },
        ModuleDescriptor { id: "M38", domain: "Tools", title_zh: "本地文件导入安装核心", title_en: "Import Core", route_name: "proxy_config" },
        ModuleDescriptor { id: "M39", domain: "Tools", title_zh: "本地 WebUI 更新器", title_en: "WebUI Updater", route_name: "web_ui" },
        ModuleDescriptor { id: "M40", domain: "Tools", title_zh: "Sub-Store 本地集成", title_en: "Sub-Store", route_name: "sub_store" },
        ModuleDescriptor { id: "M41", domain: "Tools", title_zh: "29 平台连通性与 IP 质量", title_en: "Network Diagnostics", route_name: "network_test" },
        ModuleDescriptor { id: "M42", domain: "Tools", title_zh: "网络测速与历史", title_en: "Speed Test", route_name: "network_speed_history" },
        // Domain E: Settings (M43–M52)
        ModuleDescriptor { id: "M43", domain: "Settings", title_zh: "开机自启设置", title_en: "Boot Auto-Start", route_name: "main" },
        ModuleDescriptor { id: "M44", domain: "Settings", title_zh: "应用语言切换", title_en: "Language", route_name: "main" },
        ModuleDescriptor { id: "M45", domain: "Settings", title_zh: "加速下载镜像与 Token", title_en: "Download Mirror & Token", route_name: "main" },
        ModuleDescriptor { id: "M46", domain: "Settings", title_zh: "启动与导航入口设置", title_en: "Launch & Navigation", route_name: "main" },
        ModuleDescriptor { id: "M47", domain: "Settings", title_zh: "主题与 OS4 玻璃导航", title_en: "Theme & Glass UI", route_name: "theme_settings" },
        ModuleDescriptor { id: "M48", domain: "Settings", title_zh: "三目标延迟设置页", title_en: "Latency Targets", route_name: "latency_targets" },
        ModuleDescriptor { id: "M49", domain: "Settings", title_zh: "ZIP 原子备份与恢复", title_en: "Backup & Restore", route_name: "main" },
        ModuleDescriptor { id: "M50", domain: "Settings", title_zh: "应用图标切换与关于页", title_en: "About & App Icon", route_name: "about" },
        ModuleDescriptor { id: "M51", domain: "Settings", title_zh: "开源引用库列表与详情", title_en: "Open Source Libraries", route_name: "about_libraries" },
        ModuleDescriptor { id: "M52", domain: "Settings", title_zh: "赞助支持页", title_en: "Sponsorship", route_name: "about_sponsorship" },
        // Domain F: Onboarding (M53–M54)
        ModuleDescriptor { id: "M53", domain: "Onboarding", title_zh: "首次欢迎与 Root 检查", title_en: "Welcome & Root Check", route_name: "onboarding" },
        ModuleDescriptor { id: "M54", domain: "Onboarding", title_zh: "初始核心与配置向导", title_en: "Initial Core Setup", route_name: "onboarding" },
        // Domain G: Tile & PanelSheet (M55–M56)
        ModuleDescriptor { id: "M55", domain: "System", title_zh: "系统快捷设置磁贴", title_en: "Quick Settings Tile", route_name: "main" },
        ModuleDescriptor { id: "M56", domain: "System", title_zh: "半透明快捷悬浮面板", title_en: "Panel Sheet", route_name: "panel_sheet" },
        // Domain H: Persistent Notification (M57–M58)
        ModuleDescriptor { id: "M57", domain: "System", title_zh: "前台常驻状态通知服务", title_en: "Persistent Notification", route_name: "notification_settings" },
        ModuleDescriptor { id: "M58", domain: "System", title_zh: "常驻通知定制设置页", title_en: "Notification Settings", route_name: "notification_settings" },
    ]
}
