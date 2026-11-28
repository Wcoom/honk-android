//! The Rust port of `apps/lite/src/searchIndex.ts`.

#![forbid(unsafe_code)]

use crate::js;
use dae_core::port::{conditions, js_str, regions, text};
use dae_core::utf16;
use serde_json::{Map, Value, json};
use std::collections::HashSet;

const ROUTING_ORD: &[&str] = &[
    "domainSuffix",
    "domain",
    "domainKeyword",
    "geosite",
    "dip",
    "geoip",
    "sip",
    "mac",
    "dport",
    "sport",
    "pname",
    "l4proto",
];
const DNS_REQ_ORD: &[&str] = &[
    "qnameSuffix",
    "qnameFull",
    "qnameKeyword",
    "qnameGeosite",
    "qtype",
    "sip",
];
const DNS_RESP_ORD: &[&str] = &[
    "qnameSuffix",
    "qnameFull",
    "qnameKeyword",
    "qnameGeosite",
    "qtype",
    "sip",
    "upstream",
    "answerIp",
    "answerGeoip",
];

// Compact region metadata matching `src/dae/regions.ts` (`iso`, `aliases`, `short`, `official`).
const REGION_DATA: &[(&str, &str, &str, &str)] = &[
    (
        "HK",
        "hk|hkg|hongkong|hong kong|香港|港",
        "Hong Kong|香港|香港",
        "Hong Kong SAR China|中國香港特別行政區|中国香港特别行政区",
    ),
    (
        "TW",
        "tw|tpe|taiwan|taipei|台灣|台湾|臺灣|台北",
        "Taiwan|台灣|台湾",
        "",
    ),
    (
        "JP",
        "jp|jpn|japan|tokyo|osaka|nrt|hnd|kix|日本|東京|东京|大阪",
        "",
        "",
    ),
    ("SG", "sg|sgp|singapore|sin|新加坡|獅城|狮城", "", ""),
    (
        "US",
        "us|usa|united states|america|lax|sjc|sea|nyc|los angeles|san jose|seattle|美國|美国|洛杉磯|洛杉矶|聖何塞|圣何塞|西雅圖|西雅图",
        "",
        "",
    ),
    (
        "KR",
        "kr|kor|korea|seoul|icn|south korea|南韩|南韓|韓國|韩国|首爾|首尔",
        "韓國",
        "",
    ),
    ("CA", "can|canada|toronto|yyz|加拿大", "", ""),
    (
        "GB",
        "gb|uk|united kingdom|britain|london|lhr|英國|英国|倫敦|伦敦",
        "",
        "",
    ),
    (
        "DE",
        "de|deu|germany|frankfurt|fra|德國|德国|法蘭克福|法兰克福",
        "",
        "",
    ),
    ("FR", "fr|france|paris|cdg|法國|法国|巴黎", "", ""),
    (
        "NL",
        "nl|nld|netherlands|amsterdam|ams|荷蘭|荷兰|阿姆斯特丹",
        "",
        "",
    ),
    ("RU", "ru|rus|russia|moscow|俄羅斯|俄罗斯|莫斯科", "", ""),
    (
        "AU",
        "au|aus|australia|sydney|syd|澳洲|澳大利亞|澳大利亚|悉尼",
        "",
        "",
    ),
    ("IN", "in|ind|india|mumbai|bom|印度|孟買|孟买", "", ""),
    ("TR", "tr|tur|turkey|türkiye|istanbul|土耳其", "", ""),
    ("BR", "br|bra|brazil|sao paulo|gru|巴西", "", ""),
    ("AR", "ar|arg|argentina|阿根廷", "", ""),
    ("MY", "my|mys|malaysia|kul|馬來西亞|马来西亚", "", ""),
    ("TH", "th|tha|thailand|bangkok|bkk|泰國|泰国", "", ""),
    ("VN", "vn|vnm|vietnam|越南", "", ""),
    ("PH", "ph|phl|philippines|manila|菲律賓|菲律宾", "", ""),
    (
        "ID",
        "id|idn|indonesia|jakarta|印度尼西亚|印度尼西亞|印尼",
        "",
        "",
    ),
    (
        "AE",
        "ae|are|uae|united arab emirates|阿联酋|阿聯酋|dubai|dxb|阿聯|阿联|杜拜|迪拜",
        "",
        "",
    ),
    ("CH", "ch|che|switzerland|zurich|瑞士", "", ""),
    ("SE", "se|swe|sweden|stockholm|瑞典", "", ""),
    ("FI", "fi|fin|finland|helsinki|芬蘭|芬兰", "", ""),
    ("IT", "it|ita|italy|milan|義大利|意大利", "", ""),
    ("ES", "es|esp|spain|madrid|西班牙", "", ""),
    ("PL", "pl|pol|poland|warsaw|波蘭|波兰", "", ""),
    (
        "MO",
        "mo|mac|macau|macao|澳門|澳门",
        "Macao|澳門|澳门",
        "Macao SAR China|中國澳門特別行政區|中国澳门特别行政区",
    ),
    ("NZ", "new zealand|新西兰|紐西蘭", "", ""),
    ("MX", "mexico|墨西哥", "", ""),
    ("NO", "norway|挪威", "", ""),
    ("DK", "denmark|丹麦|丹麥", "", ""),
    ("IE", "ireland|爱尔兰|愛爾蘭", "", ""),
    ("ZA", "south africa|南非", "", ""),
    ("IL", "israel|以色列", "", ""),
    ("UA", "ukraine|乌克兰|烏克蘭", "", ""),
    ("PT", "portugal|葡萄牙", "", ""),
    (
        "CN",
        "cn|chn|china|shanghai|beijing|shenzhen|中國|中国|上海|北京|深圳",
        "",
        "",
    ),
    (
        "KZ",
        "kazakhstan|almaty|哈萨克斯坦|哈薩克斯坦|哈萨克|哈薩克",
        "",
        "",
    ),
    ("AT", "austria|vienna|奥地利|奧地利", "", ""),
    ("BE", "belgium|brussels|比利时|比利時", "", ""),
    ("LU", "luxembourg|卢森堡|盧森堡", "", ""),
    ("CZ", "czech|czechia|prague|捷克", "", ""),
    ("RO", "romania|bucharest|罗马尼亚|羅馬尼亞", "", ""),
    ("IS", "iceland|reykjavik|冰岛|冰島", "", ""),
    ("CL", "chile|santiago|智利", "", ""),
    ("CO", "colombia|bogota|哥伦比亚|哥倫比亞", "", ""),
    ("SA", "saudi|riyadh|沙特|沙烏地", "", ""),
    ("PK", "pakistan|karachi|巴基斯坦", "", ""),
    ("NG", "nigeria|lagos|尼日利亚|尼日利亞|奈及利亞", "", ""),
    ("EG", "egypt|cairo|埃及", "", ""),
    ("KH", "cambodia|柬埔寨", "", ""),
];

const FLAG_REGIONS: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

#[cfg(target_arch = "wasm32")]
fn display_names(iso: &str) -> Vec<String> {
    use js_sys::{Array, Function, Object, Reflect};
    use std::cell::RefCell;
    use wasm_bindgen::{JsCast, JsValue};

    thread_local! {
        static INSTANCES: RefCell<Option<Vec<(JsValue, Function)>>> = const { RefCell::new(None) };
    }
    INSTANCES.with(|cell| {
        if cell.borrow().is_none() {
            let mut list = Vec::new();
            let global = js_sys::global();
            if let Ok(intl) = Reflect::get(&global, &JsValue::from_str("Intl"))
                && let Ok(ctor_v) = Reflect::get(&intl, &JsValue::from_str("DisplayNames"))
                && let Ok(ctor) = ctor_v.dyn_into::<Function>()
            {
                let opts = Object::new();
                let _ = Reflect::set(
                    &opts,
                    &JsValue::from_str("type"),
                    &JsValue::from_str("region"),
                );
                for loc in ["en", "zh-CN", "zh-TW"] {
                    let args = Array::of2(&Array::of1(&JsValue::from_str(loc)), &opts);
                    if let Ok(inst) = Reflect::construct(&ctor, &args)
                        && let Ok(of_v) = Reflect::get(&inst, &JsValue::from_str("of"))
                        && let Ok(of_fn) = of_v.dyn_into::<Function>()
                    {
                        list.push((inst, of_fn));
                    }
                }
            }
            *cell.borrow_mut() = Some(list);
        }
        let borrowed = cell.borrow();
        borrowed
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter_map(|(inst, f)| f.call1(inst, &JsValue::from_str(iso)).ok()?.as_string())
            .collect()
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn display_names(_iso: &str) -> Vec<String> {
    Vec::new()
}

fn region_terms(name: &str) -> Vec<String> {
    let iso = match regions::match_region(name.to_string()) {
        Ok(Value::String(s)) => s,
        _ => return Vec::new(),
    };
    let entry = REGION_DATA.iter().find(|&&(c, _, _, _)| c == iso);
    if entry.is_none() && !FLAG_REGIONS.split(' ').any(|c| c == iso) {
        return Vec::new();
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |s: String| {
        if !s.is_empty() && seen.insert(s.clone()) {
            out.push(s);
        }
    };
    push(iso.clone());
    if let Some(&(_, aliases, short, official)) = entry {
        for part in [aliases, short, official] {
            for item in part.split('|') {
                push(item.to_string());
            }
        }
    }
    for name in display_names(&iso) {
        push(name);
    }
    out
}

fn find_blocks<'a>(blocks: &'a [text::TextBlock], path: &[&str]) -> Vec<&'a text::TextBlock> {
    let mut cur: Vec<&text::TextBlock> = blocks.iter().collect();
    for (idx, &name) in path.iter().enumerate() {
        let found: Vec<&text::TextBlock> = cur.into_iter().filter(|b| b.name == name).collect();
        if idx + 1 == path.len() {
            return found;
        }
        cur = found.into_iter().flat_map(|b| b.children.iter()).collect();
    }
    Vec::new()
}

fn extra_spec(kind: &str) -> Option<(&'static str, &'static str)> {
    Some(match kind {
        "domainRegex" => ("domain", "regex"),
        "ipversion" => ("ipversion", ""),
        "dscp" => ("dscp", ""),
        "qnameRegex" => ("qname", "regex"),
        _ => return None,
    })
}

fn split_ws_and(s: &str) -> Vec<&str> {
    let (mut parts, mut start, mut i) = (Vec::new(), 0, 0);
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap_or('\0');
        if js_str::is_js_ws(ch) {
            let mut j = i;
            while j < s.len() && s[j..].chars().next().is_some_and(js_str::is_js_ws) {
                j += s[j..].chars().next().map_or(1, char::len_utf8);
            }
            if s[j..].starts_with("&&") {
                let mut k = j + 2;
                while k < s.len() && s[k..].chars().next().is_some_and(js_str::is_js_ws) {
                    k += s[k..].chars().next().map_or(1, char::len_utf8);
                }
                if k > j + 2 {
                    parts.push(&s[start..i]);
                    start = k;
                    i = k;
                    continue;
                }
            }
            i = j;
        } else {
            i += ch.len_utf8();
        }
    }
    parts.push(&s[start..]);
    parts
}

fn serialize_builder(rows: &[Value]) -> bool {
    !rows.is_empty()
        && rows.iter().all(|row| {
            let kind = row.get("kind").and_then(Value::as_str).unwrap_or("");
            let Some((name, prefix)) = extra_spec(kind) else {
                return conditions::serialize_conditions(Value::Array(vec![row.clone()]))
                    .ok()
                    .flatten()
                    .is_some();
            };
            let Some(vals) = conditions::parse_condition_values_vec(
                row.get("value").and_then(Value::as_str).unwrap_or(""),
            ) else {
                return false;
            };
            if vals.is_empty() || vals.iter().any(|v| !text::is_quotable_bool(v)) {
                return false;
            }
            let neg = if row.get("negate").and_then(Value::as_bool).unwrap_or(false) {
                "!"
            } else {
                ""
            };
            let args: Vec<String> = vals
                .iter()
                .filter_map(|v| {
                    let q = text::quote_str(v).ok()?;
                    Some(if prefix.is_empty() {
                        q
                    } else {
                        format!("{prefix}: {q}")
                    })
                })
                .collect();
            args.len() == vals.len()
                && text::is_fragment_bool(&format!("{neg}{name}({})", args.join(", ")))
        })
}

fn parse_builder(expr: &str, kind: &str) -> Option<Vec<Value>> {
    let (ord, extras): (&[&str], &[&str]) = match kind {
        "routing" => (ROUTING_ORD, &["domainRegex", "ipversion", "dscp"]),
        "request" => (DNS_REQ_ORD, &["qnameRegex"]),
        _ => (DNS_RESP_ORD, &["qnameRegex"]),
    };
    let ord_vec: Vec<String> = ord.iter().map(|s| (*s).to_string()).collect();
    if let Ok(Some(parsed)) = conditions::parse_conditions(expr.to_string(), ord_vec.clone()) {
        return Some(parsed);
    }
    let mut rows = Vec::new();
    for part in split_ws_and(expr) {
        if let Ok(Some(p)) = conditions::parse_conditions(part.to_string(), ord_vec.clone()) {
            rows.extend(p);
            continue;
        }
        let trimmed = js_str::js_trim(part);
        let (neg, rest) = match trimmed.strip_prefix('!') {
            Some(r) => (true, js_str::js_trim_start(r)),
            None => (false, trimmed),
        };
        let fn_len = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        let fn_name = &rest[..fn_len];
        let inner = rest[fn_len..]
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))?;
        if fn_name.is_empty()
            || inner
                .chars()
                .any(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
        {
            return None;
        }
        let tokens = text::scan_config_raw(inner).tokens;
        let units = utf16::units(inner);
        let raw = |n: usize| utf16::slice(&units, tokens[n].from, tokens[n].to);
        let (mut vals, mut prefix, mut i) = (Vec::new(), String::new(), 0);
        while i < tokens.len() {
            let mut p = String::new();
            if i + 1 < tokens.len() && raw(i + 1) == ":" {
                p = raw(i);
                i += 2;
            }
            if i >= tokens.len()
                || tokens[i].kind != text::TokenKind::Quoted
                || (!vals.is_empty() && p != prefix)
            {
                return None;
            }
            prefix = p;
            vals.push(text::unquote_str(&raw(i)));
            i += 1;
            if i < tokens.len() {
                if raw(i) != "," {
                    return None;
                }
                i += 1;
            }
        }
        let key = extras
            .iter()
            .copied()
            .find(|&k| extra_spec(k) == Some((fn_name, prefix.as_str())))?;
        if vals.is_empty() {
            return None;
        }
        rows.push(
            json!({ "kind": key, "value": conditions::condition_values_str(&vals), "negate": neg }),
        );
    }
    serialize_builder(&rows).then_some(rows)
}

struct ParsedRule {
    from: usize,
    anchor_text: String,
    note: String,
    expression: String,
    target: String,
    fallback: bool,
    conditions: Option<Vec<Value>>,
}

fn parse_rule_list(content: &str, kind: &str) -> Vec<ParsedRule> {
    let scan = text::scan_config_raw(content);
    let path: &[&str] = match kind {
        "routing" => &["routing"],
        _ => &["dns", "routing", kind],
    };
    let blocks = find_blocks(&scan.blocks, path);
    let [block] = blocks.as_slice() else {
        return Vec::new();
    };
    let tokens: Vec<&text::TextToken> = scan
        .tokens
        .iter()
        .filter(|t| {
            t.from > block.open && t.to <= block.close && t.kind != text::TokenKind::Comment
        })
        .collect();
    let units = utf16::units(content);
    let mut cursor = block.open + 1;
    let mut p = cursor;
    while p < units.len() && matches!(units[p], 0x20 | 0x09) {
        p += 1;
    }
    if p < units.len() && units[p] == b'\r' as u16 {
        p += 1;
    }
    if p < units.len() && units[p] == b'\n' as u16 {
        cursor = p + 1;
    }
    let mut rows = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let first = tokens[i];
        i += 1;
        let mut last = first;
        while i < tokens.len() && (tokens[i].line == last.line || tokens[i].parens > 0) {
            last = tokens[i];
            i += 1;
        }
        let to = match js_str::index_of_u16(&units, b'\n' as u16, last.to) {
            Some(nl) if nl < block.close => nl + 1,
            _ => last.to,
        };
        let actual: Vec<&text::TextToken> = tokens
            .iter()
            .copied()
            .filter(|t| t.from >= first.from && t.to <= last.to)
            .collect();
        let tok = |n: usize| {
            actual
                .get(n)
                .map(|t| utf16::slice(&units, t.from, t.to))
                .unwrap_or_default()
        };
        let fallback = matches!(tok(0).as_str(), "fallback" | "default") && tok(1) == ":";
        let arrow_idx = if fallback {
            (actual.len() > 1).then_some(1)
        } else {
            actual
                .iter()
                .position(|t| t.parens == 0 && utf16::slice(&units, t.from, t.to) == "->")
        };
        let before = arrow_idx.and_then(|idx| idx.checked_sub(1).and_then(|b| actual.get(b)));
        let expression = match (arrow_idx, before) {
            (Some(_), Some(b)) => utf16::slice(&units, first.from, b.to),
            _ => utf16::slice(&units, first.from, last.to),
        };
        let target = arrow_idx
            .map(|idx| js_str::js_trim(&utf16::slice(&units, actual[idx].to, last.to)).to_string())
            .unwrap_or_default();
        let head = utf16::slice(&units, cursor, first.from);
        let note = head
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .filter_map(|l| {
                let s = js_str::js_trim_start(l).strip_prefix('#')?;
                Some(match s.chars().next() {
                    Some(c) if js_str::is_js_ws(c) => &s[c.len_utf8()..],
                    _ => s,
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
        let conditions = if !fallback && arrow_idx.is_some() {
            parse_builder(&expression, kind)
        } else {
            None
        };
        rows.push(ParsedRule {
            from: cursor,
            anchor_text: utf16::slice(&units, cursor, to),
            note,
            expression,
            target,
            fallback,
            conditions,
        });
        cursor = to;
    }
    rows
}

// ---- JS value helpers --------------------------------------------------------------------------

fn check_obj(v: &Value, prop: &str) -> Result<(), String> {
    if v.is_null() || js::is_undefined(v) {
        let kind = if v.is_null() { "null" } else { "undefined" };
        return Err(format!(
            "TypeError: Cannot read properties of {kind} (reading '{prop}')"
        ));
    }
    Ok(())
}

fn prop(v: &Value, key: &str) -> Value {
    v.as_object()
        .filter(|_| !js::is_undefined(v))
        .and_then(|m| m.get(key).cloned())
        .unwrap_or_else(js::undefined)
}

fn require_arr<'a>(v: &'a Value, name: &str, method: &str) -> Result<&'a [Value], String> {
    check_obj(v, method)?;
    v.as_array().map(Vec::as_slice).ok_or_else(|| {
        format!("TypeError: {name}.{method} is not a function or its return value is not iterable")
    })
}

fn join_truthy(parts: &[Value]) -> String {
    parts
        .iter()
        .filter(|v| js_str::is_truthy(v))
        .map(js_str::js_to_string)
        .collect::<Vec<_>>()
        .join(" / ")
}

// ---- Exported ports ----------------------------------------------------------------------------

/// Port of `resourceIndex` (`apps/lite/src/searchIndex.ts:31`).
pub fn resource_index(
    nodes: Value,
    groups: Value,
    providers: Value,
    sources: Value,
    manual_label: String,
) -> Result<Vec<Value>, String> {
    let group_arr = require_arr(&groups, "groups", "map")?;
    let mut result = Vec::new();
    for g in group_arr {
        check_obj(g, "id")?;
        let policy = prop(g, "policy");
        check_obj(&policy, "native")?;
        let native = prop(&policy, "native");
        let desc = if js_str::is_truthy(&native) {
            native
        } else {
            prop(&policy, "kind")
        };
        let id = prop(g, "id");
        let name = prop(g, "name");
        let kw = name.as_str().map(region_terms).unwrap_or_default();
        result.push(json!({
            "id": format!("group:{}", js_str::js_to_string(&id)),
            "label": name,
            "description": desc,
            "category": "groups",
            "keywords": kw,
            "target": { "kind": "group", "id": id }
        }));
    }

    let node_arr = require_arr(&nodes, "nodes", "map")?;
    for n in node_arr {
        check_obj(n, "provider_id")?;
        let prov_arr = require_arr(&providers, "providers", "find")?;
        let pid = prop(n, "provider_id");
        let mut matched: Option<&Value> = None;
        for p in prov_arr {
            check_obj(p, "id")?;
            if prop(p, "id") == pid {
                matched = Some(p);
                break;
            }
        }
        let id = prop(n, "id");
        let name = prop(n, "name");
        let label = if js_str::is_truthy(&name) {
            name
        } else {
            id.clone()
        };
        let owner = match matched {
            Some(p) if prop(p, "kind").as_str() == Some("inline") => {
                Value::String(manual_label.clone())
            }
            Some(p) => prop(p, "name"),
            None => js::undefined(),
        };
        let kw = label.as_str().map(region_terms).unwrap_or_default();
        result.push(json!({
            "id": format!("node:{}", js_str::js_to_string(&id)),
            "label": label,
            "description": join_truthy(&[prop(n, "protocol"), owner]),
            "category": "nodes",
            "keywords": kw,
            "target": { "kind": "node", "id": id }
        }));
    }

    let prov_arr = require_arr(&providers, "providers", "filter")?;
    for p in prov_arr {
        check_obj(p, "kind")?;
        if prop(p, "kind").as_str() == Some("subscription") {
            let id = prop(p, "id");
            result.push(json!({
                "id": format!("subscription:{}", js_str::js_to_string(&id)),
                "label": prop(p, "name"),
                "description": js_str::js_to_string(&prop(p, "node_count")),
                "category": "subscriptions",
                "target": { "kind": "subscription", "id": id }
            }));
        }
    }

    let src_arr = require_arr(&sources, "sources", "Symbol.iterator")?;
    for src in src_arr {
        check_obj(src, "content")?;
        let content = prop(src, "content");
        let Some(content_s) = content.as_str() else {
            return Err("TypeError: source.content must be a string".into());
        };
        let sid = js_str::js_to_string(&prop(src, "id"));
        let is_main = prop(src, "kind").as_str() == Some("main");
        let path_val = if is_main {
            Value::String(String::new())
        } else {
            prop(src, "path")
        };
        for kind in ["routing", "request", "response"] {
            for row in parse_rule_list(content_s, kind) {
                let mut obj = Map::new();
                obj.insert(
                    "id".into(),
                    Value::String(format!("{sid}:{kind}:{}", row.from)),
                );
                obj.insert(
                    "label".into(),
                    Value::String(if row.fallback {
                        format!("fallback: {}", row.target)
                    } else {
                        format!("{} → {}", row.expression, row.target)
                    }),
                );
                obj.insert(
                    "description".into(),
                    Value::String(join_truthy(&[Value::String(row.note), path_val.clone()])),
                );
                obj.insert(
                    "preview".into(),
                    if is_main {
                        js::undefined()
                    } else {
                        Value::String(row.anchor_text.clone())
                    },
                );
                obj.insert("keywords".into(), json!([row.expression, row.target]));
                obj.insert(
                    "rule".into(),
                    json!({
                        "conditions": row.conditions,
                        "target": row.target,
                        "fallback": row.fallback,
                        "kind": kind
                    }),
                );
                obj.insert(
                    "category".into(),
                    Value::String(if kind == "routing" { "routing" } else { "dns" }.into()),
                );
                obj.insert(
                    "target".into(),
                    json!({ "kind": "rule", "ruleKind": kind, "id": row.anchor_text }),
                );
                result.push(Value::Object(obj));
            }
        }
        let scan = text::scan_config_raw(content_s);
        for block in find_blocks(&scan.blocks, &["dns", "upstream"]) {
            for field in text::block_fields_raw(content_s, block, &scan.tokens) {
                let mut obj = Map::new();
                obj.insert(
                    "id".into(),
                    Value::String(format!("{sid}:upstream:{}", field.name)),
                );
                obj.insert("label".into(), Value::String(field.name.clone()));
                obj.insert(
                    "description".into(),
                    Value::String(text::unquote_str(&field.value)),
                );
                obj.insert(
                    "preview".into(),
                    if is_main {
                        js::undefined()
                    } else {
                        Value::String(format!("{}: {}", field.name, field.value))
                    },
                );
                obj.insert("category".into(), Value::String("dns".into()));
                obj.insert(
                    "target".into(),
                    json!({ "kind": "upstream", "id": field.name }),
                );
                result.push(Value::Object(obj));
            }
        }
    }
    Ok(result)
}

/// Port of `searchNameRank` (`apps/lite/src/searchIndex.ts:91`).
pub fn search_name_rank(label: String, query: String) -> Result<f64, String> {
    let name = label.to_lowercase();
    let text = js_str::js_trim(&query).to_lowercase();
    Ok(if text.is_empty() || name == text {
        0.0
    } else if name.starts_with(&text) {
        1.0
    } else if name.contains(&text) {
        2.0
    } else {
        3.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_name_rank_tiers_and_unicode() {
        assert_eq!(
            search_name_rank("harbor".into(), " Harbor ".into()).unwrap(),
            0.0
        );
        assert_eq!(search_name_rank("任意節點".into(), "".into()).unwrap(), 0.0);
        assert_eq!(
            search_name_rank("harbor-backup".into(), " Harbor ".into()).unwrap(),
            1.0
        );
        assert_eq!(
            search_name_rank("My harbor".into(), " Harbor ".into()).unwrap(),
            2.0
        );
        assert_eq!(
            search_name_rank("香港 01".into(), " Harbor ".into()).unwrap(),
            3.0
        );
        assert_eq!(
            search_name_rank("E\u{0301}".into(), "e\u{0301}".into()).unwrap(),
            0.0
        );
    }

    #[test]
    fn resource_index_groups_nodes_providers_rules_and_upstreams() {
        let nodes = json!([
            {"id": "n1", "name": "hk-01", "protocol": "vless", "provider_id": "p1"}
        ]);
        let groups = json!([
            {"id": "g1", "name": "Japan", "policy": {"native": "score", "kind": "score"}}
        ]);
        let providers = json!([
            {"id": "p1", "name": "Inline", "kind": "inline", "node_count": 1},
            {"id": "p2", "name": "Airport", "kind": "subscription", "node_count": 12}
        ]);
        let sources = json!([
            {
                "id": "extra",
                "kind": "include",
                "path": "extra.dae",
                "content": "routing {\n  # Note line\n  domain(suffix: example.com) -> direct\n  fallback: proxy\n}\ndns {\n  upstream {\n    ali: 'udp://223.5.5.5'\n  }\n}\n"
            }
        ]);
        let items = resource_index(nodes, groups, providers, sources, "Manual".into()).unwrap();
        assert_eq!(items.len(), 6);
        assert_eq!(items[0]["id"], "group:g1");
        assert!(
            items[0]["keywords"]
                .as_array()
                .unwrap()
                .contains(&json!("日本"))
        );
        assert_eq!(items[1]["id"], "node:n1");
        assert_eq!(items[1]["description"], "vless / Manual");
        assert_eq!(items[2]["id"], "subscription:p2");
        assert_eq!(items[2]["description"], "12");
        assert_eq!(items[3]["category"], "routing");
        assert_eq!(items[3]["description"], "Note line / extra.dae");
        assert_eq!(items[4]["label"], "fallback: proxy");
        assert_eq!(items[5]["id"], "extra:upstream:ali");
        assert_eq!(items[5]["description"], "udp://223.5.5.5");
    }

    #[test]
    fn resource_index_rejects_invalid_inputs() {
        assert!(
            resource_index(json!("x"), json!(0), Value::Null, Value::Null, "M".into()).is_err()
        );
        assert!(resource_index(json!([]), json!(["a"]), json!([]), json!([]), "M".into()).is_err());
        assert!(resource_index(json!(["a"]), json!([]), json!({}), json!([]), "M".into()).is_err());
    }
}
