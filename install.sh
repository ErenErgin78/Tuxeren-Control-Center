#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Monster Control Center Installer ==="

# 1. Compile release binaries if missing
if [ ! -f "target/release/monster-control-center-gui" ]; then
    echo "[*] Building release binaries..."
    cargo build --release
fi

# 2. Setup user binary directory (~/.local/bin)
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

echo "[*] Installing binaries to $BIN_DIR..."
cp -f target/release/monster-control-center "$BIN_DIR/monster-control-center"
cp -f target/release/monster-control-center-gui "$BIN_DIR/monster-control-center-gui"
chmod +x "$BIN_DIR/monster-control-center" "$BIN_DIR/monster-control-center-gui"

# 3. Setup application icon
ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
mkdir -p "$ICON_DIR"
echo "[*] Installing application icon..."
cp -f icons/128x128.png "$ICON_DIR/monster-control-center.png"

# 4. Setup Desktop Launcher
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
echo "[*] Registering desktop application..."
cat << 'EOF' > "$DESKTOP_DIR/com.monster.controlcenter.desktop"
[Desktop Entry]
Name=Monster Control Center
Comment=Keyboard RGB & EC Control for Monster / Tongfang Laptops
Exec=monster-control-center-gui
Icon=monster-control-center
Terminal=false
Type=Application
Categories=System;HardwareSettings;
Keywords=monster;control;center;rgb;keyboard;backlight;
StartupWMClass=Monster Control Center
EOF

chmod +x "$DESKTOP_DIR/com.monster.controlcenter.desktop"

# 5. Update desktop caches
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

# 6. Ensure ACPI call module and permissions
echo "[*] Checking /proc/acpi/call and acpi_call kernel module..."
if [ -w "/proc/acpi/call" ]; then
    echo "[+] /proc/acpi/call is loaded and writable."
else
    echo "[!] /proc/acpi/call is missing or not writable. Setting up kernel module and permissions..."
    MODULE_SRC="/home/eren/Depo/ReverseEngineering/py_deneme/acpi_call/acpi_call.ko"
    SETUP_CMD="
        modprobe acpi_call 2>/dev/null || insmod '$MODULE_SRC' 2>/dev/null || true
        mkdir -p /lib/modules/\$(uname -r)/extra 2>/dev/null || true
        if [ -f '$MODULE_SRC' ]; then
            cp -f '$MODULE_SRC' /lib/modules/\$(uname -r)/extra/ 2>/dev/null || true
            depmod -a 2>/dev/null || true
        fi
        echo acpi_call > /etc/modules-load.d/acpi_call.conf 2>/dev/null || true
        mkdir -p /etc/tmpfiles.d 2>/dev/null || true
        echo 'z /proc/acpi/call 0666 root root -' > /etc/tmpfiles.d/acpi_call.conf 2>/dev/null || true
        chmod 666 /proc/acpi/call 2>/dev/null || true
    "
    if command -v pkexec &>/dev/null; then
        echo "[*] Requesting permission via pkexec..."
        pkexec bash -c "$SETUP_CMD" || echo "[-] Permission grant skipped. You can grant access inside the app."
    fi
fi

echo "=== Installation Complete ==="
echo "Launch from application menu or run:"
echo "  monster-control-center-gui"
