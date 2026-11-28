//! Unified proxy control-plane API client (ADR-009):
//! - Prefers **Honk Native API** (`/api/v1/*`, `/api/v1/x-honk/core/update`, traffic history, routing trace) when active core is `honk`
//! - Full **Clash API** HTTP client for `mihomo` / `meow` / `sing-box`
//! - Unix Domain Socket client for `clashapi-history.sock`
//! - Local `Subscription-Userinfo` parser (M14)

pub mod clash;
pub mod discovery;
pub mod history_sock;
pub mod honk;
pub mod subscription;

pub use clash::{ClashApiClient, ClashConfig, ClashConnectionsSnapshot, ClashProxyGroup};
pub use discovery::{discover_endpoint, ApiEndpoint, ApiKind};
pub use history_sock::{backoff_delay, HistoryQuery, HistorySocketClient};
pub use honk::{
    HonkCoreUpdateResponse, HonkNativeClient, HonkRouteTraceRequest, HonkTrafficPoint,
};
pub use subscription::{parse_subscription_userinfo, SubscriptionUserInfo};

pub type ApiResult<T> = std::result::Result<T, String>;
