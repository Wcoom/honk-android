//! Pure Rust WASM (Leptos CSR) frontend for Doona (`/ui/`).

pub mod mode_dae;
pub mod pages;
pub mod shell;
pub mod state;
pub mod ui;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let mount = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("root"));
    if let Some(el) = mount {
        use wasm_bindgen::JsCast;
        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
            leptos::mount::mount_to(html_el, shell::App).forget();
            return;
        }
    }
    leptos::mount::mount_to_body(shell::App);
}
