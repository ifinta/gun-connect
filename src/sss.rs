//! Shamir's Secret Sharing over GF(256).
//!
//! Splits a secret byte slice into `n` shares with a threshold `k` — any `k`
//! shares can reconstruct the original secret, but fewer than `k` reveals
//! nothing.
//!
//! Each share is `(index, Vec<u8>)` where `index` ∈ 1..=255 and the `Vec<u8>`
//! has the same length as the original secret.

use rand::RngCore;

// ── GF(256) arithmetic (irreducible polynomial x⁸ + x⁴ + x³ + x + 1) ──

/// Multiplication in GF(256).
fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result: u8 = 0;
    while b > 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        let carry = a & 0x80;
        a <<= 1;
        if carry != 0 {
            a ^= 0x1B; // x⁸ + x⁴ + x³ + x + 1
        }
        b >>= 1;
    }
    result
}

/// Multiplicative inverse in GF(256) via exponentiation (a^254 = a^{-1}).
fn gf256_inv(a: u8) -> u8 {
    if a == 0 {
        return 0; // 0 has no inverse; never called with 0 in practice
    }
    // Compute a^127 via repeated square-and-multiply
    let mut result = a;
    for _ in 0..6 {
        result = gf256_mul(result, result);
        result = gf256_mul(result, a);
    }
    // a^254 = (a^127)^2
    gf256_mul(result, result)
}

/// Evaluate polynomial `coeffs` at point `x` in GF(256).
/// coeffs[0] is the constant term (the secret byte).
fn gf256_eval(coeffs: &[u8], x: u8) -> u8 {
    // Horner's method
    let mut result: u8 = 0;
    for &c in coeffs.iter().rev() {
        result = gf256_mul(result, x) ^ c;
    }
    result
}

// ── Public API ──────────────────────────────────────────────────────────────

/// A single share of a split secret.
#[derive(Clone, Debug)]
pub struct Share {
    /// Share index (1..=255).
    pub index: u8,
    /// Share data — same length as the original secret.
    pub data: Vec<u8>,
}

/// Split `secret` into `n` shares with threshold `k`.
///
/// # Panics
/// Panics if `k < 2`, `n < k`, or `n > 255`.
pub fn split(secret: &[u8], k: u8, n: u8) -> Vec<Share> {
    assert!(k >= 2, "threshold must be at least 2");
    assert!(n >= k, "n must be >= k");
    assert!(n > 0, "n must be > 0"); // n >= k >= 2, so always true

    let mut rng = rand::rng();
    let mut shares: Vec<Share> = (1..=n)
        .map(|i| Share { index: i, data: Vec::with_capacity(secret.len()) })
        .collect();

    // For each byte of the secret, build a random polynomial of degree k-1
    // whose constant term is that secret byte.
    let mut coeffs = vec![0u8; k as usize];
    for &secret_byte in secret {
        coeffs[0] = secret_byte;
        // Fill random coefficients for degrees 1..k-1
        let random_part = &mut coeffs[1..];
        rng.fill_bytes(random_part);

        // Evaluate at each share index
        for share in shares.iter_mut() {
            share.data.push(gf256_eval(&coeffs, share.index));
        }
    }

    shares
}

/// Reconstruct the secret from `k` or more shares using Lagrange interpolation.
///
/// # Errors
/// Returns `Err` if shares have inconsistent lengths or fewer than 2 shares.
pub fn combine(shares: &[Share]) -> Result<Vec<u8>, String> {
    if shares.len() < 2 {
        return Err("need at least 2 shares".into());
    }
    let len = shares[0].data.len();
    if shares.iter().any(|s| s.data.len() != len) {
        return Err("shares have inconsistent lengths".into());
    }

    let mut secret = vec![0u8; len];

    for byte_idx in 0..len {
        // Lagrange interpolation at x = 0
        let mut value: u8 = 0;
        for (i, si) in shares.iter().enumerate() {
            let xi = si.index;
            let yi = si.data[byte_idx];

            // Compute Lagrange basis polynomial L_i(0)
            let mut basis: u8 = 1;
            for (j, sj) in shares.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = sj.index;
                // L_i(0) *= (0 - xj) / (xi - xj)  in GF(256)
                // In GF(256), subtraction = XOR, and -x = x.
                basis = gf256_mul(basis, gf256_mul(xj, gf256_inv(xi ^ xj)));
            }
            value ^= gf256_mul(yi, basis);
        }
        secret[byte_idx] = value;
    }

    Ok(secret)
}

/// Encode a share as a human-readable hex string: "XX:HHHH…"
/// where XX is the 2-digit hex share index and HHHH… is the hex-encoded data.
pub fn share_to_hex(share: &Share) -> String {
    let mut s = format!("{:02x}:", share.index);
    for &b in &share.data {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Decode a share from the hex format produced by `share_to_hex`.
pub fn share_from_hex(hex: &str) -> Result<Share, String> {
    let (idx_part, data_part) = hex.split_once(':')
        .ok_or("invalid share format: missing ':'")?;

    let index = u8::from_str_radix(idx_part, 16)
        .map_err(|e| format!("invalid share index: {}", e))?;
    if index == 0 {
        return Err("share index must be non-zero".into());
    }
    if data_part.len() % 2 != 0 {
        return Err("invalid share data: odd hex length".into());
    }
    let data: Result<Vec<u8>, _> = (0..data_part.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&data_part[i..i + 2], 16))
        .collect();
    let data = data.map_err(|e| format!("invalid share data hex: {}", e))?;

    Ok(Share { index, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_basic() {
        let secret = b"Hello Iceberg Protocol!";
        let shares = split(secret, 3, 7);
        assert_eq!(shares.len(), 7);

        // Any 3 shares should reconstruct
        let recovered = combine(&shares[0..3]).unwrap();
        assert_eq!(recovered, secret);

        // Different 3 shares
        let recovered2 = combine(&shares[2..5]).unwrap();
        assert_eq!(recovered2, secret);

        // All 7
        let recovered3 = combine(&shares).unwrap();
        assert_eq!(recovered3, secret);
    }

    #[test]
    fn hex_roundtrip() {
        let secret = b"test-secret";
        let shares = split(secret, 3, 5);
        for share in &shares {
            let hex = share_to_hex(share);
            let decoded = share_from_hex(&hex).unwrap();
            assert_eq!(decoded.index, share.index);
            assert_eq!(decoded.data, share.data);
        }
    }
}
