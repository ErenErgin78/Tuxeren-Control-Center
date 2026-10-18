#!/usr/bin/env bash
set -euo pipefail

echo "[*] Removing Monster Control Center..."

# 1. Stop and disable systemd user daemon
systemctl --user stop monster-control-center.service 2>/dev/null || true
systemctl --user disable monster-control-center.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/monster-control-center.service"
systemctl --user daemon-reload 2>/dev/null || true

# 2. Terminate running instances
killall -q monster-control-center monster-control-center-gui 2>/dev/null || true
pkill -f '(/|^)monster-control-center(-gui)?($| )' 2>/dev/null || true
rm -f "${XDG_RUNTIME_DIR:-/tmp}/monster_control.sock" /dev/shm/monster_control.sock 2>/dev/null || true

# 3. Remove user binaries, desktop entry, icon, and user config
rm -f "$HOME/.local/bin/monster-control-center" \
      "$HOME/.local/bin/monster-control-center-gui" \
      "$HOME/.local/share/applications/com.monster.controlcenter.desktop" \
      "$HOME/.local/share/icons/hicolor/128x128/apps/monster-control-center.png"
rm -rf "$HOME/.config/monster-control-center"

# 4. Remove package manager installations if present
if command -v rpm &>/dev/null && rpm -q monster-control-center &>/dev/null; then
    echo "[*] Removing RPM package..."
    sudo rpm -e monster-control-center || true
elif command -v dpkg &>/dev/null && dpkg -s monster-control-center &>/dev/null; then
    echo "[*] Removing DEB package..."
    sudo dpkg -r monster-control-center || true
fi

# 5. Refresh desktop and icon databases
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "[✓] Monster Control Center successfully uninstalled."
