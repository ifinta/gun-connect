use dioxus::prelude::*;
use qrcode::{QrCode, render::svg};
use crate::ui::state::WalletState;
use crate::ui::i18n::UiI18n;
use crate::app_version;

pub fn render_info_tab(s: WalletState, i18n: &dyn UiI18n) -> Element {
    let ver = app_version();
    let pk = s.public_key.read().clone();

    match pk {
        Some(key) => {
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
                if !ver.is_empty() {
                    p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                        "{i18n.info_version(ver)}"
                    }
                }
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
        None => {
            rsx! {
                if !ver.is_empty() {
                    p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                        "{i18n.info_version(ver)}"
                    }
                }
                div { style: "text-align: center; margin-top: 60px; color: #888;",
                    p { style: "font-size: 2em;", "ℹ️" }
                    p { "{i18n.info_no_key()}" }
                }
            }
        }
    }
}
