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
cp -f icons/128x128.png "$ICON_DIR/monster-control-center-gui.png"

# 4. Setup Desktop Launcher
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
echo "[*] Registering desktop application..."
cat << 'EOF' > "$DESKTOP_DIR/monster-control-center-gui.desktop"
[Desktop Entry]
Name=Monster Control Center
Comment=Keyboard RGB & EC Control for Monster / Tongfang Laptops
Exec=monster-control-center-gui
Icon=monster-control-center
Terminal=false
Type=Application
Categories=System;HardwareSettings;
Keywords=monster;control;center;rgb;keyboard;backlight;
StartupWMClass=monster-control-center-gui
EOF

cp -f "$DESKTOP_DIR/monster-control-center-gui.desktop" "$DESKTOP_DIR/com.monster.controlcenter.desktop"
chmod +x "$DESKTOP_DIR/monster-control-center-gui.desktop" "$DESKTOP_DIR/com.monster.controlcenter.desktop"

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
echo "[*] Configuring udev rules for LEDs and hardware control..."
UDEV_RULE_FILE="/etc/udev/rules.d/99-monster-control-center.rules"
UDEV_CONTENT='SUBSYSTEM=="leds", KERNEL=="*kbd_backlight*", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/brightness /sys%p/multi_intensity"
SUBSYSTEM=="platform", DRIVER=="tuxedo_keyboard", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/fn_lock"
SUBSYSTEM=="platform-profile", ACTION=="add", RUN+="/bin/chmod 666 /sys%p/profile /sys/firmware/acpi/platform_profile"
KERNEL=="tuxedo_io", MODE="0666"'

if [ ! -f "$UDEV_RULE_FILE" ]; then
    SETUP_CMD="
        mkdir -p /etc/modules-load.d
        printf 'tuxedo_keyboard\nuniwill_wmi\ntuxedo_io\n' > /etc/modules-load.d/tuxedo.conf
        modprobe uniwill_wmi 2>/dev/null || true
        echo '$UDEV_CONTENT' > $UDEV_RULE_FILE
        udevadm control --reload-rules 2>/dev/null || true
        udevadm trigger --subsystem-match=leds 2>/dev/null || true
        udevadm trigger --subsystem-match=platform-profile 2>/dev/null || true
        udevadm trigger --name-match=tuxedo_io 2>/dev/null || true
        chmod -R a+rX /sys/devices/platform/tuxedo_keyboard 2>/dev/null || true
        chmod -R a+rw /sys/devices/platform/tuxedo_keyboard/leds 2>/dev/null || true
        chmod 0666 /sys/devices/platform/tuxedo_keyboard/fn_lock 2>/dev/null || true
        chmod 0666 /dev/tuxedo_io 2>/dev/null || true
        chmod 0666 /sys/firmware/acpi/platform_profile* 2>/dev/null || true
        chmod 0666 /sys/devices/platform/tuxedo_keyboard/platform-profile/platform-profile-*/profile 2>/dev/null || true
        chmod 0666 /sys/class/hwmon/hwmon*/fan* 2>/dev/null || true
    "
    if command -v pkexec &>/dev/null; then
        echo "[*] Installing udev rule via pkexec..."
        pkexec bash -c "$SETUP_CMD" || echo "[-] Udev setup skipped. You can configure rules manually."
    fi
fi

echo "=== Installation Complete ==="
echo "Launch from application menu or run:"
echo "  monster-control-center-gui"
