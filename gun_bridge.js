/**
 * gun_bridge.js — Thin JS bridge between GUN.js and Rust/WASM.
 *
 * Exposes window.__gun_bridge with simple functions that Rust calls
 * via js_sys::eval / js_sys::Function / wasm_bindgen.
 *
 * GUN must be loaded before this script (via <script src="https://cdn.jsdelivr.net/npm/gun/gun.js">).
 */
(function () {
    "use strict";

    // The single GUN instance (created by gun_bridge_init)
    let _gun = null;

    // Active .on() subscriptions keyed by numeric ID
    const _subscriptions = {};
    let _nextSubId = 1;

    /**
     * Initialise the GUN instance with the given peer URLs.
     * @param {string} peersJson - JSON array of peer URLs, e.g. '["https://gun-manhattan.herokuapp.com/gun"]'
     */
    function init(peersJson) {
        console.log("[gun_bridge.init] peersJson=", peersJson);
        if (_gun) {
            console.log("[gun_bridge.init] Already initialised, adding any new peers");
            const peers = JSON.parse(peersJson || "[]");
            if (peers.length > 0) {
                _gun.opt({ peers: peers });
                console.log("[gun_bridge.init] Added peers to existing instance:", peers);
            }
            return;
        }
        const peers = JSON.parse(peersJson || "[]");
        console.log("[gun_bridge.init] Creating GUN instance with peers:", peers);
        _gun = Gun({ peers: peers });
        console.log("[gun_bridge.init] GUN instance created:", _gun ? "ok" : "null");
    }

    /**
     * Navigate a GUN chain along a path.
     * @param {string[]} path - Array of keys
     * @returns {GunInstance} - The chained reference
     */
    function _ref(path) {
        console.log("[gun_bridge._ref] Navigating path:", path);
        let ref = _gun;
        for (const key of path) {
            ref = ref.get(key);
        }
        return ref;
    }

    /**
     * gun.get(path).once(cb) — read once.
     * Returns a Promise that resolves to JSON string of the value (or "null").
     * @param {string} pathJson - JSON array of keys, e.g. '["users","alice","name"]'
     * @returns {Promise<string>} - JSON-encoded value
     */
    function get(pathJson) {
        console.log("[gun_bridge.get] pathJson=", pathJson);
        return new Promise(function (resolve) {
            const path = JSON.parse(pathJson);
            console.log("[gun_bridge.get] Parsed path:", path);
            _ref(path).once(function (data) {
                var result = JSON.stringify(data === undefined ? null : data);
                console.log("[gun_bridge.get] .once() data:", result);
                resolve(result);
            });
            // Safety timeout — if GUN finds nothing it may never call back
            setTimeout(function () {
                console.log("[gun_bridge.get] Safety timeout fired for path:", pathJson);
                resolve("null");
            }, 3000);
        });
    }

    /**
     * gun.get(path).put(value) — write/update.
     * @param {string} pathJson - JSON array of keys
     * @param {string} valueJson - JSON-encoded value to write
     * @returns {Promise<string>} - "ok" or error string
     */
    function put(pathJson, valueJson) {
        console.log("[gun_bridge.put] pathJson=", pathJson, "valueJson=", valueJson);
        return new Promise(function (resolve) {
            const path = JSON.parse(pathJson);
            const value = JSON.parse(valueJson);
            console.log("[gun_bridge.put] Parsed path:", path, "value:", value);
            _ref(path).put(value, function (ack) {
                console.log("[gun_bridge.put] ack:", ack);
                if (ack.err) {
                    console.log("[gun_bridge.put] ERROR:", ack.err);
                    resolve("err:" + ack.err);
                } else {
                    console.log("[gun_bridge.put] Success");
                    resolve("ok");
                }
            });
            // Safety timeout for the ack
            setTimeout(function () {
                console.log("[gun_bridge.put] Safety timeout fired");
                resolve("ok");
            }, 5000);
        });
    }

    /**
     * gun.get(path).on(cb) — subscribe to real-time changes.
     * The Rust side passes a callback ID; JS stores the subscription
     * and writes incoming data into a queue that Rust can poll.
     * @param {string} pathJson - JSON array of keys
     * @returns {number} subscription ID
     */
    function on(pathJson) {
        console.log("[gun_bridge.on] pathJson=", pathJson);
        const path = JSON.parse(pathJson);
        const subId = _nextSubId++;
        const queue = [];
        _subscriptions[subId] = { queue: queue, ref: null };

        const gunRef = _ref(path);
        _subscriptions[subId].ref = gunRef;

        gunRef.on(function (data, key) {
            console.log("[gun_bridge.on] subId=", subId, "data:", data, "key:", key);
            queue.push(JSON.stringify({ data: data === undefined ? null : data, key: key }));
        });

        console.log("[gun_bridge.on] Subscribed, subId=", subId);
        return subId;
    }

    /**
     * Poll queued updates for a given subscription.
     * Returns a JSON array of {data, key} objects, or "[]" if nothing new.
     * @param {number} subId
     * @returns {string} JSON array
     */
    function poll(subId) {
        const sub = _subscriptions[subId];
        if (!sub) {
            console.log("[gun_bridge.poll] No subscription for subId=", subId);
            return "[]";
        }
        const items = sub.queue.splice(0);
        if (items.length > 0) {
            console.log("[gun_bridge.poll] subId=", subId, "items count:", items.length);
        }
        return "[" + items.join(",") + "]";
    }

    /**
     * gun.get(path).off() — unsubscribe.
     * @param {number} subId
     */
    function off(subId) {
        console.log("[gun_bridge.off] subId=", subId);
        const sub = _subscriptions[subId];
        if (sub && sub.ref) {
            sub.ref.off();
            console.log("[gun_bridge.off] Unsubscribed");
        }
        delete _subscriptions[subId];
    }

    /**
     * gun.get(path).put(SEA.sign(value, pair)) — write with SEA signature.
     * Signs the value with the provided SEA key pair before storing.
     * @param {string} pathJson - JSON array of keys
     * @param {string} valueJson - JSON-encoded value to write
     * @param {string} pairJson - JSON SEA key pair { pub, priv, epub, epriv }
     * @returns {Promise<string>} - "ok" or error string
     */
    async function putSigned(pathJson, valueJson, pairJson) {
        console.log("[gun_bridge.putSigned] pathJson=", pathJson, "valueJson=", valueJson);
        var path = JSON.parse(pathJson);
        var value = JSON.parse(valueJson);
        var pair = JSON.parse(pairJson);
        console.log("[gun_bridge.putSigned] Signing value with SEA...");
        var signed = await Gun.SEA.sign(value, pair);
        console.log("[gun_bridge.putSigned] SEA.sign result:", signed);
        if (signed === undefined) {
            console.log("[gun_bridge.putSigned] ERROR: SEA sign failed");
            return "err:SEA sign failed";
        }
        return new Promise(function (resolve) {
            console.log("[gun_bridge.putSigned] Putting signed value at path:", path);
            _ref(path).put(signed, function (ack) {
                console.log("[gun_bridge.putSigned] ack:", ack);
                if (ack.err) {
                    console.log("[gun_bridge.putSigned] ERROR:", ack.err);
                    resolve("err:" + ack.err);
                } else {
                    console.log("[gun_bridge.putSigned] Success");
                    resolve("ok");
                }
            });
            setTimeout(function () {
                console.log("[gun_bridge.putSigned] Safety timeout fired");
                resolve("ok");
            }, 5000);
        });
    }

    /**
     * Dump the full local GUN graph, attempting to unwrap SEA-signed values
     * and decrypt SEA-encrypted values using the provided key pair.
     *
     * For each string value in the graph:
     *   - If it starts with "SEA": extract the "m" (message) field (signed data).
     *   - Otherwise: try Gun.SEA.decrypt(value, pair); on failure keep original.
     *
     * @param {string|null} pairJson - JSON SEA key pair, or null/empty if none
     * @returns {Promise<string>} - Pretty-printed JSON of the processed graph
     */
    async function dump(pairJson) {
        console.log("[gun_bridge.dump] starting, hasPair=", !!pairJson);
        if (!_gun) {
            console.log("[gun_bridge.dump] ERROR: GUN not initialised");
            return JSON.stringify({ error: "GUN not initialised" });
        }

        var pair = null;
        if (pairJson) {
            try { pair = JSON.parse(pairJson); } catch (e) { pair = null; }
        }

        var graph = _gun._.graph || {};
        var result = {};

        for (var soul in graph) {
            if (!graph.hasOwnProperty(soul)) continue;
            var node = graph[soul];
            var cleaned = {};
            for (var key in node) {
                if (!node.hasOwnProperty(key)) continue;
                if (key === "_") continue; // skip GUN metadata
                var val = node[key];
                if (typeof val === "string" && val.indexOf("SEA") === 0) {
                    // SEA-signed envelope: SEA{"m":"...","s":"..."}
                    try {
                        var seaJson = val.substring(3);
                        var parsed = JSON.parse(seaJson);
                        if (parsed && parsed.m !== undefined) {
                            cleaned[key] = { _sea_signed: true, value: parsed.m };
                        } else {
                            cleaned[key] = val;
                        }
                    } catch (e) {
                        cleaned[key] = val;
                    }
                } else if (typeof val === "string" && pair) {
                    // Try to decrypt with the user's key pair
                    try {
                        var dec = await Gun.SEA.decrypt(val, pair);
                        if (dec !== undefined) {
                            cleaned[key] = { _sea_decrypted: true, value: dec };
                        } else {
                            cleaned[key] = val;
                        }
                    } catch (e) {
                        cleaned[key] = val;
                    }
                } else {
                    cleaned[key] = val;
                }
            }
            result[soul] = cleaned;
        }

        var json = JSON.stringify(result, null, 2);
        console.log("[gun_bridge.dump] done, size=", json.length);
        return json;
    }

    /**
     * Add a peer to the live GUN instance.
     * @param {string} peerUrl - The peer relay URL to connect to
     */
    function addPeer(peerUrl) {
        console.log("[gun_bridge.addPeer] peerUrl=", peerUrl);
        if (!_gun) {
            console.log("[gun_bridge.addPeer] ERROR: GUN not initialised");
            return;
        }
        _gun.opt({ peers: [peerUrl] });
        console.log("[gun_bridge.addPeer] Peer added:", peerUrl);
    }

    // Expose on window
    window.__gun_bridge = {
        init: init,
        addPeer: addPeer,
        get: get,
        put: put,
        putSigned: putSigned,
        on: on,
        poll: poll,
        off: off,
        dump: dump,
    };
})();
