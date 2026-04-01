use dioxus::prelude::*;
use super::state::{WalletState, AuthState};
use super::controller::AppController;
use super::i18n::ui_i18n;
use super::tabs::Tab;
use super::tabs::{home, zsozso, networking, info, settings, log};

pub fn render_app(s: WalletState, ctrl: AppController) -> Element {
    let lang = *s.language.read();
    let i18n = ui_i18n(lang);
    let auth_state = *s.auth_state.read();

    // ── Auth failed: terminal error modal ──
    if auth_state == AuthState::Failed {
        return rsx! {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; font-family: sans-serif;",
                div { style: "background: white; padding: 40px; border-radius: 16px; max-width: 360px; width: 90%; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3);",
                    h2 { style: "margin: 0 0 12px; color: #dc3545;", "⚠️" }
                    p { style: "margin: 0 0 30px; color: #333; font-size: 1em; font-weight: bold;",
                        "{i18n.auth_failed()}"
                    }
                    button {
                        style: "padding: 14px 48px; background: #dc3545; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                        onclick: move |_| {
                            // Close window or blank the page
                            let _ = js_sys::eval("window.close() || (document.body.innerHTML = '')");
                        },
                        "{i18n.btn_exit()}"
                    }
                }
            }
        };
    }

    // ── Gate modal: pending or authenticating ──
    if auth_state != AuthState::Authenticated {
        let is_busy = auth_state == AuthState::Authenticating;
        let btn_label = if is_busy {
            i18n.authenticating()
        } else {
            i18n.btn_next()
        };
        let btn_bg = if is_busy { "#6c757d" } else { "#007bff" };

        return rsx! {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000; font-family: sans-serif;",
                div { style: "background: white; padding: 40px; border-radius: 16px; max-width: 360px; width: 90%; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3);",
                    h2 { style: "margin: 0 0 12px; color: #333;", "Zsozso" }
                    p { style: "margin: 0 0 30px; color: #666; font-size: 1em;",
                        "{i18n.gate_title()}"
                    }
                    button {
                        style: "padding: 14px 48px; background: {btn_bg}; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                        disabled: is_busy,
                        onclick: move |_| ctrl.start_auth(),
                        "{btn_label}"
                    }
                }
            }
        };
    }

    let active = *s.active_tab.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100vh; max-width: 550px; margin: auto; font-family: sans-serif;",
            // Header
            div { style: "padding: 15px 30px 0;",
                h2 { style: "margin: 0;", "Zsozso" }
            }

            // Tab content (scrollable area)
            div { style: "flex: 1; overflow-y: auto; padding: 20px 30px 90px;",
                match active {
                    Tab::Cyf => home::render_cyf_tab(s, i18n.as_ref()),
                    Tab::Zsozso => zsozso::render_zsozso_tab(s, ctrl, i18n.as_ref()),
                    Tab::Networking => networking::render_networking_tab(s, ctrl, i18n.as_ref()),
                    Tab::Info => info::render_info_tab(s, i18n.as_ref()),
                    Tab::Settings => settings::render_settings_tab(s, ctrl, i18n.as_ref()),
                    Tab::Log => log::render_log_tab(s, i18n.as_ref()),
                }
            }

            // Bottom tab bar
            {render_tab_bar(s, i18n.as_ref())}
        }

        // Clipboard modal overlay
        if *s.clipboard_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    p { style: "margin-bottom: 20px; font-size: 1em; color: #333;",
                        "{i18n.clipboard_modal_text()}"
                    }
                    button {
                        style: "padding: 12px 24px; background: #dc3545; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                        onclick: move |_| ctrl.dismiss_clipboard_modal(),
                        "{i18n.btn_clear_clipboard()}"
                    }
                }
            }
        }

        // Biometric required to save – error modal
        if *s.biometric_save_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1150;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    h3 { style: "margin: 0 0 12px; color: #dc3545;", "\u{26A0}\u{FE0F}" }
                    p { style: "margin: 0 0 20px; color: #333; font-size: 1em;",
                        "{i18n.biometric_required_to_save()}"
                    }
                    button {
                        style: "padding: 12px 24px; background: #007bff; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                        onclick: move |_| ctrl.dismiss_biometric_save_modal(),
                        "{i18n.btn_close()}"
                    }
                }
            }
        }

        // SEA key generation modal
        if *s.sea_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1200;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    h3 { style: "margin: 0 0 16px; color: #333;", "{i18n.sea_modal_title()}" }
                    input {
                        style: "width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 6px; font-size: 1em; margin-bottom: 16px; box-sizing: border-box;",
                        r#type: "password",
                        placeholder: "{i18n.sea_modal_placeholder()}",
                        value: "{s.sea_modal_input.read().as_str()}",
                        oninput: move |evt| {
                            let mut input = s.sea_modal_input;
                            input.set(zeroize::Zeroizing::new(evt.value()));
                        }
                    }
                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        button {
                            style: "padding: 12px 24px; background: #6f42c1; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.generate_sea_keys(),
                            "{i18n.btn_generate_db_keys()}"
                        }
                        button {
                            style: "padding: 12px 24px; background: #6c757d; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.close_sea_modal(),
                            "{i18n.btn_close()}"
                        }
                    }
                }
            }
        }

        // CYF not-yet-implemented modal
        if s.cyf_modal_message.read().is_some() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1300;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    p { style: "margin: 0 0 20px; color: #333; font-size: 1em;",
                        "{s.cyf_modal_message.read().as_deref().unwrap_or_default()}"
                    }
                    button {
                        style: "padding: 12px 36px; background: #007bff; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                        onclick: move |_| {
                            let mut modal = s.cyf_modal_message;
                            modal.set(None);
                        },
                        "{i18n.btn_ok()}"
                    }
                }
            }
        }

        // SSS shares modal
        if s.sss_shares.read().is_some() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1400;",
                div { style: "background: white; padding: 24px; border-radius: 12px; max-width: 460px; width: 92%; max-height: 80vh; overflow-y: auto; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    h3 { style: "margin: 0 0 8px; color: #333; text-align: center;", "{i18n.sss_modal_title()}" }
                    p { style: "margin: 0 0 16px; color: #666; font-size: 0.85em; text-align: center;",
                        "{i18n.sss_modal_description()}"
                    }
                    if let Some(shares) = s.sss_shares.read().as_ref() {
                        for (idx, share) in shares.iter().enumerate() {
                            {
                                let share_val = share.clone();
                                let label = i18n.sss_share_label(idx + 1);
                                let copy_label = i18n.btn_copy_share();
                                rsx! {
                                    div { style: "margin-bottom: 10px; background: #f8f9fa; border: 1px solid #dee2e6; border-radius: 6px; padding: 10px;",
                                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;",
                                            span { style: "font-weight: bold; font-size: 0.85em; color: #495057;", "{label}" }
                                            button {
                                                style: "padding: 4px 10px; background: #6f42c1; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 0.75em;",
                                                onclick: move |_| {
                                                    super::clipboard::copy_to_clipboard(&share_val);
                                                },
                                                "{copy_label}"
                                            }
                                        }
                                        code { style: "display: block; word-break: break-all; font-size: 0.72em; color: #333; line-height: 1.4;",
                                            "{share}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { style: "text-align: center; margin-top: 16px;",
                        button {
                            style: "padding: 12px 48px; background: #007bff; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.dismiss_sss_modal(),
                            "{i18n.btn_ok()}"
                        }
                    }
                }
            }
        }
    }
}

fn render_tab_bar(s: WalletState, i18n: &dyn super::i18n::UiI18n) -> Element {
    let active = *s.active_tab.read();
    let has_mainnet = s.mainnet_public_key.read().is_some();
    let has_testnet = s.testnet_public_key.read().is_some();

    let tabs: [(Tab, &str, &str); 6] = [
        (Tab::Cyf, "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z", i18n.tab_cyf()),
        (Tab::Zsozso, "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z", i18n.tab_zsozso()),
        (Tab::Networking, "M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9", i18n.tab_networking()),
        (Tab::Info, "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z", i18n.tab_info()),
        (Tab::Settings, "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z", i18n.tab_settings()),
        (Tab::Log, "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01m-.01 4h.01", i18n.tab_log()),
    ];

    rsx! {
        div { style: "position: fixed; bottom: 0; left: 0; right: 0; max-width: 550px; margin: auto; background: white; border-top: 1px solid #ddd; display: flex; justify-content: space-around; padding: 6px 0; z-index: 500;",
            for (tab, path, label) in tabs {
                {
                    let is_active = active == tab;
                    let is_disabled = (tab == Tab::Zsozso && !has_mainnet)
                        || (tab == Tab::Networking && !has_testnet);
                    let color = if is_disabled { "#ddd" } else if is_active { "#007bff" } else { "#999" };
                    let font_weight = if is_active { "bold" } else { "normal" };
                    let cursor = if is_disabled { "default" } else { "pointer" };
                    rsx! {
                        button {
                            key: "{label}",
                            style: "flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px; background: none; border: none; cursor: {cursor}; padding: 4px 0; color: {color};",
                            disabled: is_disabled,
                            onclick: move |_| {
                                if !is_disabled {
                                    let mut active_tab = s.active_tab;
                                    active_tab.set(tab);
                                }
                            },
                            svg {
                                width: "24",
                                height: "24",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "{color}",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "{path}" }
                            }
                            span { style: "font-size: 0.65em; font-weight: {font_weight};",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
