//! Signed-distance-field (SDF) rounded-rectangle optics, curved meniscus refraction,
//! diagonal chromatic dispersion, Fresnel rim highlight, and vibrancy color matrix.
//!
//! Ported 1:1 from `Kyant0/AndroidLiquidGlass` (`Shaders.kt`, `Lens.kt`, `ColorFilter.kt`).

use serde::{Deserialize, Serialize};

/// Per-corner radii `[top_left, top_right, bottom_right, bottom_left]` in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub fn uniform(r: f32) -> Self {
        Self {
            top_left: r,
            top_right: r,
            bottom_right: r,
            bottom_left: r,
        }
    }

    pub fn capsule(width: f32, height: f32) -> Self {
        Self::uniform(width.min(height) * 0.5)
    }

    pub fn clamped(self, width: f32, height: f32) -> [f32; 4] {
        let max_r = width.min(height) * 0.5;
        [
            self.top_left.min(max_r).max(0.0),
            self.top_right.min(max_r).max(0.0),
            self.bottom_right.min(max_r).max(0.0),
            self.bottom_left.min(max_r).max(0.0),
        ]
    }
}

/// Optical lens parameters matching `Kyant0/AndroidLiquidGlass` `BackdropEffectScope.lens`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LensParams {
    pub width: f32,
    pub height: f32,
    pub radii: CornerRadii,
    /// Depth inward from the glass boundary over which the curved meniscus bends rays (px).
    pub refraction_height: f32,
    /// Maximum ray bend magnitude (px; negative pulls backdrop inward like a convex edge).
    pub refraction_amount: f32,
    /// Additional radial depth gradient weight (`0.0` or `1.0`).
    pub depth_effect: f32,
    /// Spectral dispersion strength (`0.0` = off, `1.0` = full prism edge fringing).
    pub chromatic_aberration: f32,
}

#[inline]
pub fn radius_at(cx: f32, cy: f32, radii: [f32; 4]) -> f32 {
    if cx >= 0.0 {
        if cy <= 0.0 { radii[1] } else { radii[2] }
    } else if cy <= 0.0 {
        radii[0]
    } else {
        radii[3]
    }
}

/// Exact SDF of a rounded rectangle centered at the origin.
#[inline]
pub fn sd_rounded_rect(cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let qx = cx.abs() - (hx - r);
    let qy = cy.abs() - (hy - r);
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    let outside = (ox * ox + oy * oy).sqrt() - r;
    let inside = qx.max(qy).min(0.0);
    outside + inside
}

/// Exact normalized outward gradient of `sd_rounded_rect`.
#[inline]
pub fn grad_sd_rounded_rect(cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> (f32, f32) {
    let qx = cx.abs() - (hx - r);
    let qy = cy.abs() - (hy - r);
    let sx = if cx < 0.0 { -1.0 } else { 1.0 };
    let sy = if cy < 0.0 { -1.0 } else { 1.0 };
    if qx >= 0.0 || qy >= 0.0 {
        let ox = qx.max(0.0);
        let oy = qy.max(0.0);
        let len = (ox * ox + oy * oy).sqrt();
        if len > 1e-6 {
            (sx * ox / len, sy * oy / len)
        } else {
            (0.0, 0.0)
        }
    } else if qx >= qy {
        (sx, 0.0)
    } else {
        (0.0, sy)
    }
}

/// Circular meniscus height curve: `1 - sqrt(1 - x^2)` for `x in [0, 1]`.
#[inline]
pub fn circle_map(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    1.0 - (1.0 - t * t).max(0.0).sqrt()
}

/// Computes the refracted sample displacement `(dx, dy)` in pixels at surface coord `(px, py)`.
pub fn refraction_offset(px: f32, py: f32, p: &LensParams) -> (f32, f32) {
    if p.width <= 1.0 || p.height <= 1.0 || p.refraction_height <= 0.0 || p.refraction_amount == 0.0
    {
        return (0.0, 0.0);
    }
    let hx = p.width * 0.5;
    let hy = p.height * 0.5;
    let cx = px - hx;
    let cy = py - hy;
    let radii = p.radii.clamped(p.width, p.height);
    let r = radius_at(cx, cy, radii);

    let sd = sd_rounded_rect(cx, cy, hx, hy, r);
    if -sd >= p.refraction_height || sd > 0.0 {
        return (0.0, 0.0);
    }
    let sd_clamped = sd.min(0.0);
    let d = circle_map(1.0 - (-sd_clamped / p.refraction_height)) * (-p.refraction_amount);
    let grad_r = (r * 1.5).min(hx.min(hy));
    let (mut gx, mut gy) = grad_sd_rounded_rect(cx, cy, hx, hy, grad_r);

    if p.depth_effect != 0.0 {
        let clen = (cx * cx + cy * cy).sqrt();
        if clen > 1e-6 {
            gx += p.depth_effect * (cx / clen);
            gy += p.depth_effect * (cy / clen);
        }
    }
    let glen = (gx * gx + gy * gy).sqrt();
    if glen > 1e-6 {
        (d * gx / glen, d * gy / glen)
    } else {
        (0.0, 0.0)
    }
}

/// Computes the diagonal chromatic dispersion vector `(ddx, ddy)` from `Kyant0/AndroidLiquidGlass`.
pub fn dispersion_offset(px: f32, py: f32, p: &LensParams) -> (f32, f32) {
    if p.chromatic_aberration <= 0.0 || p.width <= 1.0 || p.height <= 1.0 {
        return (0.0, 0.0);
    }
    let (dx, dy) = refraction_offset(px, py, p);
    let hx = p.width * 0.5;
    let hy = p.height * 0.5;
    let cx = px - hx;
    let cy = py - hy;
    let intensity = p.chromatic_aberration * ((cx * cy) / (hx * hy));
    (dx * intensity, dy * intensity)
}

/// Directional Fresnel specular highlight intensity along the rounded-rect rim.
pub fn highlight_intensity(
    px: f32,
    py: f32,
    width: f32,
    height: f32,
    radii: CornerRadii,
    angle_deg: f32,
    falloff: f32,
) -> f32 {
    let hx = width * 0.5;
    let hy = height * 0.5;
    let cx = px - hx;
    let cy = py - hy;
    let r = radius_at(cx, cy, radii.clamped(width, height));
    let grad_r = (r * 1.5).min(hx.min(hy));
    let (gx, gy) = grad_sd_rounded_rect(cx, cy, hx, hy, grad_r);
    let rad = angle_deg.to_radians();
    let d = gx * rad.cos() + gy * rad.sin();
    d.abs().powf(falloff.max(0.01))
}

/// Exact 4x5 ColorMatrix matching `Kyant0/AndroidLiquidGlass` `colorControlsColorFilter`.
/// Pass `brightness = 0.0, contrast = 1.0, saturation = 1.5` for standard Vibrancy.
pub fn vibrancy_color_matrix(brightness: f32, contrast: f32, saturation: f32) -> [f32; 20] {
    let inv_sat = 1.0 - saturation;
    let r = 0.213 * inv_sat;
    let g = 0.715 * inv_sat;
    let b = 0.072 * inv_sat;
    let c = contrast;
    let t = (0.5 - c * 0.5 + brightness) * 255.0;
    let cr = c * r;
    let cg = c * g;
    let cb = c * b;
    let cs = c * saturation;
    [
        cr + cs, cg, cb, 0.0, t,
        cr, cg + cs, cb, 0.0, t,
        cr, cg, cb + cs, 0.0, t,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_center_has_zero_refraction_and_rim_curves_inward() {
        let p = LensParams {
            width: 320.0,
            height: 64.0,
            radii: CornerRadii::capsule(320.0, 64.0),
            refraction_height: 18.0,
            refraction_amount: 20.0,
            depth_effect: 0.0,
            chromatic_aberration: 1.0,
        };
        assert_eq!(refraction_offset(160.0, 32.0, &p), (0.0, 0.0));
        let (dx, dy) = refraction_offset(160.0, 2.0, &p);
        assert!(dx.abs() < 1e-3);
        assert!(dy > 1.0, "top meniscus should refract inward (dy={dy})");
    }

    #[test]
    fn vibrancy_matrix_preserves_luma_row_sum() {
        let m = vibrancy_color_matrix(0.0, 1.0, 1.5);
        for row in 0..3 {
            let sum = m[row * 5] + m[row * 5 + 1] + m[row * 5 + 2];
            assert!((sum - 1.0).abs() < 1e-5, "row {row} sum={sum}");
        }
        assert_eq!(m[18], 1.0);
    }
}
