//! Pure-Rust MVI engine covering all 58 modules (M01–M58) and 32 routes of `io.honk.android`.

pub mod action;
pub mod effect;
pub mod engine;
pub mod i18n;
pub mod state;

pub use action::AppAction;
pub use effect::AppEffect;
pub use engine::{
    BoxProxyEngine, EngineConfig, NoopPlatformBridge, PlatformBridge, StateListener,
};
pub use state::{
    all_58_modules, AppRoute, AppState, ModuleDescriptor, NotificationRenderSnapshot,
    ServiceStatus, TileStateSnapshot,
};

pub type EngineResult<T> = std::result::Result<T, String>;
