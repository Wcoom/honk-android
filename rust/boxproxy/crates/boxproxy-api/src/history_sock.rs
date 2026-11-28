//! Unix Domain Socket NDJSON client for `clashapi-history.sock` (Section 4.2.2).

use crate::ApiResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryQuery {
    pub dimension: String,
    pub window: String,
    pub limit: u16,
    pub sort: String,
    pub descending: bool,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            dimension: "outbound".into(),
            window: "30m".into(),
            limit: 10,
            sort: "download".into(),
            descending: true,
        }
    }
}

/// Exponential reconnect backoff matching `rs0.java:76`: `500ms << min(retry, 5)` capped at 10s.
pub fn backoff_delay(retry: u32) -> Duration {
    let shift = retry.min(5);
    let ms = (500u64 << shift).min(10_000);
    Duration::from_millis(ms)
}

pub struct HistorySocketClient {
    socket_path: PathBuf,
}

impl HistorySocketClient {
    pub fn for_box_home(box_home: &Path) -> Self {
        Self {
            socket_path: box_home.join("run/state/runtime/clashapi-history.sock"),
        }
    }

    pub async fn fetch_one(&self, query: &HistoryQuery) -> ApiResult<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("connect {} failed: {e}", self.socket_path.display()))?;
        let mut frame = serde_json::to_string(query).map_err(|e| e.to_string())?;
        frame.push('\n');
        stream
            .write_all(frame.as_bytes())
            .await
            .map_err(|e| format!("write history query failed: {e}"))?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut line))
            .await
            .map_err(|_| "history socket read timeout".to_string())?
            .map_err(|e| format!("read history frame failed: {e}"))?;
        serde_json::from_str(line.trim()).map_err(|e| format!("invalid history NDJSON: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_matches_apk_formula() {
        assert_eq!(backoff_delay(0), Duration::from_millis(500));
        assert_eq!(backoff_delay(1), Duration::from_millis(1000));
        assert_eq!(backoff_delay(4), Duration::from_millis(8000));
        assert_eq!(backoff_delay(5), Duration::from_millis(10_000));
        assert_eq!(backoff_delay(9), Duration::from_millis(10_000));
    }
}
