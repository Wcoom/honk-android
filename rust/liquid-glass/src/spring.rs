//! Pure-Rust damped spring oscillator and `LiquidTabBarPhysics` (1:1 port of
//! `Kyant0/AndroidLiquidGlass`'s `DampedDragAnimation` + `LiquidBottomTabs` velocity stretch).

use serde::{Deserialize, Serialize};

/// Second-order spring specification matching Compose `spring(dampingRatio, stiffness)`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpringSpec {
    pub damping_ratio: f32,
    pub stiffness: f32,
    pub epsilon: f32,
}

impl SpringSpec {
    pub const fn new(damping_ratio: f32, stiffness: f32, epsilon: f32) -> Self {
        Self {
            damping_ratio,
            stiffness,
            epsilon,
        }
    }
}

/// Stateful 1D damped spring channel stepped deterministically with sub-stepping.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpringChannel {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    pub spec: SpringSpec,
}

impl SpringChannel {
    pub const fn new(initial: f32, spec: SpringSpec) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            spec,
        }
    }

    pub fn snap_to(&mut self, v: f32) {
        self.value = v;
        self.target = v;
        self.velocity = 0.0;
    }

    pub fn set_target(&mut self, t: f32) {
        self.target = t;
    }

    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() <= self.spec.epsilon
            && self.velocity.abs() <= self.spec.epsilon * 20.0
    }

    /// Semi-implicit Euler integration with fixed <= 1/240s substeps so high stiffness
    /// (e.g. `k = 1000`) stays unconditionally stable on 60Hz, 90Hz, and 120Hz displays.
    pub fn step(&mut self, dt_secs: f32) -> f32 {
        let clamped_dt = dt_secs.clamp(0.0, 0.05);
        if clamped_dt <= 0.0 || self.is_settled() {
            if self.is_settled() {
                self.value = self.target;
                self.velocity = 0.0;
            }
            return self.value;
        }
        let max_step = 1.0 / 240.0;
        let steps = ((clamped_dt / max_step).ceil() as usize).max(1);
        let h = clamped_dt / (steps as f32);
        let k = self.spec.stiffness;
        let c = 2.0 * self.spec.damping_ratio * k.sqrt();

        for _ in 0..steps {
            let displacement = self.value - self.target;
            let accel = -k * displacement - c * self.velocity;
            self.velocity += accel * h;
            self.value += self.velocity * h;
        }

        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
        }
        self.value
    }
}

/// Complete per-frame render parameters for the Liquid Glass Bottom Bar & Droplet Pill.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiquidTabFrame {
    /// Continuous tab index (`0.0 ..= (tabs_count - 1)`).
    pub value: f32,
    /// `0.0` at rest, `1.0` when pressed/dragging/gliding.
    pub press_progress: f32,
    /// Horizontal rubber-band translation of the whole glass dock (px).
    pub panel_offset_x: f32,
    /// Subtle dock swell scale on press (`1.0` -> `1.0 + 16dp/width`).
    pub panel_scale: f32,
    /// Horizontal droplet scale (includes press expansion + velocity stretch).
    pub droplet_scale_x: f32,
    /// Vertical droplet scale (includes press expansion + volume-preserving squash).
    pub droplet_scale_y: f32,
    /// Droplet SDF meniscus refraction height (px).
    pub droplet_refraction_height: f32,
    /// Droplet SDF refraction displacement amount (px).
    pub droplet_refraction_amount: f32,
    /// Dock SDF refraction height (px).
    pub bar_refraction_height: f32,
    /// Dock SDF refraction amount (px).
    pub bar_refraction_amount: f32,
    /// Directional Fresnel highlight alpha (`0..1`).
    pub highlight_alpha: f32,
    /// Outer soft shadow alpha (`0..1`).
    pub shadow_alpha: f32,
    /// Inner caustic glow blur radius (px).
    pub inner_shadow_radius: f32,
    /// Active tab icon/label subtle magnification (`1.0` -> `1.12`).
    pub tab_content_scale: f32,
    /// Normalized `[0, 1]` center of the interactive highlight along the bar width.
    pub highlight_center_frac: f32,
    /// `true` once every spring channel has reached rest (frame loop can sleep).
    pub settled: bool,
}

/// Faithful pure-Rust implementation of Kyant0's `DampedDragAnimation` + `LiquidBottomTabs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiquidTabBarPhysics {
    pub tabs_count: usize,
    pub selected_index: usize,
    pub dragging: bool,
    pub release_pending: bool,
    pub value: SpringChannel,
    pub velocity: SpringChannel,
    pub press_progress: SpringChannel,
    pub scale_x: SpringChannel,
    pub scale_y: SpringChannel,
    pub panel_drag_px: SpringChannel,
}

impl LiquidTabBarPhysics {
    pub const PRESSED_SCALE: f32 = 78.0 / 56.0;

    pub fn new(tabs_count: usize, initial_index: usize) -> Self {
        let count = tabs_count.max(1);
        let idx = initial_index.min(count - 1);
        Self {
            tabs_count: count,
            selected_index: idx,
            dragging: false,
            release_pending: false,
            value: SpringChannel::new(idx as f32, SpringSpec::new(1.0, 1000.0, 0.001)),
            velocity: SpringChannel::new(0.0, SpringSpec::new(0.5, 300.0, 0.01)),
            press_progress: SpringChannel::new(0.0, SpringSpec::new(1.0, 1000.0, 0.001)),
            scale_x: SpringChannel::new(1.0, SpringSpec::new(0.6, 250.0, 0.001)),
            scale_y: SpringChannel::new(1.0, SpringSpec::new(0.7, 250.0, 0.001)),
            panel_drag_px: SpringChannel::new(0.0, SpringSpec::new(1.0, 300.0, 0.05)),
        }
    }

    pub fn set_tabs_count(&mut self, tabs_count: usize, current_index: usize) {
        let count = tabs_count.max(1);
        let idx = current_index.min(count - 1);
        if self.tabs_count != count {
            self.tabs_count = count;
            self.selected_index = idx;
            self.value.snap_to(idx as f32);
            self.velocity.snap_to(0.0);
        }
    }

    /// Triggers a tap transition (brief press swell + spring glide + automatic release).
    pub fn animate_to_tab(&mut self, index: usize) {
        let target = index.min(self.tabs_count.saturating_sub(1));
        self.selected_index = target;
        self.dragging = false;
        let dist = (target as f32 - self.value.value).abs();
        self.value.set_target(target as f32);
        if dist > 0.05 {
            self.press_progress.set_target(1.0);
            self.scale_x.set_target(1.16);
            self.scale_y.set_target(1.12);
            self.release_pending = true;
        }
    }

    pub fn sync_external_tab(&mut self, index: usize) {
        let target = index.min(self.tabs_count.saturating_sub(1));
        if !self.dragging && target != self.selected_index {
            self.animate_to_tab(target);
        }
    }

    pub fn on_drag_start(&mut self) {
        self.dragging = true;
        self.release_pending = false;
        self.press_progress.set_target(1.0);
        self.scale_x.set_target(Self::PRESSED_SCALE);
        self.scale_y.set_target(Self::PRESSED_SCALE);
    }

    pub fn on_drag_delta(&mut self, dx_px: f32, tab_width_px: f32) {
        if !self.dragging || tab_width_px <= 1.0 {
            return;
        }
        let max_idx = (self.tabs_count.saturating_sub(1)) as f32;
        let next = (self.value.target + dx_px / tab_width_px).clamp(0.0, max_idx);
        self.value.set_target(next);
        self.panel_drag_px.snap_to(self.panel_drag_px.value + dx_px);
    }

    /// Finishes drag, snaps target to nearest tab index, and returns the selected tab index.
    pub fn on_drag_end(&mut self) -> usize {
        self.dragging = false;
        let max_idx = self.tabs_count.saturating_sub(1);
        let nearest = (self.value.target.round() as isize).clamp(0, max_idx as isize) as usize;
        self.selected_index = nearest;
        self.value.set_target(nearest as f32);
        self.panel_drag_px.set_target(0.0);
        self.release_pending = true;
        nearest
    }

    pub fn step(&mut self, dt_secs: f32, bar_width_px: f32, density: f32) -> LiquidTabFrame {
        let prev_val = self.value.value;
        self.value.step(dt_secs);
        let span = (self.tabs_count.saturating_sub(1)).max(1) as f32;
        let dt_safe = dt_secs.clamp(1.0 / 240.0, 0.05);
        let inst_vel = (self.value.value - prev_val) / dt_safe / span;
        self.velocity.set_target(if self.dragging || !self.value.is_settled() {
            inst_vel
        } else {
            0.0
        });
        self.velocity.step(dt_secs);

        // Once the gliding droplet arrives within 3% of its target tab, release press swell.
        if self.release_pending && !self.dragging {
            let threshold = span * 0.03 + 0.02;
            if (self.value.value - self.value.target).abs() <= threshold {
                self.release_pending = false;
                self.press_progress.set_target(0.0);
                self.scale_x.set_target(1.0);
                self.scale_y.set_target(1.0);
            }
        }

        self.press_progress.step(dt_secs);
        self.scale_x.step(dt_secs);
        self.scale_y.step(dt_secs);
        if !self.dragging {
            self.panel_drag_px.step(dt_secs);
        }

        let d = density.max(1.0);
        let progress = self.press_progress.value.clamp(0.0, 1.0);
        let width_safe = bar_width_px.max(120.0);

        // Kyant0 panelOffset: 4dp * sign(f) * easeOut(|f|)
        let frac = (self.panel_drag_px.value / width_safe).clamp(-1.0, 1.0);
        let abs_f = frac.abs();
        let ease_out = 1.0 - (1.0 - abs_f) * (1.0 - abs_f);
        let panel_offset_x = 4.0 * d * frac.signum() * ease_out;

        // Subtle dock scale swell on press (up to +10dp / width).
        let panel_scale = 1.0 + (10.0 * d / width_safe) * progress;

        // Kyant0 velocity-coupled liquid stretch & squash:
        // scaleX /= 1 - clamp(v/10 * 0.75, -0.2, 0.2)
        // scaleY *= 1 - clamp(v/10 * 0.25, -0.2, 0.2)
        let v_norm = self.velocity.value / 10.0;
        let stretch_x = (v_norm * 0.75).clamp(-0.2, 0.2);
        let squash_y = (v_norm * 0.25).clamp(-0.2, 0.2);
        let droplet_scale_x = self.scale_x.value / (1.0 - stretch_x.abs());
        let droplet_scale_y = self.scale_y.value * (1.0 - squash_y.abs());

        // Keep subtle optical presence at rest and amplify during drag/glide.
        let droplet_refraction_height = (4.5 + 8.5 * progress) * d;
        let droplet_refraction_amount = (6.0 + 10.0 * progress) * d;
        let bar_refraction_height = 20.0 * d;
        let bar_refraction_amount = 22.0 * d;

        let highlight_alpha = 0.55 + 0.45 * progress;
        let shadow_alpha = 0.28 + 0.52 * progress;
        let inner_shadow_radius = (3.0 + 6.0 * progress) * d;
        let tab_content_scale = 1.0 + 0.14 * progress;
        let highlight_center_frac =
            ((self.value.value + 0.5) / (self.tabs_count as f32)).clamp(0.0, 1.0);

        let settled = !self.dragging
            && !self.release_pending
            && self.value.is_settled()
            && self.velocity.is_settled()
            && self.press_progress.is_settled()
            && self.scale_x.is_settled()
            && self.scale_y.is_settled()
            && self.panel_drag_px.is_settled();

        LiquidTabFrame {
            value: self.value.value,
            press_progress: progress,
            panel_offset_x,
            panel_scale,
            droplet_scale_x,
            droplet_scale_y,
            droplet_refraction_height,
            droplet_refraction_amount,
            bar_refraction_height,
            bar_refraction_amount,
            highlight_alpha,
            shadow_alpha,
            inner_shadow_radius,
            tab_content_scale,
            highlight_center_frac,
            settled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_transition_glides_stretches_and_settles_at_target() {
        let mut phys = LiquidTabBarPhysics::new(4, 0);
        phys.animate_to_tab(3);
        let mut saw_stretch = false;
        let mut last = phys.step(1.0 / 60.0, 360.0, 3.0);
        for _ in 0..180 {
            last = phys.step(1.0 / 60.0, 360.0, 3.0);
            if last.droplet_scale_x > 1.05 {
                saw_stretch = true;
            }
            if last.settled {
                break;
            }
        }
        assert!(saw_stretch, "droplet should visibly stretch while gliding");
        assert!(last.settled, "physics must settle cleanly");
        assert!((last.value - 3.0).abs() < 0.01);
        assert!(last.press_progress < 0.01);
    }

    #[test]
    fn drag_and_release_snaps_to_nearest_tab() {
        let mut phys = LiquidTabBarPhysics::new(4, 0);
        phys.on_drag_start();
        phys.on_drag_delta(190.0, 90.0);
        let frame_mid = phys.step(1.0 / 60.0, 360.0, 2.0);
        assert!(frame_mid.panel_offset_x > 0.0);
        let selected = phys.on_drag_end();
        assert_eq!(selected, 2);
    }
}
