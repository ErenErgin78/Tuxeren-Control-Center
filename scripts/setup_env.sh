#!/usr/bin/env bash
# Monster Control Center Development & Environment Setup Script
# Verifies Rust toolchain, Node.js dependencies, and system build libraries
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

echo "========================================================"
echo "    Monster Control Center Environment Setup"
echo "========================================================"

detect_and_install_deps() {
    local missing=()
    if command -v dnf &>/dev/null; then
        local deps=(webkit2gtk4.1-devel openssl-devel alsa-lib-devel pulseaudio-utils ImageMagick rpm-build)
        for pkg in "${deps[@]}"; do
            if ! rpm -q "$pkg" &>/dev/null; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            read -p "[?] Would you like to install missing dependencies with sudo dnf? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                sudo dnf install -y "${missing[@]}"
            fi
        else
            echo "[✓] All system dependencies are already installed (dnf)."
        fi
    elif command -v apt-get &>/dev/null; then
        local deps=(libwebkit2gtk-4.1-dev libssl-dev libasound2-dev pulseaudio-utils imagemagick rpm)
        for pkg in "${deps[@]}"; do
            if ! dpkg -s "$pkg" 2>/dev/null | grep -q "Status: install ok installed"; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            read -p "[?] Would you like to install missing dependencies with sudo apt? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                sudo apt-get update && sudo apt-get install -y "${missing[@]}"
            fi
        else
            echo "[✓] All system dependencies are already installed (apt)."
        fi
    elif command -v pacman &>/dev/null; then
        local deps=(webkit2gtk-4.1 openssl alsa-lib libpulse imagemagick rpm-tools)
        for pkg in "${deps[@]}"; do
            if ! pacman -Q "$pkg" &>/dev/null; then
                missing+=("$pkg")
            fi
        done
        if [ ${#missing[@]} -gt 0 ]; then
            echo "[!] Missing system dependencies detected: ${missing[*]}"
            read -p "[?] Would you like to install missing dependencies with sudo pacman? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                sudo pacman -S --needed "${missing[@]}"
            fi
        else
            echo "[✓] All system dependencies are already installed (pacman)."
        fi
    fi
}

detect_and_install_deps

echo "[+] Checking Rust toolchain..."
if ! command -v cargo &>/dev/null || ! command -v rustc &>/dev/null; then
    echo "[-] Rust is not installed or not in PATH."
    echo "    Please install Rust via rustup: https://rustup.rs"
    exit 1
fi
echo "    Rust: $(rustc --version)"
echo "    Cargo: $(cargo --version)"

echo "[+] Checking Node.js & npm..."
if command -v node &>/dev/null && command -v npm &>/dev/null; then
    echo "    Node: $(node --version)"
    echo "    npm: v$(npm --version)"
else
    echo "[!] Node.js or npm not found. It is recommended for Tauri CLI bundling."
fi

if [ -f "ui/package.json" ]; then
    echo "[+] Installing UI dependencies in ui/..."
    (cd ui && npm install)
fi

echo "[+] Checking tuxedo-drivers kernel interface..."
if [ -d "/sys/devices/platform/tuxedo_keyboard" ] || [ -c "/dev/tuxedo_io" ]; then
    echo "[✓] tuxedo-drivers kernel interface is active."
else
    echo "[!] tuxedo-drivers is not currently active (load via: sudo modprobe tuxedo_keyboard uniwill_wmi)."
fi

echo "[+] Checking Rust backend build..."
cargo check

echo "========================================================"
echo "  Setup Complete! Ready to launch Monster Control Center."
echo "  Run 'make dev' to start development mode."
echo "========================================================"
