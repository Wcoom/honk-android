//! The Rust port of `apps/lite/src/describe.ts`.

#![forbid(unsafe_code)]

use crate::js;
use dae_core::port::{conditions, groups, js_str, text};
use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::OnceLock;

const EMPTY_GROUP_FILTER: &str = "name(regex: '^$')";

// Compact table of regions from `src/dae/regions.ts`: `ISO:alias1|alias2|...`
const REGION_ALIASES: &[(&str, &str)] = &[
    ("HK", "hk|hkg|hongkong|hong kong|香港|港"),
    ("TW", "tw|tpe|taiwan|taipei|台灣|台湾|臺灣|台北"),
    (
        "JP",
        "jp|jpn|japan|tokyo|osaka|nrt|hnd|kix|日本|東京|东京|大阪",
    ),
    ("SG", "sg|sgp|singapore|sin|新加坡|獅城|狮城"),
    (
        "US",
        "us|usa|united states|america|lax|sjc|sea|nyc|los angeles|san jose|seattle|美國|美国|洛杉磯|洛杉矶|聖何塞|圣何塞|西雅圖|西雅图",
    ),
    (
        "KR",
        "kr|kor|korea|seoul|icn|south korea|南韩|南韓|韓國|韩国|首爾|首尔",
    ),
    ("CA", "can|canada|toronto|yyz|加拿大"),
    (
        "GB",
        "gb|uk|united kingdom|britain|london|lhr|英國|英国|倫敦|伦敦",
    ),
    (
        "DE",
        "de|deu|germany|frankfurt|fra|德國|德国|法蘭克福|法兰克福",
    ),
    ("FR", "fr|france|paris|cdg|法國|法国|巴黎"),
    (
        "NL",
        "nl|nld|netherlands|amsterdam|ams|荷蘭|荷兰|阿姆斯特丹",
    ),
    ("RU", "ru|rus|russia|moscow|俄羅斯|俄罗斯|莫斯科"),
    (
        "AU",
        "au|aus|australia|sydney|syd|澳洲|澳大利亞|澳大利亚|悉尼",
    ),
    ("IN", "in|ind|india|mumbai|bom|印度|孟買|孟买"),
    ("TR", "tr|tur|turkey|türkiye|istanbul|土耳其"),
    ("BR", "br|bra|brazil|sao paulo|gru|巴西"),
    ("AR", "ar|arg|argentina|阿根廷"),
    ("MY", "my|mys|malaysia|kul|馬來西亞|马来西亚"),
    ("TH", "th|tha|thailand|bangkok|bkk|泰國|泰国"),
    ("VN", "vn|vnm|vietnam|越南"),
    ("PH", "ph|phl|philippines|manila|菲律賓|菲律宾"),
    ("ID", "id|idn|indonesia|jakarta|印度尼西亚|印度尼西亞|印尼"),
    (
        "AE",
        "ae|are|uae|united arab emirates|阿联酋|阿聯酋|dubai|dxb|阿聯|阿联|杜拜|迪拜",
    ),
    ("CH", "ch|che|switzerland|zurich|瑞士"),
    ("SE", "se|swe|sweden|stockholm|瑞典"),
    ("FI", "fi|fin|finland|helsinki|芬蘭|芬兰"),
    ("IT", "it|ita|italy|milan|義大利|意大利"),
    ("ES", "es|esp|spain|madrid|西班牙"),
    ("PL", "pl|pol|poland|warsaw|波蘭|波兰"),
    ("MO", "mo|mac|macau|macao|澳門|澳门"),
    ("NZ", "new zealand|新西兰|紐西蘭"),
    ("MX", "mexico|墨西哥"),
    ("NO", "norway|挪威"),
    ("DK", "denmark|丹麦|丹麥"),
    ("IE", "ireland|爱尔兰|愛爾蘭"),
    ("ZA", "south africa|南非"),
    ("IL", "israel|以色列"),
    ("UA", "ukraine|乌克兰|烏克蘭"),
    ("PT", "portugal|葡萄牙"),
    (
        "CN",
        "cn|chn|china|shanghai|beijing|shenzhen|中國|中国|上海|北京|深圳",
    ),
    (
        "KZ",
        "kazakhstan|almaty|哈萨克斯坦|哈薩克斯坦|哈萨克|哈薩克",
    ),
    ("AT", "austria|vienna|奥地利|奧地利"),
    ("BE", "belgium|brussels|比利时|比利時"),
    ("LU", "luxembourg|卢森堡|盧森堡"),
    ("CZ", "czech|czechia|prague|捷克"),
    ("RO", "romania|bucharest|罗马尼亚|羅馬尼亞"),
    ("IS", "iceland|reykjavik|冰岛|冰島"),
    ("CL", "chile|santiago|智利"),
    ("CO", "colombia|bogota|哥伦比亚|哥倫比亞"),
    ("SA", "saudi|riyadh|沙特|沙烏地"),
    ("PK", "pakistan|karachi|巴基斯坦"),
    ("NG", "nigeria|lagos|尼日利亚|尼日利亞|奈及利亞"),
    ("EG", "egypt|cairo|埃及"),
    ("KH", "cambodia|柬埔寨"),
];

const FLAG_REGIONS: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

const CAPITAL_ONLY: &[&str] = &[
    "IN", "IT", "MY", "NO", "ID", "KZ", "AT", "BE", "LU", "CZ", "RO", "IS", "CL", "CO", "SA", "PK",
    "NG", "EG", "KH",
];

fn han_re() -> Option<&'static Regex> {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\p{Han}").ok()).as_ref()
}

fn latin_re() -> Option<&'static Regex> {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[\p{Latin}\p{Nd}]").ok())
        .as_ref()
}

fn is_han(s: &str) -> bool {
    han_re().is_some_and(|re| re.is_match(s))
}

fn is_latin(s: &str) -> bool {
    latin_re().is_some_and(|re| re.is_match(s))
}

fn region_flag(iso: &str) -> String {
    iso.bytes()
        .filter_map(|b| char::from_u32(0x1F1E6 + u32::from(b.saturating_sub(b'A'))))
        .collect()
}

fn escape_regex_meta(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if matches!(
            ch,
            '.' | '*' | '+' | '?' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' | '\\'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn alternatives(mut keys: Vec<String>) -> String {
    keys.sort_by(|a, b| {
        b.encode_utf16()
            .count()
            .cmp(&a.encode_utf16().count())
            .then_with(|| a.encode_utf16().cmp(b.encode_utf16()))
    });
    keys.into_iter()
        .map(|k| escape_regex_meta(&k))
        .collect::<Vec<_>>()
        .join("|")
}

fn patterns_of(iso: &str, aliases: &[&str], with_flag: bool) -> Vec<String> {
    let cap_only = CAPITAL_ONLY.contains(&iso);
    let mut keys: Vec<String> = Vec::new();
    if !aliases.is_empty() {
        let mut seen = HashSet::new();
        for &k in std::iter::once(&iso).chain(aliases.iter()) {
            if seen.insert(k.to_string()) {
                keys.push(k.to_string());
            }
        }
    }
    let han: Vec<String> = keys.iter().filter(|k| is_han(k)).cloned().collect();
    let mut latin_seen = HashSet::new();
    let mut latin: Vec<String> = Vec::new();
    for k in &keys {
        if !han.contains(k) && !(cap_only && k.to_uppercase() == iso) {
            let low = k.to_lowercase();
            if latin_seen.insert(low.clone()) {
                latin.push(low);
            }
        }
    }
    let han_filtered: Vec<String> = han
        .iter()
        .filter(|k| !han.iter().any(|o| o != *k && k.contains(o.as_str())))
        .cloned()
        .collect();
    let han_pat = alternatives(han_filtered);
    let start = r"(?:^|[^\p{L}\p{M}])";
    let end = r"(?:[^\p{L}\p{M}]|$)";
    let mut rest_parts: Vec<String> = Vec::new();
    if !han_pat.is_empty() {
        rest_parts.push(han_pat);
    }
    if with_flag {
        rest_parts.push(region_flag(iso));
    }
    let rest = rest_parts.join("|");
    let mut main_parts: Vec<String> = Vec::new();
    if !latin.is_empty() {
        main_parts.push(format!("(?i){start}(?:{}){end}", alternatives(latin)));
    }
    if !rest.is_empty() {
        main_parts.push(rest);
    }
    let mut patterns: Vec<String> = Vec::new();
    let combined = main_parts.join("|");
    if !combined.is_empty() {
        patterns.push(combined);
    }
    if cap_only {
        patterns.push(format!("{start}{iso}{end}"));
    }
    patterns
}

fn region_regexes() -> &'static Vec<(&'static str, Vec<String>)> {
    static CACHE: OnceLock<Vec<(&'static str, Vec<String>)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut all: Vec<(&'static str, Vec<&'static str>)> = REGION_ALIASES
            .iter()
            .map(|&(iso, raw)| (iso, raw.split('|').collect()))
            .collect();
        for iso in FLAG_REGIONS.split(' ') {
            if !REGION_ALIASES.iter().any(|&(c, _)| c == iso) {
                all.push((iso, Vec::new()));
            }
        }
        let mut out = Vec::new();
        for (iso, aliases) in all {
            for with_flag in [true, false] {
                let pats = patterns_of(iso, &aliases, with_flag);
                if !pats.is_empty() {
                    out.push((iso, pats));
                }
            }
        }
        out
    })
}

fn regex_literals(pattern: &str) -> Option<Vec<String>> {
    let body = pattern.strip_prefix("(?i)").unwrap_or(pattern);
    let parts: Vec<&str> = body.split('|').collect();
    if parts.is_empty() {
        return None;
    }
    let ok = parts.iter().all(|p| {
        !p.is_empty()
            && !p.bytes().any(|b| {
                matches!(
                    b,
                    b'\\'
                        | b'^'
                        | b'$'
                        | b'.'
                        | b'*'
                        | b'+'
                        | b'?'
                        | b'('
                        | b')'
                        | b'['
                        | b']'
                        | b'{'
                        | b'}'
                        | b'|'
                )
            })
    });
    ok.then(|| parts.into_iter().map(str::to_string).collect())
}

struct SubscriptionBinding {
    source: String,
    regions: Vec<String>,
    words: Vec<String>,
}

fn read_subscription_binding(filter: &str) -> Option<SubscriptionBinding> {
    let rows = conditions::parse_group_conditions(filter.to_string()).ok()??;
    if rows.is_empty() || rows.len() > 2 {
        return None;
    }
    let r0 = &rows[0];
    if r0.get("kind").and_then(Value::as_str) != Some("subtag")
        || r0.get("negate").and_then(Value::as_bool) == Some(true)
        || r0.get("alternatives").is_some()
    {
        return None;
    }
    let source = conditions::parse_condition_values_vec(r0.get("value")?.as_str()?)?;
    if source.len() != 1 {
        return None;
    }
    let mut binding = SubscriptionBinding {
        source: source[0].clone(),
        regions: Vec::new(),
        words: Vec::new(),
    };
    let Some(rest) = rows.get(1) else {
        return Some(binding);
    };
    if rest.get("negate").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let mut terms: Vec<(String, String)> = Vec::new();
    let mut push_term = |t: &Value| {
        let k = t.get("kind").and_then(Value::as_str).unwrap_or("");
        let v = t.get("value").and_then(Value::as_str).unwrap_or("");
        if let Some(vals) = conditions::parse_condition_values_vec(v) {
            for val in vals {
                terms.push((k.to_string(), val));
            }
        }
    };
    push_term(rest);
    if let Some(alts) = rest.get("alternatives").and_then(Value::as_array) {
        for alt in alts {
            push_term(alt);
        }
    }
    let regexes = region_regexes();
    let mut i = 0;
    while i < terms.len() {
        let (kind, value) = &terms[i];
        if kind == "nameKeyword" {
            binding.words.push(value.clone());
        } else if kind == "nameRegex" {
            let matched = regexes.iter().find(|(_, pats)| {
                pats.iter().enumerate().all(|(off, pat)| {
                    terms
                        .get(i + off)
                        .is_some_and(|(tk, tv)| tk == "nameRegex" && tv == pat)
                })
            });
            if let Some((iso, pats)) = matched {
                if !binding.regions.iter().any(|r| r == iso) {
                    binding.regions.push((*iso).to_string());
                }
                i += pats.len() - 1;
            } else {
                let words = regex_literals(value)?;
                binding.words.extend(words);
            }
        } else {
            return None;
        }
        i += 1;
    }
    Some(binding)
}

// ---- Phrase constructors -----------------------------------------------------------------------

fn phrase(key: &str, values: Option<Map<String, Value>>) -> Value {
    let mut m = Map::new();
    m.insert("key".into(), Value::String(key.to_string()));
    m.insert(
        "values".into(),
        values.map_or_else(js::undefined, Value::Object),
    );
    Value::Object(m)
}

fn phrase_kv(key: &str, pairs: &[(&str, Value)]) -> Value {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), v.clone());
    }
    phrase(key, Some(m))
}

fn names_list(items: Vec<String>) -> Value {
    json!({ "items": items, "join": "sep" })
}

fn quoted_list(items: Vec<String>) -> Value {
    json!({ "items": items, "join": "or", "quote": true })
}

fn node_names(items: &[String]) -> Value {
    let parts: Vec<Value> = items.iter().map(|s| describe_target_val(s)).collect();
    json!({ "parts": parts, "join": "sep" })
}

fn list_of(value: &str) -> Vec<String> {
    conditions::parse_condition_values_vec(value).unwrap_or_else(|| vec![value.to_string()])
}

fn policy_key_str(keyword: Option<&str>) -> &'static str {
    let raw = keyword.unwrap_or("select");
    let head = raw.split('(').next().unwrap_or("");
    match js_str::js_trim(head).to_lowercase().as_str() {
        "select" | "selector" | "fixed" | "random" | "min" | "min_moving_avg10" => "policyManual",
        "min_avg10" | "min_moving_avg" | "min_last_delay" | "urltest" => "policyUrlTest",
        "fallback" => "policyFallback",
        "roundrobin" | "round_robin" | "loadbalance" | "balance" => "policyBalance",
        "score" => "policyScore",
        _ => "policyManual",
    }
}

fn geosite_key(v: &str) -> Option<&'static str> {
    Some(match v {
        "cn" => "geo_site_cn",
        "gfw" => "geo_site_gfw",
        "category-ads-all" | "category-ads" => "geo_site_ads",
        "google-cn" => "geo_site_google_cn",
        "category-games" => "geo_site_games",
        "category-games@cn" => "geo_site_games_cn",
        "apple" => "geo_site_apple",
        "microsoft" => "geo_site_microsoft",
        "telegram" => "geo_site_telegram",
        "openai" => "geo_site_openai",
        "youtube" => "geo_site_youtube",
        "netflix" => "geo_site_netflix",
        "disney" => "geo_site_disney",
        "spotify" => "geo_site_spotify",
        "bilibili" => "geo_site_bilibili",
        "netease" => "geo_site_netease",
        "bahamut" => "geo_site_bahamut",
        _ => return None,
    })
}

fn geo_value_key(kind: &str, value: &str) -> Option<&'static str> {
    match kind {
        "geosite" | "qnameGeosite" => geosite_key(value),
        "geoip" | "answerGeoip" => match value {
            "private" => Some("geo_ip_private"),
            "cn" => Some("geo_ip_cn"),
            _ => None,
        },
        _ => None,
    }
}

fn is_known_kind(kind: &str) -> bool {
    matches!(
        kind,
        "domainSuffix"
            | "domain"
            | "domainKeyword"
            | "domainRegex"
            | "geosite"
            | "dip"
            | "geoip"
            | "sip"
            | "dport"
            | "sport"
            | "pname"
            | "l4proto"
            | "ipversion"
            | "mac"
            | "dscp"
            | "qnameSuffix"
            | "qnameFull"
            | "qnameKeyword"
            | "qnameGeosite"
            | "qnameRegex"
            | "qtype"
            | "upstream"
            | "answerIp"
            | "answerGeoip"
    )
}

fn describe_target_val(target: &str) -> Value {
    let trimmed = js_str::js_trim(target);
    let must_prefix = trimmed.strip_suffix("(must)").filter(|p| {
        !p.chars()
            .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    });
    let name = js_str::js_trim(must_prefix.unwrap_or(target));
    let base = match name {
        "direct" => phrase("out_direct", None),
        "block" | "reject" => phrase("out_block", None),
        "accept" => phrase("out_accept", None),
        "asis" => phrase("out_asis", None),
        _ => Value::String(name.to_string()),
    };
    if must_prefix.is_some() {
        phrase_kv("mustTarget", &[("target", base)])
    } else {
        base
    }
}

// ---- JS value helpers for phraseText & error parity --------------------------------------------

fn is_nullish(v: &Value) -> bool {
    v.is_null() || js::is_undefined(v)
}

fn check_obj(v: &Value, prop: &str) -> Result<(), String> {
    if is_nullish(v) {
        let kind = if v.is_null() { "null" } else { "undefined" };
        return Err(format!(
            "TypeError: Cannot read properties of {kind} (reading '{prop}')"
        ));
    }
    Ok(())
}

fn prop_val(v: &Value, key: &str) -> Value {
    match v {
        Value::Object(m) if !js::is_undefined(v) => {
            m.get(key).cloned().unwrap_or_else(js::undefined)
        }
        Value::Array(a) => key
            .parse::<usize>()
            .ok()
            .and_then(|i| a.get(i).cloned())
            .unwrap_or_else(js::undefined),
        Value::String(s) => key
            .parse::<usize>()
            .ok()
            .and_then(|i| s.encode_utf16().nth(i))
            .and_then(|u| char::from_u32(u32::from(u)).map(|c| Value::String(c.to_string())))
            .unwrap_or_else(js::undefined),
        _ => js::undefined(),
    }
}

fn has_key(v: &Value, key: &str) -> Result<bool, String> {
    if is_nullish(v) || js::as_number(v).is_some() || matches!(v, Value::Bool(_) | Value::String(_))
    {
        return Err(format!(
            "TypeError: Cannot use 'in' operator to search for '{key}' in {}",
            js_str::js_to_string(v)
        ));
    }
    Ok(match v {
        Value::Object(m) => m.contains_key(key),
        Value::Array(a) => key.parse::<usize>().is_ok_and(|i| i < a.len()),
        _ => false,
    })
}

fn js_array_join(items: &[String], sep: &Value) -> String {
    let sep_str = if js::is_undefined(sep) {
        ",".to_string()
    } else {
        js_str::js_to_string(sep)
    };
    items.join(&sep_str)
}

fn js_replace_first(haystack: &str, needle: &str, replacement: &str) -> String {
    let Some(pos) = haystack.find(needle) else {
        return haystack.to_string();
    };
    let before = &haystack[..pos];
    let after = &haystack[pos + needle.len()..];
    let mut expanded = String::with_capacity(replacement.len());
    let bytes = replacement.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'$' => {
                    expanded.push('$');
                    i += 2;
                    continue;
                }
                b'&' => {
                    expanded.push_str(needle);
                    i += 2;
                    continue;
                }
                b'`' => {
                    expanded.push_str(before);
                    i += 2;
                    continue;
                }
                b'\'' => {
                    expanded.push_str(after);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        if let Some(ch) = replacement[i..].chars().next() {
            expanded.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    format!("{before}{expanded}{after}")
}

fn region_short(iso: &str, locale: &str) -> Option<&'static str> {
    match (iso, locale) {
        ("HK", "en") => Some("Hong Kong"),
        ("HK", "zh-TW" | "zh-CN") => Some("香港"),
        ("TW", "en") => Some("Taiwan"),
        ("TW", "zh-TW") => Some("台灣"),
        ("TW", "zh-CN") => Some("台湾"),
        ("KR", "zh-TW") => Some("韓國"),
        ("MO", "en") => Some("Macao"),
        ("MO", "zh-TW") => Some("澳門"),
        ("MO", "zh-CN") => Some("澳门"),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn js_val_from_json(v: &Value) -> wasm_bindgen::JsValue {
    use wasm_bindgen::JsValue;
    if js::is_undefined(v) {
        return JsValue::UNDEFINED;
    }
    match v {
        Value::Null => JsValue::NULL,
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Number(n) => JsValue::from_f64(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => JsValue::from_str(s),
        _ => js_sys::Object::new().into(),
    }
}

#[cfg(target_arch = "wasm32")]
fn js_err_string(e: wasm_bindgen::JsValue) -> String {
    if let Ok(name) = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("name"))
        && let Ok(msg) = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("message"))
        && let (Some(n), Some(m)) = (name.as_string(), msg.as_string())
    {
        return format!("{n}: {m}");
    }
    e.as_string()
        .unwrap_or_else(|| "RangeError: Incorrect locale information provided".to_string())
}

#[cfg(target_arch = "wasm32")]
fn display_region_name(iso: &Value, locale: &Value) -> Result<Option<String>, String> {
    use js_sys::{Array, Function, Object, Reflect};
    use wasm_bindgen::{JsCast, JsValue};

    let global = js_sys::global();
    let intl = Reflect::get(&global, &JsValue::from_str("Intl")).map_err(js_err_string)?;
    let dn_ctor: Function = Reflect::get(&intl, &JsValue::from_str("DisplayNames"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let opts = Object::new();
    Reflect::set(
        &opts,
        &JsValue::from_str("type"),
        &JsValue::from_str("region"),
    )
    .map_err(js_err_string)?;
    let loc_arr = Array::of1(&js_val_from_json(locale));
    let names =
        Reflect::construct(&dn_ctor, &Array::of2(&loc_arr, &opts)).map_err(js_err_string)?;
    let of_fn: Function = Reflect::get(&names, &JsValue::from_str("of"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let res = of_fn
        .call1(&names, &js_val_from_json(iso))
        .map_err(js_err_string)?;
    Ok(res.as_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn display_region_name(iso: &Value, locale: &Value) -> Result<Option<String>, String> {
    let Some(loc) = locale.as_str() else {
        return Err("TypeError: Language ID should be string or object.".into());
    };
    if loc.is_empty()
        || !loc.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || loc.starts_with('-')
        || loc.ends_with('-')
    {
        return Err("RangeError: Incorrect locale information provided".into());
    }
    Ok(iso.as_str().map(str::to_string))
}

fn region_label(iso: &Value, locale: &Value) -> Result<String, String> {
    if let (Some(code), Some(loc)) = (iso.as_str(), locale.as_str())
        && let Some(short) = region_short(code, loc)
    {
        return Ok(short.to_string());
    }
    if let Some(name) = display_region_name(iso, locale)? {
        return Ok(name);
    }
    Ok(if is_nullish(iso) {
        String::new()
    } else {
        js_str::js_to_string(iso)
    })
}

fn first_u16_char(s: &str) -> Option<char> {
    s.encode_utf16()
        .next()
        .and_then(|u| char::from_u32(u32::from(u)))
}

fn last_u16_char(s: &str) -> Option<char> {
    s.encode_utf16()
        .last()
        .and_then(|u| char::from_u32(u32::from(u)))
}

fn needs_gap(a: Option<char>, b: Option<char>) -> bool {
    let (Some(ca), Some(cb)) = (a, b) else {
        return false;
    };
    let mut ba = [0u8; 4];
    let mut bb = [0u8; 4];
    let sa = ca.encode_utf8(&mut ba);
    let sb = cb.encode_utf8(&mut bb);
    (is_han(sa) && is_latin(sb)) || (is_latin(sa) && is_han(sb))
}

// ---- Exported ports ----------------------------------------------------------------------------

/// Port of `describeGroupFilter` (`apps/lite/src/describe.ts:26`).
pub fn describe_group_filter(filter: String) -> Result<Option<Value>, String> {
    let text_str = js_str::js_trim(&filter);
    if text_str == EMPTY_GROUP_FILTER {
        return Ok(None);
    }
    if let Some(binding) = read_subscription_binding(text_str)
        && !binding.regions.is_empty()
    {
        let key = if binding.words.is_empty() {
            "descSourceRegions"
        } else {
            "descSourceRegionsContains"
        };
        return Ok(Some(phrase_kv(
            key,
            &[
                ("source", Value::String(binding.source)),
                ("regions", json!({ "regions": binding.regions })),
                ("list", quoted_list(binding.words)),
            ],
        )));
    }
    if groups::exact_tokens_vec(text_str, "group").is_some() {
        let nested = groups::nested_in(json!({ "filters": [text_str] }))?;
        return Ok(Some(phrase_kv(
            "descGroups",
            &[("names", names_list(nested))],
        )));
    }
    let Some(rows) = conditions::parse_group_conditions(text_str.to_string())? else {
        return Ok(Some(phrase("descCustom", None)));
    };
    let mut sources: Vec<String> = Vec::new();
    let mut nodes: Vec<String> = Vec::new();
    let mut contains: Vec<String> = Vec::new();
    let mut subscription_contains: Vec<String> = Vec::new();
    let mut excluded_names: Vec<String> = Vec::new();
    let mut excluded_contains: Vec<String> = Vec::new();
    let mut manual = false;
    let mut pattern = false;

    for row in &rows {
        let negate = row.get("negate").and_then(Value::as_bool).unwrap_or(false);
        if let Some(alts) = row.get("alternatives").and_then(Value::as_array)
            && !alts.is_empty()
        {
            let mut terms: Vec<&Value> = Vec::with_capacity(1 + alts.len());
            terms.push(row);
            terms.extend(alts.iter());
            let all_ok = terms.iter().all(|t| {
                matches!(
                    t.get("kind").and_then(Value::as_str),
                    Some("nameKeyword" | "nameRegex")
                )
            });
            if !all_ok || negate {
                return Ok(Some(phrase("descCustom", None)));
            }
            let mut words = Vec::new();
            let mut has_non_literal = false;
            for t in &terms {
                let k = t.get("kind").and_then(Value::as_str).unwrap_or("");
                let v = t.get("value").and_then(Value::as_str).unwrap_or("");
                for item in list_of(v) {
                    if k == "nameKeyword" {
                        words.push(item);
                    } else if let Some(lits) = regex_literals(&item) {
                        words.extend(lits);
                    } else {
                        has_non_literal = true;
                    }
                }
            }
            if has_non_literal {
                pattern = true;
            } else {
                contains.extend(words);
            }
            continue;
        }
        let kind = row.get("kind").and_then(Value::as_str).unwrap_or("");
        let values = list_of(row.get("value").and_then(Value::as_str).unwrap_or(""));
        match kind {
            "group" => {
                return Ok(Some(phrase_kv(
                    "descGroups",
                    &[("names", names_list(values))],
                )));
            }
            "subtag" => {
                if negate {
                    return Ok(Some(phrase("descCustom", None)));
                }
                sources.extend(values);
            }
            "subtagRegex" => {
                if negate && values.len() == 1 && values[0] == ".*" {
                    manual = true;
                } else {
                    return Ok(Some(phrase("descCustom", None)));
                }
            }
            "subtagKeyword" => {
                if negate {
                    return Ok(Some(phrase("descCustom", None)));
                }
                subscription_contains.extend(values);
            }
            "nameExact" => {
                if negate {
                    excluded_names.extend(values);
                } else {
                    nodes.extend(values);
                }
            }
            "nameKeyword" => {
                if negate {
                    excluded_contains.extend(values);
                } else {
                    contains.extend(values);
                }
            }
            "nameRegex" => {
                let mut flat = Vec::new();
                let mut any_none = false;
                for v in &values {
                    if let Some(lits) = regex_literals(v) {
                        flat.extend(lits);
                    } else {
                        any_none = true;
                    }
                }
                if any_none {
                    if negate {
                        return Ok(Some(phrase("descCustom", None)));
                    }
                    pattern = true;
                } else if negate {
                    excluded_contains.extend(flat);
                } else {
                    contains.extend(flat);
                }
            }
            _ => return Ok(Some(phrase("descCustom", None))),
        }
    }

    let source: Option<Value> = if !sources.is_empty() {
        Some(names_list(sources))
    } else if manual {
        Some(phrase("descManualSource", None))
    } else {
        None
    };

    let mut base = if !nodes.is_empty() {
        match source {
            Some(s) => phrase_kv(
                "descSourceNodes",
                &[("source", s), ("names", node_names(&nodes))],
            ),
            None => phrase_kv("descNodes", &[("names", node_names(&nodes))]),
        }
    } else if pattern {
        match source {
            Some(s) => phrase_kv("descSourcePattern", &[("source", s)]),
            None => phrase("descPattern", None),
        }
    } else if !contains.is_empty() {
        match source {
            Some(s) => phrase_kv(
                "descSourceContains",
                &[("source", s), ("list", quoted_list(contains))],
            ),
            None => phrase_kv("descContains", &[("list", quoted_list(contains))]),
        }
    } else if !subscription_contains.is_empty() && source.is_none() {
        phrase_kv(
            "descSubscriptionContains",
            &[("list", quoted_list(subscription_contains))],
        )
    } else if let Some(s) = source {
        phrase_kv("descSourceAll", &[("source", s)])
    } else if !excluded_names.is_empty() && excluded_contains.is_empty() {
        return Ok(Some(phrase_kv(
            "descAllNodesExcept",
            &[("names", node_names(&excluded_names))],
        )));
    } else {
        phrase("descAllNodes", None)
    };

    if !excluded_names.is_empty() {
        base = phrase_kv(
            "descExcludeNames",
            &[("base", base), ("names", node_names(&excluded_names))],
        );
    }
    if !excluded_contains.is_empty() {
        base = phrase_kv(
            "descExclude",
            &[("base", base), ("list", quoted_list(excluded_contains))],
        );
    }
    Ok(Some(base))
}

/// Port of `describeGroupFilters` (`apps/lite/src/describe.ts:108`).
pub fn describe_group_filters(filters: Vec<String>) -> Result<Vec<Value>, String> {
    if filters.is_empty() {
        return Ok(vec![phrase("descAllNodes", None)]);
    }
    let mut described = Vec::new();
    for f in filters {
        if let Some(p) = describe_group_filter(f)? {
            described.push(p);
        }
    }
    if described.is_empty() {
        Ok(vec![phrase("descNone", None)])
    } else {
        Ok(described)
    }
}

/// Port of `policyKey` (`apps/lite/src/describe.ts:135`).
pub fn policy_key(keyword: Option<String>) -> Result<Value, String> {
    Ok(Value::String(
        policy_key_str(keyword.as_deref()).to_string(),
    ))
}

/// Port of `describeCondition` (`apps/lite/src/describe.ts:193`).
pub fn describe_condition(condition: Value) -> Result<Value, String> {
    check_obj(&condition, "value")?;
    let val_v = prop_val(&condition, "value");
    check_obj(&val_v, "trim")?;
    let Some(val_str) = val_v.as_str() else {
        return Err("TypeError: condition.value.trim is not a function".into());
    };
    let values = conditions::parse_condition_values_vec(val_str)
        .unwrap_or_else(|| vec![text::unquote_str(js_str::js_trim(val_str))]);
    let kind_v = prop_val(&condition, "kind");
    let kind_str = kind_v.as_str().unwrap_or("");
    let kind_phrase = if is_known_kind(kind_str) {
        phrase(&format!("kind_{kind_str}"), None)
    } else {
        phrase("customRule", None)
    };
    let parts: Vec<Value> = values
        .into_iter()
        .map(|v| {
            if let Some(gk) = geo_value_key(kind_str, &v) {
                phrase(gk, None)
            } else if kind_str == "l4proto" {
                Value::String(v.to_uppercase())
            } else {
                Value::String(v)
            }
        })
        .collect();
    let base = phrase_kv(
        "condValue",
        &[
            ("kind", kind_phrase),
            ("values", json!({ "parts": parts, "join": "sep" })),
        ],
    );
    if js_str::is_truthy(&prop_val(&condition, "negate")) {
        Ok(phrase_kv("condNegated", &[("cond", base)]))
    } else {
        Ok(base)
    }
}

/// Port of `describeTarget` (`apps/lite/src/describe.ts:204`).
pub fn describe_target(target: String) -> Result<Value, String> {
    Ok(describe_target_val(&target))
}

/// Port of `describeRule` (`apps/lite/src/describe.ts:213`).
pub fn describe_rule(conditions_opt: Option<Vec<Value>>, target: String) -> Result<Value, String> {
    let parts = match conditions_opt {
        Some(list) if !list.is_empty() => list
            .into_iter()
            .map(describe_condition)
            .collect::<Result<Vec<_>, _>>()?,
        _ => vec![phrase("customRule", None)],
    };
    Ok(phrase_kv(
        "ruleArrow",
        &[
            ("conditions", json!({ "parts": parts, "join": "and" })),
            ("target", describe_target_val(&target)),
        ],
    ))
}

/// Port of `phraseText` (`apps/lite/src/describe.ts:223`).
pub fn phrase_text(value: Value, t: Value) -> Result<String, String> {
    phrase_text_ref(&value, &t)
}

fn phrase_text_ref(value: &Value, t: &Value) -> Result<String, String> {
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    if has_key(value, "regions")? {
        let reg_val = prop_val(value, "regions");
        check_obj(&reg_val, "map")?;
        let Some(arr) = reg_val.as_array() else {
            return Err("TypeError: value.regions.map is not a function".into());
        };
        let mut rendered = Vec::with_capacity(arr.len());
        for iso in arr {
            check_obj(t, "locale")?;
            let loc = prop_val(t, "locale");
            rendered.push(region_label(iso, &loc)?);
        }
        check_obj(t, "listSep")?;
        return Ok(js_array_join(&rendered, &prop_val(t, "listSep")));
    }
    if has_key(value, "items")? {
        let items_val = prop_val(value, "items");
        let Some(arr) = items_val.as_array() else {
            check_obj(&items_val, "map")?;
            return Err("TypeError: value.items.map is not a function".into());
        };
        let quote = js_str::is_truthy(&prop_val(value, "quote"));
        let items: Vec<String> = if quote {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                check_obj(t, "listQuote")?;
                let lq = prop_val(t, "listQuote");
                check_obj(&lq, "replace")?;
                let Some(lq_s) = lq.as_str() else {
                    return Err("TypeError: t.listQuote.replace is not a function".into());
                };
                out.push(js_replace_first(lq_s, "{v}", &js_str::js_to_string(item)));
            }
            out
        } else {
            arr.iter()
                .map(|x| {
                    if is_nullish(x) {
                        String::new()
                    } else {
                        js_str::js_to_string(x)
                    }
                })
                .collect()
        };
        check_obj(t, "listSep")?;
        let sep = prop_val(t, "listSep");
        if prop_val(value, "join").as_str() == Some("sep") || items.len() < 2 {
            return Ok(js_array_join(&items, &sep));
        }
        let head = js_array_join(&items[..items.len() - 1], &sep);
        let list_or = js_str::js_to_string(&prop_val(t, "listOr"));
        let tail = items.last().map(String::as_str).unwrap_or("");
        return Ok(format!("{head}{list_or}{tail}"));
    }
    if has_key(value, "parts")? {
        let parts_val = prop_val(value, "parts");
        check_obj(&parts_val, "map")?;
        let Some(arr) = parts_val.as_array() else {
            return Err("TypeError: value.parts.map is not a function".into());
        };
        let mut rendered = Vec::with_capacity(arr.len());
        for part in arr {
            rendered.push(phrase_text_ref(part, t)?);
        }
        let sep_key = if prop_val(value, "join").as_str() == Some("and") {
            "condAnd"
        } else {
            "listSep"
        };
        check_obj(t, sep_key)?;
        return Ok(js_array_join(&rendered, &prop_val(t, sep_key)));
    }
    let key_s = js_str::js_to_string(&prop_val(value, "key"));
    check_obj(t, &key_s)?;
    let tpl_val = prop_val(t, &key_s);
    check_obj(&tpl_val, "matchAll")?;
    let Some(tpl) = tpl_val.as_str() else {
        return Err("TypeError: template.matchAll is not a function".into());
    };
    let vals_v = prop_val(value, "values");
    let vals_truthy = js_str::is_truthy(&vals_v);

    let bytes = tpl.as_bytes();
    let mut out = String::with_capacity(tpl.len());
    let mut last = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > i + 1 && j < bytes.len() && bytes[j] == b'}' {
                let name = &tpl[i + 1..j];
                let full = &tpl[i..=j];
                let before = &tpl[last..i];
                let filled = if vals_truthy && has_key(&vals_v, name)? {
                    phrase_text_ref(&prop_val(&vals_v, name), t)?
                } else {
                    full.to_string()
                };
                let next_ch = first_u16_char(&tpl[j + 1..]);
                out.push_str(before);
                let prev_ch = last_u16_char(&out);
                if needs_gap(prev_ch, first_u16_char(&filled)) {
                    out.push(' ');
                }
                out.push_str(&filled);
                if needs_gap(last_u16_char(&filled), next_ch) {
                    out.push(' ');
                }
                last = j + 1;
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out.push_str(&tpl[last..]);
    Ok(out)
}

fn change_filters(change: &Value, field: &str) -> Result<Vec<String>, String> {
    let raw = prop_val(change, field);
    check_obj(&raw, "filter")?;
    let Some(arr) = raw.as_array() else {
        return Err(format!(
            "TypeError: change.{field}.filter is not a function"
        ));
    };
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        check_obj(item, "trim")?;
        let Some(s) = item.as_str() else {
            return Err("TypeError: filter.trim is not a function".into());
        };
        out.push(s.to_string());
    }
    Ok(out)
}

fn describe_members_parts(filters: &[String]) -> Result<Value, String> {
    let mut parts = Vec::with_capacity(filters.len());
    for f in filters {
        parts.push(describe_group_filter(f.clone())?.unwrap_or_else(|| phrase("descNone", None)));
    }
    Ok(json!({ "parts": parts, "join": "sep" }))
}

/// Port of `describeGroupChanges` (`apps/lite/src/describe.ts:246`).
pub fn describe_group_changes(changes: Value) -> Result<Vec<Value>, String> {
    check_obj(&changes, "flatMap")?;
    let Some(list) = changes.as_array() else {
        return Err("TypeError: changes.flatMap is not a function".into());
    };
    let mut out = Vec::new();
    for change in list {
        check_obj(change, "kind")?;
        let kind = prop_val(change, "kind");
        match kind.as_str() {
            Some("added") => {
                let raw_filters = change_filters(change, "filters")?;
                let kept: Vec<String> = raw_filters
                    .iter()
                    .filter(|f| js_str::js_trim(f) != EMPTY_GROUP_FILTER)
                    .cloned()
                    .collect();
                let name = prop_val(change, "name");
                let added = phrase_kv("reviewGroupAdded", &[("name", name.clone())]);
                out.push(added);
                if !(!raw_filters.is_empty() && kept.is_empty()) {
                    let members = if kept.is_empty() {
                        phrase("descAllNodes", None)
                    } else {
                        describe_members_parts(&kept)?
                    };
                    out.push(phrase_kv(
                        "reviewGroupMembersAdded",
                        &[("name", name), ("members", members)],
                    ));
                }
            }
            Some("removed") => {
                out.push(phrase_kv(
                    "reviewGroupRemoved",
                    &[("name", prop_val(change, "name"))],
                ));
            }
            Some("renamed") => {
                out.push(phrase_kv(
                    "reviewGroupRenamed",
                    &[
                        ("before", prop_val(change, "before")),
                        ("after", prop_val(change, "after")),
                    ],
                ));
            }
            Some("policy") => {
                let pol = prop_val(change, "policy");
                let pol_opt = if is_nullish(&pol) {
                    None
                } else {
                    Some(js_str::js_to_string(&pol))
                };
                out.push(phrase_kv(
                    "reviewGroupPolicy",
                    &[
                        ("name", prop_val(change, "name")),
                        ("policy", phrase(policy_key_str(pol_opt.as_deref()), None)),
                    ],
                ));
            }
            Some("members") => {
                let added: Vec<String> = change_filters(change, "added")?
                    .into_iter()
                    .filter(|f| js_str::js_trim(f) != EMPTY_GROUP_FILTER)
                    .collect();
                let removed: Vec<String> = change_filters(change, "removed")?
                    .into_iter()
                    .filter(|f| js_str::js_trim(f) != EMPTY_GROUP_FILTER)
                    .collect();
                let name = prop_val(change, "name");
                if !added.is_empty() && removed.is_empty() {
                    out.push(phrase_kv(
                        "reviewGroupMembersAdded",
                        &[("name", name), ("members", describe_members_parts(&added)?)],
                    ));
                } else if !removed.is_empty() && added.is_empty() {
                    out.push(phrase_kv(
                        "reviewGroupMembersRemoved",
                        &[
                            ("name", name),
                            ("members", describe_members_parts(&removed)?),
                        ],
                    ));
                } else if added.is_empty() && removed.is_empty() {
                    out.push(phrase_kv(
                        "reviewGroupMembersAdded",
                        &[("name", name), ("members", phrase("descAllNodes", None))],
                    ));
                } else {
                    out.push(phrase_kv(
                        "reviewGroupMembersAdded",
                        &[
                            ("name", name.clone()),
                            ("members", describe_members_parts(&added)?),
                        ],
                    ));
                    out.push(phrase_kv(
                        "reviewGroupMembersRemoved",
                        &[
                            ("name", name),
                            ("members", describe_members_parts(&removed)?),
                        ],
                    ));
                }
            }
            Some("changed") => {
                out.push(phrase_kv(
                    "reviewGroupChanged",
                    &[("name", prop_val(change, "name"))],
                ));
            }
            _ => out.push(phrase("reviewOther", None)),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en_messages() -> Value {
        json!({
            "locale": "en",
            "listSep": ", ",
            "listOr": " or ",
            "listQuote": "“{v}”",
            "condAnd": " and ",
            "condValue": "{kind}: {values}",
            "condNegated": "{cond} (match when not met)",
            "ruleArrow": "{conditions} → {target}",
            "mustTarget": "{target} (forced)",
            "customRule": "Custom rule",
            "kind_domainSuffix": "Domain and subdomains",
            "kind_l4proto": "Transport protocol",
            "kind_geosite": "Site category",
            "geo_site_cn": "Mainland China sites",
            "out_direct": "Direct",
            "out_block": "Block",
            "descSourceAll": "All nodes from {source}",
            "descSourceContains": "Nodes from {source} whose names contain {list}",
            "descSourceRegions": "Nodes from {source} in {regions}",
            "descAllNodesExcept": "All nodes except {names}",
            "descCustom": "Custom filter",
            "descNone": "No nodes"
        })
    }

    #[test]
    fn group_filters_and_bindings() {
        assert_eq!(
            describe_group_filter(EMPTY_GROUP_FILTER.into()).unwrap(),
            None
        );
        let p = describe_group_filter("subtag(harbor)".into())
            .unwrap()
            .unwrap();
        assert_eq!(p["key"], "descSourceAll");
        assert_eq!(
            phrase_text(p, en_messages()).unwrap(),
            "All nodes from harbor"
        );
        let ex = describe_group_filter("!name('direct', 'block')".into())
            .unwrap()
            .unwrap();
        assert_eq!(
            phrase_text(ex, en_messages()).unwrap(),
            "All nodes except Direct, Block"
        );
        let empty = describe_group_filters(vec![EMPTY_GROUP_FILTER.into()]).unwrap();
        assert_eq!(empty[0]["key"], "descNone");
    }

    #[test]
    fn region_binding_recognized() {
        let hk_pat = &patterns_of(
            "HK",
            &["hk", "hkg", "hongkong", "hong kong", "香港", "港"],
            true,
        )[0];
        let filter = format!("subtag(air) && name(regex: '{}')", hk_pat);
        let p = describe_group_filter(filter).unwrap().unwrap();
        assert_eq!(p["key"], "descSourceRegions");
        assert_eq!(p["values"]["regions"]["regions"], json!(["HK"]));
        assert_eq!(
            phrase_text(p, en_messages()).unwrap(),
            "Nodes from air in Hong Kong"
        );
    }

    #[test]
    fn policies_conditions_targets_and_rules() {
        assert_eq!(policy_key(None).unwrap(), json!("policyManual"));
        assert_eq!(
            policy_key(Some("min_moving_avg(10)".into())).unwrap(),
            json!("policyUrlTest")
        );
        assert_eq!(
            policy_key(Some("SCORE".into())).unwrap(),
            json!("policyScore")
        );
        let rule = describe_rule(
            Some(vec![
                json!({"kind": "geosite", "value": "cn", "negate": true}),
                json!({"kind": "l4proto", "value": "udp", "negate": false}),
            ]),
            "direct(must)".into(),
        )
        .unwrap();
        assert_eq!(
            phrase_text(rule, en_messages()).unwrap(),
            "Site category: Mainland China sites (match when not met) and Transport protocol: UDP → Direct (forced)"
        );
        assert!(describe_condition(json!(0)).is_err());
    }

    #[test]
    fn phrase_text_han_latin_spacing_and_edge_errors() {
        let zh = json!({
            "locale": "zh-CN",
            "listSep": "、",
            "listOr": "或",
            "listQuote": "“{v}”",
            "descSourceAll": "{source}的全部节点"
        });
        let p = describe_group_filter("subtag(harbor)".into())
            .unwrap()
            .unwrap();
        assert_eq!(phrase_text(p, zh).unwrap(), "harbor 的全部节点");
        assert_eq!(
            phrase_text(json!({"parts": ["a", "b"], "join": "and"}), json!("x")).unwrap(),
            "a,b"
        );
        assert!(phrase_text(Value::Null, json!({})).is_err());
        assert!(phrase_text(json!({"regions": ["US"]}), json!({})).is_err());
    }

    #[test]
    fn group_changes_summary_and_errors() {
        let res = describe_group_changes(json!([
            {"kind": "added", "name": "g1", "filters": [EMPTY_GROUP_FILTER]},
            {"kind": "policy", "name": "g2", "policy": "score"},
            {"kind": "members", "name": "g3", "added": [], "removed": []}
        ]))
        .unwrap();
        assert_eq!(res.len(), 3);
        assert_eq!(res[0]["key"], "reviewGroupAdded");
        assert_eq!(res[1]["values"]["policy"]["key"], "policyScore");
        assert_eq!(res[2]["values"]["members"]["key"], "descAllNodes");
        assert!(describe_group_changes(Value::Null).is_err());
        assert!(describe_group_changes(json!(true)).is_err());
    }
}
