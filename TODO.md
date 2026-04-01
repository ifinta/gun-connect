# todo:

## simple steps:
- setup a gun server
- implement relay connection status display

## known issues:
#### (not yet solved, but it isn't mandatory to correct it):
- The status not changing - partially - if I change the language and we are in an async function

## interapp communication (select approach later):
- GUN.js real-time sync — apps share data through the decentralized GUN database (already available)
- BroadcastChannel API — same-origin inter-tab messaging (instant, no server needed)
- SharedWorker — same-origin shared background thread between app tabs
- PostMessage — cross-origin iframe/window communication
- URL deep links — pass data between apps via URL parameters
- Clipboard — copy/paste public keys, addresses between apps
- Web Share API — native share sheet to pass data between apps
- Shared IndexedDB / localStorage — same-origin apps read/write shared storage

## bigger steps:
- generate unique app icons (icon-192.png, icon-512.png): GUN symbol with a connector
- a good graphics design (styles...(learn it) use components!)
- remove unused tabs (CYF, Zsozso, Networking, Log) and their controller methods
- integrate db, ledger, store as git submodules instead of copied source

# for dev's:

## Architecture

The application targets **PWA (Progressive Web App) only** — all code compiles to WebAssembly and runs in the browser. There are no desktop or native feature flags; the single `web` feature is the default.

```
src/
├── main.rs                  # Entry point — Dioxus web launch
├── i18n.rs                  # Language enum (English, Hungarian, French, German, Spanish)
├── sss.rs                   # Shamir's Secret Sharing over GF(256)
├── db/                      # (git submodule → github.com/ifinta/db)
├── ledger/                  # (git submodule → github.com/ifinta/ledger)
├── store/                   # (git submodule → github.com/ifinta/store)
└── ui/
    ├── mod.rs               # Dioxus UI entry — app() component
    ├── clipboard.rs         # Clipboard — navigator.clipboard API
    ├── actions.rs           # Async UI actions
    ├── state.rs             # Reactive wallet state (signals)
    ├── controller.rs        # AppController — bridges state ↔ actions
    ├── status.rs            # TxStatus enum
    ├── toast.rs             # UpdateNotification — SW update toast
    ├── view.rs              # Main view layout, auth gate, tab bar
    ├── tabs/
    │   ├── mod.rs           # Tab enum
    │   ├── info.rs          # Info tab — public key QR, relay status (MAIN TAB)
    │   └── settings.rs      # Settings tab — key management, GUN relay URL config
    └── i18n/                # UiI18n trait — all UI-facing strings
        ├── mod.rs
        ├── english.rs
        ├── hungarian.rs
        ├── french.rs
        ├── german.rs
        └── spanish.rs
```

### JS Bridges

| Bridge | JS file | Rust module | Purpose |
|--------|---------|-------------|---------|
| `__gun_bridge` | `gun_bridge.js` | `db::gundb` | GUN decentralised database |
| `__sea_bridge` | `sea_bridge.js` | `db::sea` | GUN SEA crypto |
| `__passkey_bridge` | `passkey_bridge.js` | `store::passkey` | WebAuthn + AES-GCM |
| `__zsozso_log` | `log_bridge.js` | — | In-app log ring buffer |

### Service Worker Update Strategy

- `index.html` registers the SW with `updateViaCache: 'none'`
- New SW calls `skipWaiting()` + `clients.claim()` for immediate activation
- `CACHE_NAME` in `sw.js` is stamped by `build.sh` on every deploy
- Toast polls `window.__ZSOZSO_UPDATE_READY` and shows a "Refresh" button

### Internationalization

| Trait | Module | Purpose |
|-------|--------|---------|
| `UiI18n` | `ui/i18n` | All UI-facing strings |
| `LedgerI18n` | `ledger/i18n` | Blockchain operation messages |
| `StoreI18n` | `store/i18n` | Secret storage messages |
| `ScI18n` | `ledger/sc/i18n` | Smart contract messages |
| `DbI18n` | `db/i18n` | Database messages |

### Target Platforms

| Platform | Status |
|----------|--------|
| Web (WASM/PWA) | ✅ Primary target |
| iOS Safari (PWA) | ✅ Share → "Add to Home Screen" |
| Android Chrome (PWA) | ✅ Menu → "Add to Home screen" |
| Desktop Chrome/Edge (PWA) | ✅ Address bar install icon |
