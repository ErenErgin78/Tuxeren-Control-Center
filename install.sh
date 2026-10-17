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

# 6. Setup Systemd User Service for low-power background daemon
echo "[*] Setting up systemd user service for background daemon..."
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"
cat << 'EOF' > "$SERVICE_DIR/monster-control-center.service"
[Unit]
Description=Monster Control Center Background Daemon
After=default.target

[Service]
Type=simple
ExecStart=%h/.local/bin/monster-control-center daemon
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user enable --now monster-control-center.service 2>/dev/null || true

# 7. Configure udev rules for tuxedo keyboard LEDs and hardware control
echo "[*] Configuring udev rules for tuxedo_keyboard LEDs..."
UDEV_RULE_FILE="/etc/udev/rules.d/99-monster-control-center.rules"
UDEV_CONTENT='SUBSYSTEM=="leds", ACTION=="add", RUN+="/bin/chmod -R 0666 /sys%p"
KERNEL=="tuxedo_io", MODE="0666"'

if [ ! -f "$UDEV_RULE_FILE" ]; then
    SETUP_CMD="
        echo '$UDEV_CONTENT' > $UDEV_RULE_FILE
        udevadm control --reload-rules 2>/dev/null || true
        udevadm trigger --subsystem-match=leds 2>/dev/null || true
    "
    if command -v pkexec &>/dev/null; then
        echo "[*] Installing udev rule via pkexec..."
        pkexec bash -c "$SETUP_CMD" || echo "[-] Udev setup skipped. You can configure rules manually."
    fi
fi

echo "=== Installation Complete ==="
echo "Launch from application menu or run:"
echo "  monster-control-center-gui"
