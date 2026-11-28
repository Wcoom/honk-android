//! Shared SQLite schema v15 (21 tables) and domain models for `boxctl` and `io.honk.android`.
//!
//! Single source of truth for `box.db` layout, backward-compatible migration from v0..v14
//! (including legacy `wifi_match_*` and `hotspot_*` tables), default seed data, and core/network
//! enums + `network_rule_groups` boolean AST evaluation.

pub mod defaults;
pub mod migrate;
pub mod models;

pub use defaults::*;
pub use migrate::{ensure_schema_v15, seed_default_rows, table_columns, table_exists};
pub use models::*;

/// Current canonical `box.db` SQLite `PRAGMA user_version` matching `com.boxproxy.honk` v214.
pub const SCHEMA_VERSION: i64 = 15;

pub type SchemaResult<T> = std::result::Result<T, String>;
