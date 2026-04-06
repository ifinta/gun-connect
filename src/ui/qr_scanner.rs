use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Promise, Reflect, Function};

/// Get the `scan` function from the JS `__qr_scanner_bridge` object.
fn get_scan_fn() -> Result<Function, String> {
    let window = web_sys::window().ok_or("No window object")?;
    let bridge = Reflect::get(&window, &JsValue::from_str("__qr_scanner_bridge"))
        .map_err(|_| "QR scanner bridge not loaded (missing qr_scanner_bridge.js?)")?;
    let func = Reflect::get(&bridge, &JsValue::from_str("scan"))
        .map_err(|_| "No bridge method: scan")?;
    func.dyn_into::<Function>()
        .map_err(|_| "scan is not a function".to_string())
}

/// Open the camera, scan a QR code, and return its text content.
pub async fn scan_qr() -> Result<String, String> {
    let func = get_scan_fn()?;
    let promise_val = func.call0(&JsValue::NULL)
        .map_err(|e| format!("scan() call error: {:?}", e))?;
    let promise: Promise = promise_val.dyn_into()
        .map_err(|_| "Expected a Promise from JS bridge")?;
    let result = JsFuture::from(promise).await
        .map_err(|e| {
            let s = e.as_string().unwrap_or_else(|| format!("{:?}", e));
            s
        })?;
    result.as_string()
        .ok_or_else(|| "scan() returned non-string".to_string())
}
