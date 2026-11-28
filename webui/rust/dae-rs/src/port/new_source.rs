//! Port of `src/dae/newSource.ts`.

#![forbid(unsafe_code)]

use crate::port::js_str::js_trim;
use crate::port::text::{block_body_raw, scan_config_raw, uncomment_str, unquote_str};
use serde_json::{Value, json};

pub fn include_patterns(text: String) -> Result<Vec<String>, String> {
    let scan = scan_config_raw(&text);
    let mut out = Vec::new();
    for block in scan.blocks.into_iter().filter(|b| b.name == "include") {
        for line in block_body_raw(&text, &block) {
            let clean = unquote_str(js_trim(&uncomment_str(&line)));
            if !clean.is_empty() {
                out.push(clean);
            }
        }
    }
    Ok(out)
}

pub fn is_glob_bool(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

pub fn is_glob(pattern: String) -> Result<Value, String> {
    Ok(Value::Bool(is_glob_bool(&pattern)))
}

/// `*` matches `[^/]*`, `?` matches `[^/]` (one UTF-16 code unit in JS without `u`, or char),
/// every other character is literal.
pub fn glob_match_bool(pattern: &str, path: &str) -> bool {
    let pat = pattern.strip_prefix("./").unwrap_or(pattern);
    // Match on UTF-16 units so `?` matches a single UTF-16 code unit, matching JS `new RegExp('^' + body + '$')` (no `u` flag).
    let p: Vec<u16> = pat.encode_utf16().collect();
    let s: Vec<u16> = path.encode_utf16().collect();
    let n = s.len();
    let mut dp = vec![false; n + 1];
    dp[0] = true;
    for &pc in &p {
        let mut next = vec![false; n + 1];
        if pc == b'*' as u16 {
            next[0] = dp[0];
            for j in 0..n {
                next[j + 1] = dp[j + 1] || (next[j] && s[j] != b'/' as u16);
            }
        } else if pc == b'?' as u16 {
            for j in 0..n {
                next[j + 1] = dp[j] && s[j] != b'/' as u16;
            }
        } else {
            for j in 0..n {
                next[j + 1] = dp[j] && s[j] == pc;
            }
        }
        dp = next;
    }
    dp[n]
}

pub fn glob_match(pattern: String, path: String) -> Result<bool, String> {
    Ok(glob_match_bool(&pattern, &path))
}

pub fn included_by(patterns: Vec<String>, path: String) -> Result<Value, String> {
    Ok(Value::Bool(
        patterns.iter().any(|p| glob_match_bool(p, &path)),
    ))
}

pub fn resolve_include_path_str(base: Option<&str>, path: &str) -> String {
    let joined = if path.starts_with('/') {
        path.to_string()
    } else {
        let b = base.unwrap_or("");
        let prefix = match b.rfind('/') {
            Some(pos) => &b[..=pos],
            None => "",
        };
        format!("{prefix}{path}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        if part == ".." {
            parts.pop();
        } else if !part.is_empty() && part != "." {
            parts.push(part);
        }
    }
    let trailing_slash = !parts.is_empty()
        && (joined.ends_with('/') || joined.ends_with("/.") || joined.ends_with("/.."));
    let mut out = String::from("/");
    out.push_str(&parts.join("/"));
    if trailing_slash {
        out.push('/');
    }
    out
}

pub fn resolve_include_path(base: Option<String>, path: String) -> Result<Value, String> {
    Ok(Value::String(resolve_include_path_str(
        base.as_deref(),
        &path,
    )))
}

pub fn include_directory(patterns: Vec<String>) -> Result<String, String> {
    for raw in patterns {
        let pattern = raw.strip_prefix("./").unwrap_or(&raw);
        if let Some(slash) = pattern.rfind('/')
            && slash > 0
            && is_glob_bool(&pattern[slash..])
            && !is_glob_bool(&pattern[..slash])
        {
            return Ok(pattern[..=slash].to_string());
        }
    }
    Ok(String::new())
}

pub fn new_source_path_problem_str(path: &str) -> Option<&'static str> {
    if path.chars().any(|c| {
        let mut buf = [0u16; 2];
        let cu = c.encode_utf16(&mut buf)[0];
        cu < 0x20 || (0x7f..=0x9f).contains(&cu)
    }) {
        return Some("config.newSourceControl");
    }
    if path.starts_with('/') {
        return Some("config.newSourceAbsolute");
    }
    if path
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return Some("config.newSourceSegments");
    }
    let Some(stem) = path.strip_suffix(".dae") else {
        return Some("config.newSourceExtension");
    };
    if stem.chars().next_back().is_none_or(|c| c == '/') {
        return Some("config.newSourceExtension");
    }
    None
}

pub fn new_source_path_problem(path: String) -> Result<Value, String> {
    Ok(match new_source_path_problem_str(&path) {
        Some(s) => Value::String(s.to_string()),
        None => Value::Null,
    })
}

pub fn name_patterns(patterns: Vec<String>) -> Result<Vec<Value>, String> {
    let mut found: Vec<(String, String, String)> = Vec::new();
    for raw in patterns {
        let pattern = raw.strip_prefix("./").unwrap_or(&raw);
        let Some(star) = pattern.find('*') else {
            continue;
        };
        let prefix = &pattern[..star];
        let suffix = &pattern[star + 1..];
        if is_glob_bool(prefix)
            || is_glob_bool(suffix)
            || !prefix.contains('/')
            || suffix.contains('/')
            || !suffix.ends_with(".dae")
        {
            continue;
        }
        let sample = format!("{prefix}name{suffix}");
        if new_source_path_problem_str(&sample).is_some()
            || found.iter().any(|(p, _, _)| p == pattern)
        {
            continue;
        }
        found.push((pattern.to_string(), prefix.to_string(), suffix.to_string()));
    }
    Ok(found
        .into_iter()
        .map(|(pattern, prefix, suffix)| {
            json!({
                "pattern": pattern,
                "prefix": prefix,
                "suffix": suffix,
            })
        })
        .collect())
}

pub fn new_source_name_problem(name: String, pattern: Value) -> Result<Value, String> {
    if name.contains('/') {
        return Ok(Value::String("config.newSourceNameSlash".to_string()));
    }
    let po = crate::port::js_str::ensure_not_nullish(&pattern, "prefix")?;
    let prefix = po
        .and_then(|o| o.get("prefix"))
        .map(crate::port::js_str::js_to_string)
        .unwrap_or_else(|| "undefined".to_string());
    let suffix = po
        .and_then(|o| o.get("suffix"))
        .map(crate::port::js_str::js_to_string)
        .unwrap_or_else(|| "undefined".to_string());
    let combined = format!("{prefix}{name}{suffix}");
    let problem = new_source_path_problem_str(&combined).map(|p| {
        if p == "config.newSourceControl" {
            "config.newSourceNameControl"
        } else {
            p
        }
    });
    Ok(match problem {
        Some(s) => Value::String(s.to_string()),
        None => Value::Null,
    })
}

pub fn source_at(sources: Vec<Value>, path: String) -> Result<Option<Value>, String> {
    let main = sources
        .iter()
        .find(|s| s.get("kind").and_then(Value::as_str) == Some("main"));
    let directory = match main.and_then(|m| m.get("path").and_then(Value::as_str)) {
        Some(mp) => match mp.rfind('/') {
            Some(pos) => &mp[..=pos],
            None => "",
        },
        None => "",
    };
    let joined = format!("{directory}{path}");
    Ok(sources.into_iter().find(|s| {
        s.get("path")
            .and_then(Value::as_str)
            .is_some_and(|p| p == path || p == joined)
    }))
}
