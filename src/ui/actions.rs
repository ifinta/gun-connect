use zsozso_common::Language;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};

use super::status::TxStatus;

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
pub const MANAGE_DATA_KEY: &str = "gun_relay";

pub async fn submit_transaction(xdr_to_submit: String, net_env: NetworkEnvironment, lang: Language) -> TxStatus {
    if xdr_to_submit.is_empty() {
        return TxStatus::NoXdr;
    }

    let lgr = StellarLedger::new(net_env, lang);
    match lgr.submit_transaction(&xdr_to_submit).await {
        Ok(msg) => TxStatus::Success(msg),
        Err(e) => TxStatus::Error(e),
    }
}

pub async fn activate_test_account(pubkey: Option<String>, net_env: NetworkEnvironment, lang: Language) -> Option<TxStatus> {
    let pubkey = pubkey?;
    let lgr = StellarLedger::new(net_env, lang);

    Some(match lgr.activate_test_account(&pubkey).await {
        Ok(msg) => TxStatus::FaucetSuccess(msg),
        Err(e) => TxStatus::Error(e),
    })
}

pub async fn fetch_and_generate_xdr(
    secret_key: Option<String>,
    relay_url: String,
    net_env: NetworkEnvironment,
    _lang: Language,
) -> Result<(String, TxStatus), TxStatus> {
    let secret_val = secret_key.ok_or(TxStatus::NoKey)?;
    if relay_url.trim().is_empty() {
        return Err(TxStatus::Error("No relay URL configured".into()));
    }

    // Decode key
    let priv_key = match Strkey::from_string(&secret_val) {
        Ok(Strkey::PrivateKeyEd25519(pk)) => pk,
        _ => return Err(TxStatus::Error("Invalid secret key".into())),
    };
    let signing_key = SigningKey::from_bytes(&priv_key.0);
    let pub_bytes = signing_key.verifying_key().to_bytes();
    let public_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();

    // Fetch sequence number
    let horizon = horizon_url(net_env);
    let url = format!("{}/accounts/{}", horizon, public_key_str);
    let client = reqwest::Client::new();

    let response = client.get(&url).send().await
        .map_err(|e| TxStatus::Error(format!("Horizon unreachable: {}", e)))?;
    if !response.status().is_success() {
        return Err(TxStatus::Error("Account not found — activate with faucet first".into()));
    }

    #[derive(Deserialize)]
    struct Acct { sequence: String }
    let acct: Acct = response.json().await
        .map_err(|e| TxStatus::Error(format!("JSON error: {}", e)))?;
    let next_seq: i64 = acct.sequence.parse::<i64>().unwrap_or(0) + 1;

    // Build manageData operations — split relay URL across 64-byte chunks
    let url_bytes = relay_url.as_bytes();
    let chunks: Vec<&[u8]> = url_bytes.chunks(64).collect();
    let mut ops = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let key_name = if i == 0 {
            MANAGE_DATA_KEY.to_string()
        } else {
            format!("{}_{}", MANAGE_DATA_KEY, i)
        };
        ops.push(Operation {
            source_account: None,
            body: OperationBody::ManageData(ManageDataOp {
                data_name: String64(StringM::try_from(key_name.as_str())
                    .map_err(|e| TxStatus::Error(format!("Key too long: {}", e)))?),
                data_value: Some(DataValue(BytesM::try_from(chunk.to_vec())
                    .map_err(|e| TxStatus::Error(format!("Value too long: {}", e)))?)),
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
            .map_err(|e| TxStatus::Error(format!("Operations error: {}", e)))?,
        ext: TransactionExt::V0,
    };

    // Sign
    let network_id = Hash(Sha256::digest(passphrase.as_bytes()).into());
    let payload = TransactionSignaturePayload {
        network_id,
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
    };
    let tx_payload_xdr = payload.to_xdr(Limits::none())
        .map_err(|e| TxStatus::Error(format!("XDR error: {}", e)))?;
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
        .map_err(|e| TxStatus::Error(format!("XDR encode error: {}", e)))?;

    let net_name = if net_env == NetworkEnvironment::Test { "TESTNET" } else { "MAINNET" };
    Ok((xdr, TxStatus::XdrReady { net: net_name.into(), seq: next_seq }))
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

/// A relay discovered from Stellar manageData entries.
#[derive(Clone, Debug)]
pub struct DiscoveredRelay {
    pub account: String,
    pub url: String,
    pub reachable: Option<bool>,
}

/// Discover GUN relay URLs published on the Stellar testnet.
///
/// Strategy: query recent manageData operations from Horizon, filter for
/// entries with key `gun_relay`, deduplicate by account (latest wins).
pub async fn discover_relays() -> Vec<DiscoveredRelay> {
    log("[discover_relays] Querying Horizon for recent manage_data ops...");
    let horizon = horizon_url(NetworkEnvironment::Test);
    let url = format!(
        "{}/operations?type=manage_data&order=desc&limit=200",
        horizon
    );

    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log(&format!("[discover_relays] Horizon error: {}", e));
            return vec![];
        }
    };

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            log(&format!("[discover_relays] JSON parse error: {}", e));
            return vec![];
        }
    };

    let mut seen = std::collections::HashMap::<String, String>::new();

    if let Some(records) = body.pointer("/_embedded/records").and_then(|r| r.as_array()) {
        for record in records {
            let name = record.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name != MANAGE_DATA_KEY {
                continue;
            }
            let account = match record.get("source_account").and_then(|v| v.as_str()) {
                Some(a) => a.to_string(),
                None => continue,
            };
            let value_b64 = match record.get("value").and_then(|v| v.as_str()) {
                Some(v) => v,
                None => continue,
            };
            // Horizon returns the value as base64
            let url_bytes = match base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD, value_b64
            ) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let relay_url = match String::from_utf8(url_bytes) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Keep latest per account (results are ordered desc by time)
            seen.entry(account).or_insert(relay_url);
        }
    }

    log(&format!("[discover_relays] Found {} unique relays", seen.len()));
    seen.into_iter()
        .map(|(account, url)| DiscoveredRelay { account, url, reachable: None })
        .collect()
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
