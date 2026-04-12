use dioxus::prelude::*;
use zeroize::Zeroizing;
use zsozso_common::Language;
use super::tabs::Tab;
use super::actions::{DiscoveredRelay, RelayEntry};

/// Read biometric preference from localStorage (synchronous).
fn biometric_enabled_default() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("gun-connect:biometric").ok())
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Read saved relay URL from localStorage (synchronous).
fn read_relay_url() -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("gun-connect:relay_url").ok())
        .flatten()
        .unwrap_or_default()
}

/// Passkey authentication state machine.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum AuthState {
    #[default]
    Pending,        // gate modal shown, waiting for user to click
    Authenticating, // passkey dialog in progress
    Authenticated,  // success — show app tabs
    Failed,         // terminal — show error modal
}

#[derive(Clone, Copy)]
pub struct WalletState {
    pub language: Signal<Language>,
    pub public_key: Signal<Option<String>>,
    pub secret_key_hidden: Signal<Option<Zeroizing<String>>>,
    pub clipboard_modal_open: Signal<bool>,
    pub active_tab: Signal<Tab>,
    pub auth_state: Signal<AuthState>,
    pub prf_key: Signal<Option<String>>,
    /// Whether biometric (passkey) authentication is enabled.
    pub biometric_enabled: Signal<bool>,
    /// Whether the "enable biometric to save" error modal is shown.
    pub biometric_save_modal_open: Signal<bool>,
    /// Optional GUN relay URL — if the user runs their own GUN DB node.
    pub gun_relay_url: Signal<String>,
    /// Connected relays list (max MAX_RELAYS).
    pub connected_relays: Signal<Vec<RelayEntry>>,
    /// Discovered relays from Stellar testnet.
    pub discovered_relays: Signal<Vec<DiscoveredRelay>>,
    /// Whether relay discovery is in progress.
    pub discovering_relays: Signal<bool>,
    /// Stored mainnet public key.
    pub mainnet_public_key: Signal<Option<String>>,
    /// Stored testnet public key.
    pub testnet_public_key: Signal<Option<String>>,
    /// Mainnet secret key.
    pub mainnet_secret_key: Signal<Option<Zeroizing<String>>>,
    /// Testnet secret key.
    pub testnet_secret_key: Signal<Option<Zeroizing<String>>>,
    /// Mainnet import input field.
    pub mainnet_input_value: Signal<String>,
    /// Testnet import input field.
    pub testnet_input_value: Signal<String>,
    /// Whether the mainnet secret is revealed.
    pub mainnet_show_secret: Signal<bool>,
    /// Whether the testnet secret is revealed.
    pub testnet_show_secret: Signal<bool>,
    /// Localhost PIN code (used instead of passkey on localhost).
    pub pin_code: Signal<String>,
}

pub fn use_wallet_state() -> WalletState {
    let bio = biometric_enabled_default();
    WalletState {
        language: use_signal(Language::default),
        public_key: use_signal(|| None),
        secret_key_hidden: use_signal(|| None),
        clipboard_modal_open: use_signal(|| false),
        active_tab: use_signal(Tab::default),
        // If biometric is disabled, skip the auth gate entirely.
        auth_state: use_signal(move || if bio { AuthState::default() } else { AuthState::Authenticated }),
        prf_key: use_signal(|| None),
        biometric_enabled: use_signal(move || bio),
        biometric_save_modal_open: use_signal(|| false),
        gun_relay_url: use_signal(read_relay_url),
        connected_relays: use_signal(Vec::new),
        discovered_relays: use_signal(Vec::new),
        discovering_relays: use_signal(|| false),
        mainnet_public_key: use_signal(|| None),
        testnet_public_key: use_signal(|| None),
        mainnet_secret_key: use_signal(|| None),
        testnet_secret_key: use_signal(|| None),
        mainnet_input_value: use_signal(String::new),
        testnet_input_value: use_signal(String::new),
        mainnet_show_secret: use_signal(|| false),
        testnet_show_secret: use_signal(|| false),
        pin_code: use_signal(String::new),
    }
}
