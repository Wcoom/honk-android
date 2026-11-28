//! Pure-Rust Liquid Glass optics, AGSL shader generator, SVG displacement-map lens builder,
//! damped spring droplet physics, and iOS navigation motion presets.
//!
//! Optical refraction, 7-band chromatic dispersion, Fresnel edge highlight, vibrancy color
//! matrix, and damped-drag droplet physics are faithfully ported to pure Rust from
//! `Kyant0/AndroidLiquidGlass` (Apache-2.0, Copyright 2025 Kyant) so both the Android APK
//! (`libboxproxy_app.so`) and the Doona WASM WebUI (`/ui/` & `/ui/lite/`) share one
//! deterministic pure-Rust physics & optical kernel.

pub mod agsl;
pub mod ios_motion;
pub mod sdf;
pub mod spring;
pub mod svg_lens;

pub use agsl::AgslShaderBundle;
pub use ios_motion::{DesignGridTokens, IosMotionSpec, NavDirection, contrast_ratio};
pub use sdf::{CornerRadii, LensParams, vibrancy_color_matrix};
pub use spring::{LiquidTabBarPhysics, LiquidTabFrame, SpringChannel, SpringSpec};
pub use svg_lens::{SvgLensAssets, build_svg_lens_assets};
