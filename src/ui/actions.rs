use zsozso_common::Language;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};

use super::status::TxStatus;

use zsozso_store::IndexedDbStore;

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
    net_env: NetworkEnvironment,
    lang: Language,
) -> Result<(String, TxStatus), TxStatus> {
    let secret_val = secret_key.ok_or(TxStatus::NoKey)?;
    let lgr = StellarLedger::new(net_env, lang);
    let net_info = lgr.network_info();

    match lgr.build_self_payment(&secret_val, 100_000_000).await {
        Ok((xdr, seq)) => {
            let status = TxStatus::XdrReady { net: net_info.name.to_string(), seq };
            Ok((xdr, status))
        }
        Err(e) => Err(TxStatus::Error(e)),
    }
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
    IndexedDbStore::new("zsozso", "default_account", lang)
}

pub fn new_store_for_network(lang: Language, net: NetworkEnvironment) -> IndexedDbStore {
    let account = match net {
        NetworkEnvironment::Production => "mainnet_account",
        NetworkEnvironment::Test => "testnet_account",
    };
    IndexedDbStore::new("zsozso", account, lang)
}
