use dioxus::prelude::*;
use qrcode::{QrCode, render::svg};
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;
use crate::ui::status::status_text;
use crate::i18n::Language;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};

/// Check if running on localhost.
fn is_localhost() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "::1")
}

pub fn render_settings_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let lang = *s.language.read();
    let on_localhost = is_localhost();

    let status_display = status_text(&s.submission_status.read(), i18n);

    let lang_value = match lang {
        Language::English => "en",
        Language::Hungarian => "hu",
        Language::French => "fr",
        Language::German => "de",
        Language::Spanish => "es",
    };

    rsx! {
        // ── 1. Biometric / PIN Code ─────────────────────────────────────
        if on_localhost {
            // Localhost: simple PIN code input
            div { style: "display: flex; align-items: center; justify-content: space-between; padding: 12px 15px; background: #f8f9fa; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                div { style: "flex: 1; margin-right: 12px;",
                    p { style: "font-weight: bold; margin: 0; font-size: 0.95em;", "{i18n.lbl_pin_code()}" }
                    p { style: "font-size: 0.8em; color: #666; margin: 4px 0 0;", "{i18n.lbl_pin_code_desc()}" }
                }
                div { style: "display: flex; gap: 6px; align-items: center;",
                    input {
                        style: "width: 100px; padding: 6px 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 0.9em;",
                        r#type: "password",
                        placeholder: "{i18n.lbl_pin_code_ph()}",
                        value: "{s.pin_code}",
                        oninput: move |evt| {
                            ctrl.set_pin_code(evt.value());
                        }
                    }
                }
            }
        } else {
            // Production: biometric toggle
            {
                let biometric_on = *s.biometric_enabled.read();
                let track_bg = if biometric_on { "#28a745" } else { "#ccc" };
                let thumb_left = if biometric_on { "24px" } else { "2px" };
                let track_extra = if biometric_on { "opacity: 0.6; cursor: not-allowed;" } else { "cursor: pointer;" };
                let track_style = format!(
                    "position: relative; width: 50px; height: 28px; background: {}; border-radius: 28px; transition: background 0.3s; flex-shrink: 0; {}",
                    track_bg, track_extra
                );
                rsx! {
                    div { style: "display: flex; align-items: center; justify-content: space-between; padding: 12px 15px; background: #f8f9fa; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                        div { style: "flex: 1; margin-right: 12px;",
                            p { style: "font-weight: bold; margin: 0; font-size: 0.95em;", "{i18n.lbl_biometric()}" }
                            p { style: "font-size: 0.8em; color: #666; margin: 4px 0 0;", "{i18n.lbl_biometric_desc()}" }
                        }
                        div {
                            style: "{track_style}",
                            onclick: move |_| {
                                if !biometric_on {
                                    ctrl.toggle_biometric();
                                }
                            },
                            div {
                                style: "position: absolute; top: 3px; left: {thumb_left}; width: 22px; height: 22px; background: white; border-radius: 50%; transition: left 0.3s; box-shadow: 0 1px 3px rgba(0,0,0,0.3);"
                            }
                        }
                    }
                }
            }
        }

        // ── 2. Language Selector (full width) ───────────────────────────
        select {
            style: "width: 100%; padding: 12px; border: 1px solid #17a2b8; border-radius: 8px; font-weight: bold; cursor: pointer; color: #17a2b8; background: white; font-size: 1em; margin-bottom: 20px; box-sizing: border-box;",
            value: "{lang_value}",
            onchange: move |evt| ctrl.set_language(&evt.value()),
            option { value: "en", selected: lang == Language::English, "English" }
            option { value: "hu", selected: lang == Language::Hungarian, "Magyar" }
            option { value: "fr", selected: lang == Language::French, "Français" }
            option { value: "de", selected: lang == Language::German, "Deutsch" }
            option { value: "es", selected: lang == Language::Spanish, "Español" }
        }

        // ── 3. Mainnet Key Section ──────────────────────────────────────
        { render_key_section(s, ctrl, i18n, NetworkEnvironment::Production) }

        // ── 4. Testnet Key Section ──────────────────────────────────────
        { render_key_section(s, ctrl, i18n, NetworkEnvironment::Test) }

        // ── 5. Persistence (Save All / Load All) ────────────────────────
        div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
            button {
                style: "flex: 1; padding: 10px; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.save_all_to_store(),
                "{i18n.btn_save_to_os()}"
            }
            button {
                style: "flex: 1; padding: 10px; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.load_all_from_store(),
                "{i18n.btn_load()}"
            }
        }

        // ── 6. GUN DB Section ───────────────────────────────────────────
        // Nickname
        div { style: "display: flex; gap: 6px; margin-bottom: 10px; align-items: center;",
            input {
                style: "flex-grow: 1; min-width: 0; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                r#type: "text",
                maxlength: "16",
                placeholder: "{i18n.lbl_nickname_ph()}",
                value: "{s.nickname}",
                oninput: move |evt| {
                    let mut nickname = s.nickname;
                    nickname.set(evt.value());
                }
            }
            button {
                style: "padding: 8px 16px; background: #28a745; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; white-space: nowrap;",
                onclick: move |_| ctrl.save_nickname_action(),
                "{i18n.btn_save_nickname()}"
            }
        }

        // GunDB SEA key generation
        div { style: "margin-bottom: 10px;",
            button {
                style: "width: 100%; padding: 10px; background: #6f42c1; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.open_sea_modal(),
                "{i18n.btn_generate_db_secret()}"
            }
            if s.sea_key_pair.read().is_some() {
                p { style: "text-align: center; font-size: 0.8em; color: #28a745; margin-top: 5px;",
                    "{i18n.sea_keys_generated()}"
                }
            }
        }

        // GUN Node Address display
        {
            let gun_addr = s.gun_address.read().clone();
            rsx! {
                if !gun_addr.is_empty() {
                    div { style: "background: #f0f0ff; padding: 12px; border-radius: 8px; margin-bottom: 10px; border: 1px solid #c0c0e0;",
                        p { style: "font-size: 0.8em; color: #666; margin: 0 0 4px 0; font-weight: bold;", "{i18n.lbl_gun_address()}" }
                        code { style: "word-break: break-all; font-size: 0.75em; color: #333;", "{gun_addr}" }
                    }
                }
            }
        }

        // GUN Relay URL
        div { style: "display: flex; gap: 6px; margin-bottom: 10px; align-items: center;",
            input {
                style: "flex-grow: 1; min-width: 0; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 0.9em;",
                r#type: "url",
                placeholder: "{i18n.lbl_gun_relay_ph()}",
                value: "{s.gun_relay_url}",
                oninput: move |evt| {
                    let mut gun_relay_url = s.gun_relay_url;
                    gun_relay_url.set(evt.value());
                }
            }
            button {
                style: "padding: 8px 16px; background: #6f42c1; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; white-space: nowrap;",
                onclick: move |_| ctrl.save_gun_relay_action(),
                "{i18n.btn_save_gun_relay()}"
            }
        }
        p { style: "font-size: 0.7em; color: #888; margin: 2px 0 20px 0;", "{i18n.lbl_gun_relay_url()}" }

        // ── 7. XDR Generator (Testnet) ──────────────────────────────────
        button {
            style: "width: 100%; padding: 12px; background: #007bff; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer; margin-bottom: 10px;",
            onclick: move |_| ctrl.fetch_and_generate_xdr_action(),
            "{i18n.btn_generate_xdr()}"
        }

        p { style: "text-align: center; font-size: 0.9em; color: #495057; font-style: italic;",
            "{status_display}"
        }

        // Signed XDR Result Box
        if !s.generated_xdr.read().is_empty() {
            div { style: "margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 8px; border: 1px solid #ced4da;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                    span { style: "font-size: 0.8em; font-weight: bold;", "{i18n.lbl_signed_xdr()}" }
                    button {
                        style: "font-size: 0.7em; padding: 4px 8px;",
                        onclick: move |_| ctrl.copy_xdr_to_clipboard(),
                        "{i18n.copy_xdr_label()}"
                    }
                }
                pre {
                    style: "word-break: break-all; white-space: pre-wrap; font-size: 0.75em; background: white; padding: 10px; border-radius: 4px; border: 1px solid #dee2e6; max-height: 100px; overflow-y: auto;",
                    "{s.generated_xdr}"
                }
                button {
                    style: "width: 100%; margin-top: 15px; padding: 12px; background: #28a745; color: white; border: none; border-radius: 5px; font-weight: bold;",
                    onclick: move |_| ctrl.submit_transaction_action(),
                    "{i18n.btn_submit_tx()}"
                }
            }
        }
    }
}

/// Render a key management section for a specific network (Mainnet or Testnet).
fn render_key_section(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n, net: NetworkEnvironment) -> Element {
    let is_mainnet = net == NetworkEnvironment::Production;
    let lang = *s.language.read();

    let title = if is_mainnet { i18n.lbl_mainnet_keys() } else { i18n.lbl_testnet_keys() };
    let border_color = if is_mainnet { "#c8e6c9" } else { "#ffe0b2" };
    let bg_color = if is_mainnet { "#e8f5e9" } else { "#fff3e0" };
    let title_color = if is_mainnet { "#2e7d32" } else { "#e65100" };

    let pk = if is_mainnet { s.mainnet_public_key.read().clone() } else { s.testnet_public_key.read().clone() };
    let secret = if is_mainnet { s.mainnet_secret_key.read().clone() } else { s.testnet_secret_key.read().clone() };
    let show_secret = if is_mainnet { *s.mainnet_show_secret.read() } else { *s.testnet_show_secret.read() };
    let input_value = if is_mainnet { s.mainnet_input_value.read().clone() } else { s.testnet_input_value.read().clone() };

    let lgr = StellarLedger::new(net, lang);
    let has_faucet = lgr.network_info().has_faucet;

    let no_account = i18n.lbl_no_account();

    rsx! {
        div { style: "background: {bg_color}; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 1px solid {border_color};",
            // Section header with public key
            p { style: "font-size: 0.85em; color: {title_color}; margin: 0 0 8px; font-weight: bold;", "{title}" }
            code { style: "word-break: break-all; font-size: 0.7em; color: #333; display: block; margin-bottom: 10px;",
                {pk.as_deref().unwrap_or(no_account)}
            }

            // Key Input & Generation
            div { style: "display: flex; gap: 6px; margin-bottom: 10px; align-items: center;",
                button {
                    style: "padding: 5px 10px; white-space: nowrap;",
                    onclick: move |_| ctrl.generate_key_for_network(net),
                    "{i18n.btn_new_key()}"
                }
                button {
                    style: "padding: 5px 10px; background: #6f42c1; color: white; border: none; border-radius: 4px; cursor: pointer; white-space: nowrap;",
                    onclick: move |_| {
                        let input_signal = if is_mainnet { s.mainnet_input_value } else { s.testnet_input_value };
                        let mut input_signal = input_signal;
                        spawn(async move {
                            match crate::ui::qr_scanner::scan_qr().await {
                                Ok(text) => input_signal.set(text),
                                Err(e) => {
                                    if e != "cancelled" {
                                        crate::ui::log(&format!("QR scan error: {}", e));
                                    }
                                }
                            }
                        });
                    },
                    "QR"
                }
                input {
                    style: "flex-grow: 1; min-width: 0; padding: 5px;",
                    r#type: "password",
                    placeholder: "{i18n.lbl_import_ph()}",
                    value: "{input_value}",
                    oninput: move |evt| {
                        if is_mainnet {
                            let mut v = s.mainnet_input_value;
                            v.set(evt.value());
                        } else {
                            let mut v = s.testnet_input_value;
                            v.set(evt.value());
                        }
                    }
                }
                button {
                    style: "padding: 5px 10px; white-space: nowrap;",
                    onclick: move |_| ctrl.import_key_for_network(net),
                    "{i18n.btn_import()}"
                }
            }

            // Secret Key actions (if secret exists)
            if let Some(secret_val) = secret {
                div { style: "display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 10px;",
                    button {
                        onclick: move |_| {
                            if show_secret {
                                if is_mainnet {
                                    let mut ss = s.mainnet_show_secret;
                                    ss.set(false);
                                } else {
                                    let mut ss = s.testnet_show_secret;
                                    ss.set(false);
                                }
                            } else {
                                ctrl.reveal_secret_for_network(net);
                            }
                        },
                        if show_secret { "{i18n.btn_hide_secret()}" } else { "{i18n.btn_reveal_secret()}" }
                    }
                    button {
                        style: "background: #28a745; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                        onclick: move |_| ctrl.copy_secret_for_network(net),
                        "{i18n.copy_label()}"
                    }
                    if has_faucet {
                        button {
                            style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: move |_| ctrl.activate_test_account_for_testnet(),
                            "{i18n.btn_activate_faucet()}"
                        }
                    }
                }

                if show_secret {
                    div { style: "text-align: center; margin-bottom: 10px;",
                        {
                            let qr_svg = QrCode::new(secret_val.as_str().as_bytes())
                                .map(|code| {
                                    code.render::<svg::Color>()
                                        .min_dimensions(200, 200)
                                        .max_dimensions(280, 280)
                                        .quiet_zone(true)
                                        .build()
                                })
                                .unwrap_or_default();
                            rsx! {
                                div { style: "display: inline-block; padding: 12px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                                    dangerous_inner_html: "{qr_svg}"
                                }
                            }
                        }
                    }
                    p { style: "font-family: monospace; word-break: break-all; background: white; padding: 10px; border-radius: 4px; font-size: 0.8em; margin: 0;",
                        "{secret_val.as_str()}"
                    }
                }
            }
        }
    }
}
