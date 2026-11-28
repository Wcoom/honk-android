//! Complete MVI `AppAction` command vocabulary spanning M01–M58.

use crate::state::AppRoute;
use boxproxy_api::HonkRouteTraceRequest;
use boxproxy_schema::{
    CnipSettings, NetworkMatchCondition, NetworkMatchSettings, NetworkObservation,
    NetworkRuleGroup, RuntimeProfile, SharedNetworkSettings,
};
use boxproxy_store::prefs::{
    BoxAppSettings, ConnectionPrefs, CustomCoreUpdateSource, CustomWebPanel, LatencyTarget,
    OverviewPrefs, ProxySelectorPrefs,
};
use boxproxy_store::ProxyProviderEntry;
use boxproxy_tools::SmartSelectMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AppAction {
    // Navigation & OS launch
    Navigate { route: AppRoute },
    PopBack,
    SwitchTab { tab: u8 },
    HandleLaunchTarget { target: String },

    // M01–M08 Home & Service
    RefreshStatus,
    ServiceUp,
    ServiceDown,
    ServiceRestart,
    ServiceReload,
    SwitchOutboundMode { mode: String },
    ProbeLatencyTargets,
    RefreshIpGeo,
    ToggleHomeIpMode,
    RefreshSubscriptions,
    IgnoreCoreUpdateVersion { core: String, version: String },

    // M09–M16 Panel & WebUI
    RefreshOverview,
    UpdateOverviewPrefs { prefs: OverviewPrefs },
    ClearClashApiHistory,
    RefreshConnections,
    SetConnectionsPaused { paused: bool },
    CloseConnection { id: String },
    CloseAllConnections,
    UpdateConnectionPrefs { prefs: ConnectionPrefs },
    RefreshRules,
    TraceRoute { request: HonkRouteTraceRequest },
    RefreshRuleProviders,
    UpdateRuleProvider { name: String },
    UpdateAllRuleProviders,
    UpdateProxyProvider { name: String },
    HealthcheckProxyProvider { name: String },
    SelectWebPanel { panel_id: String },
    SetWebUiLocalMode { mode: String },
    AddCustomWebPanel { panel: CustomWebPanel, is_substore: bool },
    DeleteCustomWebPanel { panel_id: String, is_substore: bool },
    HandleWebMessage { raw_json: String },
    UpdateCustomApi { enabled: bool, endpoint: String, secret: String },

    // M17–M20 Proxy Selector
    RefreshProxies,
    SelectProxy { group: String, node: String },
    TestProxyDelay { node: String },
    TestGroupDelay { group: String },
    UpdateProxySelectorPrefs { prefs: ProxySelectorPrefs },

    // M21–M29 Core, App, CNIP, Bypass, Shared Network, Network Match
    SaveRuntimeProfile { profile: RuntimeProfile, apply_live: bool },
    SetConfigOverride { core: String, option: String, value: String },
    ConfigSync { manual: bool },
    ApplyResources,
    RefreshInstalledApps,
    SaveAppSelection { uids: Vec<u32>, gids: Vec<String>, proxy_mode: String },
    RunSmartSelect { mode: SmartSelectMode },
    SaveCnipSettings { settings: CnipSettings },
    SaveCnipForceApps { apps: Vec<String>, resolved_uids: Vec<u32> },
    SaveBypassRules { bypass_intranet: bool, ipv4: Vec<String>, ipv6: Vec<String>, blocked_interfaces: Vec<String> },
    ResetDefaultIntranetCidrs,
    SaveSharedNetwork { settings: SharedNetworkSettings, interfaces: Vec<String>, macs: Vec<String> },
    ScanArpClients,
    SaveNetworkMatchSettings {
        settings: NetworkMatchSettings,
        ssids: Vec<String>,
        bssids: Vec<String>,
        conditions: Vec<NetworkMatchCondition>,
    },
    UpsertNetworkRuleGroup { group: NetworkRuleGroup },
    DeleteNetworkRuleGroup { group_id: i64 },
    ApplyNetworkMatch,
    EvaluateNetworkObservation { observation: NetworkObservation },

    // M30–M35 Subscriptions, Scripts, Files, Editor, Logs
    SaveProxyProviders { entries: Vec<ProxyProviderEntry> },
    ListJsScripts { core: String },
    SaveJsScript { core: String, name: String, content: String },
    SetEnabledJsScripts { core: String, ordered: Vec<String> },
    ListShellScripts,
    SaveShellScript { name: String, content: String },
    SetShellHooks { pre_start: String, post_stop: String },
    RunShellScript { name: String, event: String },
    ListRuntimeFiles { dir: String },
    OpenEditorFile { relative_path: String },
    SaveEditorFile { relative_path: String, content: String },
    ValidateEditorText { filename: String, content: String },
    RenameRuntimeFile { from: String, to: String },
    DeleteRuntimeFile { relative_path: String },
    ListLogs,
    ReadLogTail { filename: String, max_lines: usize },
    ClearLog { filename: String },

    // M36–M42 Updaters, Sub-Store, Diagnostics
    CheckCoreUpdate { source_id: String, refresh: bool },
    InstallCoreUpdate { source_id: String },
    SaveCustomUpdateSources { sources: Vec<CustomCoreUpdateSource> },
    SetCoreChannels { mihomo_stable: bool, sing_box_stable: bool },
    UpdateLocalWebUi { target_path: String, download_url: String },
    RefreshSubStore,
    RunPlatformProbes,
    RunIpQuality,
    RunSpeedTest { source: String },
    ClearSpeedHistory,

    // M43–M58 Settings, Onboarding, Tile, Notification
    SetBootAutoStart { enabled: bool },
    SetLanguage { language: String },
    UpdateBoxAppSettings { settings: BoxAppSettings },
    SaveLatencyTargets { targets: [LatencyTarget; 3] },
    SetAppIconVariant { variant: String },
    CompleteOnboarding,
    TileToggle,
}
