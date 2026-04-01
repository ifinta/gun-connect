/**
 * sea_bridge.js — Thin JS bridge between GUN SEA and Rust/WASM.
 *
 * Exposes window.__sea_bridge with async functions that Rust calls
 * via js_sys::eval → Promise → JsFuture.
 *
 * Both gun.js AND sea.js must be loaded before this script.
 * SEA is available as Gun.SEA after sea.js loads.
 */
(function () {
    "use strict";

    var SEA = null;

    function _sea() {
        if (!SEA) {
            SEA = Gun.SEA;
            console.log("[sea_bridge._sea] SEA initialized:", SEA ? "ok" : "null");
        }
        return SEA;
    }

    /**
     * SEA.pair() — generate a new cryptographic key pair.
     * Returns JSON: { pub, priv, epub, epriv }
     * @returns {Promise<string>}
     */
    async function pair() {
        console.log("[sea_bridge.pair] Generating random key pair");
        var p = await _sea().pair();
        console.log("[sea_bridge.pair] Result:", p ? "got pair, pub=" + p.pub : "null");
        return JSON.stringify(p);
    }

    /**
     * Derive a deterministic P-256 key pair from a seed/passphrase.
     *
     * Uses PBKDF2 (100 000 iterations, SHA-256) with two fixed,
     * application-specific salts to derive independent ECDSA and ECDH
     * private key scalars.  The scalars are reduced modulo (n-1)+1 so
     * they are always valid P-256 private keys in [1, n-1].
     *
     * Public keys (x, y) are computed via P-256 scalar multiplication
     * in pure JS (BigInt arithmetic) — no PKCS#8 import needed, so this
     * works identically across all browsers including iOS Safari.
     *
     * Identical seed → identical key pair (deterministic).
     *
     * @param {string} seed - passphrase / seed to derive the key pair from
     * @returns {Promise<string>} JSON: { pub, priv, epub, epriv }
     */
    async function pairFromSeed(seed) {
        console.log("[sea_bridge.pairFromSeed] Deriving deterministic key pair, seed length=", seed.length);

        var subtle = crypto.subtle;
        var seedBytes = new TextEncoder().encode(seed);

        // Import the seed as PBKDF2 base key material
        var baseKey = await subtle.importKey(
            'raw', seedBytes, { name: 'PBKDF2' }, false, ['deriveBits']
        );

        // Derive 32 bytes for ECDSA private key
        var ecdsaBits = new Uint8Array(await subtle.deriveBits({
            name: 'PBKDF2',
            salt: new TextEncoder().encode('zsozso-sea-ecdsa'),
            iterations: 100000,
            hash: 'SHA-256'
        }, baseKey, 256));

        // Derive 32 bytes for ECDH private key (independent salt)
        var ecdhBits = new Uint8Array(await subtle.deriveBits({
            name: 'PBKDF2',
            salt: new TextEncoder().encode('zsozso-sea-ecdh'),
            iterations: 100000,
            hash: 'SHA-256'
        }, baseKey, 256));

        // ── P-256 (secp256r1) curve arithmetic ──────────────────────────
        var curveP = BigInt('0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF');
        var curveA = curveP - 3n; // a = -3
        var curveN = BigInt('0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551');
        var Gx = BigInt('0x6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296');
        var Gy = BigInt('0x4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5');

        function mod(a, m) { return ((a % m) + m) % m; }

        function modInverse(a, m) {
            var old_r = mod(a, m), r = m;
            var old_s = 1n, s = 0n;
            while (r !== 0n) {
                var q = old_r / r;
                var tmp_r = r; r = old_r - q * r; old_r = tmp_r;
                var tmp_s = s; s = old_s - q * s; old_s = tmp_s;
            }
            return mod(old_s, m);
        }

        function pointDouble(px, py) {
            if (py === 0n) return [0n, 0n];
            var s = mod((3n * px * px + curveA) * modInverse(2n * py, curveP), curveP);
            var xr = mod(s * s - 2n * px, curveP);
            var yr = mod(s * (px - xr) - py, curveP);
            return [xr, yr];
        }

        function pointAdd(x1, y1, x2, y2) {
            if (x1 === 0n && y1 === 0n) return [x2, y2];
            if (x2 === 0n && y2 === 0n) return [x1, y1];
            if (x1 === x2 && y1 === y2) return pointDouble(x1, y1);
            if (x1 === x2) return [0n, 0n];
            var s = mod((y2 - y1) * modInverse(x2 - x1, curveP), curveP);
            var xr = mod(s * s - x1 - x2, curveP);
            var yr = mod(s * (x1 - xr) - y1, curveP);
            return [xr, yr];
        }

        // Double-and-add scalar multiplication: k × P
        function scalarMult(k, px, py) {
            var rx = 0n, ry = 0n; // point at infinity
            var qx = px, qy = py;
            while (k > 0n) {
                if (k & 1n) { var t = pointAdd(rx, ry, qx, qy); rx = t[0]; ry = t[1]; }
                var t2 = pointDouble(qx, qy); qx = t2[0]; qy = t2[1];
                k >>= 1n;
            }
            return [rx, ry];
        }

        // Convert 32-byte Uint8Array → BigInt scalar in [1, n-1]
        function toValidScalar(bytes) {
            var hex = '';
            for (var i = 0; i < bytes.length; i++) hex += bytes[i].toString(16).padStart(2, '0');
            return (BigInt('0x' + hex) % (curveN - 1n)) + 1n;
        }

        // Convert BigInt → base64url string (32 bytes, zero-padded)
        function bigintToBase64url(n) {
            var hex = n.toString(16).padStart(64, '0');
            var bytes = new Uint8Array(32);
            for (var i = 0; i < 32; i++) bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
            var b64 = btoa(String.fromCharCode.apply(null, bytes));
            return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        }

        // ── Derive ECDSA key pair ───────────────────────────────────────
        var ecdsaScalar = toValidScalar(ecdsaBits);
        var ecdsaPub = scalarMult(ecdsaScalar, Gx, Gy);

        // ── Derive ECDH key pair ────────────────────────────────────────
        var ecdhScalar = toValidScalar(ecdhBits);
        var ecdhPub  = scalarMult(ecdhScalar, Gx, Gy);

        // ── Verify keys by importing via JWK (also validates correctness) ──
        var ecdsaJwk = {
            kty: 'EC', crv: 'P-256',
            x: bigintToBase64url(ecdsaPub[0]),
            y: bigintToBase64url(ecdsaPub[1]),
            d: bigintToBase64url(ecdsaScalar)
        };
        await subtle.importKey('jwk', ecdsaJwk,
            { name: 'ECDSA', namedCurve: 'P-256' }, false, ['sign']);

        var ecdhJwk = {
            kty: 'EC', crv: 'P-256',
            x: bigintToBase64url(ecdhPub[0]),
            y: bigintToBase64url(ecdhPub[1]),
            d: bigintToBase64url(ecdhScalar)
        };
        await subtle.importKey('jwk', ecdhJwk,
            { name: 'ECDH', namedCurve: 'P-256' }, false, ['deriveBits']);

        // Format in GUN SEA style: pub/epub = "x.y"  priv/epriv = "d"
        var p = {
            pub:   ecdsaJwk.x + '.' + ecdsaJwk.y,
            priv:  ecdsaJwk.d,
            epub:  ecdhJwk.x  + '.' + ecdhJwk.y,
            epriv: ecdhJwk.d,
        };

        console.log("[sea_bridge.pairFromSeed] Result:", p ? "got pair, pub=" + p.pub : "null");
        return JSON.stringify(p);
    }

    /**
     * SEA.sign(data, pair) — sign data with a key pair.
     * @param {string} data - the data to sign (plain string)
     * @param {string} pairJson - JSON key pair { pub, priv, epub, epriv }
     * @returns {Promise<string>} - the signed message string, or "" on error
     */
    async function sign(data, pairJson) {
        console.log("[sea_bridge.sign] data length=", data.length);
        var kp = JSON.parse(pairJson);
        console.log("[sea_bridge.sign] Signing with pub=", kp.pub);
        var msg = await _sea().sign(data, kp);
        console.log("[sea_bridge.sign] Result:", msg === undefined ? "undefined" : "signed");
        return (msg === undefined) ? "" : msg;
    }

    /**
     * SEA.verify(message, pub) — verify a signed message.
     * @param {string} message - the signed message from sign()
     * @param {string} pub - the public key string (pair.pub)
     * @returns {Promise<string>} - the original data if valid, or "" if invalid
     */
    async function verify(message, pub_key) {
        console.log("[sea_bridge.verify] message length=", message.length, "pub_key=", pub_key);
        var data = await _sea().verify(message, pub_key);
        console.log("[sea_bridge.verify] Result:", data === undefined ? "invalid" : "valid");
        if (data === undefined) return "";
        return (typeof data === "string") ? data : JSON.stringify(data);
    }

    /**
     * SEA.encrypt(data, pair) — encrypt data.
     * @param {string} data - plaintext to encrypt
     * @param {string} pairOrPassphrase - JSON key pair or a passphrase string
     * @returns {Promise<string>} - the encrypted message string, or "" on error
     */
    async function encrypt(data, pairOrPassphrase) {
        console.log("[sea_bridge.encrypt] data length=", data.length);
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var enc = await _sea().encrypt(data, key);
        console.log("[sea_bridge.encrypt] Result:", enc === undefined ? "undefined" : "encrypted");
        return (enc === undefined) ? "" : enc;
    }

    /**
     * SEA.decrypt(message, pair) — decrypt data.
     * @param {string} message - the encrypted message from encrypt()
     * @param {string} pairOrPassphrase - JSON key pair or a passphrase string
     * @returns {Promise<string>} - the decrypted plaintext, or "" on error
     */
    async function decrypt(message, pairOrPassphrase) {
        console.log("[sea_bridge.decrypt] message length=", message.length);
        var key;
        try { key = JSON.parse(pairOrPassphrase); } catch (e) { key = pairOrPassphrase; }
        var dec = await _sea().decrypt(message, key);
        console.log("[sea_bridge.decrypt] Result:", dec === undefined ? "undefined" : "decrypted");
        if (dec === undefined) return "";
        return (typeof dec === "string") ? dec : JSON.stringify(dec);
    }

    /**
     * SEA.work(data, salt) — proof of work / key derivation (PBKDF2).
     * @param {string} data - the data to hash
     * @param {string} saltJson - JSON key pair to use as salt, or a passphrase string
     * @returns {Promise<string>} - the derived hash, or "" on error
     */
    async function work(data, saltJson) {
        console.log("[sea_bridge.work] data length=", data.length, "salt length=", saltJson.length);
        var salt;
        try { salt = JSON.parse(saltJson); } catch (e) { salt = saltJson; }
        var hash = await _sea().work(data, salt);
        console.log("[sea_bridge.work] Result:", hash === undefined ? "undefined" : "hash computed");
        return (hash === undefined) ? "" : hash;
    }

    /**
     * SEA.secret(otherEpub, myPair) — ECDH shared secret derivation.
     * @param {string} otherEpub - the other party's epub (public encryption key)
     * @param {string} myPairJson - JSON of my full key pair
     * @returns {Promise<string>} - the shared secret string, or "" on error
     */
    async function secret(otherEpub, myPairJson) {
        console.log("[sea_bridge.secret] otherEpub length=", otherEpub.length);
        var myPair = JSON.parse(myPairJson);
        var sec = await _sea().secret(otherEpub, myPair);
        console.log("[sea_bridge.secret] Result:", sec === undefined ? "undefined" : "secret derived");
        return (sec === undefined) ? "" : sec;
    }

    // Expose on window
    window.__sea_bridge = {
        pair: pair,
        pairFromSeed: pairFromSeed,
        sign: sign,
        verify: verify,
        encrypt: encrypt,
        decrypt: decrypt,
        work: work,
        secret: secret,
    };
})();
