# Gun Connect

PWA app for connecting a GUN relay node to a mobile device. Manages GUN database relay configuration and connectivity. Part of the [Iceberg Protocol](https://zsozso.info) ecosystem, built with Rust and [Dioxus](https://dioxuslabs.com/). Runs as a PWA in the browser via WebAssembly.

## Tabs

- **Info** — Public key QR code display, relay connection status, app version
- **Settings** — Key management, biometric/PIN, language, GUN DB secret, GUN relay URL configuration, persistence

## Installation

### iOS

- In Safari open the PWA App from web
- Select Share button
- Select "To Home Screen"
- The App Icon will be reachable on the Home Screen, the App gets the latest version of the code automatically

### Android

- In Chrome open the PWA App from web
- Tap the three-dot menu (⋮) in the top-right corner
- Select "Add to Home screen" or "Install app"
- The App Icon will be reachable on the Home Screen, the App gets the latest version of the code automatically

## Build & Run

```bash
# Clone
git clone git@github.com:ifinta/gun-connect.git
cd gun-connect

# Dev server with hot-reload
dx serve --platform web

# Release build
./build.sh

# Dry run
./build.sh --dry
```

Prerequisites: `rustup target add wasm32-unknown-unknown` and `cargo install dioxus-cli`.

## Related Repositories

- [zsozso.info](https://zsozso.info) — Project website & whitepaper
- [zsozso-common](https://github.com/ifinta/zsozso-common) — Language enum, i18n traits (Cargo git dep)
- [db](https://github.com/ifinta/db) — GUN.js database (Cargo git dep)
- [ledger](https://github.com/ifinta/ledger) — Stellar blockchain (Cargo git dep)
- [store](https://github.com/ifinta/store) — Encrypted persistence (Cargo git dep)
- [zsozso-sc](https://github.com/ifinta/zsozso-sc) — Soroban smart contracts
