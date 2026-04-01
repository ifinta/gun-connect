use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::ui::i18n::UiI18n;
use crate::ui::state::WalletState;

/// Read the in-app log buffer from the JS bridge.
fn read_log_buffer() -> String {
    js_sys::eval("window.__zsozso_log ? window.__zsozso_log.get() : ''")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// Clear the in-app log buffer.
fn clear_log_buffer() {
    let _ = js_sys::eval("window.__zsozso_log && window.__zsozso_log.clear()");
}

/// Save the log buffer as a local file download. Returns a status string.
async fn save_log_buffer() -> String {
    let promise = match js_sys::eval("window.__zsozso_log && window.__zsozso_log.save()") {
        Ok(v) => v,
        Err(_) => return "ERR:eval".to_string(),
    };
    let promise: js_sys::Promise = match promise.dyn_into() {
        Ok(p) => p,
        Err(_) => return "ERR:not_a_promise".to_string(),
    };
    match JsFuture::from(promise).await {
        Ok(val) => val.as_string().unwrap_or_else(|| "ERR:unknown".to_string()),
        Err(e) => format!("ERR:{}", e.as_string().unwrap_or_default()),
    }
}

/// Dump the full local GUN graph, attempting to decrypt data with the user's SEA key pair.
/// Returns the processed JSON string, or an error string prefixed with "ERR:".
async fn dump_gun_db(pair_json: Option<String>) -> String {
    let js_code = match &pair_json {
        Some(pj) => format!(
            "window.__gun_bridge && window.__gun_bridge.dump('{}')",
            pj.replace('\\', "\\\\").replace('\'', "\\'")
        ),
        None => "window.__gun_bridge && window.__gun_bridge.dump(null)".to_string(),
    };
    let promise = match js_sys::eval(&js_code) {
        Ok(v) => v,
        Err(_) => return "ERR:eval".to_string(),
    };
    let promise: js_sys::Promise = match promise.dyn_into() {
        Ok(p) => p,
        Err(_) => return "ERR:not_a_promise".to_string(),
    };
    match JsFuture::from(promise).await {
        Ok(val) => val.as_string().unwrap_or_else(|| "ERR:unknown".to_string()),
        Err(e) => format!("ERR:{}", e.as_string().unwrap_or_default()),
    }
}

/// Save text content as a browser file download.
fn save_text_as_file(content: &str, filename: &str) -> String {
    let js_code = format!(
        r#"(function() {{
            try {{
                var blob = new Blob([{}], {{ type: 'application/json; charset=utf-8' }});
                var url = URL.createObjectURL(blob);
                var a = document.createElement('a');
                a.href = url;
                a.download = {};
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                return 'OK';
            }} catch (err) {{
                return 'ERR:' + (err.message || err);
            }}
        }})()"#,
        serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string()),
        serde_json::to_string(filename).unwrap_or_else(|_| "\"dump.json\"".to_string()),
    );
    match js_sys::eval(&js_code) {
        Ok(v) => v.as_string().unwrap_or_else(|| "ERR:unknown".to_string()),
        Err(_) => "ERR:eval".to_string(),
    }
}

pub fn render_log_tab(s: WalletState, i18n: &dyn UiI18n) -> Element {
    let mut log_text = use_signal(String::new);
    let mut save_status = use_signal(|| String::new());

    // Load log immediately when this tab is rendered (on mount)
    use_effect(move || {
        log_text.set(read_log_buffer());
    });

    // Pre-compute i18n strings we need inside async closures
    let saving_msg = i18n.log_saving().to_string();
    let save_ok_msg = i18n.log_save_ok().to_string();
    let save_empty_msg = i18n.log_save_empty().to_string();
    let dumping_msg = i18n.log_dumping().to_string();
    let dump_ok_msg = i18n.log_dump_ok().to_string();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100%;",
            // Title
            h3 { style: "margin: 0 0 10px 0; font-size: 1.1em;", "{i18n.tab_log()}" }

            // Buttons — each full width, stacked vertically
            div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 10px;",
                // Refresh button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #007bff; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: move |_| {
                        log_text.set(read_log_buffer());
                    },
                    "{i18n.log_refresh()}"
                }
                // Save button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #17a2b8; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: {
                        let saving_msg = saving_msg.clone();
                        let save_ok_msg = save_ok_msg.clone();
                        let save_empty_msg = save_empty_msg.clone();
                        move |_| {
                            let saving_msg = saving_msg.clone();
                            let save_ok_msg = save_ok_msg.clone();
                            let save_empty_msg = save_empty_msg.clone();
                            spawn(async move {
                                save_status.set(saving_msg);
                                let result = save_log_buffer().await;
                                match result.as_str() {
                                    "OK" => save_status.set(save_ok_msg),
                                    "EMPTY" => save_status.set(save_empty_msg),
                                    other => save_status.set(format!("\u{274C} {}", other)),
                                }
                            });
                        }
                    },
                    "{i18n.log_save()}"
                }
                // Dump GUN DB button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #6f42c1; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: {
                        let dumping_msg = dumping_msg.clone();
                        let dump_ok_msg = dump_ok_msg.clone();
                        move |_| {
                            let dumping_msg = dumping_msg.clone();
                            let dump_ok_msg = dump_ok_msg.clone();
                            let pair_json = s.sea_key_pair.read().as_ref().map(|p| p.to_json());
                            spawn(async move {
                                save_status.set(dumping_msg);
                                let result = dump_gun_db(pair_json).await;
                                if result.starts_with("ERR:") {
                                    save_status.set(format!("\u{274C} {}", result));
                                } else {
                                    let save_result = save_text_as_file(&result, "zsozso-gundb-dump.json");
                                    match save_result.as_str() {
                                        "OK" => save_status.set(dump_ok_msg),
                                        other => save_status.set(format!("\u{274C} {}", other)),
                                    }
                                }
                            });
                        }
                    },
                    "{i18n.btn_dump_gun_db()}"
                }
                // Clear button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #dc3545; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: move |_| {
                        clear_log_buffer();
                        log_text.set(String::new());
                        save_status.set(String::new());
                    },
                    "{i18n.log_clear()}"
                }
            }

            // Save status line (only shown when non-empty)
            if !save_status.read().is_empty() {
                p {
                    style: "margin: 0 0 8px 0; font-size: 0.85em; color: #6c757d;",
                    "{save_status.read()}"
                }
            }

            // Log output area
            pre {
                style: "flex: 1; overflow-y: auto; background: #1e1e1e; color: #d4d4d4; padding: 12px; border-radius: 8px; font-size: 0.72em; line-height: 1.5; white-space: pre-wrap; word-break: break-all; margin: 0; font-family: 'Courier New', monospace;",
                {log_text.read().clone()}
            }
        }
    }
}
