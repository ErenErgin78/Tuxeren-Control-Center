#!/bin/bash
echo "[*] Monster Control Center: Sürücü temizleniyor..."

# Modülleri kaldır
modprobe -r tuxedo_keyboard 2>/dev/null || true
modprobe -r uniwill_wmi 2>/dev/null || true
modprobe -r tuxedo_io 2>/dev/null || true

# DKMS kaydını sil
if command -v dkms &>/dev/null; then
    dkms remove -m tuxedo-drivers-monster -v 4.25.0 --all 2>/dev/null || true
fi

# Udev kuralını sil
rm -f /etc/udev/rules.d/99-monster-control-center.rules 2>/dev/null || true
udevadm control --reload-rules 2>/dev/null || true
