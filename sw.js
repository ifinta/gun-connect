// Change this two rows! No other change in this file is a need.
var MESSAGE_PREFIX = 'GUN_CONNECT';
var __BASE_PREFIX = '/gun-connect/';
var APP_NAME = 'gun-connect';
// The build.sh replaces it with a real APP_VERSION string...
var APP_VERSION = 'version';
// Cache version — it is only changes, if a need.
const CACHE_NAME = APP_VERSION+'-SW-v0.20';

function _ts() {
    const d = new Date();
    return d.getFullYear()
        + '-' + String(d.getMonth() + 1).padStart(2, '0')
        + '-' + String(d.getDate()).padStart(2, '0')
        + ' ' + String(d.getHours()).padStart(2, '0')
        + ':' + String(d.getMinutes()).padStart(2, '0')
        + ':' + String(d.getSeconds()).padStart(2, '0')
        + '.' + String(d.getMilliseconds()).padStart(3, '0');
}

// ── SW-side log ring buffer ──────────────────────────────────────────────────
var __SW_LOG_MAX = 1000;
var __swLogBuffer = [];

function _swLogPush(text) {
    __swLogBuffer.push(text);
    if (__swLogBuffer.length > __SW_LOG_MAX) __swLogBuffer.shift();
    // Push to all currently open clients
    self.clients.matchAll({ type: 'window' }).then(function(cls) {
        cls.forEach(function(c) {
            c.postMessage({ type: '__' + MESSAGE_PREFIX + '_SW_LOG', text: text });
        });
    });
}

// Grep-friendly line format: YYYY-MM-DD HH:MM:SS.MMM APP:<app> LL:SW <text>
const LOG = (...args) => {
    const text = args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ');
    const entry = _ts() + ' APP:' + APP_NAME + ' LL:SW ' + text;
    console.log(`[SW ${CACHE_NAME}]`, ...args);
    _swLogPush(entry);
};

const ERR = (...args) => {
    const text = args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ');
    const entry = _ts() + ' APP:' + APP_NAME + ' LL:SWE ' + text;
    console.error(`[SW ${CACHE_NAME}]`, ...args);
    _swLogPush(entry);
};

LOG('Script evaluated');
LOG('Base prefix:', __BASE_PREFIX);
LOG('Cache name:', CACHE_NAME);

// Legacy aliases (kept for backward compat with earlier handler code).
const LOG2 = function(...args) { LOG(...args); };
const ERR2 = function(...args) { ERR(...args); };

// Handle messages from clients (GET_LOGS, CLEAR_LOGS, SKIP_WAITING)
self.addEventListener('message', function(event) {
    if (!event.data) return;

    if (event.data.type === 'GET_LOGS') {
        var port = event.ports && event.ports[0];
        if (port) {
            port.postMessage({ logs: __swLogBuffer.slice() });
        }
    } else if (event.data.type === 'CLEAR_LOGS') {
        __swLogBuffer.length = 0;
    } else if (event.data.action === 'SKIP_WAITING') {
        LOG('SW: Received SKIP_WAITING signal from client. Taking over control...');
        self.skipWaiting();
    }
});

// ── Asset loading infrastructure ─────────────────────────────────────────────

var __ASSETS = null;

function _extractJson(html) {
    LOG('_extractJson: searching for asset tag in', html.length, 'chars');
    var el = html.indexOf('id="__EMBEDDED_ASSETS_START__"');
    if (el === -1) {
        ERR('_extractJson: asset tag not found in HTML');
        throw new Error('Asset tag not found');
    }
    var start = html.indexOf('>', el) + 1;
    var end = html.indexOf('<\/script>', start);
    if (end === -1) end = html.indexOf('</script>', start);
    LOG('_extractJson: parsing JSON from position', start, 'to', end, '(' + (end - start) + ' chars)');
    var result = JSON.parse(html.substring(start, end));
    LOG('_extractJson: parsed OK, keys:', Object.keys(result.assets).length, 'embedded assets found');
    return result;
}

function _b64ToArrayBuffer(b64) {
    var bin = atob(b64);
    var len = bin.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) bytes[i] = bin.charCodeAt(i);
    return bytes.buffer;
}

async function _loadAssets() {
    if (__ASSETS) {
        return;
    }
    LOG('_loadAssets: cache/memory miss, loading configuration from local Cache Storage…');

    var fetchUrl = __BASE_PREFIX + 'index.html';
    
    try {
        var cache = await caches.open(CACHE_NAME);
        var cachedResp = await cache.match(fetchUrl);
        
        if (!cachedResp) {
            ERR('_loadAssets: CRITICAL — index.html is missing from Cache Storage!');
            throw new Error('Application index.html missing from cache storage.');
        }

        var appHtml = await cachedResp.text();
        LOG('_loadAssets: application HTML loaded from cache, size:', appHtml.length, 'chars');
        
        __ASSETS = _extractJson(appHtml);
        LOG('_loadAssets: asset extraction complete. Cache is warm and active. ✓');
    } catch (e) {
        ERR('_loadAssets: FAILED —', e.message);
        throw e;
    }
}

function _serveEmbedded(key) {
    if (!__ASSETS) {
        ERR('_serveEmbedded: assets not loaded yet, returning null for', key);
        return null;
    }
    var data = __ASSETS.assets[key];
    if (!data) {
        LOG('_serveEmbedded: no asset found for key:', key);
        return null;
    }
    var mime = __ASSETS.mime[key] || 'application/octet-stream';
    LOG('_serveEmbedded: serving', key, '(' + mime + ',', data.length, 'base64 chars)');
    
    // Decompress: base64 → gzip bytes → DecompressionStream → raw bytes
    var gzBuf = _b64ToArrayBuffer(data);
    var ds = new DecompressionStream('gzip');
    var writer = ds.writable.getWriter();
    writer.write(new Uint8Array(gzBuf));
    writer.close();
    return new Response(ds.readable, {
        status: 200,
        headers: { 'Content-Type': mime }
    });
}

function _escapeHTML(str) {
    if (!str) return '';
    return str.replace(/&/g, '&amp;')
              .replace(/</g, '&lt;')
              .replace(/>/g, '&gt;')
              .replace(/"/g, '&quot;')
              .replace(/'/g, '&#x27;');
}

function _serve404(message) {
    ERR('_serve404: returning 404 because: ', message);
    var safeMessage = _escapeHTML(message);
    var html = '<!DOCTYPE html><html><head><meta charset="UTF-8">'
        + '<meta name="viewport" content="width=device-width,initial-scale=1.0">'
        + '<title>404 — Not Found</title>'
        + '<style>body{display:flex;align-items:center;justify-content:center;height:100vh;margin:0;'
        + 'font-family:sans-serif;background:#f5f5f5;color:#333;text-align:center}'
        + 'h1{font-size:4em;margin:0;color:#dc3545}p{color:#666;margin:8px 0}'
        + 'a{color:#17a2b8;text-decoration:none;font-weight:bold}'
        + '</style></head><body><div>'
        + '<h1>404</h1>'
        + '<p>The requested resource was not found.</p>'
        + '<p style="font-size:0.85em;font-family:monospace;word-break:break-all">' + safeMessage + '</p>'
        + '<p style="margin-top:24px"><a href="./">← Back to app</a></p>'
        + '</div></body></html>';
    return new Response(html, {
        status: 404,
        headers: { 'Content-Type': 'text/html; charset=utf-8' }
    });
}

// ── Lifecycle events ─────────────────────────────────────────────────────────

self.addEventListener('install', event => {
    LOG('Install event — triggered for version:', APP_VERSION);
    var fetchUrl = __BASE_PREFIX + 'index.html';

    event.waitUntil(
        caches.open(CACHE_NAME).then(function(cache) {
            LOG('Install: pre-caching application index.html from network:', fetchUrl);
            return cache.add(fetchUrl);
        }).then(function() {
            LOG('Install: asset pre-caching successful, activating new Service Worker ✓');
            self.skipWaiting();
        }).catch(function(e) {
            ERR('Install: pre-caching CRITICAL FAILURE —', e.message);
            throw e;
        })
    );
});

self.addEventListener('activate', event => {
    LOG('Activate event — cleaning old caches');
    // 1. Claim clients IMMEDIATELY
    LOG('Activate: claiming clients immediately');
    event.waitUntil(self.clients.claim());

    // 2. Then proceed with cleanup and notifications
    event.waitUntil(
        caches.keys().then(keys => {
            const old = keys.filter(k => k !== CACHE_NAME);
            LOG('Existing caches:', keys, '| Deleting:', old);
            const isUpdate = old.length > 0;
            return Promise.all(old.map(k => {
                LOG('Activate: deleting cache:', k);
                return caches.delete(k);
            })).then(() => isUpdate);
        }).then(isUpdate => {
            LOG('Old caches deleted, calling clients.claim()');
            return self.clients.claim().then(() => isUpdate);
        }).then(isUpdate => {
            // Only notify clients to reload when we actually replaced an older version.
            if (isUpdate) {
                return self.clients.matchAll({ type: 'window' }).then(clients => {
                    LOG('Activate: sending update notification to', clients.length, 'client(s)');
                    clients.forEach(c => {
                        c.postMessage({ type: '__'+MESSAGE_PREFIX+'_SW_UPDATED' });
                        LOG('Activate: notified client', c.id);
                    });
                    LOG('Update detected — notified', clients.length, 'client(s) to reload');
                });
            } else {
                LOG('No old caches found — not an update, skipping reload notification');
            }
        })
    );
});

// ── Fetch handler (SPA / Dioxus — all navigation → index.html) ──────────────

self.addEventListener('fetch', function(event) {
    var url = new URL(event.request.url);

    // Cross-origin — fall through to normal network fetch
    if (url.origin !== self.location.origin) {
        return;
    }

    // Navigation requests → serve the cached index.html directly (SPA / client-side routing)
    if (event.request.mode === 'navigate') {
        LOG('Fetch: navigation request for', url.pathname, '→ serving index.html from cache');
        event.respondWith(
            caches.open(CACHE_NAME).then(function(cache) {
                return cache.match(__BASE_PREFIX + 'index.html');
            }).then(function(resp) {
                if (resp) {
                    LOG('Fetch: navigation → index.html served directly from Cache Storage ✓');
                    return resp;
                }
                // Maybe it is the first run...
                return _loadAssets().then(function() {
                    return _serve404(url.pathname + ' (Cache miss on navigate)');
                });
            }).catch(function(e) {
                return _serve404(url.pathname + ' Fetch: navigation FAILED — ' + e.message);
            })
        );
        return;
    }    

    // Strip the base prefix to get the embedded-asset key.
    var relative = url.pathname;
    if (__BASE_PREFIX !== '/' && relative.startsWith(__BASE_PREFIX)) {
        relative = relative.substring(__BASE_PREFIX.length);
    } else if (relative.startsWith('/')) {
        relative = relative.substring(1);
    }

    event.respondWith(
        _loadAssets().then(function() {
            var resp = _serveEmbedded(relative);
            if (resp) {
                return resp;
            }
            return _serve404(url.pathname + ' Fetch: asset not found: ' + relative + ' → 404');
        }).catch(function(e) {
            return _serve404(url.pathname + ' Fetch: error serving ' + relative + ' — ' + e.message);
        })
    );
});
