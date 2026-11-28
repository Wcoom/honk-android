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

fn supplement_lookup(lang: Lang, key: &str) -> Option<&'static str> {
    Some(match (lang, key) {
        (Lang::ZhCn, "config.history") => "修订历史",
        (Lang::ZhTw, "config.history") => "修訂歷史",
        (Lang::En, "config.history") => "Revision history",

        (Lang::ZhCn, "conn.target") => "目标地址",
        (Lang::ZhTw, "conn.target") => "目標位址",
        (Lang::En, "conn.target") => "Target",

        (Lang::ZhCn, "conn.outbound") => "出站策略",
        (Lang::ZhTw, "conn.outbound") => "出站策略",
        (Lang::En, "conn.outbound") => "Outbound",

        (Lang::ZhCn, "conn.process") => "进程",
        (Lang::ZhTw, "conn.process") => "行程",
        (Lang::En, "conn.process") => "Process",

        (Lang::ZhCn, "dns.flush") => "清空缓存",
        (Lang::ZhTw, "dns.flush") => "清空快取",
        (Lang::En, "dns.flush") => "Flush cache",

        (Lang::ZhCn, "dns.run") => "查询",
        (Lang::ZhTw, "dns.run") => "查詢",
        (Lang::En, "dns.run") => "Query",

        (Lang::ZhCn, "dns.tabQuery") => "实时解析",
        (Lang::ZhTw, "dns.tabQuery") => "即時解析",
        (Lang::En, "dns.tabQuery") => "Resolve",

        (Lang::ZhCn, "dns.tabCache") => "DNS 缓存",
        (Lang::ZhTw, "dns.tabCache") => "DNS 快取",
        (Lang::En, "dns.tabCache") => "Cache",

        (Lang::ZhCn, "dns.tabLog") => "查询日志",
        (Lang::ZhTw, "dns.tabLog") => "查詢日誌",
        (Lang::En, "dns.tabLog") => "Query log",

        (Lang::ZhCn, "nodes.subscriptions") => "订阅源",
        (Lang::ZhTw, "nodes.subscriptions") => "訂閱源",
        (Lang::En, "nodes.subscriptions") => "Subscriptions",

        (Lang::ZhCn, "pol.clearOverride") => "自动选择",
        (Lang::ZhTw, "pol.clearOverride") => "自動選擇",
        (Lang::En, "pol.clearOverride") => "Auto select",

        (Lang::ZhCn, "pol.probe") => "探测延迟",
        (Lang::ZhTw, "pol.probe") => "探測延遲",
        (Lang::En, "pol.probe") => "Probe",

        (Lang::ZhCn, "pol.probeStarted") => "延迟探测已启动",
        (Lang::ZhTw, "pol.probeStarted") => "延遲探測已啟動",
        (Lang::En, "pol.probeStarted") => "Latency probe started",

        (Lang::ZhCn, "rules.traceRun") => "开始追踪",
        (Lang::ZhTw, "rules.traceRun") => "開始追蹤",
        (Lang::En, "rules.traceRun") => "Run trace",

        (Lang::ZhCn, "ui.add") => "添加",
        (Lang::ZhTw, "ui.add") => "新增",
        (Lang::En, "ui.add") => "Add",

        (Lang::ZhCn, "ui.failed") => "操作失败",
        (Lang::ZhTw, "ui.failed") => "操作失敗",
        (Lang::En, "ui.failed") => "Operation failed",

        (Lang::ZhCn, "ui.pause") => "暂停",
        (Lang::ZhTw, "ui.pause") => "暫停",
        (Lang::En, "ui.pause") => "Pause",

        (Lang::ZhCn, "ui.resume") => "继续",
        (Lang::ZhTw, "ui.resume") => "繼續",
        (Lang::En, "ui.resume") => "Resume",

        (Lang::ZhCn, "ov.suspended") => "已挂起",
        (Lang::ZhTw, "ov.suspended") => "已掛起",
        (Lang::En, "ov.suspended") => "Suspended",

        (Lang::ZhCn, "ov.reloading") => "重载中…",
        (Lang::ZhTw, "ov.reloading") => "重載中…",
        (Lang::En, "ov.reloading") => "Reloading…",

        (Lang::ZhCn, "ov.starting") => "启动中…",
        (Lang::ZhTw, "ov.starting") => "啟動中…",
        (Lang::En, "ov.starting") => "Starting…",

        (Lang::ZhCn, "ui.search") => "搜索…",
        (Lang::ZhTw, "ui.search") => "搜尋…",
        (Lang::En, "ui.search") => "Search…",

        _ => return None,
    })
}

pub fn lookup(lang: Lang, key: &str) -> &str {
    let map = match lang {
        Lang::ZhCn => catalog_zh_cn(),
        Lang::ZhTw => catalog_zh_tw(),
        Lang::En => catalog_en(),
    };
    map.get(key)
        .map(CatalogEntry::as_str)
        .or_else(|| supplement_lookup(lang, key))
        .or_else(|| catalog_en().get(key).map(CatalogEntry::as_str))
        .or_else(|| supplement_lookup(Lang::En, key))
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
