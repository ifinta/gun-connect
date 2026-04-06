/**
 * qr_scanner_bridge.js — Camera-based QR code scanner bridge for Rust/WASM.
 *
 * Uses the wascan library (loaded from jsdelivr CDN) for barcode/QR detection.
 *
 * Exposes window.__qr_scanner_bridge with:
 *   scan()  — opens a full-screen camera overlay, scans for a QR code,
 *             returns the decoded text as a Promise<string>.
 */
(function () {
    "use strict";

    var wascanReady = false;
    var wascanMod = null;

    // Dynamically load wascan ES module from CDN
    async function ensureWascan() {
        if (wascanReady) return;
        wascanMod = await import("./wascan.js");
        await wascanMod.default();   // init() — loads the WASM
        wascanMod.init_scanner();
        wascanReady = true;
    }

    /**
     * scan() → Promise<string>
     *
     * Creates a full-screen overlay with a <video> element, starts the
     * wascan stream scanner, and resolves with the first detected QR
     * code value.  The user can tap ✕ to cancel (rejects with "cancelled").
     */
    async function scan() {
        await ensureWascan();

        // ── Build overlay UI ──
        var overlay = document.createElement("div");
        overlay.style.cssText =
            "position:fixed;top:0;left:0;width:100%;height:100%;" +
            "background:#000;z-index:9999;display:flex;flex-direction:column;" +
            "align-items:center;justify-content:center;";

        var video = document.createElement("video");
        video.id = "__wascan_video_" + Date.now();
        video.setAttribute("playsinline", "");
        video.setAttribute("autoplay", "");
        video.setAttribute("muted", "");
        video.style.cssText = "max-width:100%;max-height:80%;border-radius:12px;";

        var cancelBtn = document.createElement("button");
        cancelBtn.textContent = "\u2715";
        cancelBtn.style.cssText =
            "position:absolute;top:16px;right:16px;font-size:2em;" +
            "background:rgba(255,255,255,0.25);color:#fff;border:none;" +
            "border-radius:50%;width:48px;height:48px;cursor:pointer;z-index:10000;";

        overlay.appendChild(video);
        overlay.appendChild(cancelBtn);
        document.body.appendChild(overlay);

        return new Promise(function (resolve, reject) {
            var settled = false;

            function cleanup() {
                if (!settled) { settled = true; }
                try { wascanMod.stop_stream_scan(); } catch (_) {}
                if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
            }

            cancelBtn.onclick = function () {
                cleanup();
                reject("cancelled");
            };

            // Register detection callback BEFORE starting the stream
            wascanMod.on_detect(function (result) {
                if (settled) return;
                if (result.success && result.value) {
                    settled = true;
                    cleanup();
                    resolve(result.value);
                }
            });

            // Start the camera stream via wascan
            try {
                wascanMod.start_stream_scan(video.id);
            } catch (err) {
                cleanup();
                reject("Camera error: " + (err.message || err));
            }
        });
    }

    window.__qr_scanner_bridge = { scan: scan };
})();
