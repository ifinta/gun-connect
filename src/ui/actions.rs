use dioxus::prelude::*;
use zsozso_common::Language;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};

use zsozso_store::IndexedDbStore;

use ed25519_dalek::{Signer, SigningKey};
use stellar_strkey::{ed25519, Strkey};
use stellar_xdr::curr::{
    MuxedAccount, Uint256, Transaction, SequenceNumber, Memo, Operation,
    OperationBody, Preconditions, TransactionExt, VecM,
    TransactionEnvelope, TransactionV1Envelope, DecoratedSignature, Hash,
    Signature, BytesM, SignatureHint, WriteXdr, Limits, TimeBounds, TimePoint,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    ManageDataOp, String64, DataValue, StringM,
};
use sha2::{Sha256, Digest};

fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }

/// The well-known manageData key used to publish GUN relay URLs on Stellar.
pub const MANAGE_DATA_KEY: &str = "gun_connect_relay";

/// Check if a Stellar data entry key is a relay key (`NN_gun_connect_relay`).
pub fn is_relay_key(key: &str) -> bool {
    key.ends_with(&format!("_{}", MANAGE_DATA_KEY))
}

/// Generate the manageData key name for a relay at the given index.
fn relay_key_name(index: usize) -> String {
    format!("{:02}_{}", index, MANAGE_DATA_KEY)
}

pub async fn activate_test_account(pubkey: Option<String>, net_env: NetworkEnvironment, lang: Language) -> Result<String, String> {
    let pubkey = pubkey.ok_or_else(|| "No public key".to_string())?;
    let lgr = StellarLedger::new(net_env, lang);
    lgr.activate_test_account(&pubkey).await
}

/// Build, sign and submit a manageData transaction that publishes all connected relay URLs.
///
/// Each relay gets its own manageData key: `00_gun_connect_relay`, `01_gun_connect_relay`, etc.
/// Old keys beyond the current list length are cleared (set to None).
pub async fn publish_relays(
    secret_key: &str,
    relay_urls: &[String],
    net_env: NetworkEnvironment,
) -> Result<(), String> {
    if relay_urls.is_empty() {
        log("[publish_relays] No relay URLs to publish");
        return Ok(());
    }

    // Decode key
    let priv_key = match Strkey::from_string(secret_key) {
        Ok(Strkey::PrivateKeyEd25519(pk)) => pk,
        _ => return Err("Invalid secret key".into()),
    };
    let signing_key = SigningKey::from_bytes(&priv_key.0);
    let pub_bytes = signing_key.verifying_key().to_bytes();
    let public_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();

    // Fetch account: sequence number + existing data keys
    let horizon = horizon_url(net_env);
    let client = reqwest::Client::new();
    let url = format!("{}/accounts/{}", horizon, public_key_str);
    let response = client.get(&url).send().await
        .map_err(|e| format!("Horizon unreachable: {}", e))?;
    if !response.status().is_success() {
        return Err("Account not found \u{2014} activate with faucet first".into());
    }

    let acct_body: serde_json::Value = response.json().await
        .map_err(|e| format!("JSON error: {}", e))?;
    let next_seq: i64 = acct_body.get("sequence")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0) + 1;

    // Collect existing relay data keys on-chain
    let existing_keys: std::collections::HashSet<String> = acct_body.get("data")
        .and_then(|d| d.as_object())
        .map(|data| {
            data.keys()
                .filter(|k| is_relay_key(k))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    log(&format!("[publish_relays] Account has {} existing relay key(s)", existing_keys.len()));

    // Build manageData operations — one per relay URL
    let mut ops = Vec::new();
    for (i, relay_url) in relay_urls.iter().enumerate() {
        let key_name = relay_key_name(i);
        ops.push(Operation {
            source_account: None,
            body: OperationBody::ManageData(ManageDataOp {
                data_name: String64(StringM::try_from(key_name.as_str())
                    .map_err(|e| format!("Key too long: {}", e))?),
                data_value: Some(DataValue(BytesM::try_from(relay_url.as_bytes().to_vec())
                    .map_err(|e| format!("Value too long: {}", e))?)),
            }),
        });
    }

    // Only delete keys that exist on-chain but aren't being overwritten
    let written_keys: std::collections::HashSet<String> = relay_urls.iter().enumerate()
        .map(|(i, _)| relay_key_name(i))
        .collect();
    for old_key in &existing_keys {
        if !written_keys.contains(old_key) {
            log(&format!("[publish_relays] Deleting old key: {}", old_key));
            ops.push(Operation {
                source_account: None,
                body: OperationBody::ManageData(ManageDataOp {
                    data_name: String64(StringM::try_from(old_key.as_str())
                        .map_err(|e| format!("Key too long: {}", e))?),
                    data_value: None,
                }),
            });
        }
    }

    let current_unix_time = (js_sys::Date::now() / 1000.0) as u64;
    let passphrase = network_passphrase(net_env);

    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(pub_bytes)),
        fee: 100 * ops.len() as u32,
        seq_num: SequenceNumber(next_seq),
        cond: Preconditions::Time(TimeBounds {
            min_time: TimePoint(0),
            max_time: TimePoint(current_unix_time + 300),
        }),
        memo: Memo::None,
        operations: VecM::try_from(ops)
            .map_err(|e| format!("Operations error: {}", e))?,
        ext: TransactionExt::V0,
    };

    // Sign
    let network_id = Hash(Sha256::digest(passphrase.as_bytes()).into());
    let payload = TransactionSignaturePayload {
        network_id,
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
    };
    let tx_payload_xdr = payload.to_xdr(Limits::none())
        .map_err(|e| format!("XDR error: {}", e))?;
    let tx_hash = Sha256::digest(&tx_payload_xdr);
    let sig_bytes = signing_key.sign(&tx_hash).to_bytes();

    let mut hint_bytes = [0u8; 4];
    hint_bytes.copy_from_slice(&pub_bytes[pub_bytes.len() - 4..]);

    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx,
        signatures: VecM::try_from(vec![
            DecoratedSignature {
                hint: SignatureHint(hint_bytes),
                signature: Signature(BytesM::try_from(sig_bytes).unwrap()),
            }
        ]).unwrap(),
    });

    let xdr = envelope.to_xdr_base64(Limits::none())
        .map_err(|e| format!("XDR encode error: {}", e))?;

    // Submit
    let lgr = StellarLedger::new(net_env, Language::English);
    match lgr.submit_transaction(&xdr).await {
        Ok(msg) => {
            log(&format!("[publish_relays] Published {} relays: {}", relay_urls.len(), msg));
            Ok(())
        }
        Err(e) => Err(format!("Submit failed: {}", e)),
    }
}

fn horizon_url(net: NetworkEnvironment) -> &'static str {
    match net {
        NetworkEnvironment::Test => "https://horizon-testnet.stellar.org",
        NetworkEnvironment::Production => "https://horizon.stellar.org",
    }
}

fn network_passphrase(net: NetworkEnvironment) -> &'static str {
    match net {
        NetworkEnvironment::Test => "Test SDF Network ; September 2015",
        NetworkEnvironment::Production => "Public Global Stellar Network ; September 2015",
    }
}

/// Maximum number of connected relays.
pub const MAX_RELAYS: usize = 5;

/// A relay in the user's connected relay list.
#[derive(Clone, Debug, PartialEq)]
pub struct RelayEntry {
    pub url: String,
    /// None = not checked yet, Some(true) = connected, Some(false) = unreachable.
    pub reachable: Option<bool>,
    /// True while a check is in progress.
    pub checking: bool,
}

/// A relay discovered from Stellar manageData entries.
#[derive(Clone, Debug)]
pub struct DiscoveredRelay {
    pub url: String,
    pub reachable: Option<bool>,
}

/// Discover GUN relay URLs published on the Stellar testnet.
///
/// Three-phase strategy:
/// 1. Query known accounts directly (`/accounts/{addr}`) for current relay data entries.
/// 2. Scan recent transactions backwards, decode envelope XDR to find ManageData ops
///    with key `NN_gun_connect_relay`, then collect the source accounts.
/// 3. Fetch data entries from newly discovered accounts.
///
/// Results are deduplicated by URL, excluding already-connected relays.
///
/// `progress` — updated with human-readable scan status after each page.
/// `stop`     — checked after each page; if true, scan aborts early.
/// Maximum: 1 000 000 transactions (5000 pages × 200).
pub async fn discover_relays(
    exclude: &std::collections::HashSet<String>,
    known_accounts: &[String],
    mut progress: Signal<String>,
    stop: Signal<bool>,
) -> (Vec<DiscoveredRelay>, Vec<String>) {
    use stellar_xdr::curr::{TransactionEnvelope, ReadXdr, OperationBody, Limits};

    let horizon = horizon_url(NetworkEnvironment::Test);
    let client = reqwest::Client::new();

    const MAX_TRANSACTIONS: usize = 1_000_000;
    const PAGE_SIZE: usize = 200;
    const MAX_PAGES: usize = MAX_TRANSACTIONS / PAGE_SIZE; // 5000

    let mut seen_urls = std::collections::HashSet::<String>::new();
    let mut relays = Vec::new();
    let mut all_accounts = std::collections::HashSet::<String>::new();
    for a in known_accounts { all_accounts.insert(a.clone()); }

    log(&format!("[discover_relays] Starting discovery. Excluding {} connected URLs, {} known accounts",
        exclude.len(), known_accounts.len()));

    let mut phase1_total_found: usize = 0;

    // ── Phase 1: Query known accounts directly ──────────────────────
    log(&format!("[discover_relays] Phase 1: querying {} known accounts...", known_accounts.len()));
    for account in known_accounts {
        log(&format!("[discover_relays] Phase 1: fetching account {}", account));
        let url = format!("{}/accounts/{}", horizon, account);
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                log(&format!("[discover_relays] Phase 1: account {} returned status {}", account, r.status()));
                continue;
            }
            Err(e) => {
                log(&format!("[discover_relays] Phase 1: account {} fetch error: {}", account, e));
                continue;
            }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                log(&format!("[discover_relays] Phase 1: account {} JSON parse error: {}", account, e));
                continue;
            }
        };
        if let Some(data) = body.get("data").and_then(|d| d.as_object()) {
            log(&format!("[discover_relays] Phase 1: account {} has {} data entries", account, data.len()));
            for (key, val) in data {
                if !is_relay_key(key) {
                    continue;
                }
                if let Some(relay_url) = decode_data_entry(val) {
                    phase1_total_found += 1;
                    if exclude.contains(&relay_url) {
                        log(&format!("[discover_relays] Phase 1: {} (already connected)", relay_url));
                    } else if seen_urls.insert(relay_url.clone()) {
                        log(&format!("[discover_relays] Phase 1: found relay URL: {}", relay_url));
                        relays.push(DiscoveredRelay { url: relay_url, reachable: None });
                    }
                }
            }
        }
    }
    log(&format!("[discover_relays] Phase 1 complete: {} relays ({} new) from known accounts", phase1_total_found, relays.len()));
    let _phase1_relay_count = relays.len();
    if phase1_total_found > 0 {
        progress.set(format!("{} relay(s) from known accounts — scanning for more...", phase1_total_found));
    }

    // ── Phase 2: Scan transactions backwards, decode XDR ────────────
    log("[discover_relays] Phase 2: scanning transactions backwards (XDR decode)...");
    let mut txn_url = format!(
        "{}/transactions?order=desc&limit={}",
        horizon, PAGE_SIZE
    );

    let mut total_scanned: usize = 0;
    let mut new_relay_accounts: Vec<String> = Vec::new();

    for page in 0..MAX_PAGES {
        // Check stop signal
        if *stop.read() {
            log(&format!("[discover_relays] Phase 2: STOPPED by user at page {}, {} txns scanned", page, total_scanned));
            progress.set(format!("Stopped — {} relay(s) found, {} new account(s) from {} txns",
                phase1_total_found, new_relay_accounts.len(), total_scanned));
            break;
        }

        let resp = match client.get(&txn_url).send().await {
            Ok(r) => r,
            Err(e) => {
                log(&format!("[discover_relays] Phase 2: Horizon error on page {}: {}", page, e));
                break;
            }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                log(&format!("[discover_relays] Phase 2: JSON parse error on page {}: {}", page, e));
                break;
            }
        };
        let records = match body.pointer("/_embedded/records").and_then(|r| r.as_array()) {
            Some(r) => r.clone(),
            None => {
                log(&format!("[discover_relays] Phase 2: no records on page {}", page));
                break;
            }
        };
        let count = records.len();
        total_scanned += count;

        progress.set(format!(
            "{} relay(s) found — scanning... {} txns, {} new account(s)",
            phase1_total_found + new_relay_accounts.len(), total_scanned, new_relay_accounts.len()
        ));

        for record in &records {
            let envelope_b64 = match record.get("envelope_xdr").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let source_account_str = match record.get("source_account").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };

            // Decode XDR
            let envelope = match TransactionEnvelope::from_xdr_base64(envelope_b64, Limits::none()) {
                Ok(env) => env,
                Err(_) => continue,
            };

            // Extract operations from the envelope
            let ops = match &envelope {
                TransactionEnvelope::TxV0(e) => e.tx.operations.as_slice(),
                TransactionEnvelope::Tx(e) => e.tx.operations.as_slice(),
                _ => continue,
            };

            let mut found_manage_data = false;
            for op in ops {
                if let OperationBody::ManageData(md) = &op.body {
                    let name = md.data_name.to_string();
                    if is_relay_key(&name) {
                        found_manage_data = true;
                        break;
                    }
                }
            }

            if found_manage_data && all_accounts.insert(source_account_str.to_string()) {
                log(&format!("[discover_relays] Phase 2: discovered NEW relay account: {}", source_account_str));
                new_relay_accounts.push(source_account_str.to_string());

                progress.set(format!(
                    "{} relay(s) found — scanning... {} txns, {} new account(s)",
                    phase1_total_found + new_relay_accounts.len(), total_scanned, new_relay_accounts.len()
                ));
            }
        }

        if count < PAGE_SIZE {
            log(&format!("[discover_relays] Phase 2: last page (only {} records)", count));
            break;
        }

        // Auto-stop if we found enough accounts (50)
        if new_relay_accounts.len() >= 50 {
            log(&format!("[discover_relays] Phase 2: reached 50 accounts, stopping"));
            progress.set(format!(
                "{} relay(s) found — {} new account(s), limit reached",
                phase1_total_found + new_relay_accounts.len(), new_relay_accounts.len()
            ));
            break;
        }

        // Next page
        match body.pointer("/_links/next/href").and_then(|v| v.as_str()) {
            Some(next) => txn_url = next.to_string(),
            None => break,
        }

        if page % 50 == 49 {
            log(&format!("[discover_relays] Phase 2: page {}, {} txns scanned, {} accounts found",
                page + 1, total_scanned, new_relay_accounts.len()));
        }
    }

    log(&format!("[discover_relays] Phase 2 complete: scanned {} txns, found {} new accounts",
        total_scanned, new_relay_accounts.len()));

    // ── Phase 3: Fetch data entries from newly discovered accounts ───
    // Always run Phase 3, even if the user pressed STOP during scanning —
    // the discovered accounts are valuable and must be queried.
    for (idx, account) in new_relay_accounts.iter().enumerate() {
        progress.set(format!(
            "Fetching relay data from account {}/{}...",
            idx + 1, new_relay_accounts.len()
        ));
        let url = format!("{}/accounts/{}", horizon, account);
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(data) = body.get("data").and_then(|d| d.as_object()) {
            for (key, val) in data {
                if !is_relay_key(key) {
                    continue;
                }
                if let Some(relay_url) = decode_data_entry(val) {
                    log(&format!("[discover_relays] Phase 3: found relay URL: {} from account {}", relay_url, account));
                    if !exclude.contains(&relay_url) && seen_urls.insert(relay_url.clone()) {
                        relays.push(DiscoveredRelay { url: relay_url, reachable: None });
                    }
                }
            }
        }
    }

    log(&format!("[discover_relays] Total unique relays discovered: {}", relays.len()));

    // If more than 20, pick random 20
    if relays.len() > 20 {
        log(&format!("[discover_relays] Truncating from {} to 20 random relays", relays.len()));
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        relays.shuffle(&mut rng);
        relays.truncate(20);
    }

    let new_accounts: Vec<String> = all_accounts.into_iter().collect();
    log(&format!("[discover_relays] Returning {} relays, {} known accounts", relays.len(), new_accounts.len()));
    (relays, new_accounts)
}

/// Decode a base64-encoded data entry value into a UTF-8 string.
pub fn decode_data_entry(val: &serde_json::Value) -> Option<String> {
    let b64 = val.as_str()?;
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD, b64
    ).ok()?;
    String::from_utf8(bytes).ok()
}

pub fn generate_keypair(net_env: NetworkEnvironment, lang: Language) -> (String, String) {
    let lgr = StellarLedger::new(net_env, lang);
    let kp = lgr.generate_keypair();
    (kp.public_key, kp.secret_key)
}

pub fn import_keypair(raw_input: String, net_env: NetworkEnvironment, lang: Language) -> Option<(String, String)> {
    let lgr = StellarLedger::new(net_env, lang);
    lgr.public_key_from_secret(&raw_input)
        .map(|pub_key_str| (pub_key_str, raw_input))
}

pub fn new_store(lang: Language) -> IndexedDbStore {
    IndexedDbStore::new("gun-connect", "default_account", lang)
}

pub fn new_store_for_network(lang: Language, net: NetworkEnvironment) -> IndexedDbStore {
    let account = match net {
        NetworkEnvironment::Production => "mainnet_account",
        NetworkEnvironment::Test => "testnet_account",
    };
    IndexedDbStore::new("gun-connect", account, lang)
}
