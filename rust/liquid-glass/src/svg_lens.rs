//! Pure-Rust SVG `<feDisplacementMap>` lens & chromatic-dispersion filter generator for WebUI.
//!
//! Computes exact Kyant0 SDF rounded-capsule refraction displacement fields in Rust and
//! encodes them as compact 32-bit uncompressed RGBA BMP `data:image/bmp;base64,...` URIs
//! (allowed under strict CSP `img-src 'self' data:`) for GPU-accelerated CSS `backdrop-filter`.

use crate::sdf::{CornerRadii, LensParams, refraction_offset};

/// Generated SVG displacement-map data URIs & filter parameters for WebUI liquid glass.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgLensAssets {
    pub bar_map_uri: String,
    pub bar_scale: f32,
    pub droplet_map_uri: String,
    pub droplet_scale: f32,
    pub droplet_red_scale: f32,
    pub droplet_blue_scale: f32,
}

/// Builds both the bottom-bar lens map and the floating chromatic-dispersion droplet map.
pub fn build_svg_lens_assets() -> SvgLensAssets {
    let bar_params = LensParams {
        width: 160.0,
        height: 44.0,
        radii: CornerRadii::capsule(160.0, 44.0),
        refraction_height: 15.0,
        refraction_amount: 14.0,
        depth_effect: 0.25,
        chromatic_aberration: 0.0,
    };
    let (bar_uri, bar_scale) = encode_lens_displacement_bmp(160, 44, &bar_params);

    let droplet_params = LensParams {
        width: 96.0,
        height: 40.0,
        radii: CornerRadii::capsule(96.0, 40.0),
        refraction_height: 13.0,
        refraction_amount: 13.0,
        depth_effect: 0.35,
        chromatic_aberration: 1.0,
    };
    let (droplet_uri, droplet_scale) = encode_lens_displacement_bmp(96, 40, &droplet_params);

    SvgLensAssets {
        bar_map_uri: bar_uri,
        bar_scale,
        droplet_map_uri: droplet_uri,
        droplet_scale,
        // Subtle prism RGB dispersion matching Kyant0's spectral shift without hurting legibility
        droplet_red_scale: droplet_scale * 1.14,
        droplet_blue_scale: droplet_scale * 0.86,
    }
}

/// Evaluates the Kyant0 SDF refraction field on a `w x h` grid and encodes `(dx, dy)` into
/// the R and G channels (`128` = neutral 0 displacement) of an uncompressed 32-bit BMP data URI.
pub fn encode_lens_displacement_bmp(w: u32, h: u32, params: &LensParams) -> (String, f32) {
    let mut offsets = Vec::with_capacity((w * h) as usize);
    let mut max_disp: f32 = 1.0;

    for y in 0..h {
        let py = (y as f32) + 0.5;
        for x in 0..w {
            let px = (x as f32) + 0.5;
            let (dx, dy) = refraction_offset(px, py, params);
            max_disp = max_disp.max(dx.abs()).max(dy.abs());
            offsets.push((dx, dy));
        }
    }

    let pixel_bytes = (w * h * 4) as usize;
    let file_size = 54 + pixel_bytes;
    let mut bmp = Vec::with_capacity(file_size);

    // BITMAPFILEHEADER (14 bytes)
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&54u32.to_le_bytes());

    // BITMAPINFOHEADER (40 bytes), top-down via negative height
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(w as i32).to_le_bytes());
    bmp.extend_from_slice(&(-(h as i32)).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&32u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&(pixel_bytes as u32).to_le_bytes());
    bmp.extend_from_slice(&2835u32.to_le_bytes());
    bmp.extend_from_slice(&2835u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());

    // BGRA pixels: R = X displacement, G = Y displacement, B = 128 neutral, A = 255
    for (dx, dy) in offsets {
        let r = (128.0 + (dx / max_disp) * 127.0).round().clamp(0.0, 255.0) as u8;
        let g = (128.0 + (dy / max_disp) * 127.0).round().clamp(0.0, 255.0) as u8;
        bmp.push(128u8); // B
        bmp.push(g); // G
        bmp.push(r); // R
        bmp.push(255u8); // A
    }

    let mut uri = String::with_capacity(24 + (bmp.len() * 4) / 3 + 4);
    uri.push_str("data:image/bmp;base64,");
    base64_encode_into(&bmp, &mut uri);
    (uri, max_disp)
}

fn base64_encode_into(input: &[u8], out: &mut String) {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0;
    while i + 3 <= input.len() {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8) | (input[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let n = (input[i] as u32) << 16;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_valid_bmp_data_uris_for_bar_and_droplet() {
        let assets = build_svg_lens_assets();
        assert!(assets.bar_map_uri.starts_with("data:image/bmp;base64,Qk"));
        assert!(assets.droplet_map_uri.starts_with("data:image/bmp;base64,Qk"));
        assert!(assets.bar_scale >= 1.0);
        assert!(assets.droplet_red_scale > assets.droplet_blue_scale);
    }
}
