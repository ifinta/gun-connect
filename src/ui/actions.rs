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
use serde::Deserialize;

fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }

/// The well-known manageData key used to publish GUN relay URLs on Stellar.
pub const MANAGE_DATA_KEY: &str = "gun_connect_relay";

pub async fn activate_test_account(pubkey: Option<String>, net_env: NetworkEnvironment, lang: Language) -> Result<String, String> {
    let pubkey = pubkey.ok_or_else(|| "No public key".to_string())?;
    let lgr = StellarLedger::new(net_env, lang);
    lgr.activate_test_account(&pubkey).await
}

/// Build, sign and submit a manageData transaction that publishes all connected relay URLs.
///
/// Each relay gets its own manageData key: `gun_connect_relay`, `gun_connect_relay_1`, etc.
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

    // Fetch sequence number
    let horizon = horizon_url(net_env);
    let client = reqwest::Client::new();
    let url = format!("{}/accounts/{}", horizon, public_key_str);
    let response = client.get(&url).send().await
        .map_err(|e| format!("Horizon unreachable: {}", e))?;
    if !response.status().is_success() {
        return Err("Account not found — activate with faucet first".into());
    }

    #[derive(Deserialize)]
    struct Acct { sequence: String }
    let acct: Acct = response.json().await
        .map_err(|e| format!("JSON error: {}", e))?;
    let next_seq: i64 = acct.sequence.parse::<i64>().unwrap_or(0) + 1;

    // Build manageData operations — one per relay URL
    let mut ops = Vec::new();
    for (i, relay_url) in relay_urls.iter().enumerate() {
        let key_name = if i == 0 {
            MANAGE_DATA_KEY.to_string()
        } else {
            format!("{}_{}", MANAGE_DATA_KEY, i)
        };
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

    // Clear old keys beyond current list (up to MAX_RELAYS)
    for i in relay_urls.len()..MAX_RELAYS {
        let key_name = if i == 0 {
            MANAGE_DATA_KEY.to_string()
        } else {
            format!("{}_{}", MANAGE_DATA_KEY, i)
        };
        ops.push(Operation {
            source_account: None,
            body: OperationBody::ManageData(ManageDataOp {
                data_name: String64(StringM::try_from(key_name.as_str())
                    .map_err(|e| format!("Key too long: {}", e))?),
                data_value: None, // None = delete the entry
            }),
        });
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
/// Two-phase strategy:
/// 1. Query known accounts directly (`/accounts/{addr}`) for current relay data entries.
///    This is reliable — data entries are account state, not lost in the operations stream.
/// 2. Scan recent global manage_data operations to discover NEW accounts.
///    Newly found accounts are added to the known list for future direct queries.
///
/// Results are deduplicated by URL, excluding already-connected relays.
pub async fn discover_relays(
    exclude: &std::collections::HashSet<String>,
    known_accounts: &[String],
) -> (Vec<DiscoveredRelay>, Vec<String>) {
    let horizon = horizon_url(NetworkEnvironment::Test);
    let client = reqwest::Client::new();
    let key_prefix = MANAGE_DATA_KEY;

    let mut seen_urls = std::collections::HashSet::<String>::new();
    let mut relays = Vec::new();
    let mut all_accounts = std::collections::HashSet::<String>::new();
    for a in known_accounts { all_accounts.insert(a.clone()); }

    // ── Phase 1: Query known accounts directly ──────────────────────
    log(&format!("[discover_relays] Querying {} known accounts...", known_accounts.len()));
    for account in known_accounts {
        let url = format!("{}/accounts/{}", horizon, account);
        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        // The `data` field is an object: { "gun_connect_relay": "base64...", ... }
        if let Some(data) = body.get("data").and_then(|d| d.as_object()) {
            for (key, val) in data {
                if key != key_prefix && !key.starts_with(&format!("{}_", key_prefix)) {
                    continue;
                }
                let b64 = match val.as_str() {
                    Some(v) => v,
                    None => continue,
                };
                let url_bytes = match base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD, b64
                ) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                let relay_url = match String::from_utf8(url_bytes) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if !exclude.contains(&relay_url) && seen_urls.insert(relay_url.clone()) {
                    relays.push(DiscoveredRelay {
                        url: relay_url,
                        reachable: None,
                    });
                }
            }
        }
    }
    log(&format!("[discover_relays] Phase 1: {} relays from known accounts", relays.len()));

    // ── Phase 2: Scan global operations to discover new accounts ────
    log("[discover_relays] Scanning global operations for new accounts...");
    let mut ops_url = format!(
        "{}/operations?type=manage_data&order=desc&limit=200",
        horizon
    );

    for page in 0..3 {
        log(&format!("[discover_relays] Fetching page {} ...", page));
        let resp = match client.get(&ops_url).send().await {
            Ok(r) => r,
            Err(e) => {
                log(&format!("[discover_relays] Horizon error: {}", e));
                break;
            }
        };
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                log(&format!("[discover_relays] JSON parse error: {}", e));
                break;
            }
        };
        let records = match body.pointer("/_embedded/records").and_then(|r| r.as_array()) {
            Some(r) => r.clone(),
            None => break,
        };
        let count = records.len();

        for record in &records {
            let name = record.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name != key_prefix && !name.starts_with(&format!("{}_", key_prefix)) {
                continue;
            }
            let account = match record.get("source_account").and_then(|v| v.as_str()) {
                Some(a) => a.to_string(),
                None => continue,
            };

            // If this is a new account we haven't queried yet, fetch its data
            if all_accounts.insert(account.clone()) {
                let acct_url = format!("{}/accounts/{}", horizon, account);
                let acct_resp = match client.get(&acct_url).send().await {
                    Ok(r) if r.status().is_success() => r,
                    _ => continue,
                };
                let acct_body: serde_json::Value = match acct_resp.json().await {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Some(data) = acct_body.get("data").and_then(|d| d.as_object()) {
                    for (key, val) in data {
                        if key != key_prefix && !key.starts_with(&format!("{}_", key_prefix)) {
                            continue;
                        }
                        let b64 = match val.as_str() {
                            Some(v) => v,
                            None => continue,
                        };
                        let url_bytes = match base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD, b64
                        ) {
                            Ok(b) => b,
                            Err(_) => continue,
                        };
                        let relay_url = match String::from_utf8(url_bytes) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if !exclude.contains(&relay_url) && seen_urls.insert(relay_url.clone()) {
                            relays.push(DiscoveredRelay {
                                url: relay_url,
                                reachable: None,
                            });
                        }
                    }
                }
            }
        }

        if count < 200 { break; }
        match body.pointer("/_links/next/href").and_then(|v| v.as_str()) {
            Some(next) => ops_url = next.to_string(),
            None => break,
        }
    }

    log(&format!("[discover_relays] Total unique relays: {}", relays.len()));

    // If more than 20, pick random 20
    if relays.len() > 20 {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        relays.shuffle(&mut rng);
        relays.truncate(20);
    }

    // Return discovered relays + the full set of known accounts (for persistence)
    let new_accounts: Vec<String> = all_accounts.into_iter().collect();
    (relays, new_accounts)
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
