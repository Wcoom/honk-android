//! Core local version probing & 4-mode comparator (`qi1.a` / `li1` parity).

use boxproxy_root::{posix_quote, RootExecutor};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VersionCompareMode {
    #[default]
    Version,
    Number,
    Build,
    Display,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionOrder {
    Older,
    Equal,
    Newer,
    Unknown,
}

pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for nc in chars.by_ref() {
                if nc.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn extract_version_line(raw: &str) -> String {
    let clean = strip_ansi(raw);
    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("version:") {
            return trimmed[8..].trim().to_string();
        }
    }
    clean
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string()
}

pub async fn probe_local_version(exec: &dyn RootExecutor, core_name: &str, core_path: &Path) -> Option<String> {
    let flags: &[&str] = match core_name {
        "xray" | "v2fly" => &["version", "-version", "--version", "-v"],
        "mihomo" | "meow" => &["-v", "--version", "version"],
        _ => &["--version", "-v", "version", "-version"],
    };
    let quoted = posix_quote(&core_path.to_string_lossy());
    for flag in flags {
        let cmd = format!("{quoted} {flag}");
        if let Ok(out) = exec.exec(&cmd, Duration::from_secs(5)).await {
            let combined = if !out.stdout.trim().is_empty() {
                &out.stdout
            } else {
                &out.stderr
            };
            let ver = extract_version_line(combined);
            if !ver.is_empty() {
                return Some(ver);
            }
        }
    }
    None
}

/// Compare `local` against `remote` under one of the 4 BoxProxy modes (`VERSION`/`NUMBER`/`BUILD`/`DISPLAY`).
pub fn compare_versions(local: &str, remote: &str, mode: VersionCompareMode) -> VersionOrder {
    let l = local.trim();
    let r = remote.trim();
    if l.is_empty() || r.is_empty() {
        return VersionOrder::Unknown;
    }
    match mode {
        VersionCompareMode::Display => VersionOrder::Unknown,
        VersionCompareMode::Build => {
            if l.eq_ignore_ascii_case(r) || l.contains(r) || r.contains(l) {
                VersionOrder::Equal
            } else {
                VersionOrder::Older
            }
        }
        VersionCompareMode::Number => {
            let ln = extract_first_digits(l);
            let rn = extract_first_digits(r);
            match (ln, rn) {
                (Some(a), Some(b)) => match a.cmp(&b) {
                    Ordering::Less => VersionOrder::Older,
                    Ordering::Equal => VersionOrder::Equal,
                    Ordering::Greater => VersionOrder::Newer,
                },
                _ => VersionOrder::Unknown,
            }
        }
        VersionCompareMode::Version => compare_semver_strings(l, r),
    }
}

fn extract_first_digits(s: &str) -> Option<u128> {
    let digits: String = s.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn parse_semver(s: &str) -> Option<(Vec<u64>, Option<String>)> {
    let token = s
        .split_whitespace()
        .find(|t| t.chars().any(|c| c.is_ascii_digit()))?
        .trim_start_matches(['v', 'V']);
    let (core, pre) = match token.split_once('-') {
        Some((c, p)) => (c, Some(p.to_string())),
        None => (token, None),
    };
    let nums: Vec<u64> = core
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .ok()
        })
        .collect::<Option<Vec<_>>>()?;
    if nums.is_empty() {
        None
    } else {
        Some((nums, pre))
    }
}

fn compare_semver_strings(local: &str, remote: &str) -> VersionOrder {
    let Some((mut ln, lpre)) = parse_semver(local) else {
        return VersionOrder::Unknown;
    };
    let Some((mut rn, rpre)) = parse_semver(remote) else {
        return VersionOrder::Unknown;
    };
    let max_len = ln.len().max(rn.len());
    ln.resize(max_len, 0);
    rn.resize(max_len, 0);
    match ln.cmp(&rn) {
        Ordering::Less => VersionOrder::Older,
        Ordering::Greater => VersionOrder::Newer,
        Ordering::Equal => match (lpre, rpre) {
            (None, None) => VersionOrder::Equal,
            (Some(_), None) => VersionOrder::Older,
            (None, Some(_)) => VersionOrder::Newer,
            (Some(a), Some(b)) => match a.cmp(&b) {
                Ordering::Less => VersionOrder::Older,
                Ordering::Equal => VersionOrder::Equal,
                Ordering::Greater => VersionOrder::Newer,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_prerelease_number_and_build() {
        assert_eq!(
            compare_versions("v1.0.9", "v1.0.10", VersionCompareMode::Version),
            VersionOrder::Older
        );
        assert_eq!(
            compare_versions("1.10.0-alpha.1", "1.10.0", VersionCompareMode::Version),
            VersionOrder::Older
        );
        assert_eq!(
            compare_versions("v1.0.10", "1.0.10", VersionCompareMode::Version),
            VersionOrder::Equal
        );
        assert_eq!(
            compare_versions("build 210", "214", VersionCompareMode::Number),
            VersionOrder::Older
        );
        assert_eq!(
            compare_versions("honk aa98568", "aa98568", VersionCompareMode::Build),
            VersionOrder::Equal
        );
        assert_eq!(strip_ansi("\x1b[32mhonk 1.0.10\x1b[0m"), "honk 1.0.10");
    }
}
