mod db;
mod i18n;
mod ledger;
pub mod sss;
mod ui;
mod store;

use dioxus::prelude::*;
use std::sync::OnceLock;

/// App version string read from window.__APP_VERSION at startup.
static APP_VERSION: OnceLock<String> = OnceLock::new();

pub fn app_version() -> &'static str {
    APP_VERSION.get().map(|s| s.as_str()).unwrap_or("")
}

/// Read the app version from window.__APP_VERSION (stamped by build.sh into index.html).
fn read_app_version() -> String {
    js_sys::eval("window.__APP_VERSION || ''")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

fn main() {
    let version = read_app_version();
    if !version.is_empty() {
        let _ = APP_VERSION.set(version);
    }
    LaunchBuilder::web().launch(ui::app);
}