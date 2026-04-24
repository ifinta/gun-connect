#!/usr/bin/env bash
# sync-bridges.sh — Copy canonical JS/web bridge files from library repos
# into this app's root directory.
#
# Source of truth:
#   ../db/     → gun.js  gun_bridge.js  sea.js  sea_bridge.js
#   ../store/  → passkey_bridge.js  bundle.js  log_bridge.template.js  sw.template.js
#
# Usage:
#   ./sync-bridges.sh          — copy files
#   ./sync-bridges.sh --check  — verify files are in sync (exit 1 if not)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DB_DIR="${SCRIPT_DIR}/../db"
STORE_DIR="${SCRIPT_DIR}/../store"

CHECK=false
[ "${1:-}" = "--check" ] && CHECK=true

APP_NAME="gun-connect"
WINDOW_LOG_API="__gun_connect_log"
SW_LOG_EVENT="__GUN_CONNECT_SW_LOG"
LOG_FILENAME_PREFIX="gun-connect-log-"
MESSAGE_PREFIX="GUN_CONNECT"
BASE_PREFIX="/gun-connect/"

DIRTY=0

sync_file() {
  local src="$1"
  local name
  name="$(basename "$src")"
  local dest="${SCRIPT_DIR}/${name}"

  if [ ! -f "$src" ]; then
    echo "⚠ Source missing: $src"
    DIRTY=1
    return
  fi

  if $CHECK; then
    if ! diff -q "$src" "$dest" >/dev/null 2>&1; then
      echo "✗ ${name} differs from $(dirname "$src")/"
      DIRTY=1
    else
      echo "✓ ${name}"
    fi
  else
    cp "$src" "$dest"
    echo "✓ ${name} ← $(dirname "$src")/"
  fi
}

sync_render() {
  local template="$1"
  local dest="$2"
  local tmp
  tmp="$(mktemp)"

  if [ ! -f "$template" ]; then
    echo "⚠ Source missing: $template"
    DIRTY=1
    rm -f "$tmp"
    return
  fi

  sed \
    -e "s#__APP_NAME__#${APP_NAME}#g" \
    -e "s#__WINDOW_LOG_API__#${WINDOW_LOG_API}#g" \
    -e "s#__SW_LOG_EVENT__#${SW_LOG_EVENT}#g" \
    -e "s#__LOG_FILENAME_PREFIX__#${LOG_FILENAME_PREFIX}#g" \
    -e "s#__MESSAGE_PREFIX__#${MESSAGE_PREFIX}#g" \
    -e "s#__BASE_PREFIX__#${BASE_PREFIX}#g" \
    "$template" > "$tmp"

  if $CHECK; then
    if ! diff -q "$tmp" "$dest" >/dev/null 2>&1; then
      echo "✗ $(basename "$dest") differs from rendered template"
      DIRTY=1
    else
      echo "✓ $(basename "$dest")"
    fi
  else
    cp "$tmp" "$dest"
    echo "✓ $(basename "$dest") ← rendered $(basename "$template")"
  fi

  rm -f "$tmp"
}

# db library bridges
sync_file "${DB_DIR}/gun.js"
sync_file "${DB_DIR}/gun_bridge.js"
sync_file "${DB_DIR}/sea.js"
sync_file "${DB_DIR}/sea_bridge.js"

# store library bridge
sync_file "${STORE_DIR}/passkey_bridge.js"
sync_file "${STORE_DIR}/bundle.js"

# store-rendered app-specific LOG assets
sync_render "${STORE_DIR}/log_bridge.template.js" "${SCRIPT_DIR}/log_bridge.js"
sync_render "${STORE_DIR}/sw.template.js" "${SCRIPT_DIR}/sw.js"

if $CHECK; then
  if [ "$DIRTY" -ne 0 ]; then
    echo ""
    echo "Bridge files are out of sync. Run ./sync-bridges.sh to update."
    exit 1
  else
    echo ""
    echo "All bridge files are in sync."
  fi
fi

