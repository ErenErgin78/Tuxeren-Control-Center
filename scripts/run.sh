#!/usr/bin/env bash
# Monster Control Center Launcher Script (Rust / Tauri)
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
cd "$DIR"

if ! command -v cargo &>/dev/null; then
    echo "[-] Error: 'cargo' not found in PATH. Please install Rust or run ./scripts/setup_env.sh"
    exit 1
fi

echo "[+] Launching Monster Control Center (Tauri Dev Mode)..."
if command -v cargo-tauri &>/dev/null; then
    exec cargo tauri dev "$@"
else
    exec npx --yes @tauri-apps/cli dev "$@"
fi
