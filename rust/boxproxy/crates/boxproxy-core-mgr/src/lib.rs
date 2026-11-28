//! 7-core manager: ELF64 AArch64 validator, 9 built-in core update sources (M36-M38),
//! local version prober + 4-mode version comparator, WebUI updater (M39), and Sub-Store (M40).

pub mod elf;
pub mod sources;
pub mod substore;
pub mod version;
pub mod webui_updater;

pub use elf::{
    extract_core_binary, validate_elf64_aarch64, PackageFormat, MAX_CORE_PACKAGE_BYTES,
};
pub use sources::{
    apply_download_prefix, builtin_core_sources, install_core_binary_atomic, CoreUpdateSource,
    UpdateCheckResult,
};
pub use substore::{build_substore_url, parse_substore_env, SubStoreEnv};
pub use version::{compare_versions, strip_ansi, VersionCompareMode, VersionOrder};
pub use webui_updater::{extract_webui_zip, WebUiUpdateResult};

pub type MgrResult<T> = std::result::Result<T, String>;
