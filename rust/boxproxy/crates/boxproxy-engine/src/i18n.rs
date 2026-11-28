//! Lightweight 4-locale (`zh-CN`, `zh-TW`, `en`, `ru`) UI & system-component string table.

pub fn normalize_locale(tag: &str) -> &'static str {
    let lower = tag.trim().to_ascii_lowercase();
    if lower.starts_with("zh-tw") || lower.starts_with("zh-hk") || lower.starts_with("zh-hant") {
        "zh-TW"
    } else if lower.starts_with("zh") {
        "zh-CN"
    } else if lower.starts_with("ru") {
        "ru"
    } else {
        "en"
    }
}

pub fn tr(locale: &str, key: &str) -> &'static str {
    let loc = normalize_locale(locale);
    match (loc, key) {
        ("zh-CN", "status_running") => "运行中",
        ("zh-TW", "status_running") => "執行中",
        ("ru", "status_running") => "Запущено",
        (_, "status_running") => "Running",

        ("zh-CN", "status_stopped") => "已停止",
        ("zh-TW", "status_stopped") => "已停止",
        ("ru", "status_stopped") => "Остановлено",
        (_, "status_stopped") => "Stopped",

        ("zh-CN", "status_working") => "处理中…",
        ("zh-TW", "status_working") => "處理中…",
        ("ru", "status_working") => "Выполняется…",
        (_, "status_working") => "Working…",

        ("zh-CN", "btn_reload") => "重载",
        ("zh-TW", "btn_reload") => "重載",
        ("ru", "btn_reload") => "Перезагрузить",
        (_, "btn_reload") => "Reload",

        ("zh-CN", "btn_stop") => "停止",
        ("zh-TW", "btn_stop") => "停止",
        ("ru", "btn_stop") => "Стоп",
        (_, "btn_stop") => "Stop",

        ("zh-CN", "btn_restart") => "重启",
        ("zh-TW", "btn_restart") => "重啟",
        ("ru", "btn_restart") => "Перезапуск",
        (_, "btn_restart") => "Restart",

        ("zh-CN", "btn_open_panel" | "btn_panel") => "面板",
        ("zh-TW", "btn_open_panel" | "btn_panel") => "面板",
        ("ru", "btn_open_panel" | "btn_panel") => "Панель",
        (_, "btn_open_panel" | "btn_panel") => "Panel",

        ("zh-CN", "btn_open_web_ui" | "btn_web_ui") => "WebUI",
        ("zh-TW", "btn_open_web_ui" | "btn_web_ui") => "WebUI",
        ("ru", "btn_open_web_ui" | "btn_web_ui") => "WebUI",
        (_, "btn_open_web_ui" | "btn_web_ui") => "WebUI",

        _ => "Box Honk",
    }
}
