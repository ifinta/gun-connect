// Shared IndexedDB-backed log ring for all same-origin PWA apps.
(function() {
    const APP_NAME = 'gun-connect';
    const SW_LOG_EVENT = '__GUN_CONNECT_SW_LOG';
    const WINDOW_LOG_API = '__gun_connect_log';
    const LOG_FILENAME_PREFIX = 'gun-connect-log-';

    // ── Ring-size config (keep in sync with sw.template.js __SW_LOG_MAX) ──
    // MEMORY_MAX      — client-side in-memory buffer (fast path for get())
    // MAX_DB_ENTRIES  — IndexedDB rolling cap (persistent, cross-tab, cross-app same-origin)
    // SW __SW_LOG_MAX — service-worker in-memory ring (see sw.template.js)
    const MEMORY_MAX = 300;
    const MAX_DB_ENTRIES = 4000;
    const DB_NAME = 'zsozso_logs';
    const DB_VERSION = 1;
    const STORE = 'entries';

    // Fixed length of the timestamp prefix produced by ts() (YYYY-MM-DD HH:MM:SS.MMM + trailing space).
    // Used to strip the timestamp when computing dedup keys for SW lines.
    const TS_PREFIX_LEN = 24;

    const buffer = [];
    const seenSwLines = new Set();

    let dbPromise = null;
    let trimInProgress = false;

    function ts() {
        const d = new Date();
        return d.getFullYear()
            + '-' + String(d.getMonth() + 1).padStart(2, '0')
            + '-' + String(d.getDate()).padStart(2, '0')
            + ' ' + String(d.getHours()).padStart(2, '0')
            + ':' + String(d.getMinutes()).padStart(2, '0')
            + ':' + String(d.getSeconds()).padStart(2, '0')
            + '.' + String(d.getMilliseconds()).padStart(3, '0');
    }

    function toText(args) {
        return Array.from(args).map(a => {
            if (typeof a === 'string') return a;
            try { return JSON.stringify(a); } catch (_) { return String(a); }
        }).join(' ');
    }

    function pushMemory(line) {
        buffer.push(line);
        if (buffer.length > MEMORY_MAX) buffer.shift();
    }

    function escapeRegex(text) {
        return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    function openDb() {
        if (dbPromise) return dbPromise;
        dbPromise = new Promise(function(resolve, reject) {
            const req = indexedDB.open(DB_NAME, DB_VERSION);
            req.onupgradeneeded = function() {
                const db = req.result;
                if (!db.objectStoreNames.contains(STORE)) {
                    const os = db.createObjectStore(STORE, { keyPath: 'id', autoIncrement: true });
                    os.createIndex('by_time', 'time');
                    os.createIndex('by_app', 'app');
                }
            };
            req.onsuccess = function() { resolve(req.result); };
            req.onerror = function() { reject(req.error || new Error('IndexedDB open failed')); };
        });
        return dbPromise;
    }

    async function trimDbIfNeeded() {
        if (trimInProgress) return;
        trimInProgress = true;
        try {
            const db = await openDb();
            const count = await new Promise(function(resolve, reject) {
                const tx = db.transaction(STORE, 'readonly');
                const req = tx.objectStore(STORE).count();
                req.onsuccess = function() { resolve(req.result || 0); };
                req.onerror = function() { reject(req.error); };
            });
            if (count <= MAX_DB_ENTRIES) return;
            const toDelete = count - MAX_DB_ENTRIES;
            await new Promise(function(resolve, reject) {
                let removed = 0;
                const tx = db.transaction(STORE, 'readwrite');
                const os = tx.objectStore(STORE);
                const cursorReq = os.openCursor();
                cursorReq.onsuccess = function() {
                    const cursor = cursorReq.result;
                    if (!cursor || removed >= toDelete) return;
                    os.delete(cursor.primaryKey);
                    removed += 1;
                    cursor.continue();
                };
                cursorReq.onerror = function() { reject(cursorReq.error); };
                tx.oncomplete = function() { resolve(); };
                tx.onerror = function() { reject(tx.error); };
            });
        } catch (_) {
            // best effort
        } finally {
            trimInProgress = false;
        }
    }

    async function persistLine(level, line) {
        try {
            const db = await openDb();
            await new Promise(function(resolve, reject) {
                const tx = db.transaction(STORE, 'readwrite');
                tx.objectStore(STORE).add({
                    app: APP_NAME,
                    level: level,
                    line: line,
                    time: Date.now(),
                });
                tx.oncomplete = function() { resolve(); };
                tx.onerror = function() { reject(tx.error); };
            });
            trimDbIfNeeded();
        } catch (_) {
            // best effort
        }
    }

    async function readAllLines() {
        try {
            const db = await openDb();
            return await new Promise(function(resolve, reject) {
                const tx = db.transaction(STORE, 'readonly');
                const req = tx.objectStore(STORE).getAll();
                req.onsuccess = function() {
                    const rows = Array.isArray(req.result) ? req.result : [];
                    rows.sort(function(a, b) {
                        const at = typeof a.time === 'number' ? a.time : 0;
                        const bt = typeof b.time === 'number' ? b.time : 0;
                        return at - bt;
                    });
                    resolve(rows.map(function(r) { return String(r.line || ''); }).join('\n'));
                };
                req.onerror = function() { reject(req.error); };
            });
        } catch (_) {
            return buffer.join('\n');
        }
    }

    async function clearDb() {
        try {
            const db = await openDb();
            await new Promise(function(resolve, reject) {
                const tx = db.transaction(STORE, 'readwrite');
                tx.objectStore(STORE).clear();
                tx.oncomplete = function() { resolve(); };
                tx.onerror = function() { reject(tx.error); };
            });
        } catch (_) {
            // best effort
        }
    }

    // Grep-friendly line format:
    //   YYYY-MM-DD HH:MM:SS.MMM APP:<app> LL:<level> <text>
    // e.g. grep 'APP:admin LL:ERR' for all admin errors.
    function append(level, args) {
        const line = ts() + ' APP:' + APP_NAME + ' LL:' + level + ' ' + toText(args);
        pushMemory(line);
        persistLine(level, line);
    }

    function pushSwLine(line) {
        // Dedup key ignores the timestamp prefix so lines produced once but
        // observed twice (GET_LOGS pull + streamed postMessage) collapse, even if
        // they arrive with slightly different timestamps is avoided by using content only.
        const key = line.length > TS_PREFIX_LEN ? line.substring(TS_PREFIX_LEN) : line;
        if (seenSwLines.has(key)) return Promise.resolve();
        seenSwLines.add(key);
        if (seenSwLines.size > MEMORY_MAX * 2) {
            // Keep the NEWEST MEMORY_MAX keys (Set iteration is insertion-ordered).
            const arr = Array.from(seenSwLines);
            const tail = arr.slice(arr.length - MEMORY_MAX);
            seenSwLines.clear();
            for (const v of tail) seenSwLines.add(v);
        }
        pushMemory(line);
        return persistLine('SW', line);
    }

    const origLog = console.log.bind(console);
    const origErr = console.error.bind(console);

    console.log = function() {
        append('LOG', arguments);
        origLog.apply(console, arguments);
    };

    console.error = function() {
        append('ERR', arguments);
        origErr.apply(console, arguments);
    };

    function pullSwLogs() {
        if (!navigator.serviceWorker || !navigator.serviceWorker.controller) {
            return Promise.resolve();
        }
        return new Promise(function(resolve) {
            let settled = false;
            const done = function() { if (!settled) { settled = true; resolve(); } };
            try {
                const ch = new MessageChannel();
                ch.port1.onmessage = function(e) {
                    if (e.data && Array.isArray(e.data.logs)) {
                        const pending = e.data.logs.map(function(line) { return pushSwLine(line); });
                        Promise.all(pending).then(done, done);
                    } else {
                        done();
                    }
                };
                navigator.serviceWorker.controller.postMessage({ type: 'GET_LOGS' }, [ch.port2]);
                // Safety: if SW never answers, don't hang reads.
                setTimeout(done, 1500);
            } catch (_) { done(); }
        });
    }

    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.addEventListener('message', function(event) {
            if (event.data && event.data.type === SW_LOG_EVENT) {
                pushSwLine(event.data.text);
            }
        });
        navigator.serviceWorker.ready.then(function() {
            pullSwLogs();
        });
    }

    async function saveLogs() {
        const body = await readAllLines();
        if (!body) return 'EMPTY';
        try {
            const blob = new Blob([body], { type: 'text/plain; charset=utf-8' });
            const url = URL.createObjectURL(blob);
            const d = new Date();
            const stamp = d.getFullYear()
                + String(d.getMonth() + 1).padStart(2, '0')
                + String(d.getDate()).padStart(2, '0')
                + '-' + String(d.getHours()).padStart(2, '0')
                + String(d.getMinutes()).padStart(2, '0')
                + String(d.getSeconds()).padStart(2, '0');
            const filename = LOG_FILENAME_PREFIX + stamp + '.log';
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            return 'OK';
        } catch (err) {
            return 'ERR:' + (err.message || err);
        }
    }

    const versionRe = new RegExp('\\b(' + escapeRegex(APP_NAME) + '-[\\w.-]+)\\b');

    window[WINDOW_LOG_API] = {
        get: function() {
            pullSwLogs();
            return buffer.join('\n');
        },
        get_all: async function() {
            await pullSwLogs();
            return readAllLines();
        },
        count: function() { return buffer.length; },
        clear: function() {
            buffer.length = 0;
            seenSwLines.clear();
            clearDb();
            if (navigator.serviceWorker && navigator.serviceWorker.controller) {
                try {
                    navigator.serviceWorker.controller.postMessage({ type: 'CLEAR_LOGS' });
                } catch (_) {}
            }
        },
        save: function() {
            return saveLogs();
        },
        version: function() {
            for (let i = buffer.length - 1; i >= 0; i--) {
                const m = buffer[i].match(versionRe);
                if (m) return m[1];
            }
            return 'detecting...';
        }
    };
})();
