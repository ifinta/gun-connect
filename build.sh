#!/usr/bin/env bash
# build.sh — build gun-connect and create deployment bundle
#
# Pipeline:
#   1. Run `dx build --release --platform web --features web`
#   2. Stage output into dist/gun-connect/ (preserves manually-added files like gun.js)
#   3. Stamp a fresh APP_VERSION into dist/gun-connect/sw.js and dist/gun-connect/index.html
#   4. Run bundle.js to create the deployment in deploy/<prefix>/
#
# Usage:
#   ./build.sh          — build + bundle for live server (https://zsozso.info/gun-connect)
#   ./build.sh -ghpages — build + bundle for GitHub Pages (/gun-connect/)
#   ./build.sh --dry    — print the new APP_VERSION without building

set -euo pipefail

DRY=false
GHPAGES=false
for arg in "$@"; do
  case "$arg" in
    --dry) DRY=true ;;
    -ghpages) GHPAGES=true ;;
  esac
done

# ── 1. Generate CACHE_NAME ────────────────────────────────────────────────────
BUILD_TS="$(date +%Y%m%d-%H%M)"
GIT_HASH="$(git rev-parse --short=8 HEAD)"

APP_NAME="gun-connect"
# Deployment prefix: /gun-connect/ for live server, /gun-connect-dioxus/ for GitHub Pages
if $GHPAGES; then
  PREFIX="gun-connect"
  APP_VERSION="${APP_NAME}-gh-${BUILD_TS}-${GIT_HASH}"
else
  PREFIX="gun-connect"
  APP_VERSION="${APP_NAME}-app-${BUILD_TS}-${GIT_HASH}"
fi

echo "APP_VERSION → ${APP_VERSION}"
$DRY && exit 0

# ── Sync JS bridge files from library repos ───────────────────────────────────
if [ -z "${CI:-}" ]; then
  ./sync-bridges.sh
else
  echo "CI detected — skipping sync-bridges.sh (bridge files already in repo)"
fi

# For different builds, patch Dioxus.toml paths to match the right prefix
sed -i "s|.*base_path =.*|base_path = \"${PREFIX}\"|g" Dioxus.toml
echo "Patched Dioxus.toml for different (-ghpages for Github Pages) deployments"

# ── 2. Build ──────────────────────────────────────────────────────────────────
echo "Running: dx build --release --platform web --features web"
dx build --release --platform web --features web

# ── 3. Stage to dist/app/ ────────────────────────────────────────────────────
DX_OUT="target/dx/${APP_NAME}/release/web/public"
DIST_DIR="dist/${PREFIX}"

echo "Staging ${DX_OUT}/ → ${DIST_DIR}/"
rm -rf "${DIST_DIR}/assets"
mkdir -p "${DIST_DIR}/assets"
cp -r "${DX_OUT}/." "${DIST_DIR}/"
rm -rf "${DX_OUT}"

# Copy root static assets into dist (in CI there is no persistent dist/)
cp sw.js manifest.json favicon.ico icon-192.png icon-512.png \
   gun.js gun_bridge.js sea.js sea_bridge.js log_bridge.js \
   passkey_bridge.js \
   "${DIST_DIR}/"

# ── 4. Stamp CACHE_NAME ──────────────────────────────────────────────────────
sed -i "s|.*var APP_VERSION =.*|var APP_VERSION = '${APP_VERSION}';|" "${DIST_DIR}/sw.js"
echo "Stamped ${DIST_DIR}/sw.js"

sed -i "s|window.__APP_VERSION = '.*'|window.__APP_VERSION = '${APP_VERSION}'|" "${DIST_DIR}/index.html"
echo "Stamped ${DIST_DIR}/index.html"

# ── 5. Bundle for deployment ─────────────────────────────────────────────────
# For GitHub Pages builds, patch manifest.json paths and index.html to match the /gun-connect-dioxus/ prefix
if $GHPAGES; then
  sed -i 's|.*var __BASE_PREFIX =.*|var __BASE_PREFIX = '"'"'/gun-connect/'"'"';|g' "${DIST_DIR}/sw.js"
  sed -i 's|.*let PREFIX =.*|        let PREFIX = "gun-connect";|g' "${DIST_DIR}/index.html"
  sed -i 's|.*"id":.*|    "id": "/gun-connect/",|g' "${DIST_DIR}/manifest.json"
  sed -i 's|.*"start_url":.*|    "start_url": "/gun-connect/",|g' "${DIST_DIR}/manifest.json"
  sed -i 's|.*"scope":.*|    "scope": "/gun-connect/",|g' "${DIST_DIR}/manifest.json"
  echo "Patched manifest.json and index.html for -ghpages (GitHub Pages) deployment (/gun-connect/)"
fi

echo "Running: node bundle.js ${DIST_DIR} deploy ${PREFIX}"
node bundle.js "${DIST_DIR}" deploy "${PREFIX}"

echo ""
echo "Copying icons and manifest file to deploy folder"
cp manifest.json favicon.ico icon-192.png icon-512.png "deploy/${PREFIX}/"

echo ""
echo "✓ Build complete — APP_VERSION: ${APP_VERSION}"
echo "  Deploy from: deploy/${PREFIX}/"
echo "  Test:        npx serve deploy/ -l 8080  →  http://localhost:8080/${PREFIX}/"

