//! iOS-inspired navigation motion presets and 8pt UI design-system tokens & contrast checks.

use serde::{Deserialize, Serialize};

/// Spatial navigation transition direction for iOS-like push/pop and horizontal tab transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavDirection {
    /// Deeper hierarchical push (incoming from right `+100%`, outgoing parallax `-28%`).
    Push,
    /// Hierarchical pop / back (current exits right `+100%`, underlying restores `-28% -> 0%`).
    Pop,
    /// Horizontal sibling tab switch moving right.
    TabForward,
    /// Horizontal sibling tab switch moving left.
    TabBackward,
    /// Subtle in-place crossfade (initial load or reduced motion).
    CrossFade,
}

impl NavDirection {
    pub const fn css_attr(self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Pop => "pop",
            Self::TabForward => "tab-forward",
            Self::TabBackward => "tab-backward",
            Self::CrossFade => "fade",
        }
    }
}

/// Canonical iOS UIKit / SwiftUI transition timing & spring constants.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IosMotionSpec {
    /// iOS native navigation cubic-bezier control points: `(0.32, 0.72, 0.0, 1.0)`.
    pub ios_ease: [f32; 4],
    pub nav_push_ms: u32,
    pub nav_pop_ms: u32,
    pub nav_parallax_ratio: f32,
    pub nav_scrim_alpha: f32,
    pub tab_switch_ms: u32,
    pub tab_shift_dp: f32,
    pub tab_scale_from: f32,
    pub sheet_enter_ms: u32,
    pub sheet_exit_ms: u32,
    pub sheet_scale_from: f32,
    pub press_scale: f32,
    pub switch_damping: f32,
    pub switch_stiffness: f32,
}

impl Default for IosMotionSpec {
    fn default() -> Self {
        Self {
            ios_ease: [0.32, 0.72, 0.0, 1.0],
            nav_push_ms: 340,
            nav_pop_ms: 300,
            nav_parallax_ratio: 0.28,
            nav_scrim_alpha: 0.14,
            tab_switch_ms: 230,
            tab_shift_dp: 18.0,
            tab_scale_from: 0.985,
            sheet_enter_ms: 280,
            sheet_exit_ms: 180,
            sheet_scale_from: 0.94,
            press_scale: 0.975,
            switch_damping: 0.78,
            switch_stiffness: 480.0,
        }
    }
}

/// 8pt spacing grid & typographic hierarchy tokens shared across APK and WebUI.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DesignGridTokens {
    pub space_xs: u16,
    pub space_sm: u16,
    pub space_md: u16,
    pub space_lg: u16,
    pub space_xl: u16,
    pub card_radius_dp: u16,
    pub dock_height_dp: u16,
    pub dock_clearance_dp: u16,
    pub large_title_tracking_em: f32,
}

impl Default for DesignGridTokens {
    fn default() -> Self {
        Self {
            space_xs: 4,
            space_sm: 8,
            space_md: 16,
            space_lg: 24,
            space_xl: 32,
            card_radius_dp: 20,
            dock_height_dp: 64,
            dock_clearance_dp: 104,
            large_title_tracking_em: -0.025,
        }
    }
}

/// Computes WCAG 2.1 relative luminance and contrast ratio between two `0xRRGGBB` colors.
pub fn contrast_ratio(fg_rgb: u32, bg_rgb: u32) -> f32 {
    let l1 = relative_luminance(fg_rgb);
    let l2 = relative_luminance(bg_rgb);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

fn relative_luminance(rgb: u32) -> f32 {
    let chan = |shift: u32| -> f32 {
        let s = (((rgb >> shift) & 0xFF) as f32) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * chan(16) + 0.7152 * chan(8) + 0.0722 * chan(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boxproxy_light_and_dark_palettes_meet_wcag_aa_contrast() {
        // Light palette: textPrimary #1A1C1E on card #FFFFFF and bg #F3F4F8
        assert!(contrast_ratio(0x1A1C1E, 0xFFFFFF) >= 15.0);
        assert!(contrast_ratio(0x5B616E, 0xFFFFFF) >= 5.5);
        assert!(contrast_ratio(0x455A8A, 0xFFFFFF) >= 6.5);
        // Dark palette: textPrimary #F2F4F8 on card #15171C
        assert!(contrast_ratio(0xF2F4F8, 0x15171C) >= 15.0);
        assert!(contrast_ratio(0x9BA2B0, 0x15171C) >= 6.5);
        assert!(contrast_ratio(0x7D96D4, 0x15171C) >= 5.5);
    }
}
