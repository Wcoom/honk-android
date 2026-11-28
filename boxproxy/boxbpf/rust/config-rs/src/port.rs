//! Pure Rust port of `config.c` (`boxproxy/boxbpf/config.c`).

use std::ffi::{OsStr, c_int};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub const PIN_DIR: &str = "/sys/fs/bpf/box";
pub const PIN_CIDR_OUT4: &str = "/sys/fs/bpf/box/box_cidr_out4";
pub const PIN_CIDR_OUT6: &str = "/sys/fs/bpf/box/box_cidr_out6";
pub const PIN_CIDR_PRE4: &str = "/sys/fs/bpf/box/box_cidr_pre4";
pub const PIN_CIDR_PRE6: &str = "/sys/fs/bpf/box/box_cidr_pre6";
pub const PIN_FORCE_OUT4: &str = "/sys/fs/bpf/box/box_force_out4";
pub const PIN_FORCE_OUT6: &str = "/sys/fs/bpf/box/box_force_out6";
pub const PIN_APP_OUT4: &str = "/sys/fs/bpf/box/box_uid_out4";
pub const PIN_APP_OUT6: &str = "/sys/fs/bpf/box/box_uid_out6";

pub const MAP_RUNTIME: &str = "/sys/fs/bpf/box/box_runtime_cfg";
pub const MAP_CIDR4: &str = "/sys/fs/bpf/box/box_cidr4_lpm";
pub const MAP_CIDR6: &str = "/sys/fs/bpf/box/box_cidr6_lpm";
pub const MAP_FORCE_UID: &str = "/sys/fs/bpf/box/box_force_uid_set";
pub const MAP_APP_UID: &str = "/sys/fs/bpf/box/box_app_uid_set";

pub const MAX_PATH_LEN: usize = 512;

const BPF_MAP_UPDATE_ELEM: c_int = 2;
const BPF_ANY: u64 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lpm4Key {
    pub prefixlen: u32,
    pub addr: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lpm6Key {
    pub prefixlen: u32,
    pub addr: [u8; 16],
}

#[repr(C)]
struct BpfMapElemAttr {
    map_fd: u32,
    _pad: u32,
    key: u64,
    value: u64,
    flags: u64,
    _rest: [u8; 168 - 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub ipv6: bool,
    pub cidr4_file: String,
    pub cidr6_file: String,
    pub force_uid_file: String,
    pub app_uid_file: String,
    pub pin_cidr_out4: String,
    pub pin_cidr_out6: String,
    pub pin_cidr_pre4: String,
    pub pin_cidr_pre6: String,
    pub pin_force_out4: String,
    pub pin_force_out6: String,
    pub pin_app_out4: String,
    pub pin_app_out6: String,
    pub map_cidr4: String,
    pub map_cidr6: String,
    pub map_force_uid: String,
    pub map_app_uid: String,
}

impl Default for Config {
    fn default() -> Self {
        config_defaults()
    }
}

pub fn config_defaults() -> Config {
    Config {
        ipv6: true,
        cidr4_file: String::new(),
        cidr6_file: String::new(),
        force_uid_file: String::new(),
        app_uid_file: String::new(),
        pin_cidr_out4: PIN_CIDR_OUT4.to_string(),
        pin_cidr_out6: PIN_CIDR_OUT6.to_string(),
        pin_cidr_pre4: PIN_CIDR_PRE4.to_string(),
        pin_cidr_pre6: PIN_CIDR_PRE6.to_string(),
        pin_force_out4: PIN_FORCE_OUT4.to_string(),
        pin_force_out6: PIN_FORCE_OUT6.to_string(),
        pin_app_out4: PIN_APP_OUT4.to_string(),
        pin_app_out6: PIN_APP_OUT6.to_string(),
        map_cidr4: MAP_CIDR4.to_string(),
        map_cidr6: MAP_CIDR6.to_string(),
        map_force_uid: MAP_FORCE_UID.to_string(),
        map_app_uid: MAP_APP_UID.to_string(),
    }
}

fn find_json_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let quoted = format!("\"{key}\"");
    let idx = json.find(&quoted)?;
    let rest = json[idx + quoted.len()..].trim_start_matches(|c: char| c.is_ascii_whitespace());
    let rest = rest.strip_prefix(':')?;
    Some(rest.trim_start_matches(|c: char| c.is_ascii_whitespace()))
}

fn read_json_string(json: &str, key: &str) -> Option<String> {
    let rest = find_json_key(json, key)?;
    let bytes = rest.as_bytes();
    if bytes.first().copied() != Some(b'"') {
        return None;
    }
    let mut i = 1usize;
    let mut out = Vec::new();
    while i < bytes.len() && bytes[i] != b'"' && out.len() + 1 < MAX_PATH_LEN {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1;
        }
        out.push(bytes[i]);
        i += 1;
    }
    if bytes.get(i).copied() != Some(b'"') {
        return None;
    }
    Some(String::from_utf8_lossy(&out).into_owned())
}

fn read_json_bool(json: &str, key: &str) -> Option<bool> {
    let rest = find_json_key(json, key)?;
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else if rest.starts_with('1') {
        Some(true)
    } else if rest.starts_with('0') {
        Some(false)
    } else {
        None
    }
}

pub fn read_config_json(path: &Path) -> Option<Config> {
    let mut config = config_defaults();
    let raw = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            eprintln!("read config failed: {}", path.display());
            return None;
        }
    };
    let json = String::from_utf8_lossy(&raw);

    if let Some(v) = read_json_bool(&json, "ipv6") {
        config.ipv6 = v;
    }
    if let Some(v) = read_json_string(&json, "cidr4") {
        config.cidr4_file = v;
    }
    if let Some(v) = read_json_string(&json, "cidr6") {
        config.cidr6_file = v;
    }
    if let Some(v) = read_json_string(&json, "forceUids") {
        config.force_uid_file = v;
    }
    if let Some(v) = read_json_string(&json, "appUids") {
        config.app_uid_file = v;
    }
    if let Some(v) = read_json_string(&json, "pinCidrOut4") {
        config.pin_cidr_out4 = v;
    }
    if let Some(v) = read_json_string(&json, "pinCidrOut6") {
        config.pin_cidr_out6 = v;
    }
    if let Some(v) = read_json_string(&json, "pinCidrPre4") {
        config.pin_cidr_pre4 = v;
    }
    if let Some(v) = read_json_string(&json, "pinCidrPre6") {
        config.pin_cidr_pre6 = v;
    }
    if let Some(v) = read_json_string(&json, "pinForceOut4") {
        config.pin_force_out4 = v;
    }
    if let Some(v) = read_json_string(&json, "pinForceOut6") {
        config.pin_force_out6 = v;
    }
    if let Some(v) = read_json_string(&json, "pinAppOut4") {
        config.pin_app_out4 = v;
    }
    if let Some(v) = read_json_string(&json, "pinAppOut6") {
        config.pin_app_out6 = v;
    }
    if let Some(v) = read_json_string(&json, "mapCidr4") {
        config.map_cidr4 = v;
    }
    if let Some(v) = read_json_string(&json, "mapCidr6") {
        config.map_cidr6 = v;
    }
    if let Some(v) = read_json_string(&json, "mapForceUid") {
        config.map_force_uid = v;
    }
    if let Some(v) = read_json_string(&json, "mapAppUid") {
        config.map_app_uid = v;
    }

    if config.cidr4_file.is_empty() {
        eprintln!("missing config field: cidr4");
        return None;
    }
    if config.ipv6 && config.cidr6_file.is_empty() {
        eprintln!("missing config field: cidr6");
        return None;
    }
    Some(config)
}

fn trim_line(line: &str) -> &str {
    let without_comment = line.split('#').next().unwrap_or("");
    without_comment.trim_matches([' ', '\t', '\r', '\n'])
}

fn parse_c_ulong(s: &str) -> Option<u64> {
    let trimmed = s.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let (neg, digits_start) = if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else {
        (false, trimmed)
    };
    let mut acc: u64 = 0;
    let mut count = 0usize;
    let mut overflow = false;
    for b in digits_start.bytes() {
        if !b.is_ascii_digit() {
            break;
        }
        count += 1;
        let digit = u64::from(b - b'0');
        match acc.checked_mul(10).and_then(|v| v.checked_add(digit)) {
            Some(v) => acc = v,
            None => {
                overflow = true;
                acc = u64::MAX;
            }
        }
    }
    if count == 0 {
        return None;
    }
    if overflow {
        return Some(u64::MAX);
    }
    Some(if neg { acc.wrapping_neg() } else { acc })
}

#[derive(Debug, PartialEq, Eq)]
enum ParsedCidr {
    V4(Lpm4Key),
    V6(Lpm6Key),
}

fn parse_cidr(line: &str) -> Option<ParsedCidr> {
    let value = trim_line(line);
    if value.is_empty() {
        return None;
    }
    let (ip_str, prefix_str) = value.split_once('/')?;
    let prefix = parse_c_ulong(prefix_str)?;
    if ip_str.contains(':') {
        if prefix > 128 {
            return None;
        }
        let addr: Ipv6Addr = ip_str.parse().ok()?;
        Some(ParsedCidr::V6(Lpm6Key {
            prefixlen: prefix as u32,
            addr: addr.octets(),
        }))
    } else {
        if prefix > 32 {
            return None;
        }
        let addr: Ipv4Addr = ip_str.parse().ok()?;
        Some(ParsedCidr::V4(Lpm4Key {
            prefixlen: prefix as u32,
            addr: addr.octets(),
        }))
    }
}

fn parse_uid(line: &str) -> Option<u32> {
    let value = trim_line(line);
    if value.is_empty() {
        return None;
    }
    let parsed = parse_c_ulong(value)?;
    u32::try_from(parsed).ok()
}

fn bpf_update_elem<K>(map_fd: c_int, key: &K, value: &u8) -> c_int {
    let attr = BpfMapElemAttr {
        map_fd: map_fd as u32,
        _pad: 0,
        key: std::ptr::from_ref(key) as usize as u64,
        value: std::ptr::from_ref(value) as usize as u64,
        flags: BPF_ANY,
        _rest: [0u8; 168 - 32],
    };
    // SAFETY: `attr` is a valid, zero-padded `bpf_attr` buffer of size 168 bytes;
    // `key` and `value` remain live for the duration of the synchronous `bpf(2)` syscall.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_bpf,
            BPF_MAP_UPDATE_ELEM,
            std::ptr::from_ref(&attr),
            std::mem::size_of::<BpfMapElemAttr>(),
        )
    };
    rc as c_int
}

/// Port of `int load_cidr_file(int map_fd, const char *path, bool ipv6)` (`config.c:184`).
pub fn load_cidr_file(map_fd: c_int, path: Option<&[u8]>, ipv6: bool) -> c_int {
    let Some(path_bytes) = path else {
        return 0;
    };
    let path_str = OsStr::from_bytes(path_bytes);
    let Ok(file) = File::open(path_str) else {
        eprintln!(
            "CIDR file unavailable, using empty map: {}",
            String::from_utf8_lossy(path_bytes)
        );
        return 0;
    };

    let mut loaded: c_int = 0;
    let mut failed: c_int = 0;
    let value: u8 = 1;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    while reader.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
        let line = String::from_utf8_lossy(&buf);
        if let Some(parsed) = parse_cidr(&line) {
            let rc = match (ipv6, parsed) {
                (true, ParsedCidr::V6(key)) => Some(bpf_update_elem(map_fd, &key, &value)),
                (false, ParsedCidr::V4(key)) => Some(bpf_update_elem(map_fd, &key, &value)),
                _ => None,
            };
            if let Some(rc) = rc {
                if rc == 0 {
                    loaded += 1;
                } else {
                    failed += 1;
                }
            }
        }
        buf.clear();
    }

    if failed > 0 {
        eprintln!(
            "CIDR map dropped {} entries (map full or kernel error): {}",
            failed,
            String::from_utf8_lossy(path_bytes)
        );
    }
    loaded
}

/// Port of `int load_uid_file(int map_fd, const char *path)` (`config.c:238`).
pub fn load_uid_file(map_fd: c_int, path: Option<&[u8]>) -> c_int {
    let Some(path_bytes) = path else {
        return 0;
    };
    if path_bytes.is_empty() {
        return 0;
    }
    let path_str = OsStr::from_bytes(path_bytes);
    let Ok(file) = File::open(path_str) else {
        eprintln!(
            "UID file unavailable, using empty map: {}",
            String::from_utf8_lossy(path_bytes)
        );
        return 0;
    };

    let value: u8 = 1;
    let mut loaded: c_int = 0;
    let mut failed: c_int = 0;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    while reader.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
        let line = String::from_utf8_lossy(&buf);
        if let Some(uid) = parse_uid(&line) {
            if bpf_update_elem(map_fd, &uid, &value) == 0 {
                loaded += 1;
            } else {
                failed += 1;
            }
        }
        buf.clear();
    }

    if failed > 0 {
        eprintln!(
            "UID map dropped {} entries (map full or kernel error): {}",
            failed,
            String::from_utf8_lossy(path_bytes)
        );
    }
    loaded
}

/// Port of `bool uid_file_has_entries(const char *path)` (`config.c:264`).
pub fn uid_file_has_entries(path: Option<&[u8]>) -> bool {
    let Some(path_bytes) = path else {
        return false;
    };
    if path_bytes.is_empty() {
        return false;
    }
    let Ok(file) = File::open(OsStr::from_bytes(path_bytes)) else {
        return false;
    };
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    while reader.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
        let line = String::from_utf8_lossy(&buf);
        if parse_uid(&line).is_some() {
            return true;
        }
        buf.clear();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v4_and_v6_cidrs_and_rejects_invalid() {
        assert_eq!(
            parse_cidr("  1.1.1.0/24 # cn\n"),
            Some(ParsedCidr::V4(Lpm4Key {
                prefixlen: 24,
                addr: [1, 1, 1, 0],
            }))
        );
        assert_eq!(
            parse_cidr("240e::/16"),
            Some(ParsedCidr::V6(Lpm6Key {
                prefixlen: 16,
                addr: Ipv6Addr::new(0x240e, 0, 0, 0, 0, 0, 0, 0).octets(),
            }))
        );
        assert_eq!(parse_cidr("1.2.3.4/33"), None);
        assert_eq!(parse_cidr("2001:db8::/129"), None);
        assert_eq!(parse_cidr("# comment"), None);
        assert_eq!(parse_cidr("not-a-cidr"), None);
    }

    #[test]
    fn parses_uids_within_u32_range() {
        assert_eq!(parse_uid("  10234 # app\n"), Some(10234));
        assert_eq!(parse_uid("4294967295"), Some(u32::MAX));
        assert_eq!(parse_uid("4294967296"), None);
        assert_eq!(parse_uid("invalid"), None);
    }

    #[test]
    fn reads_config_json_defaults_and_overrides() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("boxbpf_cfg_test_{}.json", std::process::id()));
        std::fs::write(&path, r#"{"ipv6": 0, "cidr4": "/tmp/a.txt"}"#).unwrap();
        let cfg = read_config_json(&path).expect("valid ipv4-only config");
        let _ = std::fs::remove_file(&path);
        assert!(!cfg.ipv6);
        assert_eq!(cfg.cidr4_file, "/tmp/a.txt");
        assert_eq!(cfg.pin_cidr_out4, PIN_CIDR_OUT4);
    }
}
