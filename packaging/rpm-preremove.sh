#!/bin/bash
# DKMS kaydını sadece özellikle istendiğinde kaldır (REMOVE_DKMS=1)
if [ "${REMOVE_DKMS:-0}" = "1" ] && command -v dkms &>/dev/null; then
    echo "[*] Tuxeren Control Center (TxeCtrl): DKMS sürücüsü siliniyor..."
    modprobe -r tuxedo_keyboard uniwill_wmi tuxedo_io 2>/dev/null || true
    dkms remove -m tuxedo-drivers-monster -v 4.25.0 --all 2>/dev/null || true
    rm -f /etc/udev/rules.d/99-tuxeren-control-center.rules /etc/udev/rules.d/99-monster-control-center.rules 2>/dev/null || true
    rm -f /etc/modules-load.d/tuxedo.conf 2>/dev/null || true
    udevadm control --reload-rules 2>/dev/null || true
fi
