//! Pure-Rust SVG QR code generator replacing `qrcode` JS dependency in Doona Lite ShareDialog.

use qrcode::render::svg;
use qrcode::QrCode;

pub fn render_svg_qr(payload: &str) -> Result<String, String> {
    let code = QrCode::new(payload.as_bytes()).map_err(|e| format!("QR encode failed: {e}"))?;
    let svg_doc = code
        .render::<svg::Color>()
        .min_dimensions(180, 180)
        .dark_color(svg::Color("#111111"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(svg_doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_svg_qr_markup() {
        let out = render_svg_qr("vless://test@example.com:443#Demo").unwrap();
        assert!(out.contains("<svg"));
        assert!(out.contains("</svg>"));
    }
}
