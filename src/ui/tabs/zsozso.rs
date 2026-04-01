use dioxus::prelude::*;
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;
use crate::ledger::NetworkEnvironment;

pub fn render_zsozso_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let net_env = *s.current_network.read();
    let is_testnet = net_env == NetworkEnvironment::Test;

    // On testnet, show a disabled message
    if is_testnet {
        return rsx! {
            div { style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 16px; opacity: 0.5;",
                // Greyed out ZS coin
                svg {
                    width: "100",
                    height: "100",
                    view_box: "0 0 100 100",
                    circle { cx: "50", cy: "50", r: "47", fill: "#ccc", stroke: "#999", stroke_width: "3" }
                    circle { cx: "50", cy: "50", r: "37", fill: "none", stroke: "#999", stroke_width: "1.5" }
                    text {
                        x: "50",
                        y: "57",
                        text_anchor: "middle",
                        font_size: "26",
                        font_weight: "bold",
                        fill: "#666",
                        font_family: "sans-serif",
                        "ZS"
                    }
                }
                p { style: "text-align: center; color: #888; font-size: 1em; max-width: 300px;",
                    "{i18n.zs_mainnet_only()}"
                }
            }
        };
    }

    let has_key = s.public_key.read().is_some();

    if !has_key {
        return rsx! {
            div { style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 16px;",
                svg {
                    width: "100",
                    height: "100",
                    view_box: "0 0 100 100",
                    circle { cx: "50", cy: "50", r: "47", fill: "#4A90D9", stroke: "#2E6AB0", stroke_width: "3" }
                    circle { cx: "50", cy: "50", r: "37", fill: "none", stroke: "#2E6AB0", stroke_width: "1.5" }
                    text {
                        x: "50",
                        y: "57",
                        text_anchor: "middle",
                        font_size: "26",
                        font_weight: "bold",
                        fill: "#1A3A6B",
                        font_family: "sans-serif",
                        "ZS"
                    }
                }
                p { style: "text-align: center; color: #888; font-size: 1em;",
                    "{i18n.zs_no_key()}"
                }
            }
        };
    }

    let xlm_display = s.xlm_balance.read().clone().unwrap_or_else(|| "\u{2014}".to_string());
    let zsozso_display = s.zsozso_balance.read().clone().unwrap_or_else(|| "\u{2014}".to_string());
    let locked_display = s.locked_zsozso.read().clone().unwrap_or_else(|| "\u{2014}".to_string());
    let status = s.zs_status.read().clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px; height: 100%;",

            // Amount input
            div { style: "display: flex; gap: 8px; align-items: center;",
                label { style: "font-size: 0.9em; font-weight: bold; color: #555; white-space: nowrap;",
                    "{i18n.lbl_amount()}"
                }
                input {
                    r#type: "number",
                    min: "1",
                    style: "flex: 1; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 1em;",
                    placeholder: "0",
                    value: "{s.lock_amount.read()}",
                    oninput: move |e| {
                        let mut amount = s.lock_amount;
                        amount.set(e.value());
                    },
                }
            }

            // Lock button (blue, top)
            button {
                style: "width: 100%; padding: 16px; background: #2E6AB0; color: white; border: none; border-radius: 10px; font-size: 1.1em; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.lock_zsozso_action(),
                "{i18n.btn_lock()}"
            }

            // White canvas with coin + balances
            div { style: "background: white; border-radius: 12px; border: 1px solid #e0e0e0; padding: 16px 24px;",
                div { style: "display: flex; align-items: center; gap: 24px;",
                    // ZS Coin (blue theme)
                    div { style: "flex-shrink: 0;",
                        svg {
                            width: "80",
                            height: "80",
                            view_box: "0 0 80 80",
                            circle { cx: "40", cy: "40", r: "38", fill: "#4A90D9", stroke: "#2E6AB0", stroke_width: "3" }
                            circle { cx: "40", cy: "40", r: "30", fill: "none", stroke: "#2E6AB0", stroke_width: "1.5" }
                            text {
                                x: "40",
                                y: "45",
                                text_anchor: "middle",
                                font_size: "20",
                                font_weight: "bold",
                                fill: "#1A3A6B",
                                font_family: "sans-serif",
                                "ZS"
                            }
                        }
                    }

                    // Balances column
                    div { style: "flex: 1;",
                        // XLM balance
                        div { style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px;",
                            span { style: "font-size: 0.8em; color: #888; font-weight: bold;", "{i18n.lbl_xlm_balance()}" }
                            span { style: "font-size: 1.2em; font-weight: bold; color: #333;", "{xlm_display}" }
                        }
                        // ZSOZSO balance
                        div { style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px;",
                            span { style: "font-size: 0.8em; color: #888; font-weight: bold;", "{i18n.lbl_zsozso_balance()}" }
                            span { style: "font-size: 1.2em; font-weight: bold; color: #2E6AB0;", "{zsozso_display}" }
                        }
                        // Locked ZSOZSO
                        div { style: "display: flex; justify-content: space-between; align-items: baseline;",
                            span { style: "font-size: 0.8em; color: #888; font-weight: bold;", "{i18n.lbl_locked_zsozso()}" }
                            span { style: "font-size: 1.2em; font-weight: bold; color: #28a745;", "{locked_display}" }
                        }
                    }
                }
            }

            // Status message
            if let Some(status_msg) = status {
                p { style: "text-align: center; font-size: 0.9em; color: #666; font-style: italic; margin: 0;",
                    "{status_msg}"
                }
            }

            // Refresh button
            button {
                style: "width: 100%; padding: 12px; background: #f8f9fa; color: #333; border: 1px solid #ddd; border-radius: 10px; font-size: 1em; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.fetch_balances_action(),
                "{i18n.btn_refresh_balances()}"
            }

            // Unlock button (orange, bottom)
            button {
                style: "width: 100%; padding: 16px; background: #fd7e14; color: white; border: none; border-radius: 10px; font-size: 1.1em; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.unlock_zsozso_action(),
                "{i18n.btn_unlock()}"
            }
        }
    }
}
