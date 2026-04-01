mod clipboard;
pub mod actions;
pub mod i18n;
pub mod status;
pub mod tabs;
pub mod view;
pub mod state;
pub mod controller;
pub mod qr_scanner;
pub mod toast;

use dioxus::prelude::*;
use state::{use_wallet_state, AuthState};
use controller::AppController;
use toast::UpdateNotification;

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

    // Load network data when public key becomes available or changes
    use_effect(move || {
        let pk = state.public_key.read();
        if pk.is_some() {
            ctrl.load_network_action();
        }
    });

    rsx! {
        {view::render_app(state, ctrl)}
        //UpdateNotification {}
    }
}

pub fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }
