//! AGSL runtime shader bundle generated in pure Rust for Android 13+ (`RuntimeShader`).
//!
//! Optical shader algorithms are adapted from `Kyant0/AndroidLiquidGlass` (Apache-2.0,
//! Copyright 2025 Kyant) and served directly from `libboxproxy_app.so` via JNI so all
//! optical math and motion parameters stay single-source-of-truth in Rust.

use serde::{Deserialize, Serialize};

use crate::ios_motion::{DesignGridTokens, IosMotionSpec};
use crate::sdf::vibrancy_color_matrix;

const ROUNDED_RECT_SDF_AGSL: &str = r#"
float radiusAt(float2 centeredCoord, float4 radii) {
    if (centeredCoord.x >= 0.0) {
        if (centeredCoord.y <= 0.0) return radii.y;
        else return radii.z;
    } else {
        if (centeredCoord.y <= 0.0) return radii.x;
        else return radii.w;
    }
}

float sdRoundedRect(float2 coord, float2 halfSize, float radius) {
    float2 cornerCoord = abs(coord) - (halfSize - float2(radius));
    float outside = length(max(cornerCoord, 0.0)) - radius;
    float inside = min(max(cornerCoord.x, cornerCoord.y), 0.0);
    return outside + inside;
}

float2 gradSdRoundedRect(float2 coord, float2 halfSize, float radius) {
    float2 cornerCoord = abs(coord) - (halfSize - float2(radius));
    if (cornerCoord.x >= 0.0 || cornerCoord.y >= 0.0) {
        return sign(coord) * normalize(max(cornerCoord, 0.0));
    } else {
        float gradX = step(cornerCoord.y, cornerCoord.x);
        return sign(coord) * float2(gradX, 1.0 - gradX);
    }
}
"#;

pub fn rounded_rect_refraction_agsl() -> String {
    format!(
        r#"
uniform shader content;
uniform float2 size;
uniform float2 offset;
uniform float4 cornerRadii;
uniform float refractionHeight;
uniform float refractionAmount;
uniform float depthEffect;
{ROUNDED_RECT_SDF_AGSL}
float circleMap(float x) {{
    return 1.0 - sqrt(max(0.0, 1.0 - x * x));
}}

half4 main(float2 coord) {{
    float2 halfSize = size * 0.5;
    float2 centeredCoord = (coord + offset) - halfSize;
    float radius = radiusAt(centeredCoord, cornerRadii);

    float sd = sdRoundedRect(centeredCoord, halfSize, radius);
    if (-sd >= refractionHeight) {{
        return content.eval(coord);
    }}
    sd = min(sd, 0.0);

    float d = circleMap(1.0 - -sd / max(refractionHeight, 0.001)) * refractionAmount;
    float gradRadius = min(radius * 1.5, min(halfSize.x, halfSize.y));
    float2 baseGrad = gradSdRoundedRect(centeredCoord, halfSize, gradRadius);
    float2 radial = length(centeredCoord) > 0.001 ? normalize(centeredCoord) : float2(0.0);
    float2 grad = normalize(baseGrad + depthEffect * radial);

    return content.eval(coord + d * grad);
}}
"#
    )
}

pub fn rounded_rect_refraction_dispersion_agsl() -> String {
    format!(
        r#"
uniform shader content;
uniform float2 size;
uniform float2 offset;
uniform float4 cornerRadii;
uniform float refractionHeight;
uniform float refractionAmount;
uniform float depthEffect;
uniform float chromaticAberration;
{ROUNDED_RECT_SDF_AGSL}
float circleMap(float x) {{
    return 1.0 - sqrt(max(0.0, 1.0 - x * x));
}}

half4 main(float2 coord) {{
    float2 halfSize = size * 0.5;
    float2 centeredCoord = (coord + offset) - halfSize;
    float radius = radiusAt(centeredCoord, cornerRadii);

    float sd = sdRoundedRect(centeredCoord, halfSize, radius);
    if (-sd >= refractionHeight) {{
        return content.eval(coord);
    }}
    sd = min(sd, 0.0);

    float d = circleMap(1.0 - -sd / max(refractionHeight, 0.001)) * refractionAmount;
    float gradRadius = min(radius * 1.5, min(halfSize.x, halfSize.y));
    float2 baseGrad = gradSdRoundedRect(centeredCoord, halfSize, gradRadius);
    float2 radial = length(centeredCoord) > 0.001 ? normalize(centeredCoord) : float2(0.0);
    float2 grad = normalize(baseGrad + depthEffect * radial);

    float2 refractedCoord = coord + d * grad;
    float dispersionIntensity = chromaticAberration * ((centeredCoord.x * centeredCoord.y) / max(halfSize.x * halfSize.y, 1.0));
    float2 dispersedCoord = d * grad * dispersionIntensity;

    half4 color = half4(0.0);

    half4 red = content.eval(refractedCoord + dispersedCoord);
    color.r += red.r / 3.5;
    color.a += red.a / 7.0;

    half4 orange = content.eval(refractedCoord + dispersedCoord * (2.0 / 3.0));
    color.r += orange.r / 3.5;
    color.g += orange.g / 7.0;
    color.a += orange.a / 7.0;

    half4 yellow = content.eval(refractedCoord + dispersedCoord * (1.0 / 3.0));
    color.r += yellow.r / 3.5;
    color.g += yellow.g / 3.5;
    color.a += yellow.a / 7.0;

    half4 green = content.eval(refractedCoord);
    color.g += green.g / 3.5;
    color.a += green.a / 7.0;

    half4 cyan = content.eval(refractedCoord - dispersedCoord * (1.0 / 3.0));
    color.g += cyan.g / 3.5;
    color.b += cyan.b / 3.0;
    color.a += cyan.a / 7.0;

    half4 blue = content.eval(refractedCoord - dispersedCoord * (2.0 / 3.0));
    color.b += blue.b / 3.0;
    color.a += blue.a / 7.0;

    half4 purple = content.eval(refractedCoord - dispersedCoord);
    color.r += purple.r / 7.0;
    color.b += purple.b / 3.0;
    color.a += purple.a / 7.0;

    return color;
}}
"#
    )
}

pub fn default_highlight_agsl() -> String {
    format!(
        r#"
uniform float2 size;
uniform float4 cornerRadii;
layout(color) uniform half4 color;
uniform float angle;
uniform float falloff;
{ROUNDED_RECT_SDF_AGSL}
half4 main(float2 coord) {{
    float2 halfSize = size * 0.5;
    float2 centeredCoord = coord - halfSize;
    float radius = radiusAt(centeredCoord, cornerRadii);

    float gradRadius = min(radius * 1.5, min(halfSize.x, halfSize.y));
    float2 grad = gradSdRoundedRect(centeredCoord, halfSize, gradRadius);
    float2 normal = float2(cos(angle), sin(angle));
    float d = dot(grad, normal);
    float intensity = pow(abs(d), falloff);
    return color * intensity;
}}
"#
    )
}

pub fn ambient_highlight_agsl() -> String {
    format!(
        r#"
uniform float2 size;
uniform float4 cornerRadii;
uniform float angle;
uniform float falloff;
{ROUNDED_RECT_SDF_AGSL}
half4 main(float2 coord) {{
    float2 halfSize = size * 0.5;
    float2 centeredCoord = coord - halfSize;
    float radius = radiusAt(centeredCoord, cornerRadii);

    float gradRadius = min(radius * 1.5, min(halfSize.x, halfSize.y));
    float2 grad = gradSdRoundedRect(centeredCoord, halfSize, gradRadius);
    float2 normal = float2(cos(angle), sin(angle));
    float d = dot(grad, normal);
    float intensity = pow(abs(d), falloff);
    float t = step(0.0, d);
    return half4(t, t, t, 1.0) * intensity;
}}
"#
    )
}

/// Complete AGSL + vibrancy + iOS motion + 8pt design-grid payload returned over JNI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgslShaderBundle {
    pub refraction: String,
    pub refraction_dispersion: String,
    pub default_highlight: String,
    pub ambient_highlight: String,
    pub vibrancy_matrix: [f32; 20],
    pub ios_motion: IosMotionSpec,
    pub design_grid: DesignGridTokens,
}

impl AgslShaderBundle {
    pub fn build() -> Self {
        Self {
            refraction: rounded_rect_refraction_agsl(),
            refraction_dispersion: rounded_rect_refraction_dispersion_agsl(),
            default_highlight: default_highlight_agsl(),
            ambient_highlight: ambient_highlight_agsl(),
            vibrancy_matrix: vibrancy_color_matrix(0.0, 1.0, 1.5),
            ios_motion: IosMotionSpec::default(),
            design_grid: DesignGridTokens::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agsl_bundle_serializes_cleanly_with_all_entry_points() {
        let bundle = AgslShaderBundle::build();
        assert!(bundle.refraction.contains("half4 main(float2 coord)"));
        assert!(bundle.refraction_dispersion.contains("chromaticAberration"));
        assert!(bundle.default_highlight.contains("layout(color) uniform half4 color"));
        assert!(bundle.ambient_highlight.contains("step(0.0, d)"));
        let json = serde_json::to_string(&bundle).expect("serialize bundle");
        assert!(json.contains("vibrancy_matrix"));
    }
}
