//! The Rust port of `src/api/hash.ts`.

#![forbid(unsafe_code)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const DATAVIEW_BOUNDS_ERR: &str = "RangeError: Offset is outside the bounds of the DataView";

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn f64_to_uint8(n: f64) -> u8 {
    if !n.is_finite() || n == 0.0 {
        return 0;
    }
    n.trunc().rem_euclid(256.0) as u8
}

fn str_to_uint8(s: &str) -> u8 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return 0;
    }
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_or(0, |v| (v & 0xff) as u8);
    }
    trimmed.parse::<f64>().map_or(0, f64_to_uint8)
}

fn value_to_uint8(value: &Value) -> u8 {
    if let Some(n) = crate::js::as_number(value) {
        return f64_to_uint8(n);
    }
    match value {
        Value::Null => 0,
        Value::Bool(b) => u8::from(*b),
        Value::String(s) => str_to_uint8(s),
        Value::Array(items) if items.is_empty() => 0,
        Value::Array(items) if items.len() == 1 => value_to_uint8(&items[0]),
        _ => 0,
    }
}

fn extract_input_bytes(input: &Value) -> Result<Vec<u8>, String> {
    if input.is_null() {
        return Err("TypeError: Cannot read properties of null (reading 'length')".into());
    }
    if crate::js::is_undefined(input) {
        return Err("TypeError: Cannot read properties of undefined (reading 'length')".into());
    }
    if let Some(units) = crate::js::string_units(input) {
        return Ok(units
            .into_iter()
            .map(|u| match u {
                0x30..=0x39 => (u - 0x30) as u8,
                _ => 0,
            })
            .collect());
    }
    match input {
        Value::Array(items) => Ok(items.iter().map(value_to_uint8).collect()),
        Value::Object(map) => {
            if map.len() == 1
                && let Some(items) = map
                    .get("$bytes")
                    .or_else(|| map.get("$buffer"))
                    .and_then(Value::as_array)
            {
                return Ok(items.iter().map(value_to_uint8).collect());
            }
            let Some(len) = map.get("length").and_then(crate::js::as_number) else {
                return Err(DATAVIEW_BOUNDS_ERR.into());
            };
            if !len.is_finite() || len < 0.0 {
                return Err(DATAVIEW_BOUNDS_ERR.into());
            }
            let count = len.trunc() as usize;
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let b = map.get(&i.to_string()).map_or(0, value_to_uint8);
                out.push(b);
            }
            Ok(out)
        }
        _ => Err(DATAVIEW_BOUNDS_ERR.into()),
    }
}

fn format_uuid(bytes: &[u8; 16]) -> String {
    let hex = hex_encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Port of `sha256` (`src/api/hash.ts:2`).
pub async fn sha256(text: String) -> Result<String, String> {
    let digest = Sha256::digest(text.as_bytes());
    Ok(hex_encode(&digest))
}

/// Port of `sha256Bytes` (`src/api/hash.ts:17`).
pub fn sha256_bytes(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let bytes = extract_input_bytes(&input)?;
    let digest = Sha256::digest(&bytes);
    Ok(json!({ "$bytes": digest.as_slice() }))
}

/// Pure-Rust cryptographic RFC 4122 UUID v4 generator (kept out of `rust-anything` exact-replay diff
/// because UUID v4 output is non-deterministic, while still used directly by `doona-wasm`).
pub mod random {
    use super::format_uuid;

    pub fn uuid() -> Result<String, String> {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).map_err(|e| format!("Error: {e}"))?;
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Ok(format_uuid(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_fips_test_vectors() {
        let digest = Sha256::digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            hex_encode(&digest),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        let out = sha256_bytes(json!({ "$bytes": [1, 2, 3] })).unwrap();
        let bytes = out["$bytes"].as_array().unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn sha256_bytes_matches_js_coercion_and_errors() {
        assert_eq!(
            sha256_bytes(json!(["bad", 257])).unwrap(),
            sha256_bytes(json!({ "$bytes": [0, 1] })).unwrap()
        );
        assert_eq!(
            sha256_bytes(json!("z")).unwrap(),
            sha256_bytes(json!({ "$bytes": [0] })).unwrap()
        );
        assert!(sha256_bytes(Value::Null).is_err());
        assert!(sha256_bytes(json!({})).is_err());
        assert!(sha256_bytes(json!(42)).is_err());
    }

    #[test]
    fn uuid_generates_valid_v4_rfc4122_strings() {
        let a = random::uuid().unwrap();
        let b = random::uuid().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36);
        let parts: Vec<&str> = a.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            vec![8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('4'));
        assert!(matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'));
    }
}
