//! M15 WebView Zashboard detection, dynamic Monet CSS generator, and `BoxAndroid` message handler
//! (Section 4.2.4: `ZashboardKsuBridge` + `BoxAndroid` + `mui.kernelsu.org/internal/colors.css`).

use crate::ToolsResult;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub const MONET_COLORS_CSS_URL: &str = "https://mui.kernelsu.org/internal/colors.css";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MonetPaletteInput {
    pub is_dark: bool,
    pub primary: String,
    pub on_primary: String,
    pub primary_container: String,
    pub surface: String,
    pub surface_container: String,
    pub on_surface: String,
    pub outline: String,
}

impl Default for MonetPaletteInput {
    fn default() -> Self {
        Self {
            is_dark: true,
            primary: "#4FB1D0".into(),
            on_primary: "#003544".into(),
            primary_container: "#004D61".into(),
            surface: "#111417".into(),
            surface_container: "#1D2024".into(),
            on_surface: "#E2E2E6".into(),
            outline: "#8C9198".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WebMessageCommand {
    ZashboardDetected {
        #[serde(rename = "pageUrl")]
        page_url: String,
        token: String,
    },
    SaveFile {
        base64: String,
        #[serde(rename = "fileName")]
        file_name: String,
        #[serde(rename = "mimeType", default)]
        mime_type: String,
    },
}

pub fn is_monet_css_url(url: &str) -> bool {
    url.trim().eq_ignore_ascii_case(MONET_COLORS_CSS_URL)
}

/// JS snippet injected on page finish (`pm0.java:234` parity) with one-time anti-spoof token.
pub fn zashboard_detection_script(one_time_token: &str) -> String {
    format!(
        r#"(function(){{
  try {{
    var hasApp = !!document.querySelector('#app');
    var title = (document.title || '').trim().toLowerCase();
    var meta = document.querySelector('meta[name="description"]');
    var desc = (meta && meta.content ? meta.content : '').trim().toLowerCase();
    if (hasApp && (title === 'zashboard' || desc === 'a dashboard using clash api')) {{
      if (window.BoxAndroid && window.BoxAndroid.postMessage) {{
        window.BoxAndroid.postMessage(JSON.stringify({{
          type: 'zashboard-detected',
          pageUrl: location.href,
          token: '{one_time_token}'
        }}));
      }}
    }}
  }} catch (e) {{}}
}})();"#
    )
}

pub fn generate_monet_colors_css(p: &MonetPaletteInput) -> String {
    format!(
        ":root {{\n\
  --monet-primary: {primary};\n\
  --monet-on-primary: {on_primary};\n\
  --monet-primary-container: {primary_container};\n\
  --monet-surface: {surface};\n\
  --monet-surface-container: {surface_container};\n\
  --monet-on-surface: {on_surface};\n\
  --monet-outline: {outline};\n\
  color-scheme: {scheme};\n\
}}\n",
        primary = p.primary,
        on_primary = p.on_primary,
        primary_container = p.primary_container,
        surface = p.surface,
        surface_container = p.surface_container,
        on_surface = p.on_surface,
        outline = p.outline,
        scheme = if p.is_dark { "dark" } else { "light" },
    )
}

pub fn decode_save_file_payload(b64: &str) -> ToolsResult<Vec<u8>> {
    let cleaned = b64
        .trim()
        .split_once(",")
        .map(|(_, rhs)| rhs)
        .unwrap_or(b64.trim());
    base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .map_err(|e| format!("invalid base64 save-file payload: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_monet_css_and_decodes_save_file() {
        assert!(is_monet_css_url("HTTPS://MUI.KERNELSU.ORG/INTERNAL/COLORS.CSS"));
        let css = generate_monet_colors_css(&MonetPaletteInput::default());
        assert!(css.contains("--monet-primary: #4FB1D0;"));
        let raw = decode_save_file_payload("data:text/plain;base64,aGVsbG8=").unwrap();
        assert_eq!(raw, b"hello");
    }
}
