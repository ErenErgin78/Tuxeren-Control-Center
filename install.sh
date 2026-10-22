#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Tuxeren Control Center (TxeCtrl) Installer ==="

# 1. Compile release binaries if missing
if [ ! -f "target/release/txectrl-gui" ]; then
    echo "[*] Building release binaries..."
    cargo build --release
fi

# 2. Setup user binary directory (~/.local/bin)
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

echo "[*] Installing binaries to $BIN_DIR..."
cp -f target/release/txectrl "$BIN_DIR/txectrl"
cp -f target/release/txectrl-gui "$BIN_DIR/txectrl-gui"
chmod +x "$BIN_DIR/txectrl" "$BIN_DIR/txectrl-gui"

# 3. Setup application icons
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps" \
         "$HOME/.local/share/icons/hicolor/128x128/apps" \
         "$HOME/.local/share/icons/hicolor/64x64/apps" \
         "$HOME/.local/share/icons/hicolor/scalable/apps" \
         "$HOME/.local/share/pixmaps"
echo "[*] Installing application icons..."
cp -f icons/256x256.png "$HOME/.local/share/icons/hicolor/256x256/apps/tuxeren-control-center.png"
cp -f icons/256x256.png "$HOME/.local/share/icons/hicolor/256x256/apps/txectrl-gui.png"
cp -f icons/256x256.png "$HOME/.local/share/pixmaps/tuxeren-control-center.png"
cp -f icons/256x256.png "$HOME/.local/share/pixmaps/txectrl-gui.png"
cp -f icons/128x128.png "$HOME/.local/share/icons/hicolor/128x128/apps/tuxeren-control-center.png"
cp -f icons/128x128.png "$HOME/.local/share/icons/hicolor/128x128/apps/txectrl-gui.png"
cp -f icons/64x64.png "$HOME/.local/share/icons/hicolor/64x64/apps/tuxeren-control-center.png"
cp -f icons/64x64.png "$HOME/.local/share/icons/hicolor/64x64/apps/txectrl-gui.png"
cp -f icons/tuxeren-control-center.svg "$HOME/.local/share/icons/hicolor/scalable/apps/tuxeren-control-center.svg"
cp -f icons/128x128.png "$HOME/.local/share/icons/hicolor/128x128/apps/txectrl.png"

# 4. Setup Desktop Launcher and AppStream MetaInfo
DESKTOP_DIR="$HOME/.local/share/applications"
METAINFO_DIR="$HOME/.local/share/metainfo"
SWCATALOG_DIR="$HOME/.local/share/swcatalog/xml"
mkdir -p "$DESKTOP_DIR" "$METAINFO_DIR" "$SWCATALOG_DIR"

echo "[*] Registering desktop application and AppStream metadata..."
cp -f packaging/tuxeren-control-center.metainfo.xml "$METAINFO_DIR/tuxeren-control-center.metainfo.xml"
cp -f packaging/tuxeren-control-center.swcatalog.xml "$SWCATALOG_DIR/tuxeren-control-center.xml"

cat << 'EOF' > "$DESKTOP_DIR/tuxeren-control-center.desktop"
[Desktop Entry]
Name=Tuxeren Control Center
Comment=Hardware, Keyboard RGB and EC Control for Uniwill / Tongfang Laptops
Comment[tr]=Uniwill / Tongfang Laptoplar İçin Donanım, Klavye RGB ve EC Kontrol Merkezi
Exec=txectrl-gui
Icon=tuxeren-control-center
Terminal=false
Type=Application
Categories=Settings;HardwareSettings;
Keywords=txectrl;tuxeren;monster;control;center;rgb;keyboard;backlight;uniwill;tongfang;
StartupWMClass=txectrl-gui
EOF

cp -f "$DESKTOP_DIR/tuxeren-control-center.desktop" "$DESKTOP_DIR/txectrl-gui.desktop"
cp -f "$DESKTOP_DIR/tuxeren-control-center.desktop" "$DESKTOP_DIR/org.txectrl.tcc.desktop"
chmod +x "$DESKTOP_DIR/tuxeren-control-center.desktop" "$DESKTOP_DIR/txectrl-gui.desktop" "$DESKTOP_DIR/org.txectrl.tcc.desktop"

# 5. Update desktop caches
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
if command -v appstreamcli &>/dev/null; then
    appstreamcli refresh-cache --force 2>/dev/null || true
fi

# 6. Setup Systemd User Service for low-power background daemon
echo "[*] Setting up systemd user service for background daemon..."
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"
cat << 'EOF' > "$SERVICE_DIR/txectrl.service"
[Unit]
Description=Tuxeren Control Center Background Daemon
After=default.target

[Service]
Type=simple
ExecStart=%h/.local/bin/txectrl daemon
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user enable --now txectrl.service 2>/dev/null || true

# 7. Configure udev rules for tuxedo keyboard LEDs and hardware control
echo "[*] Configuring udev rules for LEDs and hardware control..."
UDEV_RULE_FILE="/etc/udev/rules.d/99-tuxeren-control-center.rules"
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
echo "  txectrl-gui"
