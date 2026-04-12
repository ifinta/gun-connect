mod clipboard;
pub mod actions;
pub mod i18n;
pub mod tabs;
pub mod view;
pub mod state;
pub mod controller;

use dioxus::prelude::*;
use state::{use_wallet_state, AuthState};
use controller::AppController;

pub fn app() -> Element {
    let state = use_wallet_state();
    let ctrl = AppController::new(state);
    let mut auto_loaded = use_signal(|| false);

    // Clear clipboard when the tab/browser is closed
    use_hook(|| {
        clipboard::register_beforeunload_cleanup();
    });

    // Auto-load secrets from store after authentication
    use_effect(move || {
        let auth = *state.auth_state.read();
        if auth == AuthState::Authenticated && !auto_loaded() {
            auto_loaded.set(true);
            ctrl.load_all_from_store();
        }
    });

    rsx! {
        {view::render_app(state, ctrl)}
    }
}

pub fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }
