//! Shared Doona UI primitives matching upstream `.rp-*` CSS classes, inline SVG icons, sparklines, and modals.

use doona_common::format::sparkline_paths;
use leptos::prelude::*;

use crate::state::{HUBS, page_title_key, use_app_state};

#[component]
pub fn SparkSvg(
    values: Vec<f64>,
    color: &'static str,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 32.0)] height: f64,
) -> impl IntoView {
    let (line, area) = sparkline_paths(&values, width, height);
    let view_box = format!("0 0 {width} {height}");
    view! {
        <svg class="rp-spark" viewBox=view_box preserveAspectRatio="none" aria-hidden="true" style="width:100%;height:32px;display:block;">
            <path d=area fill=color fill-opacity="0.16" stroke="none" />
            <path d=line fill="none" stroke=color stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}

#[component]
pub fn DualAreaSvg(
    primary: Vec<f64>,
    secondary: Vec<f64>,
    color_a: &'static str,
    color_b: &'static str,
) -> impl IntoView {
    let width = 480.0;
    let height = 140.0;
    let (line_a, area_a) = sparkline_paths(&primary, width, height);
    let (line_b, area_b) = sparkline_paths(&secondary, width, height);
    view! {
        <svg viewBox="0 0 480 140" preserveAspectRatio="none" aria-hidden="true" style="width:100%;height:140px;display:block;">
            <path d=area_a fill=color_a fill-opacity="0.18" stroke="none" />
            <path d=line_a fill="none" stroke=color_a stroke-width="2" stroke-linecap="round" />
            <path d=area_b fill=color_b fill-opacity="0.14" stroke="none" />
            <path d=line_b fill="none" stroke=color_b stroke-width="1.8" stroke-linecap="round" stroke-dasharray="4 2" />
        </svg>
    }
}

#[component]
pub fn StatusLight(tone: &'static str, text: String) -> impl IntoView {
    let cls = format!("rp-light {tone}");
    view! {
        <span class=cls>
            <span class="rp-dot" aria-hidden="true"></span>
            <span>{text}</span>
        </span>
    }
}

#[component]
pub fn ToastViewport() -> impl IntoView {
    let state = use_app_state();
    view! {
        <div class="rp-toasts" aria-live="polite" style="position:fixed;bottom:16px;right:16px;z-index:1000;display:flex;flex-direction:column;gap:8px;max-width:360px;">
            <For
                each=move || state.toasts.get()
                key=|t| t.id
                children=move |item| {
                    let cls = format!("rp-toast {}", item.tone);
                    let id = item.id;
                    view! {
                        <div class=cls role="status">
                            <div class="rp-toast-body">
                                <span class="rp-toast-text">{item.text}</span>
                                {item.detail.map(|d| view! { <span class="rp-toast-detail">{d}</span> })}
                            </div>
                            <button
                                type="button"
                                class="rp-btn quiet sm icon"
                                aria-label="Close"
                                on:click=move |_| state.dismiss_toast(id)
                            >
                                "×"
                            </button>
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn SearchModal() -> impl IntoView {
    let state = use_app_state();
    let filter = RwSignal::new(String::new());

    view! {
        <Show when=move || state.search_open.get()>
            <div
                class="rp-modal"
                style="position:fixed;inset:0;z-index:900;background:rgba(0,0,0,0.45);display:flex;align-items:flex-start;justify-content:center;padding-top:12vh;"
                on:click=move |_| state.search_open.set(false)
            >
                <div
                    class="rp-dialog rp-card"
                    role="dialog"
                    aria-modal="true"
                    style="width:min(520px,92vw);max-height:70vh;overflow:auto;padding:16px;"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="rp-between" style="margin-bottom:12px;">
                        <input
                            class="rp-input"
                            type="search"
                            placeholder=move || state.t("shell.search")
                            prop:value=move || filter.get()
                            on:input=move |ev| filter.set(event_target_value(&ev))
                            style="flex:1;"
                        />
                        <button
                            type="button"
                            class="rp-btn quiet sm"
                            on:click=move |_| state.search_open.set(false)
                        >
                            "Esc"
                        </button>
                    </div>
                    <div class="rp-list" style="display:flex;flex-direction:column;gap:4px;">
                        {move || {
                            let q = filter.get().to_lowercase();
                            HUBS.iter()
                                .flat_map(|h| h.pages.iter().map(move |&p| (h.title_key, p)))
                                .filter(|(hk, p)| {
                                    if q.is_empty() {
                                        return true;
                                    }
                                    let title = state.t(page_title_key(p)).to_lowercase();
                                    let hub_t = state.t(hk).to_lowercase();
                                    title.contains(&q) || hub_t.contains(&q) || p.contains(&q)
                                })
                                .map(|(hk, p)| {
                                    let title = state.t(page_title_key(p));
                                    let hub_label = state.t(hk);
                                    view! {
                                        <button
                                            type="button"
                                            class="rp-btn quiet"
                                            style="justify-content:space-between;width:100%;"
                                            on:click=move |_| {
                                                state.navigate(p, None);
                                                state.search_open.set(false);
                                            }
                                        >
                                            <span>{title}</span>
                                            <span class="rp-label">{hub_label}</span>
                                        </button>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn AboutModal() -> impl IntoView {
    let state = use_app_state();
    let quacked = RwSignal::new(false);
    view! {
        <Show when=move || state.about_open.get()>
            <div
                class="rp-modal"
                style="position:fixed;inset:0;z-index:900;background:rgba(0,0,0,0.45);display:flex;align-items:center;justify-content:center;"
                on:click=move |_| state.about_open.set(false)
            >
                <div
                    class="rp-dialog rp-card rp-about"
                    role="dialog"
                    aria-modal="true"
                    style="width:min(420px,92vw);padding:20px;display:flex;flex-direction:column;gap:12px;"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="rp-between">
                        <h2 class="rp-h3">{move || state.t("about.title")}</h2>
                        <button type="button" class="rp-btn quiet sm icon" on:click=move |_| state.about_open.set(false)>"×"</button>
                    </div>
                    <div class="rp-cluster">
                        <img
                            src="./logo.svg"
                            alt="doona"
                            style="width:48px;height:48px;cursor:pointer;"
                            on:click=move |_| quacked.update(|q| *q = !*q)
                        />
                        <div>
                            <div class="rp-about-name"><strong>"doona (Pure Rust WASM)"</strong></div>
                            <div class="rp-label">
                                {move || {
                                    let eng = state
                                        .version
                                        .get()
                                        .and_then(|v| v.engine)
                                        .map(|e| format!("{} {}", e.name.unwrap_or_default(), e.version.unwrap_or_default()))
                                        .unwrap_or_else(|| "honk".to_string());
                                    state.t_args("about.tagline", &[("engine", &eng)])
                                }}
                            </div>
                        </div>
                    </div>
                    <Show when=move || quacked.get()>
                        <div class="rp-badge">{move || state.t("about.quack")}</div>
                    </Show>
                    <p class="rp-label">{move || state.t("about.privacy")}</p>
                    <div class="rp-cluster">
                        <a class="rp-btn secondary sm" href="./lite/" rel="noopener">"Doona Lite"</a>
                        <button
                            type="button"
                            class="rp-btn accent sm"
                            on:click=move |_| state.about_open.set(false)
                        >
                            {move || state.t("ui.close")}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Crisp inline SVG icons matching Doona's Adobe Spectrum icon set (zero external icon runtime).
pub mod icons {
    use leptos::prelude::*;

    pub fn hub_icon(id: &str) -> AnyView {
        match id {
            "activity" => view! {
                <svg class="rp-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M12 12l4-4" /><path d="M3.34 19a10 10 0 1 1 17.32 0" />
                </svg>
            }.into_any(),
            "traffic" => view! {
                <svg class="rp-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                </svg>
            }.into_any(),
            "routing" => view! {
                <svg class="rp-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M12 19V5" /><path d="M5 12l7-7 7 7" />
                </svg>
            }.into_any(),
            _ => view! {
                <svg class="rp-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" />
                </svg>
            }.into_any(),
        }
    }

    pub fn search_icon() -> impl IntoView {
        view! {
            <svg class="rp-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
        }
    }

    pub fn refresh_icon() -> impl IntoView {
        view! {
            <svg class="rp-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <polyline points="23 4 23 10 17 10" /><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
        }
    }

    pub fn theme_icon() -> impl IntoView {
        view! {
            <svg class="rp-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <circle cx="12" cy="12" r="9" /><path d="M12 3v18a9 9 0 0 0 0-18z" fill="currentColor" />
            </svg>
        }
    }
}
