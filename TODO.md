# todo:

---

## Development Methodology

We follow **manual test-driven development** within an agile workflow:
1. Pick a small target from the TODO or a near-term goal
2. Implement the minimal change
3. Test manually in the browser (PWA on mobile and/or desktop)
4. If it works → commit. If not → iterate (fix → test again)
5. Update documentation **only when a step target is reached** — not at every micro-change

> **Rule:** Don't update docs speculatively. Document what **is**, not what **might be**.

---

## Code Structure Vision

**Current (Phase 1):** 6 PWA apps + 4 shared Rust libraries, all as Cargo git dependencies.

**Next (Phase 2):** Extract reusable **Dioxus UI components** into shared git libraries. Dioxus follows the React component model — this makes components naturally shareable across apps. Examples: auth gate, settings panel, key management UI, log viewer, tab bar.

**Later (Phase 3):** Add **Soroban smart contract** projects to the ecosystem. Two drafts already exist:
- `proof-of-zsozso-sc` — a vault SC for ZSOZSO token locking on Stellar Mainnet
- `zsozso-sc` — a template SC (first working draft, testnet ping/upgrade/admin)

---

## Near-Term Targets

- **LOG:** Every app sends logs to admin. Admin collects, filters, and displays logs from all apps. Concept still being explored — some ideas exist, needs iteration.
- **GUN relay sharing:** `gun-connect` app manages relay discovery + sharing between apps/users. Some relay functionality was temporarily moved to `admin` (which has a working log tab for debugging). Will be redistributed after iteration.
- **MLM network:** `mlm` and `merlin` apps — build and manage the Antarctica MLM hierarchy. Merlin is the root node.
- **Biometric sharing:** Understand and handle WebAuthn passkey sharing between PWA apps on the same device. Needs research + iteration to find a good solution.

---

## Tooling & Build Commands

### Everyday commands

```bash
# Dev server with hot-reload
dx serve --platform web

# Release build
./build.sh

# Dry run — show CACHE_NAME without building
./build.sh --dry
```

### When dependencies change or builds fail

```bash
# Remove lock file and re-resolve all dependencies
rm Cargo.lock
cargo update
```

This is needed when:
- A shared library (`zsozso-common`, `zsozso-db`, `zsozso-ledger`, `zsozso-store`) was updated
- Dependency conflicts arise after editing `Cargo.toml`
- Build errors point to version mismatches

### Updating dioxus-cli

```bash
# Fast: install pre-built binary (seconds, no compilation)
cargo binstall dioxus-cli --force

# Slow: build from source (minutes, compiles everything)
cargo install dioxus-cli --force
```

`cargo-binstall` downloads a pre-compiled binary from GitHub releases — **much faster** (seconds vs. 5-10+ minutes). The downside: the binary may lag behind the latest source by a few days. For day-to-day work, `binstall` is the right choice. Use source install only if you need a bleeding-edge fix not yet in a release.

To install `cargo-binstall` itself (one-time):
```bash
cargo install cargo-binstall
```

---

## simple steps:
- setup a gun server
- ~~implement relay connection status display~~ (done — Info tab shows relay status with check button)

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
- ~~generate unique app icons~~ (done)
- a good graphics design (styles...(learn it) use components!)
- remove unused tabs (CYF, Zsozso, Networking, Log) and their controller methods
- ~~integrate db, ledger, store as git submodules instead of copied source~~ (done — now Cargo git deps via zsozso-common/zsozso-db/zsozso-ledger/zsozso-store)

# for dev's:

## Architecture

The application targets **PWA (Progressive Web App) only** — all code compiles to WebAssembly and runs in the browser. There are no desktop or native feature flags; the single `web` feature is the default.

```
src/
├── main.rs                  # Entry point — Dioxus web launch
├── i18n.rs                  # Language enum (English, Hungarian, French, German, Spanish)
├── sss.rs                   # Shamir's Secret Sharing over GF(256)
└── ui/                      # (db, ledger, store are Cargo git deps — not in src/)
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
| `__gun_connect_log` | `log_bridge.js` | — | In-app log ring buffer |

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
