#!/usr/bin/env bash
# build.sh — build zsozso and create deployment bundle
#
# Pipeline:
#   1. Run `dx build --release --platform web --features web`
#   2. Stage output into dist/app/ (preserves manually-added files like gun.js)
#   3. Stamp a fresh APP_VERSION into dist/app/sw.js and dist/app/index.html
#   4. Run bundle.js to create the deployment in deploy/<prefix>/
#
# Usage:
#   ./build.sh          — build + bundle for live server (https://zsozso.info/app)
#   ./build.sh -ghpages — build + bundle for GitHub Pages (/zsozso-dioxus/)
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

APP_NAME="zsozso"
# Deployment prefix: /app/ for live server, /zsozso-dioxus/ for GitHub Pages
if $GHPAGES; then
  PREFIX="zsozso-dioxus"
  APP_VERSION="${APP_NAME}-gh-${BUILD_TS}-${GIT_HASH}"
else
  PREFIX="app"
  APP_VERSION="${APP_NAME}-app-${BUILD_TS}-${GIT_HASH}"
fi

echo "APP_VERSION → ${APP_VERSION}"
$DRY && exit 0

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
   passkey_bridge.js qr_scanner_bridge.js wascan_bg.wasm wascan.js \
   "${DIST_DIR}/"

# ── 4. Stamp CACHE_NAME ──────────────────────────────────────────────────────
sed -i "s|.*var APP_VERSION =.*|var APP_VERSION = '${APP_VERSION}';|" "${DIST_DIR}/sw.js"
echo "Stamped ${DIST_DIR}/sw.js"

sed -i "s|window.__APP_VERSION = '.*'|window.__APP_VERSION = '${APP_VERSION}'|" "${DIST_DIR}/index.html"
echo "Stamped ${DIST_DIR}/index.html"

# ── 5. Bundle for deployment ─────────────────────────────────────────────────
# For GitHub Pages builds, patch manifest.json paths and index.html to match the /zsozso-dioxus/ prefix
if $GHPAGES; then
  sed -i 's|.*var __BASE_PREFIX =.*|var __BASE_PREFIX = '/zsozso-dioxus/';|g' "${DIST_DIR}/sw.js"
  sed -i 's|.*let PREFIX =.*|        let PREFIX = "zsozso-dioxus";|g' "${DIST_DIR}/index.html"
  sed -i 's|.*"id":.*|    "id": "/zsozso-dioxus/",|g' "${DIST_DIR}/manifest.json"
  sed -i 's|.*"start-url":.*|    "start_url": "/zsozso-dioxus/",|g' "${DIST_DIR}/manifest.json"
  sed -i 's|.*"scope":.*|    "scupe": "/zsozso-dioxus/",|g' "${DIST_DIR}/manifest.json"
  echo "Patched manifest.json and index.html for -ghpages (GitHub Pages) deployment (/zsozso-dioxus/)"
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

