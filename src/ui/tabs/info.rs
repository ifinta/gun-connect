use dioxus::prelude::*;
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;
use crate::ui::actions::MAX_RELAYS;
use crate::app_version;

pub fn render_info_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let ver = app_version();
    let connected = s.connected_relays.read().clone();
    let discovered = s.discovered_relays.read().clone();
    let discovering = *s.discovering_relays.read();
    let discover_hint = s.discover_status.read().clone();
    let discover_progress = s.discover_progress.read().clone();
    let at_limit = connected.len() >= MAX_RELAYS;

    rsx! {
        // ── Version ─────────────────────────────────────────────────
        if !ver.is_empty() {
            p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                "{i18n.info_version(ver)}"
            }
        }

        // ── Connected Relays ────────────────────────────────────────
        div { style: "margin-top: 16px;",
            div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;",
                p { style: "margin: 0; font-weight: bold; font-size: 0.95em; color: #333;",
                    "{i18n.lbl_relay_status()}"
                }
                if !connected.is_empty() {
                    button {
                        style: "padding: 6px 14px; background: #6f42c1; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 0.8em;",
                        onclick: move |_| ctrl.check_all_relays_action(),
                        "{i18n.btn_check_relay()}"
                    }
                }
            }

            if connected.is_empty() {
                p { style: "font-size: 0.85em; color: #999; font-style: italic;",
                    "{i18n.relay_status_not_configured()}"
                }
            }

            for entry in connected.iter() {
                {
                    let (dot, color) = if entry.checking {
                        ("\u{1F7E1}", "#ffc107")
                    } else {
                        match entry.reachable {
                            Some(true) => ("\u{1F7E2}", "#28a745"),
                            Some(false) => ("\u{1F534}", "#dc3545"),
                            None => ("\u{26AA}", "#999"),
                        }
                    };
                    let status_text = if entry.checking {
                        i18n.relay_status_checking()
                    } else {
                        match entry.reachable {
                            Some(true) => i18n.relay_status_connected(),
                            Some(false) => i18n.relay_status_unreachable(),
                            None => i18n.relay_status_checking(),
                        }
                    };
                    let url_for_remove = entry.url.clone();
                    rsx! {
                        div { style: "padding: 10px; margin-bottom: 6px; background: #f8f9fa; border: 1px solid #dee2e6; border-radius: 8px;",
                            div { style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "font-size: 1.1em;", "{dot}" }
                                div { style: "flex: 1; min-width: 0;",
                                    p { style: "margin: 0; font-family: monospace; font-size: 0.8em; color: #333; word-break: break-all;",
                                        "{entry.url}"
                                    }
                                    p { style: "margin: 2px 0 0; font-size: 0.75em; color: {color};",
                                        "{status_text}"
                                    }
                                }
                                button {
                                    style: "padding: 4px 10px; background: #dc3545; color: white; border: none; border-radius: 4px; font-size: 0.75em; cursor: pointer;",
                                    onclick: move |_| ctrl.remove_relay_action(url_for_remove.clone()),
                                    "{i18n.btn_remove_relay()}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Find Relays ─────────────────────────────────────────────
        div { style: "margin-top: 16px;",
            div { style: "display: flex; gap: 8px;",
                button {
                    style: "flex: 1; padding: 12px; background: #17a2b8; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 0.95em;",
                    disabled: discovering,
                    onclick: move |_| ctrl.discover_relays_action(),
                    if discovering {
                        "{i18n.relay_discovering()}"
                    } else {
                        "{i18n.btn_find_relays()}"
                    }
                }
                if discovering {
                    button {
                        style: "padding: 12px 20px; background: #dc3545; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 0.95em;",
                        onclick: move |_| ctrl.stop_discover_relays(),
                        "{i18n.btn_stop_search()}"
                    }
                }
            }
            if !discover_progress.is_empty() {
                p { style: "text-align: center; font-size: 0.8em; color: #17a2b8; font-family: monospace; margin-top: 6px;",
                    "{discover_progress}"
                }
            }
            if !discover_hint.is_empty() {
                p { style: "text-align: center; font-size: 0.8em; color: #495057; font-style: italic; margin-top: 4px;",
                    "{discover_hint}"
                }
            }
        }

        // ── Discovered Relays ───────────────────────────────────────
        if !discovered.is_empty() {
            div { style: "margin-top: 16px;",
                p { style: "font-weight: bold; font-size: 0.9em; color: #333; margin-bottom: 8px;",
                    "{i18n.lbl_discovered_relays()}"
                }
                for relay in discovered.iter().filter(|r| !connected.iter().any(|c| c.url == r.url)) {
                    {
                        let dot = match relay.reachable {
                            Some(true) => "\u{1F7E2}",
                            Some(false) => "\u{1F534}",
                            None => "\u{1F7E1}",
                        };
                        let url_for_add = relay.url.clone();
                        rsx! {
                            div { style: "padding: 10px; margin-bottom: 6px; background: #f8f9fa; border: 1px solid #dee2e6; border-radius: 8px;",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-size: 1.1em;", "{dot}" }
                                    div { style: "flex: 1; min-width: 0;",
                                        p { style: "margin: 0; font-family: monospace; font-size: 0.8em; color: #333; word-break: break-all;",
                                            "{relay.url}"
                                        }
                                    }
                                    if !at_limit {
                                        button {
                                            style: "padding: 4px 10px; background: #28a745; color: white; border: none; border-radius: 4px; font-size: 0.75em; cursor: pointer;",
                                            onclick: move |_| ctrl.add_relay_action(url_for_add.clone()),
                                            "{i18n.btn_connect_relay()}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
