# todo:

---

## GitHub Pages deploy — eljárás új app felhúzásához

Minta: `gun-connect`, `admin`, `proof-of-zsozso`. GH Pages URL ugyanaz mint a
live, csak `ifinta.github.io/<prefix>/` alatt. Prefix ugyanaz mindkettő
esetén (itt `gun-connect`), kivéve ha az app neve ütközik egy meglévő repo-val
— pl. `proof-of-zsozso` GH Pages prefixe `proof-of-zsozso-dioxus` a régi
monolit miatt.

### Lokálisan (egyszer per app)

1. **`build.sh`** — `-ghpages` flag támogatás:
   - `GHPAGES=false` arg parser, `APP_VERSION` infix `-gh-` vagy `-app-`.
   - `if [ -z "${CI:-}" ]` guard a `sync-bridges.sh` köré (CI-ben a
     bridge fájlok már a repo-ban vannak, a sibling repo nincs meg).
   - `node bundle.js` ELŐTT `if $GHPAGES; then` blokk, sed-del patcheli:
     - `sw.js` → `var __BASE_PREFIX = '/<prefix>/';`
     - `index.html` → `let PREFIX = "<prefix>";`
     - `manifest.json` → `"id"` / `"start_url"` / `"scope"` = `/<prefix>/`
   - Egyszeres quote-okat `'"'"'` szekvenciával escape-eld (lásd admin).
2. **`.github/workflows/deploy.yml`** — másolás gun-connect-ből, csak az
   `upload-pages-artifact` lépés `path:` sorát kell az app deploy mappájára
   cserélni (`deploy/<prefix>`).
3. **Commit + push** main ágra.

### GitHub UI (egyszer per repo)

- **Settings → Secrets and variables → Actions**: `CARGO_SSH_KEY`
  (ugyanaz az SSH private key, amit a többi appnál; deploy key read-only
  access-szel a privát `zsozso-common` / `-db` / `-ledger` / `-store` repókhoz).
  Secrets nem oszthatók meg repók között — minden új appban újra kell rakni.
- **Settings → Pages → Source**: `GitHub Actions` (NEM "Deploy from a branch").
- (Opcionális) **Settings → Environments → github-pages**: `main`-re szűkített
  deployment branches.

### Lokális teszt `-ghpages` build-del

```bash
./build.sh -ghpages
npx serve deploy/ -l 8080   # → http://localhost:8080/<prefix>/
```

### Ismert buktatók

- Ha `cargo fetch` "Permission denied (publickey)" hibával bukik → a
  `CARGO_SSH_KEY` secret hiányzik vagy nincs bekötve minden privát
  függőség repóba.
- Ha a deploy job panaszkodik hogy "Pages site not yet created" → a
  Settings → Pages source még nincs `GitHub Actions`-re állítva.
- `Dioxus.toml` `base_path` lokálisan módosul a `build.sh` sed-jétől
  (dirty working tree) — commit előtt visszaállítani.

---

## LOG rendszer — peer-review pontok (2026-04)

Kanonikus forrás: `../store/log_bridge.template.js` + `../store/sw.template.js`,
alkalmazás-specifikus render: `./sync-bridges.sh`.

Sor-formátum (grep-barát):
`YYYY-MM-DD HH:MM:SS.MMM APP:<app> LL:<level> <szöveg>` — `<level>` ∈ {LOG, ERR, SW, SWE}.

### Kész

- [x] **7** — `get_all()` verseny: `pullSwLogs()` promise-alapú, awaitelt olvasás, 1500 ms timeout.
- [x] **9** — Időbélyeg `YYYY-MM-DD HH:MM:SS.MMM` (ISO-közeli).
- [x] **10** — Három ring-méret (MEMORY_MAX / MAX_DB_ENTRIES / __SW_LOG_MAX) kommentált + kereszthivatkozott.
- [x] **Formátum** — `APP:` és `LL:` előtagok.

### To remember

- [ ] **3** — `pushSwLine` dedup már TS-levágott kulcsot használ és a legfrissebbet tartja meg; okosabb LRU-t később.
- [ ] **6** — `readAllLines` sort stabilitásra épít; ha kell, secondary key-ként az `id`-t vesszük.

### After test — we have time for it

- [ ] **1** — `trimDbIfNeeded` fire-and-forget a `persistLine`-ban: `catch` / burst-védelem.
- [ ] **2** — SW ring nem perzisztens (iOS): SW-ből közvetlenül IDB, vagy IDB-bootolás.
- [ ] **4** — `MESSAGE_PREFIX` vs `SW_LOG_EVENT` drift: 5 s-os silent-SW warning.
- [ ] **5** — `version()` cold-start detektálás a DB-ből is.

JSON payload + szűrő UI törölve — marad a plain-text grep-barát formátum.


---

## Development Methodology

We follow **manual test-driven development** within an agile workflow:
1. Pick a small target from the TODO or a near-term goal
2. Implement the minimal change
3. Test manually in the browser (PWA on mobile and/or desktop)
4. If it works → commit. If not → iterate (fix → test again)
5. Update documentation **only when a step target is reached** — not at every micro-change

The architecture and critical logic of this project are the results of human-led AI-assisted engineering. This unique workflow ensures industrial-grade reliability and accelerated deployment.

> **Rule:** Don't update docs speculatively. Document what **is**, not what **might be**.

---

## Code Structure Vision

**Current (Phase 1):** 6 PWA apps + 4 shared Rust libraries, all as Cargo git dependencies. Relay management logic (publish, discover, types, helpers) consolidated into `zsozso-ledger::relay` module — both admin and gun-connect delegate to it.

**Next (Phase 2):** Extract reusable **Dioxus UI components** into shared git libraries. Dioxus follows the React component model — this makes components naturally shareable across apps. Examples: auth gate, settings panel, key management UI, log viewer, tab bar.

**Later (Phase 3):** Libraries will be published as open source alongside gun-connect. Add **Soroban smart contract** projects to the ecosystem. Two drafts already exist:
- `proof-of-zsozso-sc` — a vault SC for ZSOZSO token locking on Stellar Mainnet
- `zsozso-sc` — a template SC (first working draft, testnet ping/upgrade/admin)

---

## Near-Term Targets

- **LOG:** Every app sends logs to admin. Admin collects, filters, and displays logs from all apps. Concept still being explored — some ideas exist, needs iteration.
- **GUN relay sharing:** `gun-connect` app manages relay discovery + sharing between apps/users. Relay discovery (3-phase XDR scanning) and publishing (ManageData transactions) now live in `zsozso-ledger::relay`. Both admin and gun-connect use thin wrappers that adapt Dioxus Signals to the library's callback interface. The `Ledger` trait now also exposes `horizon_url()` and `network_passphrase()`.
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
cargo install cargo-binstall --locked
```
or install a precompiled binary: https://github.com/cargo-bins/cargo-binstall/issues/1634
((because the desktop:pici (bookworm) we use dioxus@0.7.5 . needs better clib :D I don't want to use sid, only for nvidia :D gpu ...))
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
