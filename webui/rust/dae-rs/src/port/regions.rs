//! Port of `src/dae/regions.ts` and `src/dae/flags.ts`.

#![forbid(unsafe_code)]

use crate::utf16;
use fancy_regex::Regex;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

struct RegionSpec {
    iso: &'static str,
    aliases: &'static [&'static str],
    short: &'static [(&'static str, &'static str)],
    official: &'static [&'static str],
}

const REGIONS: &[RegionSpec] = &[
    RegionSpec {
        iso: "HK",
        aliases: &["hk", "hkg", "hongkong", "hong kong", "香港", "港"],
        short: &[("en", "Hong Kong"), ("zh-TW", "香港"), ("zh-CN", "香港")],
        official: &[
            "Hong Kong SAR China",
            "中國香港特別行政區",
            "中国香港特别行政区",
        ],
    },
    RegionSpec {
        iso: "TW",
        aliases: &[
            "tw", "tpe", "taiwan", "taipei", "台灣", "台湾", "臺灣", "台北",
        ],
        short: &[("en", "Taiwan"), ("zh-TW", "台灣"), ("zh-CN", "台湾")],
        official: &[],
    },
    RegionSpec {
        iso: "JP",
        aliases: &[
            "jp", "jpn", "japan", "tokyo", "osaka", "nrt", "hnd", "kix", "日本", "東京", "东京",
            "大阪",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "SG",
        aliases: &["sg", "sgp", "singapore", "sin", "新加坡", "獅城", "狮城"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "US",
        aliases: &[
            "us",
            "usa",
            "united states",
            "america",
            "lax",
            "sjc",
            "sea",
            "nyc",
            "los angeles",
            "san jose",
            "seattle",
            "美國",
            "美国",
            "洛杉磯",
            "洛杉矶",
            "聖何塞",
            "圣何塞",
            "西雅圖",
            "西雅图",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "KR",
        aliases: &[
            "kr",
            "kor",
            "korea",
            "seoul",
            "icn",
            "south korea",
            "南韩",
            "南韓",
            "韓國",
            "韩国",
            "首爾",
            "首尔",
        ],
        short: &[("zh-TW", "韓國")],
        official: &[],
    },
    RegionSpec {
        iso: "CA",
        aliases: &["can", "canada", "toronto", "yyz", "加拿大"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "GB",
        aliases: &[
            "gb",
            "uk",
            "united kingdom",
            "britain",
            "london",
            "lhr",
            "英國",
            "英国",
            "倫敦",
            "伦敦",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "DE",
        aliases: &[
            "de",
            "deu",
            "germany",
            "frankfurt",
            "fra",
            "德國",
            "德国",
            "法蘭克福",
            "法兰克福",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "FR",
        aliases: &["fr", "france", "paris", "cdg", "法國", "法国", "巴黎"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "NL",
        aliases: &[
            "nl",
            "nld",
            "netherlands",
            "amsterdam",
            "ams",
            "荷蘭",
            "荷兰",
            "阿姆斯特丹",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "RU",
        aliases: &[
            "ru",
            "rus",
            "russia",
            "moscow",
            "俄羅斯",
            "俄罗斯",
            "莫斯科",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "AU",
        aliases: &[
            "au",
            "aus",
            "australia",
            "sydney",
            "syd",
            "澳洲",
            "澳大利亞",
            "澳大利亚",
            "悉尼",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "IN",
        aliases: &[
            "in", "ind", "india", "mumbai", "bom", "印度", "孟買", "孟买",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "TR",
        aliases: &["tr", "tur", "turkey", "türkiye", "istanbul", "土耳其"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "BR",
        aliases: &["br", "bra", "brazil", "sao paulo", "gru", "巴西"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "AR",
        aliases: &["ar", "arg", "argentina", "阿根廷"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "MY",
        aliases: &["my", "mys", "malaysia", "kul", "馬來西亞", "马来西亚"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "TH",
        aliases: &["th", "tha", "thailand", "bangkok", "bkk", "泰國", "泰国"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "VN",
        aliases: &["vn", "vnm", "vietnam", "越南"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "PH",
        aliases: &["ph", "phl", "philippines", "manila", "菲律賓", "菲律宾"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "ID",
        aliases: &[
            "id",
            "idn",
            "indonesia",
            "jakarta",
            "印度尼西亚",
            "印度尼西亞",
            "印尼",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "AE",
        aliases: &[
            "ae",
            "are",
            "uae",
            "united arab emirates",
            "阿联酋",
            "阿聯酋",
            "dubai",
            "dxb",
            "阿聯",
            "阿联",
            "杜拜",
            "迪拜",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "CH",
        aliases: &["ch", "che", "switzerland", "zurich", "瑞士"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "SE",
        aliases: &["se", "swe", "sweden", "stockholm", "瑞典"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "FI",
        aliases: &["fi", "fin", "finland", "helsinki", "芬蘭", "芬兰"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "IT",
        aliases: &["it", "ita", "italy", "milan", "義大利", "意大利"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "ES",
        aliases: &["es", "esp", "spain", "madrid", "西班牙"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "PL",
        aliases: &["pl", "pol", "poland", "warsaw", "波蘭", "波兰"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "MO",
        aliases: &["mo", "mac", "macau", "macao", "澳門", "澳门"],
        short: &[("en", "Macao"), ("zh-TW", "澳門"), ("zh-CN", "澳门")],
        official: &[
            "Macao SAR China",
            "中國澳門特別行政區",
            "中国澳门特别行政区",
        ],
    },
    RegionSpec {
        iso: "NZ",
        aliases: &["new zealand", "新西兰", "紐西蘭"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "MX",
        aliases: &["mexico", "墨西哥"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "NO",
        aliases: &["norway", "挪威"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "DK",
        aliases: &["denmark", "丹麦", "丹麥"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "IE",
        aliases: &["ireland", "爱尔兰", "愛爾蘭"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "ZA",
        aliases: &["south africa", "南非"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "IL",
        aliases: &["israel", "以色列"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "UA",
        aliases: &["ukraine", "乌克兰", "烏克蘭"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "PT",
        aliases: &["portugal", "葡萄牙"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "CN",
        aliases: &[
            "cn", "chn", "china", "shanghai", "beijing", "shenzhen", "中國", "中国", "上海",
            "北京", "深圳",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "KZ",
        aliases: &[
            "kazakhstan",
            "almaty",
            "哈萨克斯坦",
            "哈薩克斯坦",
            "哈萨克",
            "哈薩克",
        ],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "AT",
        aliases: &["austria", "vienna", "奥地利", "奧地利"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "BE",
        aliases: &["belgium", "brussels", "比利时", "比利時"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "LU",
        aliases: &["luxembourg", "卢森堡", "盧森堡"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "CZ",
        aliases: &["czech", "czechia", "prague", "捷克"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "RO",
        aliases: &["romania", "bucharest", "罗马尼亚", "羅馬尼亞"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "IS",
        aliases: &["iceland", "reykjavik", "冰岛", "冰島"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "CL",
        aliases: &["chile", "santiago", "智利"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "CO",
        aliases: &["colombia", "bogota", "哥伦比亚", "哥倫比亞"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "SA",
        aliases: &["saudi", "riyadh", "沙特", "沙烏地"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "PK",
        aliases: &["pakistan", "karachi", "巴基斯坦"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "NG",
        aliases: &["nigeria", "lagos", "尼日利亚", "尼日利亞", "奈及利亞"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "EG",
        aliases: &["egypt", "cairo", "埃及"],
        short: &[],
        official: &[],
    },
    RegionSpec {
        iso: "KH",
        aliases: &["cambodia", "柬埔寨"],
        short: &[],
        official: &[],
    },
];

const FLAG_REGIONS_STR: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

const CAPITAL_ONLY: &[&str] = &[
    "IN", "IT", "MY", "NO", "ID", "KZ", "AT", "BE", "LU", "CZ", "RO", "IS", "CL", "CO", "SA", "PK",
    "NG", "EG", "KH",
];

fn flag_regions() -> &'static Vec<&'static str> {
    static LIST: OnceLock<Vec<&'static str>> = OnceLock::new();
    LIST.get_or_init(|| FLAG_REGIONS_STR.split(' ').collect())
}

pub fn region_flag_str(region: &str) -> String {
    let mut out = String::new();
    for ch in region.chars() {
        let mut buf = [0u16; 2];
        let cu = ch.encode_utf16(&mut buf)[0] as u32;
        if let Some(c) = char::from_u32(0x1f1e6 + cu - 65) {
            out.push(c);
        }
    }
    out
}

pub fn region_flag(region: String) -> Result<Value, String> {
    Ok(Value::String(region_flag_str(&region)))
}

struct AliasMember {
    iso: &'static str,
    rank: usize,
}

struct Matcher {
    members: Vec<AliasMember>,
    pattern: Regex,
}

struct RegionMatchers {
    matchers: [Matcher; 2],
    by_flag: HashMap<String, &'static str>,
}

fn has_han(s: &str) -> bool {
    static HAN: OnceLock<regex::Regex> = OnceLock::new();
    let re = HAN.get_or_init(|| {
        // INFALLIBLE: constant valid regex pattern
        regex::Regex::new(r"\p{Han}").expect("valid regex")
    });
    re.is_match(s)
}

fn region_matchers() -> &'static RegionMatchers {
    static INSTANCE: OnceLock<RegionMatchers> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        struct RawAlias {
            iso: &'static str,
            key: String,
            exact_case: bool,
            rank: usize,
        }
        let mut aliases: Vec<RawAlias> = Vec::new();
        for spec in REGIONS {
            let mut seen = HashSet::new();
            let mut keys = Vec::new();
            for &k in std::iter::once(&spec.iso).chain(spec.aliases.iter()) {
                let norm = if k.to_uppercase() == spec.iso {
                    spec.iso.to_string()
                } else {
                    k.to_string()
                };
                if seen.insert(norm.clone()) {
                    keys.push(norm);
                }
            }
            let cap_only = CAPITAL_ONLY.contains(&spec.iso);
            for key in keys {
                let exact_case = key.to_uppercase() == spec.iso && cap_only;
                let rank = aliases.len();
                aliases.push(RawAlias {
                    iso: spec.iso,
                    key,
                    exact_case,
                    rank,
                });
            }
        }
        let build = |exact_case: bool| -> Matcher {
            let mut members: Vec<&RawAlias> = aliases
                .iter()
                .filter(|a| a.exact_case == exact_case)
                .collect();
            members.sort_by(|a, b| {
                b.key
                    .encode_utf16()
                    .count()
                    .cmp(&a.key.encode_utf16().count())
                    .then_with(|| a.rank.cmp(&b.rank))
            });
            let (han, latin): (Vec<_>, Vec<_>) = members.into_iter().partition(|a| has_han(&a.key));
            let group = |list: &[&RawAlias]| -> String {
                list.iter()
                    .map(|a| format!("({})", a.key))
                    .collect::<Vec<_>>()
                    .join("|")
            };
            let mut parts = Vec::new();
            if !latin.is_empty() {
                parts.push(format!(
                    r"(?<![\p{{L}}\p{{M}}])(?:{})(?![\p{{L}}\p{{M}}])",
                    group(&latin)
                ));
            }
            if !han.is_empty() {
                parts.push(group(&han));
            }
            let src = if exact_case {
                parts.join("|")
            } else {
                format!("(?i:{})", parts.join("|"))
            };
            let ordered: Vec<AliasMember> = latin
                .into_iter()
                .chain(han)
                .map(|a| AliasMember {
                    iso: a.iso,
                    rank: a.rank,
                })
                .collect();
            // INFALLIBLE: constant alias table generates valid regex
            let pattern = Regex::new(&src).expect("valid region regex");
            Matcher {
                members: ordered,
                pattern,
            }
        };
        let mut by_flag = HashMap::new();
        for &iso in flag_regions() {
            by_flag.insert(region_flag_str(iso), iso);
        }
        RegionMatchers {
            matchers: [build(false), build(true)],
            by_flag,
        }
    })
}

fn match_flag(
    name: &str,
    by_flag: &HashMap<String, &'static str>,
) -> Option<(&'static str, usize)> {
    static FLAG_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = FLAG_RE.get_or_init(|| {
        // INFALLIBLE: constant valid regex pattern
        regex::Regex::new("[\u{1F1E6}-\u{1F1FF}]{2}").expect("valid regex")
    });
    for m in re.find_iter(name) {
        if let Some(&iso) = by_flag.get(m.as_str()) {
            return Some((iso, utf16::utf16_offset(name, m.start())));
        }
    }
    None
}

pub fn match_region(name: String) -> Result<Value, String> {
    let rm = region_matchers();
    let mut chosen: Option<(&'static str, usize, usize, usize)> = None;
    for matcher in &rm.matchers {
        if let Ok(Some(caps)) = matcher.pattern.captures(&name)
            && let Some(full) = caps.get(0)
        {
            let mut found_idx = None;
            for idx in 1..caps.len() {
                if caps.get(idx).is_some() {
                    found_idx = Some(idx - 1);
                    break;
                }
            }
            if let Some(member_idx) = found_idx {
                let member = &matcher.members[member_idx];
                let index = utf16::utf16_offset(&name, full.start());
                let length = full.as_str().encode_utf16().count();
                let better = match chosen {
                    None => true,
                    Some((_, c_idx, c_len, c_rank)) => {
                        index < c_idx
                            || (index == c_idx
                                && (length > c_len || (length == c_len && member.rank < c_rank)))
                    }
                };
                if better {
                    chosen = Some((member.iso, index, length, member.rank));
                }
            }
        }
    }
    let flag = match_flag(&name, &rm.by_flag);
    Ok(match (flag, chosen) {
        (Some((iso, _)), None) => Value::String(iso.to_string()),
        (Some((iso, f_idx)), Some((_, c_idx, _, _))) if f_idx < c_idx => {
            Value::String(iso.to_string())
        }
        (_, Some((iso, _, _, _))) => Value::String(iso.to_string()),
        (None, None) => Value::Null,
    })
}

// --- flags.ts ---

pub fn flag_key(name: String) -> Result<Value, String> {
    Ok(Value::String(format!("node:{name}")))
}

pub fn valid_flag_name(name: String) -> Result<Value, String> {
    let len = name.encode_utf16().count();
    Ok(Value::Bool(len > 0 && len <= 512))
}

pub fn has_embedded_flag(name: String) -> Result<Value, String> {
    static EMBEDDED: OnceLock<regex::Regex> = OnceLock::new();
    let re = EMBEDDED.get_or_init(|| {
        // INFALLIBLE: constant valid regex pattern
        regex::Regex::new("[\u{1F1E6}-\u{1F1FF}]{2}|\u{1F3F4}[\u{E0061}-\u{E007A}]+\u{E007F}")
            .expect("valid regex")
    });
    Ok(Value::Bool(re.is_match(&name)))
}

pub fn valid_flag_choice(value: Value) -> Result<bool, String> {
    Ok(value
        .as_str()
        .is_some_and(|s| s == "none" || flag_regions().contains(&s)))
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
fn build_flag_choices_wasm(locale: &str) -> Result<Vec<Value>, String> {
    use js_sys::{Array, Function, Object, Reflect};
    use wasm_bindgen::{JsCast, JsValue};

    let global = js_sys::global();
    let intl = Reflect::get(&global, &JsValue::from_str("Intl")).map_err(js_err_string)?;
    let dn_ctor: Function = Reflect::get(&intl, &JsValue::from_str("DisplayNames"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let loc_ctor: Function = Reflect::get(&intl, &JsValue::from_str("Locale"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let col_ctor: Function = Reflect::get(&intl, &JsValue::from_str("Collator"))
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

    let loc_arr = Array::of1(&JsValue::from_str(locale));
    let names =
        Reflect::construct(&dn_ctor, &Array::of2(&loc_arr, &opts)).map_err(js_err_string)?;
    let loc_obj = Reflect::construct(&loc_ctor, &Array::of1(&JsValue::from_str(locale)))
        .map_err(js_err_string)?;
    let language = Reflect::get(&loc_obj, &JsValue::from_str("language"))
        .map_err(js_err_string)?
        .as_string()
        .unwrap_or_default();

    let en_arr = Array::of1(&JsValue::from_str("en"));
    let english =
        Reflect::construct(&dn_ctor, &Array::of2(&en_arr, &opts)).map_err(js_err_string)?;
    let collator = Reflect::construct(&col_ctor, &Array::of1(&JsValue::from_str(locale)))
        .map_err(js_err_string)?;

    let names_of: Function = Reflect::get(&names, &JsValue::from_str("of"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let english_of: Function = Reflect::get(&english, &JsValue::from_str("of"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;
    let compare_fn: Function = Reflect::get(&collator, &JsValue::from_str("compare"))
        .map_err(js_err_string)?
        .dyn_into()
        .map_err(js_err_string)?;

    let known: HashMap<&str, (&RegionSpec, usize)> = REGIONS
        .iter()
        .enumerate()
        .map(|(rank, spec)| (spec.iso, (spec, rank)))
        .collect();

    struct Item {
        id: &'static str,
        label: String,
        flag: String,
        keywords: String,
        rank: usize,
    }

    let mut items = Vec::with_capacity(flag_regions().len());
    for &id in flag_regions() {
        let region = known.get(id);
        let official = names_of
            .call1(&names, &JsValue::from_str(id))
            .map_err(js_err_string)?
            .as_string()
            .unwrap_or_else(|| id.to_string());
        let en_name = english_of
            .call1(&english, &JsValue::from_str(id))
            .map_err(js_err_string)?
            .as_string();
        let label = region
            .and_then(|(spec, _)| {
                spec.short
                    .iter()
                    .find(|(k, _)| *k == locale)
                    .or_else(|| spec.short.iter().find(|(k, _)| *k == language))
                    .map(|(_, v)| (*v).to_string())
            })
            .unwrap_or_else(|| official.clone());
        let mut kw: Vec<String> = Vec::new();
        kw.push(id.to_string());
        kw.push(official);
        kw.push(en_name.unwrap_or_default());
        if let Some((spec, _)) = region {
            for &o in spec.official {
                kw.push(o.to_string());
            }
            for &a in spec.aliases {
                kw.push(a.to_string());
            }
        }
        items.push(Item {
            id,
            label,
            flag: region_flag_str(id),
            keywords: kw.join(" "),
            rank: region.map_or(REGIONS.len(), |(_, r)| *r),
        });
    }

    let mut sort_err: Option<String> = None;
    items.sort_by(|a, b| {
        let ord = a.rank.cmp(&b.rank);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
        match compare_fn.call2(
            &collator,
            &JsValue::from_str(&a.label),
            &JsValue::from_str(&b.label),
        ) {
            Ok(v) => {
                let n = v.as_f64().unwrap_or(0.0);
                if n < 0.0 {
                    std::cmp::Ordering::Less
                } else if n > 0.0 {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            Err(e) => {
                if sort_err.is_none() {
                    sort_err = Some(js_err_string(e));
                }
                std::cmp::Ordering::Equal
            }
        }
    });
    if let Some(e) = sort_err {
        return Err(e);
    }

    Ok(items
        .into_iter()
        .map(|it| {
            json!({
                "id": it.id,
                "label": it.label,
                "flag": it.flag,
                "keywords": it.keywords,
            })
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
fn build_flag_choices_host(locale: &str) -> Result<Vec<Value>, String> {
    if locale.is_empty()
        || !locale
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || locale.starts_with('-')
        || locale.ends_with('-')
    {
        return Err("RangeError: Incorrect locale information provided".to_string());
    }
    let language = locale.split('-').next().unwrap_or(locale);
    let known: HashMap<&str, (&RegionSpec, usize)> = REGIONS
        .iter()
        .enumerate()
        .map(|(rank, spec)| (spec.iso, (spec, rank)))
        .collect();
    let mut items: Vec<(&str, String, String, String, usize)> = flag_regions()
        .iter()
        .map(|&id| {
            let region = known.get(id);
            let label = region
                .and_then(|(spec, _)| {
                    spec.short
                        .iter()
                        .find(|(k, _)| *k == locale)
                        .or_else(|| spec.short.iter().find(|(k, _)| *k == language))
                        .map(|(_, v)| (*v).to_string())
                })
                .unwrap_or_else(|| id.to_string());
            let mut kw = vec![id.to_string(), id.to_string(), id.to_string()];
            if let Some((spec, _)) = region {
                kw.extend(spec.official.iter().map(|s| (*s).to_string()));
                kw.extend(spec.aliases.iter().map(|s| (*s).to_string()));
            }
            (
                id,
                label,
                region_flag_str(id),
                kw.join(" "),
                region.map_or(REGIONS.len(), |(_, r)| *r),
            )
        })
        .collect();
    items.sort_by(|a, b| a.4.cmp(&b.4).then_with(|| a.1.cmp(&b.1)));
    Ok(items
        .into_iter()
        .map(|(id, label, flag, keywords, _)| {
            json!({
                "id": id,
                "label": label,
                "flag": flag,
                "keywords": keywords,
            })
        })
        .collect())
}

pub fn flag_choices(locale: String) -> Result<Vec<Value>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::RefCell;
        thread_local! {
            static CACHE: RefCell<HashMap<String, Vec<Value>>> = RefCell::new(HashMap::new());
        }
        if let Some(cached) = CACHE.with(|c| c.borrow().get(&locale).cloned()) {
            return Ok(cached);
        }
        let built = build_flag_choices_wasm(&locale)?;
        CACHE.with(|c| c.borrow_mut().insert(locale, built.clone()));
        Ok(built)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        build_flag_choices_host(&locale)
    }
}
