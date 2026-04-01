# Copilot Instructions — Gun Connect

## Overview

Gun Connect is a **PWA app** for connecting a GUN relay node to a mobile device. Manages GUN database relay configuration and connectivity. Part of the [Iceberg Protocol](https://zsozso.info) ecosystem.

Built with **Dioxus 0.7** (Rust → WASM), runs as a PWA only — no native/desktop target.

## Build & Run

```bash
# Clone with submodules
git clone --recurse-submodules git@github.com:ifinta/gun-connect.git
# Or if already cloned:
git submodule update --init --recursive

# Dev server with hot-reload
dx serve --platform web

# Release build
./build.sh

# Dry run — show CACHE_NAME without building
./build.sh --dry
```

Prerequisites: `rustup target add wasm32-unknown-unknown` and `cargo install dioxus-cli`.

### Git submodules

`src/db`, `src/ledger`, and `src/store` are **git submodules** — shared libraries across all Iceberg Protocol apps. The `i18n.rs` (Language enum) and JS bridge files live in the app itself, not in the submodules.

No tests exist. No linter beyond `cargo check`.

## App Tabs

This app has **2 tabs**:

1. **Info tab** — Displays app version, QR code of public key, public key as text. Relay connection status.
2. **Settings tab** — Key management (generate/import/reveal/copy for mainnet+testnet), biometric/PIN toggle, language selector, nickname, GUN DB secret, GUN relay URL configuration, save/load persistence.

### Tabs NOT in this app (to be removed)

- ~~CYF tab~~ (burn/mint CYF — belongs to cyf app)
- ~~Zsozso tab~~ (lock/unlock ZS tokens — belongs to proof-of-zsozso app)
- ~~Networking tab~~ (MLM hierarchy — belongs to mlm app)
- ~~Log tab~~ (log buffer, GUN dump — belongs to admin app)

## Architecture

### Shared libraries (git submodules)

| Library | Purpose |
|---------|---------|
| `db` | GUN.js decentralized database (Db, Sea, NetworkGraph traits) |
| `ledger` | Stellar blockchain (Ledger, Cyf, SmartContract traits) |
| `store` | Encrypted secret persistence (Store trait, passkey, IndexedDB) |

### Key focus: GUN relay connectivity

This app's primary purpose is managing the GUN relay connection:
- Configure relay URL via settings
- Verify connectivity to the relay
- Display connection status
- The GUN DB config (`GunConfig` in `db/mod.rs`) specifies relay URLs

### Module layout

- **`src/ui/`** — Dioxus components, state, controller, tabs
- **`src/ledger/`** — (from submodule) Blockchain abstraction
- **`src/store/`** — (from submodule) Secret persistence
- **`src/db/`** — (from submodule) Decentralized database
- **`src/i18n.rs`** — `Language` enum
- **`src/sss.rs`** — Shamir's Secret Sharing

### JS Bridges

| Bridge | JS file | Rust module |
|--------|---------|-------------|
| `__gun_bridge` | `gun_bridge.js` | `db::gundb` |
| `__sea_bridge` | `sea_bridge.js` | `db::gundb::sea` |
| `__passkey_bridge` | `passkey_bridge.js` | `store::passkey` |
| `__zsozso_log` | `log_bridge.js` | (for remote log upload) |

## Key Controller Methods (relevant to this app)

| Method | Purpose |
|--------|---------|
| `save_gun_relay_action()` | Store relay URL in GUN DB (SEA-signed) |
| `open_sea_modal()` / `generate_sea_keys()` | Generate DB secret for SEA authentication |
| Key management methods | generate/import/reveal/copy keys |
| Store methods | save/load persistence |

## Key Conventions

### Coding style

- All trait async methods use `#[allow(async_fn_in_trait)]`
- Errors are `Result<T, String>` — no custom error types
- Secret keys wrapped in `Zeroizing<String>`
- All styles are inline CSS via Rust format strings in `rsx!` — no CSS files

### Component pattern

Tab components are free functions returning `Element`:
```rust
pub fn render_info_tab(i18n: &dyn UiI18n) -> Element { rsx! { ... } }
```

### I18n pattern

Every user-facing string is behind an i18n trait method. Factory functions select implementation.
Method naming: `btn_*()` for buttons, `lbl_*()` for labels, `fmt_*()` for format helpers.

### State management

`WalletState` — struct of Dioxus `Signal<T>` fields. Initialized via `use_wallet_state()`.
`AppController` — bridges state and actions. Sync methods mutate signals; async methods use `spawn()` with `*_action()` suffix.

### Service worker & deployment

`build.sh` stamps date+commit CACHE_NAME. SW uses `skipWaiting()` + `clients.claim()`. App served under `/app/` (base_path in Dioxus.toml).

## Ecosystem

- **Live app**: https://zsozso.info/app/
- **Project website**: https://zsozso.info
- **Smart contracts**: https://github.com/ifinta/zsozso-sc
- **Original monolith**: zsozso-dioxus (now experimental only)
