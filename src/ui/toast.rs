use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use super::state::{use_wallet_state, AuthState};
use super::i18n::ui_i18n;

#[component]
pub fn UpdateNotification() -> Element {
    let mut show_update = use_signal(|| false);
    let state = use_wallet_state();

    // Sync with the JavaScript bridge
    use_effect(move || {
        spawn(async move {
            loop {
                // Check if the JS flag is set
                let is_ready = js_sys::eval("window.__ZSOZSO_UPDATE_READY === true")
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if is_ready {
                    // // If not yet authenticated, reload immediately
                    // if *state.auth_state.read() != AuthState::Authenticated {
                    //     let _ = js_sys::eval("window.location.reload()");
                    //     return;
                    // }
                    show_update.set(true);
                    break; // Stop polling once detected
                }
                // Check every 5 seconds if not already detected
                TimeoutFuture::new(5_000).await;
            }
        });
    });

    if !show_update() {
        return rsx! {};
    }

    let lang = *state.language.read();
    let i18n = ui_i18n(lang);

    rsx! {
        div {
            class: "fixed top-4 right-4 bg-info text-white p-4 rounded-lg shadow-lg flex flex-col gap-2 z-50",
            style: "background-color: #17a2b8; min-width: 250px;",
            p { "{i18n.toast_update_available()}" }
            button {
                class: "bg-white text-info font-bold py-1 px-3 rounded hover:bg-gray-100",
                style: "color: #17a2b8;",
                onclick: move |_| {
                    // Trigger the reload via JS
                    let _ = js_sys::eval("window.location.reload()");
                },
                "{i18n.btn_update_now()}"
            }
        }
    }
}
