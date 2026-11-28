//! Persistence layer for `io.honk.android`:
//! - Full transactional CRUD over all 21 tables of `box.db` v15 (`db_repo`)
//! - Strongly-typed persistence + Android XML import/export for all 24 `shared_prefs` (`prefs`)
//! - Atomic `boxapp.backup.zip` snapshot backup & rollback-safe restore (`backup`, M49)
//! - Runtime files, JS override scripts, proxy providers YAML, shell scripts, and logs (`files`, M30–M35)

pub mod backup;
pub mod db_repo;
pub mod files;
pub mod prefs;

pub use backup::{BackupManifest, BackupScope, BackupService};
pub use db_repo::BoxDbRepo;
pub use files::{
    FileEntry, JsScriptItem, LogFileInfo, ProxyProviderEntry, RuntimeFileStore, ShellScriptItem,
};
pub use prefs::{AllPrefs, PrefsStore};

pub type StoreResult<T> = std::result::Result<T, String>;
