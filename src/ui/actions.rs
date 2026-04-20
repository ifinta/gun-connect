use dioxus::prelude::*;
use zsozso_common::Language;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};

use zsozso_store::IndexedDbStore;

// Re-export relay types and functions from the library.
pub use zsozso_ledger::relay::{
    MAX_RELAYS, RelayEntry, DiscoveredRelay, publish_relays,
};

pub async fn activate_test_account(pubkey: Option<String>, net_env: NetworkEnvironment, lang: Language) -> Result<String, String> {
    let pubkey = pubkey.ok_or_else(|| "No public key".to_string())?;
    let lgr = StellarLedger::new(net_env, lang);
    lgr.activate_test_account(&pubkey).await
}

/// Thin wrapper around the library's `discover_relays` that adapts Dioxus Signals
/// to the library's callback interface.
pub async fn discover_relays(
    exclude: &std::collections::HashSet<String>,
    known_accounts: &[String],
    mut progress: Signal<String>,
    stop: Signal<bool>,
) -> (Vec<DiscoveredRelay>, Vec<String>) {
    zsozso_ledger::relay::discover_relays(
        exclude,
        known_accounts,
        &mut |msg| progress.set(msg.to_string()),
        &|| *stop.read(),
    ).await
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
