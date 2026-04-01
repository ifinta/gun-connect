use dioxus::prelude::*;

/// Copy text to clipboard using navigator.clipboard API.
pub fn copy_to_clipboard(text: &str) {
    let text = text.to_string();
    let _ = js_sys::eval("window.__zsozso_clipboard_dirty = true");
    spawn(async move {
        write_to_web_clipboard(&text).await;
    });
}

/// Clear the clipboard content.
pub fn clear_clipboard() {
    let _ = js_sys::eval("window.__zsozso_clipboard_dirty = false");
    spawn(async move {
        write_to_web_clipboard("").await;
    });
}

async fn write_to_web_clipboard(text: &str) -> bool {
    use wasm_bindgen_futures::JsFuture;

    let Some(window) = web_sys::window() else { return false; };
    let clipboard = window.navigator().clipboard();
    let promise = clipboard.write_text(text);
    JsFuture::from(promise).await.is_ok()
}

/// Register beforeunload and pagehide handlers that clear the clipboard
/// when the user closes the tab or navigates away.
/// Implemented in pure JS (via eval) to minimize overhead during page teardown.
/// Only clears when the dirty flag is set (i.e., something was copied).
pub fn register_beforeunload_cleanup() {
    let _ = js_sys::eval(r#"
        if (!window.__zsozso_unload_registered) {
            window.__zsozso_clipboard_dirty = false;
            window.__zsozso_unload_registered = true;
            function __zsozso_clear_clipboard() {
                if (!window.__zsozso_clipboard_dirty) return;
                window.__zsozso_clipboard_dirty = false;
                // Use a copy event handler to synchronously override clipboard data.
                // This is the only reliable way to clear the clipboard during page unload.
                var copyHandler = function(e) {
                    e.clipboardData.setData('text/plain', '');
                    e.preventDefault();
                };
                document.addEventListener('copy', copyHandler, true);
                try {
                    var ta = document.createElement('textarea');
                    ta.value = '.';
                    ta.style.position = 'fixed';
                    ta.style.opacity = '0';
                    document.body.appendChild(ta);
                    ta.select();
                    document.execCommand('copy');
                    document.body.removeChild(ta);
                } catch(e) {}
                document.removeEventListener('copy', copyHandler, true);
            }
            window.addEventListener('beforeunload', __zsozso_clear_clipboard);
            window.addEventListener('pagehide', __zsozso_clear_clipboard);
        }
    "#);
}
