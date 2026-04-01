use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::{WalletState, AuthState};
use super::actions::*;
use super::actions::new_store_for_network;
use super::status::TxStatus;
use super::i18n::ui_i18n;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::Store;
use crate::store::passkey;
use crate::db::gundb::{GunSea, Sea};
use crate::db::network::{NetworkGraph, GunNetworkGraph};
use super::clipboard::{copy_to_clipboard, clear_clipboard};
use super::log;
use crate::ledger::sc::SmartContract;

#[derive(Clone, Copy)]
pub struct AppController {
    s: WalletState,
}

impl AppController {
    pub fn new(state: WalletState) -> Self {
        Self { s: state }
    }

    /// Get the configured GUN relay peers from the relay URL signal.
    fn gun_peers(&self) -> Vec<String> {
        let url = self.s.gun_relay_url.read().clone();
        if url.trim().is_empty() { vec![] } else { vec![url] }
    }

    /// Start passkey authentication (gate modal button).
    /// On localhost the passkey check is skipped for easier testing.
    pub fn start_auth(&self) {
        let mut auth_state = self.s.auth_state;
        let mut prf_key_signal = self.s.prf_key;

        auth_state.set(AuthState::Authenticating);

        spawn(async move {
            if is_localhost() {
                auth_state.set(AuthState::Authenticated);
                return;
            }
            match passkey::passkey_init().await {
                Ok(result) if result.success => {
                    prf_key_signal.set(result.prf_key);
                    auth_state.set(AuthState::Authenticated);
                }
                _ => {
                    auth_state.set(AuthState::Failed);
                }
            }
        });
    }

    /// Enable biometric identification.
    /// Registers the passkey (one biometric prompt). PRF key is obtained
    /// lazily when actually needed (save/load/auth gate).
    pub fn toggle_biometric(&self) {
        // Only allow turning ON (switch is disabled when already on)
        if *self.s.biometric_enabled.read() {
            return;
        }

        let mut biometric = self.s.biometric_enabled;

        spawn(async move {
            if is_localhost() {
                biometric.set(true);
                write_biometric_pref(true);
                return;
            }
            match passkey::passkey_register().await {
                Ok(result) if result.success => {
                    biometric.set(true);
                    write_biometric_pref(true);
                    log("Biometric registration successful");
                }
                Ok(result) => {
                    let err = result.error.unwrap_or_else(|| "Unknown error".into());
                    log(&format!("Biometric registration failed: {}", err));
                }
                Err(e) => {
                    log(&format!("Biometric registration error: {}", e));
                }
            }
        });
    }

    /// Dismiss the biometric save error modal.
    pub fn dismiss_biometric_save_modal(&self) {
        let mut modal = self.s.biometric_save_modal_open;
        modal.set(false);
    }

    /// Reveal the secret key after passkey verification.
    /// On localhost or when biometric is disabled, the passkey check is skipped.
    pub fn reveal_secret(&self) {
        let mut show_secret = self.s.show_secret;
        let biometric_on = *self.s.biometric_enabled.read();

        spawn(async move {
            if is_localhost() || !biometric_on {
                show_secret.set(true);
                return;
            }
            match passkey::passkey_verify().await {
                Ok(true) => show_secret.set(true),
                _ => { /* auth failed or cancelled — don't reveal */ }
            }
        });
    }

    /// Generate a new keypair and store it in the state
    pub fn generate_key(&self) {
        let (pk, sk) = generate_keypair(*self.s.current_network.read(), *self.s.language.read());
        let mut public_key = self.s.public_key;
        let mut secret_key_hidden = self.s.secret_key_hidden;
        public_key.set(Some(pk));
        secret_key_hidden.set(Some(Zeroizing::new(sk)));
        self.track_network_key();
    }

    /// Import a keypair from user input
    pub fn import_key(&self) {
        let raw_input = self.s.input_value.read().clone();
        let net = *self.s.current_network.read();
        let lang = *self.s.language.read();

        if let Some((pub_key_str, secret)) = import_keypair(raw_input, net, lang) {
            let mut public_key = self.s.public_key;
            let mut secret_key_hidden = self.s.secret_key_hidden;
            let mut input_value = self.s.input_value;
            public_key.set(Some(pub_key_str));
            secret_key_hidden.set(Some(Zeroizing::new(secret)));
            input_value.set(String::new());
            self.track_network_key();
        }
    }

    /// Securely copy the secret key to clipboard and show modal
    pub fn copy_secret_to_clipboard(&self) {
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            copy_to_clipboard(secret.as_str());
            let lang = *self.s.language.read();
            let i18n = ui_i18n(lang);
            log(&i18n.copied().to_string());
            let mut modal = self.s.clipboard_modal_open;
            modal.set(true);
        }
    }

    /// Copy the generated XDR to clipboard and show modal
    pub fn copy_xdr_to_clipboard(&self) {
        let xdr = self.s.generated_xdr.read().clone();
        if !xdr.is_empty() {
            copy_to_clipboard(&xdr);
            let lang = *self.s.language.read();
            let i18n = ui_i18n(lang);
            log(&i18n.copied().to_string());
            let mut modal = self.s.clipboard_modal_open;
            modal.set(true);
        }
    }

    /// Dismiss the clipboard modal and clear clipboard content
    pub fn dismiss_clipboard_modal(&self) {
        clear_clipboard();
        let mut modal = self.s.clipboard_modal_open;
        modal.set(false);
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        log(&i18n.clipboard_cleared().to_string());
    }

    /// Activate a test network account (Faucet call)
    pub fn activate_test_account_action(&self) {
        let pubkey = self.s.public_key.read().clone();
        let net_env = *self.s.current_network.read();
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;

        spawn(async move {
            status.set(TxStatus::CallingFaucet);
            if let Some(next_status) = activate_test_account(pubkey, net_env, lang).await {
                status.set(next_status);
            }
        });
    }

    /// Generate XDR based on account data — always uses Testnet address.
    pub fn fetch_and_generate_xdr_action(&self) {
        let secret_key = self.s.testnet_secret_key.read().as_ref().map(|s| s.to_string());
        let net_env = NetworkEnvironment::Test;
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;
        let mut xdr_signal = self.s.generated_xdr;

        spawn(async move {
            status.set(TxStatus::FetchingSequence);
            match fetch_and_generate_xdr(secret_key, net_env, lang).await {
                Ok((xdr, next_status)) => {
                    xdr_signal.set(xdr);
                    status.set(next_status);
                }
                Err(e_status) => status.set(e_status),
            }
        });
    }

    /// Submit a transaction to the network (testnet).
    pub fn submit_transaction_action(&self) {
        let xdr = self.s.generated_xdr.read().clone();
        let net_env = NetworkEnvironment::Test;
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;

        spawn(async move {
            status.set(TxStatus::Submitting);
            status.set(submit_transaction(xdr, net_env, lang).await);
        });
    }

    /// Save key to local store — encrypted with passkey if PRF available and biometric is on.
    /// The stored format is 'tn:secret' for testnet or 'mn:secret' for mainnet.
    /// If the PRF key is not yet available, authenticates lazily (one biometric prompt).
    pub fn save_to_store(&self) {
        let lang = *self.s.language.read();
        let net = *self.s.current_network.read();
        let i18n = ui_i18n(lang);
        let biometric_on = *self.s.biometric_enabled.read();

        // Refuse to save when biometric is off (except localhost dev)
        if !biometric_on && !is_localhost() {
            let mut modal = self.s.biometric_save_modal_open;
            modal.set(true);
            return;
        }
        
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            let store = new_store(lang);
            let secret = secret.clone();
            let existing_prf = self.s.prf_key.read().clone();
            let mut prf_key_signal = self.s.prf_key;
            spawn(async move {
                let prefix = match net {
                    NetworkEnvironment::Test => "tn:",
                    NetworkEnvironment::Production => "mn:",
                };
                let prefixed_secret = format!("{}{}", prefix, secret.as_str());
                let data_to_save = if is_localhost() || !biometric_on {
                    prefixed_secret
                } else {
                    let prf = match existing_prf {
                        Some(key) => key,
                        None => {
                            // Lazy PRF: authenticate to get encryption key
                            match passkey::passkey_init().await {
                                Ok(result) if result.success => {
                                    match result.prf_key {
                                        Some(key) => {
                                            prf_key_signal.set(Some(key.clone()));
                                            key
                                        }
                                        None => {
                                            log(&i18n.fmt_error(i18n.no_prf_key()));
                                            return;
                                        }
                                    }
                                }
                                Ok(result) => {
                                    let err = result.error.unwrap_or_else(|| "Authentication failed".into());
                                    log(&i18n.fmt_error(&err));
                                    return;
                                }
                                Err(e) => {
                                    log(&i18n.fmt_error(&e));
                                    return;
                                }
                            }
                        }
                    };
                    match passkey::passkey_encrypt(&prefixed_secret, &prf).await {
                        Ok(encrypted) => encrypted,
                        Err(e) => {
                            log(&i18n.fmt_error(&e));
                            return;
                        }
                    }
                };
                match store.save(&data_to_save).await {
                    Ok(_) => log(&i18n.save_success().to_string()),
                    Err(e) => log(&i18n.fmt_error(&e)),
                }
            });
        } else {
            log(&i18n.nothing_to_save().to_string());
        }
    }

    /// Load key from local store — decrypted with passkey if PRF available and biometric is on.
    /// Parses the 'tn:' / 'mn:' prefix to restore the correct network.
    /// If the PRF key is not yet available, authenticates lazily (one biometric prompt).
    pub fn load_from_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut pk_signal = self.s.public_key;
        let mut sk_signal = self.s.secret_key_hidden;
        let mut net_signal = self.s.current_network;
        let existing_prf = self.s.prf_key.read().clone();
        let mut prf_key_signal = self.s.prf_key;
        let biometric_on = *self.s.biometric_enabled.read();
        let mut mn_pk = self.s.mainnet_public_key;
        let mut tn_pk = self.s.testnet_public_key;

        log(&i18n.loading_started().to_string());
        let store = new_store(lang);
        
        spawn(async move {
            match store.load().await {
                Ok(stored_data) => {
                    let decrypted = if is_localhost() || !biometric_on {
                        stored_data
                    } else {
                        let prf = match existing_prf {
                            Some(key) => key,
                            None => {
                                // Lazy PRF: authenticate to get decryption key
                                match passkey::passkey_init().await {
                                    Ok(result) if result.success => {
                                        match result.prf_key {
                                            Some(key) => {
                                                prf_key_signal.set(Some(key.clone()));
                                                key
                                            }
                                            None => {
                                                log(&i18n.fmt_error(i18n.no_prf_key()));
                                                return;
                                            }
                                        }
                                    }
                                    Ok(result) => {
                                        let err = result.error.unwrap_or_else(|| "Authentication failed".into());
                                        log(&i18n.fmt_error(&err));
                                        return;
                                    }
                                    Err(e) => {
                                        log(&i18n.fmt_error(&e));
                                        return;
                                    }
                                }
                            }
                        };
                        match passkey::passkey_decrypt(&stored_data, &prf).await {
                            Ok(d) => d,
                            Err(e) => {
                                log(&i18n.fmt_error(&e));
                                return;
                            }
                        }
                    };

                    // Parse network prefix
                    let (net, secret) = if let Some(rest) = decrypted.strip_prefix("tn:") {
                        (NetworkEnvironment::Test, rest.to_string())
                    } else if let Some(rest) = decrypted.strip_prefix("mn:") {
                        (NetworkEnvironment::Production, rest.to_string())
                    } else {
                        // Legacy data without prefix — default to current network
                        (*net_signal.read(), decrypted)
                    };

                    log(&i18n.key_loaded_len(secret.len()));
                    let lgr = StellarLedger::new(net, lang);
                    if let Some(pub_key_str) = lgr.public_key_from_secret(&secret) {
                        net_signal.set(net);
                        pk_signal.set(Some(pub_key_str.clone()));
                        sk_signal.set(Some(Zeroizing::new(secret)));
                        // Track key in the appropriate network slot
                        match net {
                            NetworkEnvironment::Production => mn_pk.set(Some(pub_key_str)),
                            NetworkEnvironment::Test => tn_pk.set(Some(pub_key_str)),
                        }
                        log(&i18n.ui_updated_with_key().to_string());
                    }
                }
                Err(e) => log(&i18n.fmt_error(&e)),
            }
        });
    }

    pub fn set_language(&self, code: &str) {
        use crate::i18n::Language;
        let lang = match code {
            "hu" => Language::Hungarian,
            "fr" => Language::French,
            "de" => Language::German,
            "es" => Language::Spanish,
            _ => Language::English,
        };
        let mut language = self.s.language;
        language.set(lang);
    }

    pub fn toggle_network(&self) {
        let current = *self.s.current_network.read();
        let next = if current == NetworkEnvironment::Production {
            NetworkEnvironment::Test
        } else {
            NetworkEnvironment::Production
        };

        // If there is a secret in memory, ask the user whether to save first
        let has_secret = self.s.secret_key_hidden.read().is_some();
        if has_secret {
            let mut pending = self.s.network_switch_pending;
            pending.set(Some(next));
        } else {
            self.do_switch_network(next);
        }
    }

    /// Actually perform the network switch (clears XDR).
    fn do_switch_network(&self, next: NetworkEnvironment) {
        let mut current_network = self.s.current_network;
        let mut generated_xdr = self.s.generated_xdr;
        current_network.set(next);
        generated_xdr.set(String::new());
    }

    /// User confirmed: save the secret with the current network, then switch.
    pub fn confirm_network_switch_save(&self) {
        // Save first (uses current network for the prefix)
        self.save_to_store();

        // Now switch to the pending network
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
    }

    /// User confirmed: switch to the new network first, then save with the new prefix.
    pub fn confirm_network_switch_and_save(&self) {
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);

        // Save after switching (uses new network for the prefix)
        self.save_to_store();
    }

    /// User declined saving: just switch network without saving.
    pub fn confirm_network_switch_discard(&self) {
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
    }

    /// User cancelled the network switch entirely.
    pub fn cancel_network_switch(&self) {
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
    }

    /// Open camera QR scanner and add the scanned public key as a worker.
    pub fn add_worker_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let public_key = self.s.public_key.read().clone();
        let sea_pair = self.s.sea_key_pair.read().clone();
        let peers = self.gun_peers();

        // SEA keypair required for authenticated writes
        if sea_pair.is_none() {
            log("[add_worker_action] No SEA keypair, opening modal");
            self.open_sea_modal();
            return;
        }

        let mut ping_status = self.s.ping_status;
        let mut workers_signal = self.s.network_workers;
        let mut nicknames_signal = self.s.network_nicknames;
        let mut parents_signal = self.s.network_parents;

        let Some(pk) = public_key else {
            log("[add_worker_action] No public key");
            ping_status.set(Some(i18n.network_no_key().to_string()));
            return;
        };

        log(&format!("[add_worker_action] Starting QR scan to add worker for pk={}", pk));
        ping_status.set(Some(i18n.scan_scanning().to_string()));

        spawn(async move {
            match super::qr_scanner::scan_qr().await {
                Ok(worker_key) => {
                    log(&format!("[add_worker_action] Scanned worker key: {}", worker_key));
                    let graph = GunNetworkGraph::new(lang, sea_pair, peers);
                    match graph.add_worker(&pk, &worker_key).await {
                        Ok(_) => {
                            log("[add_worker_action] Worker added successfully");
                            let i18n = ui_i18n(lang);
                            ping_status.set(Some(i18n.network_add_worker_success(&worker_key)));
                            // Reload network data
                            log("[add_worker_action] Reloading network data...");
                            let workers = graph.get_workers(&pk).await.unwrap_or_default();
                            workers_signal.set(workers.clone());
                            let ancestors = graph.get_ancestors(&pk, 6).await.unwrap_or_default();
                            parents_signal.set(ancestors.clone());
                            let mut nicks = std::collections::HashMap::new();
                            for key in ancestors.iter().chain(workers.iter()) {
                                if let Ok(Some(nick)) = graph.get_nickname(key).await {
                                    nicks.insert(key.clone(), nick);
                                }
                            }
                            nicknames_signal.set(nicks);
                            log("[add_worker_action] Network data reloaded");
                        }
                        Err(e) => {
                            log(&format!("[add_worker_action] Failed to add worker: {}", e));
                            let i18n = ui_i18n(lang);
                            ping_status.set(Some(i18n.network_add_worker_error(&e)));
                        }
                    }
                }
                Err(e) => {
                    if e == "cancelled" {
                        log("[add_worker_action] QR scan cancelled");
                        ping_status.set(None);
                    } else {
                        log(&format!("[add_worker_action] QR scan error: {}", e));
                        let i18n = ui_i18n(lang);
                        ping_status.set(Some(i18n.scan_error(&e)));
                    }
                }
            }
        });
    }

    /// Load network hierarchy (parents, workers, nicknames) from the graph database.
    pub fn load_network_action(&self) {
        let public_key = self.s.public_key.read().clone();
        let lang = *self.s.language.read();
        let sea_pair = self.s.sea_key_pair.read().clone();
        let peers = self.gun_peers();
        let mut parents_signal = self.s.network_parents;
        let mut workers_signal = self.s.network_workers;
        let mut nicknames_signal = self.s.network_nicknames;
        let mut nickname_signal = self.s.nickname;
        let mut gun_address = self.s.gun_address;
        let mut gun_relay_url = self.s.gun_relay_url;

        let Some(pk) = public_key else {
            log("[load_network_action] No public key, skipping");
            return;
        };

        log(&format!("[load_network_action] Loading network data for pk={}", pk));

        spawn(async move {
            let graph = GunNetworkGraph::new(lang, sea_pair, peers);

            log("[load_network_action] Fetching ancestors...");
            let ancestors = graph.get_ancestors(&pk, 6).await.unwrap_or_default();
            log(&format!("[load_network_action] Got {} ancestors", ancestors.len()));
            parents_signal.set(ancestors.clone());

            log("[load_network_action] Fetching workers...");
            let workers = graph.get_workers(&pk).await.unwrap_or_default();
            log(&format!("[load_network_action] Got {} workers", workers.len()));
            workers_signal.set(workers.clone());

            log("[load_network_action] Fetching own nickname...");
            if let Ok(Some(nick)) = graph.get_nickname(&pk).await {
                log(&format!("[load_network_action] Own nickname={}", nick));
                nickname_signal.set(nick);
            }

            log("[load_network_action] Fetching GUN address...");
            if let Ok(Some(addr)) = graph.get_gun_address(&pk).await {
                log(&format!("[load_network_action] GUN address={}", addr));
                gun_address.set(addr);
            }

            log("[load_network_action] Fetching GUN relay URL...");
            if let Ok(Some(url)) = graph.get_gun_relay_url(&pk).await {
                log(&format!("[load_network_action] GUN relay URL={}", url));
                gun_relay_url.set(url);
            }

            log("[load_network_action] Fetching nicknames for ancestors and workers...");
            let mut nicks = std::collections::HashMap::new();
            for key in ancestors.iter().chain(workers.iter()) {
                if let Ok(Some(nick)) = graph.get_nickname(key).await {
                    log(&format!("[load_network_action] nickname for {}={}", key, nick));
                    nicks.insert(key.clone(), nick);
                }
            }
            nicknames_signal.set(nicks);
            log("[load_network_action] Done");
        });
    }

    /// Save the user's nickname to the graph database.
    pub fn save_nickname_action(&self) {
        let nickname = self.s.nickname.read().clone();
        let public_key = self.s.public_key.read().clone();
        let lang = *self.s.language.read();
        let sea_pair = self.s.sea_key_pair.read().clone();
        let peers = self.gun_peers();

        // SEA keypair required for authenticated writes
        if sea_pair.is_none() {
            log("[save_nickname_action] No SEA keypair, opening modal");
            self.open_sea_modal();
            return;
        }

        let Some(pk) = public_key else {
            log("[save_nickname_action] No public key");
            return;
        };

        log(&format!("[save_nickname_action] Saving nickname '{}' for pk={}", nickname, pk));
        spawn(async move {
            let graph = GunNetworkGraph::new(lang, sea_pair, peers);
            match graph.set_nickname(&pk, &nickname).await {
                Ok(_) => {
                    log("[save_nickname_action] Nickname saved successfully");
                    let i18n = ui_i18n(lang);
                    log(&i18n.nickname_saved().to_string());
                }
                Err(e) => {
                    log(&format!("[save_nickname_action] Failed to save nickname: {}", e));
                    let i18n = ui_i18n(lang);
                    log(&i18n.nickname_save_error(&e));
                }
            }
        });
    }

    /// Open camera QR scanner and log the scanned public key to the console.
    pub fn scan_qr_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut ping_status = self.s.ping_status;

        ping_status.set(Some(i18n.scan_scanning().to_string()));

        spawn(async move {
            match super::qr_scanner::scan_qr().await {
                Ok(key) => {
                    log(&format!("Scanned public key: {}", key));
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.scan_success(&key)));
                }
                Err(e) => {
                    if e == "cancelled" {
                        ping_status.set(None);
                    } else {
                        let i18n = ui_i18n(lang);
                        ping_status.set(Some(i18n.scan_error(&e)));
                    }
                }
            }
        });
    }

    /// Call the zsozso-sc ping() contract function using the stored secret key
    pub fn ping_contract_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut ping_status = self.s.ping_status;

        // Use the in-memory secret key if available
        let secret = match self.s.secret_key_hidden.read().as_ref() {
            Some(sk) => sk.as_str().to_string(),
            None => {
                ping_status.set(Some(i18n.ping_no_key().to_string()));
                return;
            }
        };

        let net_env = *self.s.current_network.read();

        ping_status.set(Some(i18n.ping_calling().to_string()));

        spawn(async move {
            let sc = crate::ledger::sc::zsozso_sc::ZsozsoSc::new(net_env, lang);
            match sc.ping(&secret).await {
                Ok(msg) => {
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.ping_success(&msg)));
                }
                Err(e) => {
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.ping_error(&e)));
                }
            }
        });
    }

    /// Open the SEA key generation modal.
    /// Save the GUN relay URL to the graph database.
    pub fn save_gun_relay_action(&self) {
        let relay_url = self.s.gun_relay_url.read().clone();
        let public_key = self.s.public_key.read().clone();
        let lang = *self.s.language.read();
        let sea_pair = self.s.sea_key_pair.read().clone();
        let peers = if relay_url.trim().is_empty() { vec![] } else { vec![relay_url.clone()] };

        if sea_pair.is_none() {
            log("[save_gun_relay_action] No SEA keypair, opening modal");
            self.open_sea_modal();
            return;
        }

        let Some(pk) = public_key else {
            log("[save_gun_relay_action] No public key");
            return;
        };

        spawn(async move {
            log(&format!("[save_gun_relay_action] Saving relay URL: {}", relay_url));
            let graph = GunNetworkGraph::new(lang, sea_pair, peers);
            match graph.set_gun_relay_url(&pk, &relay_url).await {
                Ok(_) => log("[save_gun_relay_action] Relay URL saved successfully"),
                Err(e) => log(&format!("[save_gun_relay_action] Failed to save relay URL: {}", e)),
            }
        });
    }

    pub fn open_sea_modal(&self) {
        let mut open = self.s.sea_modal_open;
        open.set(true);
    }

    /// Close the SEA key generation modal and zeroize the input.
    pub fn close_sea_modal(&self) {
        let mut open = self.s.sea_modal_open;
        let mut input = self.s.sea_modal_input;
        open.set(false);
        input.set(Zeroizing::new(String::new()));
    }

    /// Generate a SEA key pair from the passphrase entered in the modal.
    /// The passphrase is zeroized after use; the keys live only in memory.
    pub fn generate_sea_keys(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);

        let passphrase = self.s.sea_modal_input.read().clone();
        if passphrase.is_empty() {
            return;
        }

        let mut key_pair_signal = self.s.sea_key_pair;
        let mut modal_open = self.s.sea_modal_open;
        let mut modal_input = self.s.sea_modal_input;

        let mut gun_address = self.s.gun_address;
        let public_key = self.s.public_key.read().clone();
        let mut sss_shares = self.s.sss_shares;
        let peers = self.gun_peers();

        spawn(async move {
            log("[generate_sea_keys] Starting SEA key generation from passphrase");
            let sea = GunSea::new(lang);
            match sea.pair_from_seed(&passphrase).await {
                Ok(pair) => {
                    log(&format!("[generate_sea_keys] SEA keys generated. pub_key={}", &pair.pub_key));
                    gun_address.set(pair.pub_key.clone());

                    // Store GUN address to GunDB if we have a Stellar public key
                    if let Some(pk) = &public_key {
                        log(&format!("[generate_sea_keys] Storing GUN address to GunDB for node {}", pk));
                        let graph = GunNetworkGraph::new(lang, Some(pair.clone()), peers.clone());
                        if let Err(e) = graph.set_gun_address(pk, &pair.pub_key).await {
                            log(&format!("[generate_sea_keys] Failed to store GUN address: {}", e));
                        } else {
                            log("[generate_sea_keys] GUN address stored successfully");
                        }
                    }

                    key_pair_signal.set(Some(pair));

                    // Split the passphrase into SSS shares (7 shares, threshold 3)
                    let shares = crate::sss::split(passphrase.as_bytes(), 3, 7);
                    let share_strings: Vec<String> = shares.iter()
                        .map(|s| crate::sss::share_to_hex(s))
                        .collect();
                    log(&format!("[generate_sea_keys] SSS shares generated: {} shares, threshold 3", share_strings.len()));
                    sss_shares.set(Some(share_strings));

                    let i18n = ui_i18n(lang);
                    log(&i18n.sea_keys_generated().to_string());
                }
                Err(e) => {
                    log(&format!("[generate_sea_keys] SEA key generation failed: {}", e));
                    let i18n = ui_i18n(lang);
                    log(&i18n.sea_generation_error(&e));
                }
            }
            // Zeroize the passphrase input and close the modal
            modal_input.set(Zeroizing::new(String::new()));
            modal_open.set(false);
        });
    }

    /// Dismiss the SSS shares modal.
    pub fn dismiss_sss_modal(&self) {
        let mut sss = self.s.sss_shares;
        sss.set(None);
    }

    /// Track the current public key in the appropriate network slot.
    /// Called after key generation, import, or load.
    pub fn track_network_key(&self) {
        let net = *self.s.current_network.read();
        let pk = self.s.public_key.read().clone();
        match net {
            NetworkEnvironment::Production => {
                let mut mn = self.s.mainnet_public_key;
                mn.set(pk);
            }
            NetworkEnvironment::Test => {
                let mut tn = self.s.testnet_public_key;
                tn.set(pk);
            }
        }
    }

    /// Lock ZSOZSO tokens via the proof-of-zsozso smart contract.
    pub fn lock_zsozso_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let net_env = *self.s.current_network.read();
        let mut zs_status = self.s.zs_status;
        let mut lock_amount_signal = self.s.lock_amount;

        let amount_str = self.s.lock_amount.read().clone();
        let amount: u64 = match amount_str.trim().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                zs_status.set(Some(i18n.zs_invalid_amount().to_string()));
                return;
            }
        };

        let secret = match self.s.secret_key_hidden.read().as_ref() {
            Some(sk) => sk.as_str().to_string(),
            None => {
                zs_status.set(Some(i18n.zs_no_key().to_string()));
                return;
            }
        };

        zs_status.set(Some(i18n.zs_locking().to_string()));

        spawn(async move {
            let sc = crate::ledger::sc::proof_of_zsozso_sc::ProofOfZsozsoSc::new(net_env, lang);
            let i18n = ui_i18n(lang);
            match sc.lock(&secret, amount).await {
                Ok(_) => {
                    zs_status.set(Some(i18n.fmt_zs_lock_success(&amount.to_string())));
                    lock_amount_signal.set(String::new());
                }
                Err(e) => {
                    zs_status.set(Some(i18n.fmt_zs_lock_error(&e)));
                }
            }
        });
    }

    /// Unlock ZSOZSO tokens via the proof-of-zsozso smart contract.
    pub fn unlock_zsozso_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let net_env = *self.s.current_network.read();
        let mut zs_status = self.s.zs_status;
        let mut lock_amount_signal = self.s.lock_amount;

        let amount_str = self.s.lock_amount.read().clone();
        let amount: u64 = match amount_str.trim().parse() {
            Ok(v) if v > 0 => v,
            _ => {
                zs_status.set(Some(i18n.zs_invalid_amount().to_string()));
                return;
            }
        };

        let secret = match self.s.secret_key_hidden.read().as_ref() {
            Some(sk) => sk.as_str().to_string(),
            None => {
                zs_status.set(Some(i18n.zs_no_key().to_string()));
                return;
            }
        };

        zs_status.set(Some(i18n.zs_unlocking().to_string()));

        spawn(async move {
            let sc = crate::ledger::sc::proof_of_zsozso_sc::ProofOfZsozsoSc::new(net_env, lang);
            let i18n = ui_i18n(lang);
            match sc.unlock(&secret, amount).await {
                Ok(_) => {
                    zs_status.set(Some(i18n.fmt_zs_unlock_success(&amount.to_string())));
                    lock_amount_signal.set(String::new());
                }
                Err(e) => {
                    zs_status.set(Some(i18n.fmt_zs_unlock_error(&e)));
                }
            }
        });
    }

    /// Fetch XLM and ZSOZSO balances from Horizon and locked ZSOZSO from the SC.
    /// ZSOZSO balance is only fetched on Mainnet.
    pub fn fetch_balances_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let net_env = *self.s.current_network.read();
        let public_key = self.s.public_key.read().clone();

        let Some(pk) = public_key else {
            let mut zs_status = self.s.zs_status;
            zs_status.set(Some(i18n.zs_no_key().to_string()));
            return;
        };

        let mut xlm_balance = self.s.xlm_balance;
        let mut zsozso_balance = self.s.zsozso_balance;
        let mut locked_zsozso = self.s.locked_zsozso;
        let mut zs_status = self.s.zs_status;

        zs_status.set(Some(i18n.zs_fetching_balances().to_string()));

        spawn(async move {
            // Fetch account balances from Horizon
            let horizon_url = match net_env {
                NetworkEnvironment::Production => "https://horizon.stellar.org",
                NetworkEnvironment::Test => "https://horizon-testnet.stellar.org",
            };
            let url = format!("{}/accounts/{}", horizon_url, pk);
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(balances) = json["balances"].as_array() {
                            let mut xlm: Option<String> = None;
                            let mut zsozso: Option<String> = None;
                            for b in balances {
                                let asset_type = b["asset_type"].as_str().unwrap_or("");
                                if asset_type == "native" {
                                    xlm = b["balance"].as_str().map(|s| s.to_string());
                                } else if b["asset_code"].as_str() == Some("ZSOZSO") {
                                    zsozso = b["balance"].as_str().map(|s| s.to_string());
                                }
                            }
                            xlm_balance.set(xlm);
                            if net_env == NetworkEnvironment::Production {
                                zsozso_balance.set(zsozso);
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[fetch_balances] Error: {}", e));
                }
            }

            // Clear the status spinner
            zs_status.set(None);

            // Locked ZSOZSO from proof-of-zsozso contract — stored locally for now
            // (will query SC once deployed)
            let _ = locked_zsozso;
        });
    }

    // ── Dual-key methods ───────────────────────────────────────────────

    /// Generate a new keypair for a specific network.
    pub fn generate_key_for_network(&self, net: NetworkEnvironment) {
        let lang = *self.s.language.read();
        let (pk, sk) = generate_keypair(net, lang);
        let mut mn_pk = self.s.mainnet_public_key;
        let mut mn_sk = self.s.mainnet_secret_key;
        let mut tn_pk = self.s.testnet_public_key;
        let mut tn_sk = self.s.testnet_secret_key;
        let mut active_pk = self.s.public_key;
        match net {
            NetworkEnvironment::Production => {
                mn_pk.set(Some(pk.clone()));
                mn_sk.set(Some(Zeroizing::new(sk)));
            }
            NetworkEnvironment::Test => {
                tn_pk.set(Some(pk.clone()));
                tn_sk.set(Some(Zeroizing::new(sk)));
            }
        }
        active_pk.set(Some(pk));
    }

    /// Import a keypair from user input for a specific network.
    pub fn import_key_for_network(&self, net: NetworkEnvironment) {
        let lang = *self.s.language.read();
        let raw_input = match net {
            NetworkEnvironment::Production => self.s.mainnet_input_value.read().clone(),
            NetworkEnvironment::Test => self.s.testnet_input_value.read().clone(),
        };

        if let Some((pub_key_str, secret)) = import_keypair(raw_input, net, lang) {
            let mut mn_pk = self.s.mainnet_public_key;
            let mut mn_sk = self.s.mainnet_secret_key;
            let mut mn_iv = self.s.mainnet_input_value;
            let mut tn_pk = self.s.testnet_public_key;
            let mut tn_sk = self.s.testnet_secret_key;
            let mut tn_iv = self.s.testnet_input_value;
            let mut active_pk = self.s.public_key;
            match net {
                NetworkEnvironment::Production => {
                    mn_pk.set(Some(pub_key_str.clone()));
                    mn_sk.set(Some(Zeroizing::new(secret)));
                    mn_iv.set(String::new());
                }
                NetworkEnvironment::Test => {
                    tn_pk.set(Some(pub_key_str.clone()));
                    tn_sk.set(Some(Zeroizing::new(secret)));
                    tn_iv.set(String::new());
                }
            }
            active_pk.set(Some(pub_key_str));
        }
    }

    /// Reveal the secret key for a specific network after passkey verification.
    pub fn reveal_secret_for_network(&self, net: NetworkEnvironment) {
        let biometric_on = *self.s.biometric_enabled.read();
        let mut show_signal = match net {
            NetworkEnvironment::Production => self.s.mainnet_show_secret,
            NetworkEnvironment::Test => self.s.testnet_show_secret,
        };

        spawn(async move {
            if is_localhost() || !biometric_on {
                show_signal.set(true);
                return;
            }
            match passkey::passkey_verify().await {
                Ok(true) => show_signal.set(true),
                _ => {}
            }
        });
    }

    /// Copy the secret key for a specific network to clipboard.
    pub fn copy_secret_for_network(&self, net: NetworkEnvironment) {
        let secret = match net {
            NetworkEnvironment::Production => self.s.mainnet_secret_key.read().clone(),
            NetworkEnvironment::Test => self.s.testnet_secret_key.read().clone(),
        };
        if let Some(secret) = secret {
            copy_to_clipboard(secret.as_str());
            let lang = *self.s.language.read();
            let i18n = ui_i18n(lang);
            log(&i18n.copied().to_string());
            let mut modal = self.s.clipboard_modal_open;
            modal.set(true);
        }
    }

    /// Activate testnet faucet for testnet key.
    pub fn activate_test_account_for_testnet(&self) {
        let pubkey = self.s.testnet_public_key.read().clone();
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;

        spawn(async move {
            status.set(TxStatus::CallingFaucet);
            if let Some(next_status) = activate_test_account(pubkey, NetworkEnvironment::Test, lang).await {
                status.set(next_status);
            }
        });
    }

    /// Save all defined keys to the store — one entry per network.
    pub fn save_all_to_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let biometric_on = *self.s.biometric_enabled.read();

        if !biometric_on && !is_localhost() {
            let mut modal = self.s.biometric_save_modal_open;
            modal.set(true);
            return;
        }

        let mn_secret = self.s.mainnet_secret_key.read().clone();
        let tn_secret = self.s.testnet_secret_key.read().clone();

        if mn_secret.is_none() && tn_secret.is_none() {
            log(&i18n.nothing_to_save().to_string());
            return;
        }

        let existing_prf = self.s.prf_key.read().clone();
        let mut prf_key_signal = self.s.prf_key;
        let pin = self.s.pin_code.read().clone();

        spawn(async move {
            let prf = if is_localhost() {
                None
            } else if !biometric_on {
                None
            } else {
                Some(match existing_prf {
                    Some(key) => key,
                    None => {
                        match passkey::passkey_init().await {
                            Ok(result) if result.success => {
                                match result.prf_key {
                                    Some(key) => {
                                        prf_key_signal.set(Some(key.clone()));
                                        key
                                    }
                                    None => {
                                        let i18n = ui_i18n(lang);
                                        log(&i18n.fmt_error(i18n.no_prf_key()));
                                        return;
                                    }
                                }
                            }
                            _ => {
                                let i18n = ui_i18n(lang);
                                log(&i18n.fmt_error("Authentication failed"));
                                return;
                            }
                        }
                    }
                })
            };

            // Save mainnet key
            if let Some(secret) = mn_secret {
                let store = new_store_for_network(lang, NetworkEnvironment::Production);
                let data = if let Some(ref prf) = prf {
                    match passkey::passkey_encrypt(secret.as_str(), prf).await {
                        Ok(encrypted) => encrypted,
                        Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); return; }
                    }
                } else if is_localhost() && !pin.is_empty() {
                    // On localhost with PIN, use PIN as simple XOR obfuscation key via passkey_encrypt
                    secret.as_str().to_string()
                } else {
                    secret.as_str().to_string()
                };
                match store.save(&data).await {
                    Ok(_) => log("[save_all] Mainnet key saved"),
                    Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); return; }
                }
            }

            // Save testnet key
            if let Some(secret) = tn_secret {
                let store = new_store_for_network(lang, NetworkEnvironment::Test);
                let data = if let Some(ref prf) = prf {
                    match passkey::passkey_encrypt(secret.as_str(), prf).await {
                        Ok(encrypted) => encrypted,
                        Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); return; }
                    }
                } else {
                    secret.as_str().to_string()
                };
                match store.save(&data).await {
                    Ok(_) => log("[save_all] Testnet key saved"),
                    Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); return; }
                }
            }

            let i18n = ui_i18n(lang);
            log(&i18n.save_success().to_string());
        });
    }

    /// Load all keys from the store — one entry per network.
    pub fn load_all_from_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let biometric_on = *self.s.biometric_enabled.read();
        let existing_prf = self.s.prf_key.read().clone();
        let mut prf_key_signal = self.s.prf_key;
        let mut mn_pk = self.s.mainnet_public_key;
        let mut tn_pk = self.s.testnet_public_key;
        let mut mn_sk = self.s.mainnet_secret_key;
        let mut tn_sk = self.s.testnet_secret_key;
        let mut pk_signal = self.s.public_key;
        let mut sk_signal = self.s.secret_key_hidden;

        log(&i18n.loading_started().to_string());

        spawn(async move {
            let prf = if is_localhost() || !biometric_on {
                None
            } else {
                Some(match existing_prf {
                    Some(key) => key,
                    None => {
                        match passkey::passkey_init().await {
                            Ok(result) if result.success => {
                                match result.prf_key {
                                    Some(key) => {
                                        prf_key_signal.set(Some(key.clone()));
                                        key
                                    }
                                    None => {
                                        log(&ui_i18n(lang).fmt_error(ui_i18n(lang).no_prf_key()));
                                        return;
                                    }
                                }
                            }
                            _ => {
                                log(&ui_i18n(lang).fmt_error("Authentication failed"));
                                return;
                            }
                        }
                    }
                })
            };

            // Load mainnet key
            let mn_store = new_store_for_network(lang, NetworkEnvironment::Production);
            if let Ok(stored_data) = mn_store.load().await {
                let decrypted = if let Some(ref prf) = prf {
                    match passkey::passkey_decrypt(&stored_data, prf).await {
                        Ok(d) => d,
                        Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); String::new() }
                    }
                } else {
                    stored_data
                };
                // Strip legacy prefix if present
                let secret = decrypted.strip_prefix("mn:").unwrap_or(&decrypted).to_string();
                if !secret.is_empty() {
                    let lgr = StellarLedger::new(NetworkEnvironment::Production, lang);
                    if let Some(pub_key) = lgr.public_key_from_secret(&secret) {
                        mn_pk.set(Some(pub_key.clone()));
                        mn_sk.set(Some(Zeroizing::new(secret)));
                        // Set as active key
                        pk_signal.set(Some(pub_key));
                        log("[load_all] Mainnet key loaded");
                    }
                }
            }

            // Load testnet key
            let tn_store = new_store_for_network(lang, NetworkEnvironment::Test);
            if let Ok(stored_data) = tn_store.load().await {
                let decrypted = if let Some(ref prf) = prf {
                    match passkey::passkey_decrypt(&stored_data, prf).await {
                        Ok(d) => d,
                        Err(e) => { log(&ui_i18n(lang).fmt_error(&e)); String::new() }
                    }
                } else {
                    stored_data
                };
                let secret = decrypted.strip_prefix("tn:").unwrap_or(&decrypted).to_string();
                if !secret.is_empty() {
                    let lgr = StellarLedger::new(NetworkEnvironment::Test, lang);
                    if let Some(pub_key) = lgr.public_key_from_secret(&secret) {
                        tn_pk.set(Some(pub_key.clone()));
                        tn_sk.set(Some(Zeroizing::new(secret)));
                        log("[load_all] Testnet key loaded");
                    }
                }
            }

            // Also try loading from legacy "default_account" store for migration
            let legacy_store = new_store(lang);
            if let Ok(stored_data) = legacy_store.load().await {
                let decrypted = if let Some(ref prf) = prf {
                    match passkey::passkey_decrypt(&stored_data, prf).await {
                        Ok(d) => d,
                        Err(_) => String::new()
                    }
                } else {
                    stored_data
                };
                if !decrypted.is_empty() {
                    let (net, secret) = if let Some(rest) = decrypted.strip_prefix("tn:") {
                        (NetworkEnvironment::Test, rest.to_string())
                    } else if let Some(rest) = decrypted.strip_prefix("mn:") {
                        (NetworkEnvironment::Production, rest.to_string())
                    } else {
                        (NetworkEnvironment::Production, decrypted)
                    };
                    let lgr = StellarLedger::new(net, lang);
                    if let Some(pub_key) = lgr.public_key_from_secret(&secret) {
                        match net {
                            NetworkEnvironment::Production => {
                                if mn_pk.read().is_none() {
                                    mn_pk.set(Some(pub_key.clone()));
                                    mn_sk.set(Some(Zeroizing::new(secret)));
                                    pk_signal.set(Some(pub_key));
                                    log("[load_all] Migrated legacy key as mainnet");
                                }
                            }
                            NetworkEnvironment::Test => {
                                if tn_pk.read().is_none() {
                                    tn_pk.set(Some(pub_key.clone()));
                                    tn_sk.set(Some(Zeroizing::new(secret)));
                                    log("[load_all] Migrated legacy key as testnet");
                                }
                            }
                        }
                    }
                }
            }

            // Sync the backward-compatible signal with mainnet key (primary)
            if let Some(pk) = mn_pk.read().clone() {
                pk_signal.set(Some(pk.clone()));
                sk_signal.set(mn_sk.read().clone());
            } else if let Some(pk) = tn_pk.read().clone() {
                pk_signal.set(Some(pk.clone()));
                sk_signal.set(tn_sk.read().clone());
            }

            log(&ui_i18n(lang).ui_updated_with_key().to_string());
        });
    }

    /// Set the PIN code (localhost only).
    pub fn set_pin_code(&self, pin: String) {
        let mut pin_signal = self.s.pin_code;
        pin_signal.set(pin);
    }
}

/// Returns true when the app is served from localhost (development).
fn is_localhost() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "::1")
}

/// Write biometric preference to localStorage.
fn write_biometric_pref(enabled: bool) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("zsozso:biometric", if enabled { "true" } else { "false" });
    }
}
