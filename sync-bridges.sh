#!/usr/bin/env bash
# sync-bridges.sh — Copy canonical JS bridge files from library repos
# into this app's root directory.
#
# Source of truth:
#   ../db/     → gun.js  gun_bridge.js  sea.js  sea_bridge.js
#   ../store/  → passkey_bridge.js
#
# App-specific files (NOT synced):
#   log_bridge.js  qr_scanner_bridge.js  wascan.js  bundle.js  sw.js
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

# db library bridges
sync_file "${DB_DIR}/gun.js"
sync_file "${DB_DIR}/gun_bridge.js"
sync_file "${DB_DIR}/sea.js"
sync_file "${DB_DIR}/sea_bridge.js"

# store library bridge
sync_file "${STORE_DIR}/passkey_bridge.js"

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

