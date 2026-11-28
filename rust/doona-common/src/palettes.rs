//! Doona 16-palette appearance definitions and `localStorage` preference keys.

pub const PALETTES: &[(&str, &str)] = &[
    ("rose-pine/main", "Rose Pine"),
    ("rose-pine/moon", "Rose Pine Moon"),
    ("catppuccin/frappe", "Catppuccin Frappe"),
    ("catppuccin/macchiato", "Catppuccin Macchiato"),
    ("catppuccin/mocha", "Catppuccin Mocha"),
    ("arco/arco", "Arco"),
    ("semi/semi", "Semi"),
    ("glass/glass", "Glass"),
    ("glass/clear", "Glass Clear"),
    ("glass/frosted", "Glass Frosted"),
    ("glass/tinted", "Glass Tinted"),
    ("nord/nord", "Nord"),
    ("kary/kary", "Kary"),
    ("antd/antd", "Ant Design"),
    ("qiangguo/qiangguo", "Qiangguo"),
    ("halloween/halloween", "Halloween"),
];

pub const DEFAULT_PALETTE: &str = "rose-pine/moon";

pub fn split_palette(palette: &str) -> (&str, &str) {
    let valid = PALETTES.iter().any(|(id, _)| *id == palette);
    let chosen = if valid { palette } else { DEFAULT_PALETTE };
    chosen.split_once('/').unwrap_or(("rose-pine", "moon"))
}

#[cfg(target_arch = "wasm32")]
pub mod browser {
    use super::split_palette;

    pub fn read_storage(key: &str) -> Option<String> {
        web_sys::window()?
            .local_storage()
            .ok()??
            .get_item(key)
            .ok()?
    }

    pub fn write_storage(key: &str, value: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item(key, value);
        }
    }

    pub fn remove_storage(key: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.remove_item(key);
        }
    }

    pub fn apply_appearance(scheme: &str, palette: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(doc) = window.document() else {
            return;
        };
        let Some(root) = doc.document_element() else {
            return;
        };
        let prefers_dark = window
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .is_some_and(|m| m.matches());
        let resolved = match scheme {
            "dark" => "dark",
            "light" => "light",
            _ if prefers_dark => "dark",
            _ => "light",
        };
        let (family, flavour) = split_palette(palette);
        let _ = root.set_attribute("data-scheme", resolved);
        let _ = root.set_attribute("data-theme", scheme);
        let _ = root.set_attribute("data-family", family);
        let _ = root.set_attribute("data-flavour", flavour);
    }
}
