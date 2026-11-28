//! Pure-Rust Liquid Glass Bottom Navigation Bar for Doona WebUI (`/ui/`).
//!
//! Powered by our shared `liquid-glass` crate (1:1 port of `@Kyant0/AndroidLiquidGlass`):
//! - Generates Kyant0 SDF rounded-capsule displacement maps in Rust (`data:image/bmp;base64,...`)
//!   for GPU-accelerated SVG `<feDisplacementMap>` lens refraction + RGB chromatic dispersion.
//! - Steps `LiquidTabBarPhysics` via `requestAnimationFrame` on tap or horizontal finger/mouse
//!   drag with velocity-coupled liquid droplet stretch/squash and zero CPU usage once settled.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use liquid_glass::{LiquidTabBarPhysics, LiquidTabFrame, build_svg_lens_assets};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::state::{HUBS, use_app_state};
use crate::ui::icons;

fn hub_index_for_route(route: &str) -> usize {
    HUBS.iter()
        .position(|h| h.pages.contains(&route))
        .unwrap_or(0)
}

#[component]
pub fn LiquidHubBar() -> impl IntoView {
    let state = use_app_state();
    let lens = StoredValue::new(build_svg_lens_assets());
    let tabs_count = HUBS.len();
    let initial_idx = hub_index_for_route(&state.route.get_untracked());

    let physics = Rc::new(RefCell::new(LiquidTabBarPhysics::new(
        tabs_count,
        initial_idx,
    )));
    let frame_sig = RwSignal::new({
        let mut p = physics.borrow_mut();
        p.step(1.0 / 60.0, 360.0, 1.0)
    });
    let raf_active = Rc::new(RefCell::new(false));
    let bar_width = Rc::new(RefCell::new(360.0f32));
    let drag_state = Rc::new(RefCell::new(None::<(i32, f32, bool)>)); // (pointer_id, last_x, moved)
    let bar_ref = NodeRef::<leptos::html::Div>::new();

    // Helper to wake the 60/120fps requestAnimationFrame stepper (sleeps automatically when settled).
    let wake_raf: Rc<dyn Fn()> = {
        let physics = physics.clone();
        let raf_active = raf_active.clone();
        let bar_width = bar_width.clone();
        Rc::new(move || {
            if *raf_active.borrow() {
                return;
            }
            *raf_active.borrow_mut() = true;
            let phys_c = physics.clone();
            let active_c = raf_active.clone();
            let width_c = bar_width.clone();
            let last_ms = Rc::new(RefCell::new(None::<f64>));

            type RafClosureSlot = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
            let cb_slot: RafClosureSlot = Rc::new(RefCell::new(None));
            let cb_slot_inner = cb_slot.clone();

            *cb_slot.borrow_mut() = Some(Closure::wrap(Box::new(move |now_ms: f64| {
                let dt = {
                    let mut prev = last_ms.borrow_mut();
                    let dt = match *prev {
                        Some(p) => ((now_ms - p) / 1000.0).clamp(0.001, 0.05) as f32,
                        None => 1.0 / 60.0,
                    };
                    *prev = Some(now_ms);
                    dt
                };
                let w = (*width_c.borrow()).max(160.0);
                let frame: LiquidTabFrame = phys_c.borrow_mut().step(dt, w, 1.0);
                frame_sig.set(frame);
                if frame.settled {
                    *active_c.borrow_mut() = false;
                    let _ = cb_slot_inner.borrow_mut().take();
                } else if let Some(win) = web_sys::window() {
                    if let Some(ref closure) = *cb_slot_inner.borrow() {
                        let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
                    }
                } else {
                    *active_c.borrow_mut() = false;
                }
            }) as Box<dyn FnMut(f64)>));

            if let Some(win) = web_sys::window() {
                if let Some(ref closure) = *cb_slot.borrow() {
                    let _ = win.request_animation_frame(closure.as_ref().unchecked_ref());
                }
            } else {
                *raf_active.borrow_mut() = false;
            }
        })
    };

    // Sync droplet whenever route changes externally (sidebar click, Ctrl+K, browser back/forward).
    {
        let physics = physics.clone();
        let wake = wake_raf.clone();
        Effect::new(move |_| {
            let route = state.route.get();
            let idx = hub_index_for_route(&route);
            physics.borrow_mut().sync_external_tab(idx);
            wake();
        });
    }

    let on_pointer_down = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let bar_width = bar_width.clone();
        let wake = wake_raf.clone();
        move |ev: web_sys::PointerEvent| {
            if ev.button() != 0 {
                return;
            }
            if let Some(el) = bar_ref.get() {
                let rect = el.get_bounding_client_rect();
                if rect.width() > 40.0 {
                    *bar_width.borrow_mut() = rect.width() as f32;
                }
                let _ = el.set_pointer_capture(ev.pointer_id());
            }
            *drag_state.borrow_mut() = Some((ev.pointer_id(), ev.client_x() as f32, false));
            physics.borrow_mut().on_drag_start();
            wake();
        }
    };

    let on_pointer_move = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let bar_width = bar_width.clone();
        let wake = wake_raf.clone();
        move |ev: web_sys::PointerEvent| {
            let mut ds = drag_state.borrow_mut();
            let Some((pid, ref mut last_x, ref mut moved)) = *ds else {
                return;
            };
            if ev.pointer_id() != pid {
                return;
            }
            let x = ev.client_x() as f32;
            let dx = x - *last_x;
            if dx.abs() > 1.5 || *moved {
                *moved = true;
                *last_x = x;
                let w = (*bar_width.borrow()).max(160.0);
                let tab_w = ((w - 8.0) / (tabs_count as f32)).max(24.0);
                physics.borrow_mut().on_drag_delta(dx, tab_w);
                wake();
            }
        }
    };

    let finish_drag = {
        let physics = physics.clone();
        let drag_state = drag_state.clone();
        let wake = wake_raf.clone();
        Rc::new(move |ev: web_sys::PointerEvent| {
            let taken = {
                let mut ds = drag_state.borrow_mut();
                if ds.as_ref().is_some_and(|(pid, _, _)| *pid == ev.pointer_id()) {
                    ds.take()
                } else {
                    None
                }
            };
            let Some((_, _, moved)) = taken else {
                return;
            };
            if let Some(el) = bar_ref.get() {
                let _ = el.release_pointer_capture(ev.pointer_id());
            }
            let chosen = physics.borrow_mut().on_drag_end();
            wake();
            if moved && let Some(hub) = HUBS.get(chosen) {
                let cur_idx = hub_index_for_route(&state.route.get_untracked());
                if chosen != cur_idx {
                    state.navigate(hub.pages[0], None);
                }
            }
        })
    };
    let on_pointer_up = {
        let f = finish_drag.clone();
        move |ev: web_sys::PointerEvent| f(ev)
    };
    let on_pointer_cancel = move |ev: web_sys::PointerEvent| finish_drag(ev);

    let bar_style = move || {
        let f = frame_sig.get();
        format!(
            "transform:translate3d({:.2}px,0,0) scale({:.4});--lg-hl-x:{:.2}%;--lg-press:{:.3};",
            f.panel_offset_x,
            f.panel_scale,
            f.highlight_center_frac * 100.0,
            f.press_progress
        )
    };

    let droplet_style = move || {
        let f = frame_sig.get();
        let pct_w = 100.0 / (tabs_count as f32);
        let pct_x = f.value * 100.0;
        format!(
            "width:calc({pct_w:.4}% - 2px);transform:translate3d({pct_x:.2}%,0,0) scale({:.4},{:.4});--lg-press:{:.3};",
            f.droplet_scale_x, f.droplet_scale_y, f.press_progress
        )
    };

    view! {
        // Hidden pure-Rust Kyant0 SDF refraction + RGB chromatic dispersion SVG filter definitions.
        <svg class="rp-lg-defs" aria-hidden="true" width="0" height="0" style="position:fixed;width:0;height:0;pointer-events:none;">
            <defs>
                {move || {
                    let a = lens.get_value();
                    let bar_scale = format!("{:.2}", a.bar_scale);
                    let r_scale = format!("{:.2}", a.droplet_red_scale);
                    let g_scale = format!("{:.2}", a.droplet_scale);
                    let b_scale = format!("{:.2}", a.droplet_blue_scale);
                    view! {
                        <filter id="doona-lg-bar" x="-5%" y="-5%" width="110%" height="110%" color-interpolation-filters="sRGB">
                            <feImage href=a.bar_map_uri.clone() result="barDisp" preserveAspectRatio="none" />
                            <feDisplacementMap
                                in="SourceGraphic"
                                in2="barDisp"
                                scale=bar_scale
                                xChannelSelector="R"
                                yChannelSelector="G"
                                result="refracted"
                            />
                            <feColorMatrix in="refracted" type="saturate" values="1.45" />
                        </filter>
                        <filter id="doona-lg-droplet" x="-10%" y="-10%" width="120%" height="120%" color-interpolation-filters="sRGB">
                            <feImage href=a.droplet_map_uri.clone() result="dropDisp" preserveAspectRatio="none" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=r_scale xChannelSelector="R" yChannelSelector="G" result="dispR" />
                            <feColorMatrix in="dispR" type="matrix" values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0" result="chanR" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=g_scale xChannelSelector="R" yChannelSelector="G" result="dispG" />
                            <feColorMatrix in="dispG" type="matrix" values="0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0" result="chanG" />
                            <feDisplacementMap in="SourceGraphic" in2="dropDisp" scale=b_scale xChannelSelector="R" yChannelSelector="G" result="dispB" />
                            <feColorMatrix in="dispB" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0" result="chanB" />
                            <feBlend in="chanR" in2="chanG" mode="screen" result="rg" />
                            <feBlend in="rg" in2="chanB" mode="screen" result="rgb" />
                            <feColorMatrix in="rgb" type="saturate" values="1.35" />
                        </filter>
                    }
                }}
            </defs>
        </svg>

        <nav class="rp-lg-dock-wrap" aria-label=move || state.t("shell.hubs")>
            <div
                node_ref=bar_ref
                class="rp-lg-dock"
                style=bar_style
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
                on:pointercancel=on_pointer_cancel
            >
                <div class="rp-lg-surface" aria-hidden="true"></div>
                <div class="rp-lg-track">
                    <div class="rp-lg-droplet" style=droplet_style aria-hidden="true"></div>
                    {HUBS.iter().enumerate().map(|(idx, hub)| {
                        let first = hub.pages[0];
                        let href = format!("#{first}");
                        let phys_tap = physics.clone();
                        let wake_tap = wake_raf.clone();
                        let on_tap = move |ev: web_sys::MouseEvent| {
                            ev.prevent_default();
                            phys_tap.borrow_mut().animate_to_tab(idx);
                            wake_tap();
                            state.navigate(first, None);
                        };
                        let tab_style = move || {
                            let f = frame_sig.get();
                            let focus = (1.0 - (f.value - (idx as f32)).abs()).clamp(0.0, 1.0);
                            let sc = 1.0 + 0.05 * focus + 0.05 * focus * f.press_progress;
                            format!("transform:scale({sc:.4});")
                        };
                        view! {
                            <a
                                class="rp-lg-tab"
                                href=href
                                style=tab_style
                                aria-current=move || {
                                    let cur = state.route.get();
                                    hub.pages.contains(&cur.as_str()).then_some("page")
                                }
                                on:click=on_tap
                            >
                                <span class="rp-lg-tab-icon">{icons::hub_icon(hub.id)}</span>
                                <span class="rp-lg-tab-label">{move || state.t(hub.title_key)}</span>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>
        </nav>
    }
}
