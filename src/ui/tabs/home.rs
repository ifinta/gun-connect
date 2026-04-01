use dioxus::prelude::*;
use crate::ui::state::WalletState;
use crate::ui::i18n::UiI18n;

pub fn render_cyf_tab(s: WalletState, i18n: &dyn UiI18n) -> Element {
    let balance_text = "—";
    let burn_label = i18n.btn_burn();
    let mint_label = i18n.btn_mint();
    let burn_msg = i18n.cyf_not_implemented("burn");
    let mint_msg = i18n.cyf_not_implemented("mint");

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px; height: 100%;",

            // Burn button (red, top)
            button {
                style: "width: 100%; padding: 16px; background: #dc3545; color: white; border: none; border-radius: 10px; font-size: 1.1em; font-weight: bold; cursor: pointer;",
                onclick: move |_| {
                    let mut modal = s.cyf_modal_message;
                    modal.set(Some(burn_msg.clone()));
                },
                "{burn_label}"
            }

            // White canvas with coin + balance
            div { style: "background: white; border-radius: 12px; border: 1px solid #e0e0e0; display: flex; align-items: center; justify-content: center; padding: 16px 24px;",

                // Coin image (SVG)
                div { style: "flex-shrink: 0; margin-right: 24px;",
                    svg {
                        width: "80",
                        height: "80",
                        view_box: "0 0 80 80",
                        // Outer ring
                        circle { cx: "40", cy: "40", r: "38", fill: "#FFD700", stroke: "#DAA520", stroke_width: "3" }
                        // Inner ring
                        circle { cx: "40", cy: "40", r: "30", fill: "none", stroke: "#DAA520", stroke_width: "1.5" }
                        // CYF text
                        text {
                            x: "40",
                            y: "45",
                            text_anchor: "middle",
                            font_size: "20",
                            font_weight: "bold",
                            fill: "#8B6914",
                            font_family: "sans-serif",
                            "CYF"
                        }
                    }
                }

                // Balance
                div { style: "text-align: right;",
                    p { style: "margin: 0; font-size: 2em; font-weight: bold; color: #333;",
                        "{balance_text}"
                    }
                    p { style: "margin: 4px 0 0; font-size: 0.85em; color: #888;",
                        "CYF"
                    }
                }
            }

            // Mint button (green, bottom)
            button {
                style: "width: 100%; padding: 16px; background: #28a745; color: white; border: none; border-radius: 10px; font-size: 1.1em; font-weight: bold; cursor: pointer;",
                onclick: move |_| {
                    let mut modal = s.cyf_modal_message;
                    modal.set(Some(mint_msg.clone()));
                },
                "{mint_label}"
            }
        }
    }
}
