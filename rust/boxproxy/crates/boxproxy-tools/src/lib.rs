//! Pure-Rust diagnostics & utilities covering:
//! - M05 3-target latency prober & M06 WAN GeoIP
//! - M15 WebView Zashboard detector, dynamic Monet CSS generator & Base64 save-file decoder
//! - M25 Smart Select app classifier
//! - M34 JSON5/YAML/DAE/CIDR/Sing-Box schema validator
//! - M41 29-platform connectivity/unlock prober + Net.Coffee IP quality
//! - M42 3-source speed test (Cloudflare / Apple / Ookla)
//! - M57/M58 12-placeholder persistent notification template renderer

pub mod notification;
pub mod platforms;
pub mod speedtest;
pub mod validator;
pub mod webview;

pub use notification::{render_notification_template, NotificationRenderVars};
pub use platforms::{
    builtin_platforms, probe_all_platforms, probe_ip_quality, probe_wan_geo, IpGeoInfo,
    IpQualityResult, PlatformCategory, PlatformProbeResult, PlatformSpec,
};
pub use speedtest::{
    probe_latency_targets, run_speed_test, LatencyProbeResult, SpeedTestResult, SpeedTestSource,
};
pub use validator::{
    classify_smart_select, validate_cidr_v4, validate_cidr_v6, validate_config_text,
    InstalledAppInfo, SmartSelectMode, ValidationDiagnostic,
};
pub use webview::{
    decode_save_file_payload, generate_monet_colors_css, is_monet_css_url,
    zashboard_detection_script, MonetPaletteInput, WebMessageCommand,
};

pub type ToolsResult<T> = std::result::Result<T, String>;
