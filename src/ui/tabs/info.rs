use dioxus::prelude::*;
use qrcode::{QrCode, render::svg};
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;
use crate::app_version;

pub fn render_info_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let ver = app_version();
    let pk = s.public_key.read().clone();
    let relay_url = s.gun_relay_url.read().clone();
    let relay_status = *s.relay_status.read();
    let relay_checking = *s.relay_checking.read();
    let discovered = s.discovered_relays.read().clone();
    let discovering = *s.discovering_relays.read();

    // Relay status indicator
    let (status_dot, status_text, status_color) = if relay_url.trim().is_empty() {
        ("\u{26AA}", i18n.relay_status_not_configured(), "#999")  // ⚪
    } else if relay_checking {
        ("\u{1F7E1}", i18n.relay_status_checking(), "#ffc107")  // 🟡
    } else {
        match relay_status {
            Some(true) => ("\u{1F7E2}", i18n.relay_status_connected(), "#28a745"),   // 🟢
            Some(false) => ("\u{1F534}", i18n.relay_status_unreachable(), "#dc3545"), // 🔴
            None => ("\u{26AA}", i18n.relay_status_not_configured(), "#999"),         // ⚪
        }
    };

    let has_relay = !relay_url.trim().is_empty();

    rsx! {
        if !ver.is_empty() {
            p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                "{i18n.info_version(ver)}"
            }
        }

        // ── Relay Status Card ───────────────────────────────────────
        div { style: "margin-top: 20px; padding: 16px; background: #f8f9fa; border-radius: 12px; border: 1px solid #dee2e6;",
            div { style: "display: flex; align-items: center; justify-content: space-between;",
                div { style: "display: flex; align-items: center; gap: 10px;",
                    span { style: "font-size: 1.4em;", "{status_dot}" }
                    div {
                        p { style: "margin: 0; font-weight: bold; font-size: 0.95em; color: #333;",
                            "{i18n.lbl_relay_status()}"
                        }
                        p { style: "margin: 2px 0 0; font-size: 0.85em; color: {status_color};",
                            "{status_text}"
                        }
                    }
                }
                if has_relay {
                    button {
                        style: "padding: 8px 16px; background: #6f42c1; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 0.85em;",
                        disabled: relay_checking,
                        onclick: move |_| ctrl.check_relay_action(),
                        "{i18n.btn_check_relay()}"
                    }
                }
            }
            if has_relay {
                p { style: "margin: 10px 0 0; font-family: monospace; font-size: 0.75em; color: #666; word-break: break-all;",
                    "{relay_url}"
                }
            }
        }

        // ── Find Relays on Stellar Testnet ──────────────────────────
        div { style: "margin-top: 16px;",
            button {
                style: "width: 100%; padding: 12px; background: #17a2b8; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 0.95em;",
                disabled: discovering,
                onclick: move |_| ctrl.discover_relays_action(),
                if discovering {
                    "{i18n.relay_discovering()}"
                } else {
                    "{i18n.btn_find_relays()}"
                }
            }
        }

        // ── Discovered Relays List ──────────────────────────────────
        if !discovered.is_empty() {
            div { style: "margin-top: 16px;",
                p { style: "font-weight: bold; font-size: 0.9em; color: #333; margin-bottom: 8px;",
                    "{i18n.lbl_discovered_relays()}"
                }
                for relay in discovered.iter() {
                    {
                        let dot = match relay.reachable {
                            Some(true) => "\u{1F7E2}",   // 🟢
                            Some(false) => "\u{1F534}",   // 🔴
                            None => "\u{1F7E1}",          // 🟡 checking
                        };
                        let short_account = if relay.account.len() > 12 {
                            format!("{}...{}", &relay.account[..6], &relay.account[relay.account.len()-6..])
                        } else {
                            relay.account.clone()
                        };
                        rsx! {
                            div { style: "padding: 10px; margin-bottom: 6px; background: #f8f9fa; border: 1px solid #dee2e6; border-radius: 8px;",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-size: 1.1em;", "{dot}" }
                                    div { style: "flex: 1; min-width: 0;",
                                        p { style: "margin: 0; font-family: monospace; font-size: 0.8em; color: #333; word-break: break-all;",
                                            "{relay.url}"
                                        }
                                        p { style: "margin: 2px 0 0; font-size: 0.7em; color: #888;",
                                            "{short_account}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if !discovering && *s.discovering_relays.peek() == false && discovered.is_empty() {
            // Only show "no results" if we've actually searched (not on first load)
        }

        // ── Public Key QR ───────────────────────────────────────────
        match pk {
            Some(key) => {
                {
                    let qr_svg = QrCode::new(key.as_bytes())
                        .map(|code| {
                            code.render::<svg::Color>()
                                .min_dimensions(200, 200)
                                .max_dimensions(280, 280)
                                .quiet_zone(true)
                                .build()
                        })
                        .unwrap_or_default();

                    rsx! {
                        div { style: "text-align: center; margin-top: 30px;",
                            p { style: "font-size: 0.9em; color: #666; margin-bottom: 10px;",
                                "{i18n.info_public_key_label()}"
                            }
                            div { style: "display: inline-block; padding: 12px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                                dangerous_inner_html: "{qr_svg}"
                            }
                            p { style: "margin-top: 16px; font-family: monospace; font-size: 0.78em; word-break: break-all; padding: 12px; background: #f8f9fa; border-radius: 8px; border: 1px solid #ddd;",
                                "{key}"
                            }
                        }
                    }
                }
            }
            None => {
                rsx! {
                    div { style: "text-align: center; margin-top: 60px; color: #888;",
                        p { style: "font-size: 2em;", "\u{2139}\u{FE0F}" }
                        p { "{i18n.info_no_key()}" }
                    }
                }
            }
        }
    }
}
