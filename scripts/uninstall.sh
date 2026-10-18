#!/usr/bin/env bash
set -euo pipefail

echo "[*] Removing Tuxeren Control Center (TxeCtrl)..."

# 1. Stop and disable systemd user daemon
systemctl --user stop txectrl.service monster-control-center.service 2>/dev/null || true
systemctl --user disable txectrl.service monster-control-center.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/txectrl.service" \
      "$HOME/.config/systemd/user/monster-control-center.service"
systemctl --user daemon-reload 2>/dev/null || true

# 2. Terminate running instances
killall -q txectrl txectrl-gui monster-control-center monster-control-center-gui 2>/dev/null || true
pkill -f '(/|^)(txectrl|monster-control-center)(-gui)?($| )' 2>/dev/null || true
rm -f "${XDG_RUNTIME_DIR:-/tmp}/txectrl.sock" /dev/shm/txectrl.sock \
      "${XDG_RUNTIME_DIR:-/tmp}/monster_control.sock" /dev/shm/monster_control.sock 2>/dev/null || true

# 3. Remove user binaries, desktop entry, icon, and user config
rm -f "$HOME/.local/bin/txectrl" \
      "$HOME/.local/bin/txectrl-gui" \
      "$HOME/.local/bin/monster-control-center" \
      "$HOME/.local/bin/monster-control-center-gui" \
      "$HOME/.local/share/applications/txectrl-gui.desktop" \
      "$HOME/.local/share/applications/org.txectrl.app.desktop" \
      "$HOME/.local/share/applications/com.monster.controlcenter.desktop" \
      "$HOME/.local/share/icons/hicolor/128x128/apps/txectrl.png" \
      "$HOME/.local/share/icons/hicolor/128x128/apps/txectrl-gui.png" \
      "$HOME/.local/share/icons/hicolor/128x128/apps/monster-control-center.png"
rm -rf "$HOME/.config/txectrl" "$HOME/.config/monster-control-center"

# 4. Remove package manager installations if present
if command -v rpm &>/dev/null; then
    rpm -q txectrl &>/dev/null && sudo rpm -e txectrl || true
    rpm -q monster-control-center &>/dev/null && sudo rpm -e monster-control-center || true
elif command -v dpkg &>/dev/null; then
    dpkg -s txectrl &>/dev/null && sudo dpkg -r txectrl || true
    dpkg -s monster-control-center &>/dev/null && sudo dpkg -r monster-control-center || true
fi

# 5. Refresh desktop and icon databases
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "[✓] Tuxeren Control Center successfully uninstalled."
