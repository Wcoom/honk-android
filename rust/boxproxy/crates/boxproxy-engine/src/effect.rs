//! Outbound platform side effects emitted from Rust to the ultra-thin Android DEX shell.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum AppEffect {
    Toast { message: String, is_error: bool },
    OpenPanelSheet,
    OpenWebUi { url: String },
    RefreshTile,
    SyncNotificationService { enabled: bool },
    SetComponentAlias { alias_name: String, enabled: bool },
    SaveFile { file_name: String, mime_type: String, base64: String },
    RequestPermission { permission: String },
    OpenExternalUrl { url: String },
    ShareText { title: String, content: String },
}
