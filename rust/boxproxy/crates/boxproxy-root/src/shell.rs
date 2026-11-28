//! POSIX single-quote escaping and persistent Root (`su`) Shell pool (max 2 sessions,
//! 120 commands or 5 min idle recycle — faithful to `com.boxproxy.honk` `v16.java`).

use crate::RootResult;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex, Semaphore};

pub const MAX_POOL_SESSIONS: usize = 2;
pub const MAX_COMMANDS_PER_SESSION: usize = 120;
pub const SESSION_IDLE_TTL: Duration = Duration::from_secs(300);

/// POSIX single-quote shell escaper (`'` -> `'\''`).
pub fn posix_quote(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('\'');
    for ch in input.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ShellOutput {
    pub fn ok(&self) -> bool {
        self.exit_code == 0
    }

    pub fn error_text(&self) -> String {
        let err = self.stderr.trim();
        if !err.is_empty() {
            err.to_string()
        } else {
            self.stdout.trim().to_string()
        }
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait RootExecutor: Send + Sync {
    fn exec<'a>(&'a self, script: &'a str, timeout: Duration) -> BoxFuture<'a, RootResult<ShellOutput>>;
}

pub async fn check_root_granted(exec: &dyn RootExecutor) -> bool {
    exec.exec("id -u", Duration::from_secs(5))
        .await
        .map(|o| o.ok() && o.stdout.trim() == "0")
        .unwrap_or(false)
}

struct PersistentSuSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    commands_run: usize,
    last_used: Instant,
}

impl PersistentSuSession {
    async fn spawn() -> RootResult<Self> {
        let mut child = Command::new("su")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("spawn su failed: {e}"))?;
        let stdin = child.stdin.take().ok_or("su stdin missing")?;
        let stdout = child.stdout.take().ok_or("su stdout missing")?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            commands_run: 0,
            last_used: Instant::now(),
        })
    }

    fn is_expired(&mut self) -> bool {
        if self.commands_run >= MAX_COMMANDS_PER_SESSION
            || self.last_used.elapsed() >= SESSION_IDLE_TTL
        {
            return true;
        }
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    async fn run_command(&mut self, script: &str, timeout: Duration) -> RootResult<ShellOutput> {
        let tag = format!(
            "__BOX_END_{}_{}__",
            self.commands_run,
            self.last_used.elapsed().as_nanos()
        );
        let framed = format!(
            "__box_err=$(mktemp /data/local/tmp/.boxerr.XXXXXX 2>/dev/null || echo /tmp/.boxerr.$$)\n\
             ( {script} ) 2>\"$__box_err\"\n\
             __box_rc=$?\n\
             printf '\\n{tag}:RC:%d\\n' \"$__box_rc\"\n\
             cat \"$__box_err\" 2>/dev/null || true\n\
             rm -f \"$__box_err\" 2>/dev/null || true\n\
             printf '\\n{tag}:DONE\\n'\n"
        );
        self.stdin
            .write_all(framed.as_bytes())
            .await
            .map_err(|e| format!("write to su session failed: {e}"))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| format!("flush su session failed: {e}"))?;

        let rc_prefix = format!("{tag}:RC:");
        let done_marker = format!("{tag}:DONE");
        let read_fut = async {
            let mut stdout = String::new();
            let mut stderr = String::new();
            let mut in_stderr = false;
            let mut exit_code = -1i32;
            let mut line = String::new();
            loop {
                line.clear();
                let n = self
                    .stdout
                    .read_line(&mut line)
                    .await
                    .map_err(|e| format!("read from su session failed: {e}"))?;
                if n == 0 {
                    return Err("su session closed unexpectedly".to_string());
                }
                let trimmed = line.trim_end_matches(['\r', '\n']);
                if !in_stderr {
                    if let Some(rc_str) = trimmed.strip_prefix(&rc_prefix) {
                        exit_code = rc_str.trim().parse().unwrap_or(1);
                        in_stderr = true;
                        continue;
                    }
                    stdout.push_str(&line);
                } else {
                    if trimmed == done_marker {
                        break;
                    }
                    stderr.push_str(&line);
                }
            }
            Ok(ShellOutput {
                exit_code,
                stdout: stdout.trim_end_matches('\n').to_string(),
                stderr: stderr.trim_end_matches('\n').to_string(),
            })
        };

        let out = tokio::time::timeout(timeout, read_fut)
            .await
            .map_err(|_| format!("root command timed out after {}s", timeout.as_secs()))??;
        self.commands_run += 1;
        self.last_used = Instant::now();
        Ok(out)
    }
}

pub struct SuShellPool {
    permits: Arc<Semaphore>,
    idle: Arc<Mutex<VecDeque<PersistentSuSession>>>,
}

impl Default for SuShellPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SuShellPool {
    pub fn new() -> Self {
        Self {
            permits: Arc::new(Semaphore::new(MAX_POOL_SESSIONS)),
            idle: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_POOL_SESSIONS))),
        }
    }

    async fn exec_one_shot(script: &str, timeout: Duration) -> RootResult<ShellOutput> {
        let mut cmd = Command::new("su");
        cmd.arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let child = cmd.spawn().map_err(|e| format!("spawn su -c failed: {e}"))?;
        let out = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| format!("root command timed out after {}s", timeout.as_secs()))?
            .map_err(|e| format!("wait su -c failed: {e}"))?;
        Ok(ShellOutput {
            exit_code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        })
    }
}

impl RootExecutor for SuShellPool {
    fn exec<'a>(
        &'a self,
        script: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, RootResult<ShellOutput>> {
        Box::pin(async move {
            let _permit = self
                .permits
                .acquire()
                .await
                .map_err(|_| "root pool closed".to_string())?;
            let maybe_session = {
                let mut guard = self.idle.lock().await;
                let mut picked = None;
                while let Some(mut s) = guard.pop_front() {
                    if !s.is_expired() {
                        picked = Some(s);
                        break;
                    }
                }
                picked
            };

            let mut session = match maybe_session {
                Some(s) => s,
                None => match PersistentSuSession::spawn().await {
                    Ok(s) => s,
                    Err(_) => return Self::exec_one_shot(script, timeout).await,
                },
            };

            match session.run_command(script, timeout).await {
                Ok(out) => {
                    if !session.is_expired() {
                        self.idle.lock().await.push_back(session);
                    }
                    Ok(out)
                }
                Err(_) => Self::exec_one_shot(script, timeout).await,
            }
        })
    }
}

/// Deterministic mock executor for host unit & MVI engine tests.
#[derive(Default)]
pub struct MockRootExecutor {
    pub recorded: Mutex<Vec<String>>,
    pub canned: Mutex<VecDeque<RootResult<ShellOutput>>>,
}

impl MockRootExecutor {
    pub fn with_outputs(outputs: Vec<RootResult<ShellOutput>>) -> Self {
        Self {
            recorded: Mutex::new(Vec::new()),
            canned: Mutex::new(outputs.into()),
        }
    }

    pub async fn commands(&self) -> Vec<String> {
        self.recorded.lock().await.clone()
    }
}

impl RootExecutor for MockRootExecutor {
    fn exec<'a>(
        &'a self,
        script: &'a str,
        _timeout: Duration,
    ) -> BoxFuture<'a, RootResult<ShellOutput>> {
        Box::pin(async move {
            self.recorded.lock().await.push(script.to_string());
            if let Some(next) = self.canned.lock().await.pop_front() {
                next
            } else {
                Ok(ShellOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_quotes_embedded_single_quotes() {
        assert_eq!(posix_quote("/data/user/0/io.honk.android"), "'/data/user/0/io.honk.android'");
        assert_eq!(posix_quote("a'b'c"), "'a'\\''b'\\''c'");
    }

    #[tokio::test]
    async fn root_check_accepts_only_uid_zero() {
        let ok_exec = MockRootExecutor::with_outputs(vec![Ok(ShellOutput {
            exit_code: 0,
            stdout: "0\n".into(),
            stderr: String::new(),
        })]);
        assert!(check_root_granted(&ok_exec).await);

        let deny_exec = MockRootExecutor::with_outputs(vec![Ok(ShellOutput {
            exit_code: 0,
            stdout: "2000\n".into(),
            stderr: String::new(),
        })]);
        assert!(!check_root_granted(&deny_exec).await);
    }
}
