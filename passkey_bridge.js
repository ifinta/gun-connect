/**
 * passkey_bridge.js — WebAuthn Passkey + Web Crypto bridge for Rust/WASM.
 *
 * Exposes window.__passkey_bridge with:
 *   register_only()     — register passkey (one biometric prompt, no PRF)
 *   init()              — authenticate with PRF (credential must exist)
 *   verify()            — lightweight auth check (no PRF)
 *   encrypt(b64, key)   — AES-GCM encrypt with PRF-derived key
 *   decrypt(b64, key)   — AES-GCM decrypt with PRF-derived key
 */
(function () {
    "use strict";

    const DB_NAME = "ZsozsoPasskey";
    const STORE_NAME = "credentials";

    // Fixed PRF salt (deterministic key derivation per credential)
    const PRF_SALT = new TextEncoder().encode("zsozso-passkey-encryption-v1");

    // ── IndexedDB helpers for credential ID persistence ──

    function openDb() {
        return new Promise(function (resolve, reject) {
            var req = indexedDB.open(DB_NAME, 1);
            req.onupgradeneeded = function () { req.result.createObjectStore(STORE_NAME); };
            req.onsuccess = function () { resolve(req.result); };
            req.onerror = function () { reject(req.error); };
        });
    }

    async function saveCredentialId(id) {
        var db = await openDb();
        var tx = db.transaction(STORE_NAME, "readwrite");
        tx.objectStore(STORE_NAME).put(id, "current");
        return new Promise(function (resolve, reject) {
            tx.oncomplete = function () { resolve(); };
            tx.onerror = function () { reject(tx.error); };
        });
    }

    async function loadCredentialId() {
        var db = await openDb();
        var tx = db.transaction(STORE_NAME, "readonly");
        var req = tx.objectStore(STORE_NAME).get("current");
        return new Promise(function (resolve, reject) {
            req.onsuccess = function () { resolve(req.result || null); };
            req.onerror = function () { reject(req.error); };
        });
    }

    // ── Base64 helpers ──

    function bufToBase64(buf) {
        return btoa(String.fromCharCode.apply(null, new Uint8Array(buf)));
    }

    function base64ToBuf(b64) {
        var bin = atob(b64);
        var arr = new Uint8Array(bin.length);
        for (var i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
        return arr.buffer;
    }

    // ── WebAuthn: register ──

    async function register() {
        var challenge = crypto.getRandomValues(new Uint8Array(32));

        var credential = await navigator.credentials.create({
            publicKey: {
                challenge: challenge,
                rp: { name: "Zsozso Wallet", id: location.hostname },
                user: {
                    id: crypto.getRandomValues(new Uint8Array(16)),
                    name: "zsozso-user",
                    displayName: "Zsozso Wallet User",
                },
                pubKeyCredParams: [
                    { alg: -7, type: "public-key" },    // ES256
                    { alg: -257, type: "public-key" },   // RS256
                ],
                authenticatorSelection: {
                    residentKey: "preferred",
                    userVerification: "required",
                },
                extensions: { prf: {} },
            },
        });

        var credId = bufToBase64(credential.rawId);
        await saveCredentialId(credId);
        return credId;
    }

    // ── WebAuthn: authenticate with PRF ──

    async function authenticateWithPrf() {
        var credIdB64 = await loadCredentialId();
        var challenge = crypto.getRandomValues(new Uint8Array(32));

        var opts = {
            publicKey: {
                challenge: challenge,
                userVerification: "required",
                extensions: {
                    prf: { eval: { first: PRF_SALT } },
                },
            },
        };

        if (credIdB64) {
            opts.publicKey.allowCredentials = [{
                id: base64ToBuf(credIdB64),
                type: "public-key",
            }];
        }

        var assertion = await navigator.credentials.get(opts);

        var prfResult = assertion.getClientExtensionResults().prf;
        if (prfResult && prfResult.results && prfResult.results.first) {
            return bufToBase64(prfResult.results.first);
        }
        return null; // PRF not available, auth still succeeded
    }

    // ── Public: register_only (one biometric prompt, no PRF) ──

    async function registerOnly() {
        if (!window.PublicKeyCredential) {
            return JSON.stringify({ success: false, error: "passkeys_not_supported" });
        }
        try {
            var credIdB64 = await loadCredentialId();
            if (!credIdB64) {
                await register();
            }
            return JSON.stringify({ success: true, error: null });
        } catch (e) {
            return JSON.stringify({ success: false, error: e.message || String(e) });
        }
    }

    // ── Public: init (authenticate with PRF — credential must already exist) ──

    async function init() {
        if (!window.PublicKeyCredential) {
            return JSON.stringify({ success: false, prfKey: null, error: "passkeys_not_supported" });
        }

        try {
            var credIdB64 = await loadCredentialId();
            if (!credIdB64) {
                // No credential yet — caller should use register_only() first
                return JSON.stringify({ success: false, prfKey: null, error: "not_registered" });
            }

            var prfKey = await authenticateWithPrf();
            return JSON.stringify({ success: true, prfKey: prfKey, error: null });
        } catch (e) {
            return JSON.stringify({ success: false, prfKey: null, error: e.message || String(e) });
        }
    }

    // ── Public: verify (lightweight auth, no PRF) ──

    async function verify() {
        try {
            var credIdB64 = await loadCredentialId();
            if (!credIdB64) return false;

            var challenge = crypto.getRandomValues(new Uint8Array(32));
            await navigator.credentials.get({
                publicKey: {
                    challenge: challenge,
                    userVerification: "required",
                    allowCredentials: [{
                        id: base64ToBuf(credIdB64),
                        type: "public-key",
                    }],
                },
            });
            return true;
        } catch (_) {
            return false;
        }
    }

    // ── Web Crypto: AES-GCM key derivation from PRF output ──

    async function deriveKey(prfKeyB64) {
        var prfBytes = base64ToBuf(prfKeyB64);
        var keyMaterial = await crypto.subtle.importKey(
            "raw", prfBytes, "HKDF", false, ["deriveKey"]
        );
        return crypto.subtle.deriveKey(
            {
                name: "HKDF",
                hash: "SHA-256",
                salt: new TextEncoder().encode("zsozso-aes-key-v1"),
                info: new TextEncoder().encode("encryption"),
            },
            keyMaterial,
            { name: "AES-GCM", length: 256 },
            false,
            ["encrypt", "decrypt"]
        );
    }

    // ── Public: encrypt plaintext (base64) with PRF-derived AES key ──

    async function encrypt(plaintextB64, prfKeyB64) {
        var key = await deriveKey(prfKeyB64);
        var iv = crypto.getRandomValues(new Uint8Array(12));
        var plaintext = base64ToBuf(plaintextB64);
        var ciphertext = await crypto.subtle.encrypt(
            { name: "AES-GCM", iv: iv }, key, plaintext
        );
        // Prepend IV to ciphertext
        var result = new Uint8Array(iv.length + ciphertext.byteLength);
        result.set(iv, 0);
        result.set(new Uint8Array(ciphertext), iv.length);
        return bufToBase64(result.buffer);
    }

    // ── Public: decrypt ciphertext (base64) with PRF-derived AES key ──

    async function decrypt(ciphertextB64, prfKeyB64) {
        var key = await deriveKey(prfKeyB64);
        var data = new Uint8Array(base64ToBuf(ciphertextB64));
        var iv = data.slice(0, 12);
        var ciphertext = data.slice(12);
        var plaintext = await crypto.subtle.decrypt(
            { name: "AES-GCM", iv: iv }, key, ciphertext
        );
        return bufToBase64(plaintext);
    }

    // ── Expose bridge ──

    window.__passkey_bridge = {
        register_only: registerOnly,
        init: init,
        verify: verify,
        encrypt: encrypt,
        decrypt: decrypt,
    };
})();
