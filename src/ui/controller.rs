use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::{WalletState, AuthState};
use super::actions::*;
use super::actions::{new_store_for_network, discover_relays};
use super::i18n::ui_i18n;
use zsozso_ledger::{Ledger, NetworkEnvironment, StellarLedger};
use zsozso_store::Store;
use zsozso_store::passkey;
use zsozso_db::gundb::GunDb;
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

    /// Dismiss the clipboard modal and clear clipboard content
    pub fn dismiss_clipboard_modal(&self) {
        clear_clipboard();
        let mut modal = self.s.clipboard_modal_open;
        modal.set(false);
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        log(&i18n.clipboard_cleared().to_string());
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

    /// Save the GUN relay URL to the graph database.
    pub fn save_gun_relay_action(&self) {
        let relay_url = self.s.gun_relay_url.read().clone();

        // Always persist relay URL to localStorage
        write_relay_url(&relay_url);

        // Always add to connected relays list (triggers publish to Stellar)
        if !relay_url.trim().is_empty() {
            self.add_relay_action(relay_url.clone());
        }
    }

    /// Check all connected relays for reachability.
    pub fn check_all_relays_action(&self) {
        let mut relays = self.s.connected_relays;
        let entries = relays.read().clone();
        if entries.is_empty() { return; }

        // Mark all as checking
        relays.set(entries.iter().map(|r| RelayEntry {
            checking: true,
            reachable: r.reachable,
            ..r.clone()
        }).collect());

        spawn(async move {
            let entries = relays.read().clone();
            for (i, entry) in entries.iter().enumerate() {
                log(&format!("[check_all_relays] Checking {}: {}", i, entry.url));
                let ok = GunDb::check_relay(&entry.url, 5000).await.unwrap_or(false);
                let mut current = relays.read().clone();
                if let Some(r) = current.get_mut(i) {
                    r.reachable = Some(ok);
                    r.checking = false;
                }
                relays.set(current);
            }
        });
    }

    /// Add a relay URL to the connected relays list (if not already present and under limit).
    pub fn add_relay_action(&self, url: String) {
        let mut relays = self.s.connected_relays;
        let current = relays.read().clone();
        if current.len() >= MAX_RELAYS { return; }
        if current.iter().any(|r| r.url == url) { return; }

        // Persist to localStorage
        add_persisted_relay(&url);

        let mut new_list = current;
        new_list.push(RelayEntry { url: url.clone(), reachable: None, checking: true });
        relays.set(new_list);

        let secret_key = self.s.testnet_secret_key.read().as_ref().map(|s| s.to_string());

        // Check the newly added relay, then publish all to Stellar
        spawn(async move {
            log(&format!("[add_relay] Checking new relay: {}", url));
            let ok = GunDb::check_relay(&url, 5000).await.unwrap_or(false);
            let mut current = relays.read().clone();
            if let Some(r) = current.iter_mut().find(|r| r.url == url) {
                r.reachable = Some(ok);
                r.checking = false;
            }
            relays.set(current);

            // Publish all connected relays to Stellar
            if let Some(sk) = secret_key {
                let urls: Vec<String> = relays.read().iter().map(|r| r.url.clone()).collect();
                if !urls.is_empty() {
                    match publish_relays(&sk, &urls, NetworkEnvironment::Test).await {
                        Ok(()) => {
                            log(&format!("[add_relay] Published {} relays to Stellar", urls.len()));
                        }
                        Err(e) => {
                            log(&format!("[add_relay] Publish failed: {}", e));
                        }
                    }
                }
            }
        });
    }

    /// Remove a relay URL from the connected relays list.
    pub fn remove_relay_action(&self, url: String) {
        let mut relays = self.s.connected_relays;
        let mut current = relays.read().clone();
        current.retain(|r| r.url != url);
        relays.set(current);

        // Remove from localStorage
        remove_persisted_relay(&url);
    }

    /// Discover GUN relays published on Stellar testnet and check their connectivity.
    pub fn discover_relays_action(&self) {
        let mut relays_signal = self.s.discovered_relays;
        let mut discovering = self.s.discovering_relays;
        let connected = self.s.connected_relays;
        let own_address = self.s.testnet_public_key.read().clone();

        discovering.set(true);
        relays_signal.set(vec![]);

        spawn(async move {
            let connected_urls: std::collections::HashSet<String> =
                connected.read().iter().map(|r| r.url.clone()).collect();

            // Build known accounts list: our own address + previously discovered
            let mut known = read_known_accounts();
            if let Some(ref addr) = own_address {
                if !known.contains(addr) {
                    known.push(addr.clone());
                }
            }

            let (relays, updated_accounts) = discover_relays(&connected_urls, &known).await;

            // Persist the updated known accounts list
            write_known_accounts(&updated_accounts);

            log(&format!("[discover_relays_action] Found {} relays, checking connectivity...", relays.len()));

            relays_signal.set(relays.clone());

            for (i, relay) in relays.iter().enumerate() {
                let ok = GunDb::check_relay(&relay.url, 4000).await.unwrap_or(false);
                let mut current = relays_signal.read().clone();
                if let Some(r) = current.get_mut(i) {
                    r.reachable = Some(ok);
                }
                relays_signal.set(current);
            }

            discovering.set(false);
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

        spawn(async move {
            let _ = activate_test_account(pubkey, NetworkEnvironment::Test, lang).await;
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
        let mut connected_relays = self.s.connected_relays;

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

            // Load saved relay URLs into connected relays and check reachability
            let saved = read_relay_urls();
            for relay_url in &saved {
                let current = connected_relays.read().clone();
                if current.iter().any(|r| r.url == *relay_url) { continue; }
                let mut list = current;
                list.push(RelayEntry { url: relay_url.clone(), reachable: None, checking: true });
                connected_relays.set(list);

                let ok = GunDb::check_relay(relay_url, 5000).await.unwrap_or(false);
                let mut current = connected_relays.read().clone();
                if let Some(r) = current.iter_mut().find(|r| r.url == *relay_url) {
                    r.reachable = Some(ok);
                    r.checking = false;
                }
                connected_relays.set(current);
                log(&format!("[load_all] Relay {} reachable: {}", relay_url, ok));
            }

            // Auto-create testnet key if none was loaded
            if tn_pk.read().is_none() {
                log("[load_all] No testnet key found, generating one...");
                let (pk, sk) = generate_keypair(NetworkEnvironment::Test, lang);
                tn_pk.set(Some(pk.clone()));
                tn_sk.set(Some(Zeroizing::new(sk.clone())));
                pk_signal.set(Some(pk.clone()));
                sk_signal.set(Some(Zeroizing::new(sk.clone())));

                // Save the new key to store immediately
                let store = new_store_for_network(lang, NetworkEnvironment::Test);
                let data = if let Some(ref prf) = prf {
                    match passkey::passkey_encrypt(&sk, prf).await {
                        Ok(encrypted) => encrypted,
                        Err(_) => sk.clone(),
                    }
                } else {
                    sk
                };
                let _ = store.save(&data).await;
                log("[load_all] Testnet key auto-generated and saved");

                // Activate via faucet
                let _ = activate_test_account(Some(pk), NetworkEnvironment::Test, lang).await;
            }

            // Re-publish all connected relays to Stellar Testnet
            let relay_urls: Vec<String> = connected_relays.read().iter().map(|r| r.url.clone()).collect();
            if let Some(ref sk) = *tn_sk.read() {
                if !relay_urls.is_empty() {
                    log(&format!("[load_all] Publishing {} relays to Stellar...", relay_urls.len()));
                    match publish_relays(sk.as_str(), &relay_urls, NetworkEnvironment::Test).await {
                        Ok(()) => {
                            log("[load_all] Relays published to Stellar");
                        }
                        Err(e) => {
                            log(&format!("[load_all] Publish failed: {}", e));
                        }
                    }
                }
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

/// Write relay URL to localStorage.
fn write_relay_url(url: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("gun-connect:relay_url", url);
    }
}

/// Read all saved relay URLs from localStorage.
fn read_relay_urls() -> Vec<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("gun-connect:relay_urls").ok())
        .flatten()
        .map(|s| s.lines().filter(|l| !l.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

/// Write all saved relay URLs to localStorage.
fn write_relay_urls(urls: &[String]) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("gun-connect:relay_urls", &urls.join("\n"));
    }
}

/// Add a relay URL to the persisted list (dedup, max 5).
fn add_persisted_relay(url: &str) {
    let mut urls = read_relay_urls();
    if !urls.iter().any(|u| u == url) {
        urls.push(url.to_string());
        if urls.len() > MAX_RELAYS {
            urls.remove(0);
        }
        write_relay_urls(&urls);
    }
}

/// Remove a relay URL from the persisted list.
fn remove_persisted_relay(url: &str) {
    let mut urls = read_relay_urls();
    urls.retain(|u| u != url);
    write_relay_urls(&urls);
}

/// Read known relay operator accounts from localStorage.
fn read_known_accounts() -> Vec<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("gun-connect:known_accounts").ok())
        .flatten()
        .map(|s| s.lines().filter(|l| !l.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

/// Write known relay operator accounts to localStorage.
fn write_known_accounts(accounts: &[String]) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("gun-connect:known_accounts", &accounts.join("\n"));
    }
}
