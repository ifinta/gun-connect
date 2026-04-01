use std::collections::HashMap;
use dioxus::prelude::*;
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;

/// Show abbreviated key or nickname for a network node button.
fn node_label(key: &str, nicknames: &HashMap<String, String>) -> String {
    if let Some(nick) = nicknames.get(key) {
        if !nick.is_empty() {
            return nick.clone();
        }
    }
    let chars: Vec<char> = key.chars().collect();
    if chars.len() > 19 {
        let head: String = chars[..8].iter().collect();
        let tail: String = chars[chars.len() - 8..].iter().collect();
        format!("{}...{}", head, tail)
    } else {
        key.to_string()
    }
}

pub fn render_networking_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let ping_display = match &*s.ping_status.read() {
        Some(msg) => msg.clone(),
        None => String::new(),
    };

    let parents = s.network_parents.read().clone();
    let workers = s.network_workers.read().clone();
    let nicknames = s.network_nicknames.read().clone();

    rsx! {
        div { style: "padding: 0 20px;",
            // Parents section
            h3 { style: "margin-top: 24px; margin-bottom: 8px; color: #28a745; font-size: 1em;",
                "{i18n.lbl_parents()}"
            }
            if parents.is_empty() {
                p { style: "font-size: 0.85em; color: #999; font-style: italic; margin: 4px 0;",
                    "—"
                }
            }
            for parent_key in parents.iter().take(6) {
                {
                    let label = node_label(parent_key, &nicknames);
                    rsx! {
                        button {
                            style: "width: 100%; margin-bottom: 8px; padding: 12px 0; background: #28a745; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            "{label}"
                        }
                    }
                }
            }

            // Ping button (between Parents and Workers)
            button {
                style: "width: 100%; margin-top: 12px; margin-bottom: 12px; padding: 14px 0; background: #007bff; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                onclick: move |_| ctrl.ping_contract_action(),
                "{i18n.btn_ping()}"
            }

            // Status display
            if !ping_display.is_empty() {
                p { style: "text-align: center; margin-top: 0; margin-bottom: 12px; font-size: 0.95em; color: #495057; font-style: italic; padding: 0;",
                    "{ping_display}"
                }
            }

            // Workers section
            h3 { style: "margin-top: 12px; margin-bottom: 8px; color: #007bff; font-size: 1em;",
                "{i18n.lbl_workers()}"
            }
            for worker_key in workers.iter() {
                {
                    let label = node_label(worker_key, &nicknames);
                    rsx! {
                        button {
                            style: "width: 100%; margin-bottom: 8px; padding: 12px 0; background: #007bff; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            "{label}"
                        }
                    }
                }
            }

            // New worker button
            button {
                style: "width: 100%; margin-top: 4px; margin-bottom: 20px; padding: 12px 0; background: #5bc0de; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1em;",
                onclick: move |_| ctrl.add_worker_action(),
                "{i18n.btn_new_worker()}"
            }
        }
    }
}
