//! The Rust port of `apps/lite/src/groupBindings.ts`.

#![forbid(unsafe_code)]

use dae_core::port::conditions::parse_condition_values_vec;
use dae_core::port::js_str::{ensure_not_nullish, is_truthy, js_to_string, js_trim, obj_defined};
use dae_core::port::regions::region_flag_str;
use dae_core::port::text::quote_str;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::OnceLock;

const ALIAS_REGIONS: &str = "\
HK:hk,hkg,hongkong,hong kong,香港,港;\
TW:tw,tpe,taiwan,taipei,台灣,台湾,臺灣,台北;\
JP:jp,jpn,japan,tokyo,osaka,nrt,hnd,kix,日本,東京,东京,大阪;\
SG:sg,sgp,singapore,sin,新加坡,獅城,狮城;\
US:us,usa,united states,america,lax,sjc,sea,nyc,los angeles,san jose,seattle,美國,美国,洛杉磯,洛杉矶,聖何塞,圣何塞,西雅圖,西雅图;\
KR:kr,kor,korea,seoul,icn,south korea,南韩,南韓,韓國,韩国,首爾,首尔;\
CA:can,canada,toronto,yyz,加拿大;\
GB:gb,uk,united kingdom,britain,london,lhr,英國,英国,倫敦,伦敦;\
DE:de,deu,germany,frankfurt,fra,德國,德国,法蘭克福,法兰克福;\
FR:fr,france,paris,cdg,法國,法国,巴黎;\
NL:nl,nld,netherlands,amsterdam,ams,荷蘭,荷兰,阿姆斯特丹;\
RU:ru,rus,russia,moscow,俄羅斯,俄罗斯,莫斯科;\
AU:au,aus,australia,sydney,syd,澳洲,澳大利亞,澳大利亚,悉尼;\
IN:in,ind,india,mumbai,bom,印度,孟買,孟买;\
TR:tr,tur,turkey,türkiye,istanbul,土耳其;\
BR:br,bra,brazil,sao paulo,gru,巴西;\
AR:ar,arg,argentina,阿根廷;\
MY:my,mys,malaysia,kul,馬來西亞,马来西亚;\
TH:th,tha,thailand,bangkok,bkk,泰國,泰国;\
VN:vn,vnm,vietnam,越南;\
PH:ph,phl,philippines,manila,菲律賓,菲律宾;\
ID:id,idn,indonesia,jakarta,印度尼西亚,印度尼西亞,印尼;\
AE:ae,are,uae,united arab emirates,阿联酋,阿聯酋,dubai,dxb,阿聯,阿联,杜拜,迪拜;\
CH:ch,che,switzerland,zurich,瑞士;\
SE:se,swe,sweden,stockholm,瑞典;\
FI:fi,fin,finland,helsinki,芬蘭,芬兰;\
IT:it,ita,italy,milan,義大利,意大利;\
ES:es,esp,spain,madrid,西班牙;\
PL:pl,pol,poland,warsaw,波蘭,波兰;\
MO:mo,mac,macau,macao,澳門,澳门;\
NZ:new zealand,新西兰,紐西蘭;MX:mexico,墨西哥;NO:norway,挪威;DK:denmark,丹麦,丹麥;\
IE:ireland,爱尔兰,愛爾蘭;ZA:south africa,南非;IL:israel,以色列;UA:ukraine,乌克兰,烏克蘭;\
PT:portugal,葡萄牙;CN:cn,chn,china,shanghai,beijing,shenzhen,中國,中国,上海,北京,深圳;\
KZ:kazakhstan,almaty,哈萨克斯坦,哈薩克斯坦,哈萨克,哈薩克;AT:austria,vienna,奥地利,奧地利;\
BE:belgium,brussels,比利时,比利時;LU:luxembourg,卢森堡,盧森堡;CZ:czech,czechia,prague,捷克;\
RO:romania,bucharest,罗马尼亚,羅馬尼亞;IS:iceland,reykjavik,冰岛,冰島;CL:chile,santiago,智利;\
CO:colombia,bogota,哥伦比亚,哥倫比亞;SA:saudi,riyadh,沙特,沙烏地;PK:pakistan,karachi,巴基斯坦;\
NG:nigeria,lagos,尼日利亚,尼日利亞,奈及利亞;EG:egypt,cairo,埃及;KH:cambodia,柬埔寨";

const FLAG_REGIONS: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

const CAPITAL_ONLY: &[&str] = &[
    "IN", "IT", "MY", "NO", "ID", "KZ", "AT", "BE", "LU", "CZ", "RO", "IS", "CL", "CO", "SA", "PK",
    "NG", "EG", "KH",
];

fn all_regions() -> &'static Vec<(&'static str, Vec<&'static str>)> {
    static ALL: OnceLock<Vec<(&'static str, Vec<&'static str>)>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut list: Vec<(&'static str, Vec<&'static str>)> = ALIAS_REGIONS
            .split(';')
            .filter_map(|item| item.split_once(':'))
            .map(|(iso, rest)| (iso, rest.split(',').collect()))
            .collect();
        let known: HashSet<&str> = list.iter().map(|(iso, _)| *iso).collect();
        for code in FLAG_REGIONS.split(' ') {
            if !known.contains(code) {
                list.push((code, Vec::new()));
            }
        }
        list
    })
}

fn regex_literals_opt(pattern: &str) -> Option<Vec<String>> {
    let body = pattern.strip_prefix("(?i)").unwrap_or(pattern);
    let parts: Vec<String> = body.split('|').map(str::to_string).collect();
    let ok = !parts.is_empty()
        && parts.iter().all(|part| {
            !part.is_empty()
                && !part.bytes().any(|b| {
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
    ok.then_some(parts)
}

/// Port of `regexLiterals` (`apps/lite/src/groupBindings.ts:11`).
pub fn regex_literals(pattern: String) -> Result<Option<Vec<String>>, String> {
    Ok(regex_literals_opt(&pattern))
}

fn js_strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(Value::Null), Some(Value::Null)) => true,
        (Some(Value::Bool(x)), Some(Value::Bool(y))) => x == y,
        (Some(Value::String(x)), Some(Value::String(y))) => x == y,
        (Some(x), Some(y)) => {
            matches!((crate::js::as_number(x), crate::js::as_number(y)), (Some(nx), Some(ny)) if nx == ny)
        }
        _ => false,
    }
}

fn quote_val(v: Option<&Value>) -> Result<String, String> {
    match v {
        None => Err("TypeError: Cannot read properties of undefined (reading 'includes')".into()),
        Some(Value::Null) => {
            Err("TypeError: Cannot read properties of null (reading 'includes')".into())
        }
        Some(Value::String(s)) => quote_str(s),
        Some(_) => Err("TypeError: value.includes is not a function".into()),
    }
}

/// Port of `nodeBinding` (`apps/lite/src/groupBindings.ts:17`).
pub fn node_binding(node: Value, providers: Value, manual: String) -> Result<Value, String> {
    ensure_not_nullish(&providers, "find")?;
    let Some(prov_arr) = providers.as_array() else {
        return Err("TypeError: providers.find is not a function".into());
    };
    let mut provider: Option<&Value> = None;
    for item in prov_arr {
        let item_obj = ensure_not_nullish(item, "id")?;
        let node_obj = ensure_not_nullish(&node, "provider_id")?;
        if js_strict_eq(
            item_obj.and_then(|o| obj_defined(o, "id")),
            node_obj.and_then(|o| obj_defined(o, "provider_id")),
        ) {
            provider = Some(item);
            break;
        }
    }
    let node_obj = ensure_not_nullish(&node, "subscription_tag")?;
    let node_prop = |k: &str| node_obj.and_then(|o| obj_defined(o, k));
    let sub_from_prov = provider
        .and_then(Value::as_object)
        .filter(|o| obj_defined(o, "kind").and_then(Value::as_str) == Some("subscription"))
        .map(|o| obj_defined(o, "name"));
    let subscription = match node_prop("subscription_tag").filter(|v| is_truthy(v)) {
        Some(v) => Some(v),
        None => sub_from_prov.unwrap_or(Some(&Value::Null)),
    };
    let id_val = node_prop("id");
    let name_val = node_prop("name");
    let id = format!(
        "node:{}",
        id_val.map_or_else(|| "undefined".to_string(), js_to_string)
    );
    let label = name_val
        .filter(|v| is_truthy(v))
        .or(id_val)
        .cloned()
        .unwrap_or_else(crate::js::undefined);
    let mut desc_parts = Vec::new();
    if let Some(p) = node_prop("protocol").filter(|v| is_truthy(v)) {
        desc_parts.push(js_to_string(p));
    }
    if let Some(s) = subscription.filter(|v| is_truthy(v)) {
        desc_parts.push(js_to_string(s));
    } else if !manual.is_empty() {
        desc_parts.push(manual);
    }
    let prefix = if subscription.is_some_and(is_truthy) {
        format!("subtag({}) && ", quote_val(subscription)?)
    } else {
        "!subtag(regex: '.*') && ".to_string()
    };
    let filter = format!("{prefix}name({})", quote_val(name_val)?);
    Ok(json!({
        "id": id,
        "label": label,
        "description": desc_parts.join(" / "),
        "filter": filter,
    }))
}

fn valid_rust_regex(regex: &str) -> bool {
    if regex.bytes().rev().take_while(|&b| b == b'\\').count() % 2 == 1 {
        return false;
    }
    let filter = if !regex.contains('\'') {
        format!("name('x', regex: '{regex}')")
    } else if !regex.contains('"') {
        format!("name('x', regex: \"{regex}\")")
    } else {
        let safe = regex.replace('\'', "%");
        format!("name('x', regex: '{safe}')")
    };
    matches!(
        dae_core::port::group_admits(vec![filter], json!({"name": "x"})),
        Ok(Value::Bool(true))
    )
}

/// Port of `subscriptionFilter` (`apps/lite/src/groupBindings.ts:27`).
pub fn subscription_filter(name: String, regex: String) -> Result<Value, String> {
    if !regex.is_empty() && !valid_rust_regex(&regex) {
        return Ok(Value::Null);
    }
    let sub = format!("subtag({})", quote_str(&name)?);
    if regex.is_empty() {
        Ok(Value::String(sub))
    } else {
        Ok(Value::String(format!(
            "{sub} && name(regex: {})",
            quote_str(&regex)?
        )))
    }
}

/// Port of `honkPolicy` (`apps/lite/src/groupBindings.ts:31`).
pub fn honk_policy(value: String) -> Result<String, String> {
    let base = js_trim(value.split('(').next().unwrap_or("")).to_lowercase();
    let policy = match base.as_str() {
        "min_moving_avg" | "min_avg10" | "min_last_delay" | "urltest" => "urltest",
        "roundrobin" | "round_robin" | "loadbalance" | "balance" => "roundrobin",
        "fallback" => "fallback",
        "score" => "score",
        _ => "select",
    };
    Ok(policy.to_string())
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
            .then_with(|| a.cmp(b))
    });
    keys.into_iter()
        .map(|k| escape_regex_meta(&k))
        .collect::<Vec<_>>()
        .join("|")
}

fn patterns_of(iso: &str, with_flag: bool) -> Vec<String> {
    let Some((_, aliases)) = all_regions().iter().find(|(code, _)| *code == iso) else {
        return Vec::new();
    };
    let capital_only = CAPITAL_ONLY.contains(&iso);
    let mut keys: Vec<&str> = Vec::new();
    if !aliases.is_empty() {
        let mut seen = HashSet::new();
        for &k in std::iter::once(&iso).chain(aliases.iter()) {
            if seen.insert(k) {
                keys.push(k);
            }
        }
    }
    let is_han = |s: &str| s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
    let han: Vec<&str> = keys.iter().copied().filter(|k| is_han(k)).collect();
    let mut latin_seen = HashSet::new();
    let mut latin: Vec<String> = Vec::new();
    for &k in &keys {
        if !is_han(k) && !(capital_only && k.to_uppercase() == iso) {
            let low = k.to_lowercase();
            if latin_seen.insert(low.clone()) {
                latin.push(low);
            }
        }
    }
    let han_reduced: Vec<String> = han
        .iter()
        .copied()
        .filter(|&k| !han.iter().any(|&other| other != k && k.contains(other)))
        .map(str::to_string)
        .collect();
    let han_pat = alternatives(han_reduced);
    let start = r"(?:^|[^\p{L}\p{M}])";
    let end = r"(?:[^\p{L}\p{M}]|$)";
    let mut rest_parts = Vec::new();
    if !han_pat.is_empty() {
        rest_parts.push(han_pat);
    }
    if with_flag {
        rest_parts.push(region_flag_str(iso));
    }
    let rest = rest_parts.join("|");
    let mut head_parts = Vec::new();
    if !latin.is_empty() {
        head_parts.push(format!("(?i){start}(?:{}){end}", alternatives(latin)));
    }
    if !rest.is_empty() {
        head_parts.push(rest);
    }
    let mut patterns = Vec::new();
    let main = head_parts.join("|");
    if !main.is_empty() {
        patterns.push(main);
    }
    if capital_only {
        patterns.push(format!("{start}{iso}{end}"));
    }
    patterns
}

fn region_regexes() -> &'static Vec<(&'static str, Vec<String>)> {
    static TABLE: OnceLock<Vec<(&'static str, Vec<String>)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut out = Vec::new();
        for &(iso, _) in all_regions() {
            for with_flag in [true, false] {
                let pats = patterns_of(iso, with_flag);
                if !pats.is_empty() {
                    out.push((iso, pats));
                }
            }
        }
        out
    })
}

/// Port of `regionPatterns` (`apps/lite/src/groupBindings.ts:61`).
pub fn region_patterns(iso: String) -> Result<Value, String> {
    Ok(json!(patterns_of(&iso, true)))
}

/// Port of `subscriptionBinding` (`apps/lite/src/groupBindings.ts:67`).
pub fn subscription_binding(
    name: String,
    regions: Vec<String>,
    words: Vec<String>,
) -> Result<String, String> {
    let mut terms = Vec::new();
    for iso in &regions {
        for pat in patterns_of(iso, true) {
            terms.push(format!("regex: {}", quote_str(&pat)?));
        }
    }
    for word in &words {
        terms.push(format!("keyword: {}", quote_str(word)?));
    }
    let sub = format!("subtag({})", quote_str(&name)?);
    if terms.is_empty() {
        Ok(sub)
    } else {
        Ok(format!("{sub} && name({})", terms.join(", ")))
    }
}

/// Port of `readSubscriptionBinding` (`apps/lite/src/groupBindings.ts:73`).
pub fn read_subscription_binding(filter: String) -> Result<Option<Value>, String> {
    let Some(rows) = dae_core::port::parse_group_conditions(filter)? else {
        return Ok(None);
    };
    if rows.is_empty() || rows.len() > 2 {
        return Ok(None);
    }
    let Some(first) = rows[0].as_object() else {
        return Ok(None);
    };
    if first.get("kind").and_then(Value::as_str) != Some("subtag")
        || first
            .get("negate")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || first.get("alternatives").is_some()
    {
        return Ok(None);
    }
    let Some(sources) = first
        .get("value")
        .and_then(Value::as_str)
        .and_then(parse_condition_values_vec)
        .filter(|v| v.len() == 1)
    else {
        return Ok(None);
    };
    let source = sources[0].clone();
    let mut regions: Vec<String> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let Some(rest) = rows.get(1).and_then(Value::as_object) else {
        return Ok(Some(
            json!({"source": source, "regions": regions, "words": words}),
        ));
    };
    if rest.get("negate").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(None);
    }
    let mut terms: Vec<(String, String)> = Vec::new();
    let alts = rest
        .get("alternatives")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for item in std::iter::once(&rows[1]).chain(alts.iter()) {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let kind = obj.get("kind").and_then(Value::as_str).unwrap_or("");
        if let Some(vals) = obj
            .get("value")
            .and_then(Value::as_str)
            .and_then(parse_condition_values_vec)
        {
            for val in vals {
                terms.push((kind.to_string(), val));
            }
        }
    }
    let table = region_regexes();
    let mut i = 0;
    while i < terms.len() {
        match terms[i].0.as_str() {
            "nameKeyword" => {
                words.push(terms[i].1.clone());
                i += 1;
            }
            "nameRegex" => {
                let matched = table.iter().find(|(_, pats)| {
                    i + pats.len() <= terms.len()
                        && pats.iter().enumerate().all(|(off, pat)| {
                            terms[i + off].0 == "nameRegex" && terms[i + off].1 == *pat
                        })
                });
                if let Some((iso, pats)) = matched {
                    if !regions.iter().any(|r| r == iso) {
                        regions.push((*iso).to_string());
                    }
                    i += pats.len();
                } else {
                    let Some(lits) = regex_literals_opt(&terms[i].1) else {
                        return Ok(None);
                    };
                    words.extend(lits);
                    i += 1;
                }
            }
            _ => return Ok(None),
        }
    }
    Ok(Some(
        json!({"source": source, "regions": regions, "words": words}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_group_bindings_behavior() {
        assert_eq!(
            regex_literals("(?i)jp|日本".into()),
            Ok(Some(vec!["jp".into(), "日本".into()]))
        );
        assert_eq!(regex_literals("^JP".into()), Ok(None));
        assert_eq!(honk_policy("min_avg10(5)".into()), Ok("urltest".into()));
        assert_eq!(honk_policy("balance".into()), Ok("roundrobin".into()));
        assert_eq!(honk_policy("unknown".into()), Ok("select".into()));
        assert_eq!(
            subscription_filter("feed".into(), "(?=x)".into()),
            Ok(Value::Null)
        );
        assert_eq!(
            subscription_filter("feed".into(), "(?i)HK|香港".into()),
            Ok(Value::String(
                "subtag('feed') && name(regex: '(?i)HK|香港')".into()
            ))
        );
        assert!(subscription_filter("o'neil".into(), "ok".into()).is_err());
    }

    #[test]
    fn regions_and_roundtrip_bindings() {
        let hk = region_patterns("HK".into()).expect("hk");
        assert_eq!(hk.as_array().map(Vec::len), Some(1));
        let in_pats = region_patterns("IN".into()).expect("in");
        assert_eq!(in_pats.as_array().map(Vec::len), Some(2));
        assert_eq!(region_patterns("ZZZ".into()), Ok(json!([])));
        let binding = subscription_binding(
            "harbor".into(),
            vec!["HK".into(), "IN".into()],
            vec!["IPLC".into()],
        )
        .expect("binding");
        let parsed = read_subscription_binding(binding).expect("parsed");
        assert_eq!(
            parsed,
            Some(json!({"source": "harbor", "regions": ["HK", "IN"], "words": ["IPLC"]}))
        );
    }

    #[test]
    fn node_binding_edge_cases() {
        let out = node_binding(
            json!({"id": "n1", "name": "node-1", "protocol": "vless", "provider_id": "p1"}),
            json!([{"id": "p1", "name": "feed", "kind": "subscription"}]),
            "Manual".into(),
        )
        .expect("node binding");
        assert_eq!(out["filter"], "subtag('feed') && name('node-1')");
        assert_eq!(out["description"], "vless / feed");
        assert!(node_binding(json!({}), json!(null), "Manual".into()).is_err());
        assert!(node_binding(json!({}), json!([]), "Manual".into()).is_err());
    }
}
