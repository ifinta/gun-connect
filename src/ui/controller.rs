use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::{WalletState, AuthState};
use super::actions::*;
use super::actions::{new_store_for_network, discover_relays};
use super::status::TxStatus;
use super::i18n::ui_i18n;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};
use zsozso_store::Store;
use zsozso_store::passkey;
use zsozso_db::gundb::{GunSea, Sea};
use zsozso_db::network::{NetworkGraph, GunNetworkGraph};
use super::clipboard::{copy_to_clipboard, clear_clipboard};
use super::log;

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

    /// Generate XDR for a manageData transaction that publishes the GUN relay URL on Testnet.
    pub fn fetch_and_generate_xdr_action(&self) {
        let secret_key = self.s.testnet_secret_key.read().as_ref().map(|s| s.to_string());
        let relay_url = self.s.gun_relay_url.read().clone();
        let net_env = NetworkEnvironment::Test;
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;
        let mut xdr_signal = self.s.generated_xdr;

        spawn(async move {
            status.set(TxStatus::FetchingSequence);
            match fetch_and_generate_xdr(secret_key, relay_url, net_env, lang).await {
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

    pub fn set_language(&self, code: &str) {
        use zsozso_common::Language;
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

    /// Check if the configured GUN relay is reachable.
    pub fn check_relay_action(&self) {
        let relay_url = self.s.gun_relay_url.read().clone();
        let mut status = self.s.relay_status;
        let mut checking = self.s.relay_checking;

        if relay_url.trim().is_empty() {
            status.set(None);
            return;
        }

        checking.set(true);

        spawn(async move {
            log(&format!("[check_relay] Checking relay: {}", relay_url));
            let js_code = format!(
                "window.__gun_bridge.checkRelay('{}')",
                relay_url.replace('\'', "\\'")
            );
            let result = match js_sys::eval(&js_code) {
                Ok(val) => {
                    match wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(val)).await {
                        Ok(json_val) => {
                            let json_str = json_val.as_string().unwrap_or_default();
                            log(&format!("[check_relay] Result: {}", json_str));
                            json_str.contains("\"ok\":true")
                        }
                        Err(e) => {
                            log(&format!("[check_relay] Promise error: {:?}", e));
                            false
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[check_relay] Eval error: {:?}", e));
                    false
                }
            };
            status.set(Some(result));
            checking.set(false);
        });
    }

    /// Discover GUN relays published on Stellar testnet and check their connectivity.
    pub fn discover_relays_action(&self) {
        let mut relays_signal = self.s.discovered_relays;
        let mut discovering = self.s.discovering_relays;

        discovering.set(true);
        relays_signal.set(vec![]);

        spawn(async move {
            let relays = discover_relays().await;
            log(&format!("[discover_relays_action] Found {} relays, checking connectivity...", relays.len()));

            // Set them immediately (reachable = None) so the UI shows them
            relays_signal.set(relays.clone());

            // Now check each relay's connectivity
            for (i, relay) in relays.iter().enumerate() {
                let js_code = format!(
                    "window.__gun_bridge.checkRelay('{}', 4000)",
                    relay.url.replace('\'', "\\'")
                );
                let reachable = match js_sys::eval(&js_code) {
                    Ok(val) => {
                        match wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(val)).await {
                            Ok(json_val) => {
                                let json_str = json_val.as_string().unwrap_or_default();
                                json_str.contains("\"ok\":true")
                            }
                            Err(_) => false,
                        }
                    }
                    Err(_) => false,
                };
                // Update the single relay's reachable status
                let mut current = relays_signal.read().clone();
                if let Some(r) = current.get_mut(i) {
                    r.reachable = Some(reachable);
                }
                relays_signal.set(current);
            }

            discovering.set(false);
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
        let _ = storage.set_item("gun-connect:biometric", if enabled { "true" } else { "false" });
    }
}
