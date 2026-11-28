//! 3-language i18n catalog loader embedding Doona's exact JSON locale tables (`zh-CN`, `zh-TW`, `en`).

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum CatalogEntry {
    Text(String),
    Plural {
        one: Option<String>,
        other: String,
    },
}

impl CatalogEntry {
    fn as_str(&self) -> &str {
        match self {
            CatalogEntry::Text(s) => s.as_str(),
            CatalogEntry::Plural { other, .. } => other.as_str(),
        }
    }

    fn for_count(&self, count: u64) -> &str {
        match self {
            CatalogEntry::Text(s) => s.as_str(),
            CatalogEntry::Plural { one, other } => {
                if count == 1 {
                    one.as_deref().unwrap_or(other.as_str())
                } else {
                    other.as_str()
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    ZhCn,
    ZhTw,
    En,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::ZhCn, Lang::ZhTw, Lang::En];

    pub fn code(self) -> &'static str {
        match self {
            Lang::ZhCn => "zh-CN",
            Lang::ZhTw => "zh-TW",
            Lang::En => "en",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::ZhCn => "简体中文",
            Lang::ZhTw => "繁體中文",
            Lang::En => "English",
        }
    }

    pub fn from_code(code: &str) -> Self {
        let lower = code.to_ascii_lowercase();
        if lower.starts_with("zh-tw") || lower.starts_with("zh-hant") || lower.starts_with("zh-hk") {
            Lang::ZhTw
        } else if lower.starts_with("zh") {
            Lang::ZhCn
        } else {
            Lang::En
        }
    }
}

fn catalog_zh_cn() -> &'static HashMap<String, CatalogEntry> {
    static MAP: OnceLock<HashMap<String, CatalogEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/zh-CN.json"))
            .expect("valid zh-CN.json catalog")
    })
}

fn catalog_zh_tw() -> &'static HashMap<String, CatalogEntry> {
    static MAP: OnceLock<HashMap<String, CatalogEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/zh-TW.json"))
            .expect("valid zh-TW.json catalog")
    })
}

fn catalog_en() -> &'static HashMap<String, CatalogEntry> {
    static MAP: OnceLock<HashMap<String, CatalogEntry>> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/en.json"))
            .expect("valid en.json catalog")
    })
}

pub fn lookup(lang: Lang, key: &str) -> &str {
    let map = match lang {
        Lang::ZhCn => catalog_zh_cn(),
        Lang::ZhTw => catalog_zh_tw(),
        Lang::En => catalog_en(),
    };
    map.get(key)
        .or_else(|| catalog_en().get(key))
        .map(CatalogEntry::as_str)
        .unwrap_or(key)
}

pub fn tr(lang: Lang, key: &str) -> String {
    lookup(lang, key).to_string()
}

pub fn tr_args(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let mut text = lookup(lang, key).to_string();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

pub fn tr_plural(lang: Lang, key: &str, count: u64) -> String {
    let map = match lang {
        Lang::ZhCn => catalog_zh_cn(),
        Lang::ZhTw => catalog_zh_tw(),
        Lang::En => catalog_en(),
    };
    let template = map
        .get(key)
        .or_else(|| catalog_en().get(key))
        .map(|entry| entry.for_count(count))
        .unwrap_or(key);
    template.replace("{n}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_exact_doona_zh_cn_and_en_keys() {
        assert_eq!(lookup(Lang::ZhCn, "act.global"), "节点策略选择");
        assert_eq!(lookup(Lang::ZhTw, "act.global"), "節點策略選擇");
        assert_eq!(lookup(Lang::En, "act.global"), "Node policy");
        assert_eq!(
            tr_args(
                Lang::ZhCn,
                "act.globalRuleApplied",
                &[("target", "自建 CDN 优选"), ("mode", "规则")]
            ),
            "节点策略已切换为 自建 CDN 优选（保持规则模式）"
        );
    }
}
