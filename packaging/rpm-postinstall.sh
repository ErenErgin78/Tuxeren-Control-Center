#!/bin/bash
set -e

echo "[*] Monster Control Center: Donanım sürücüsü yapılandırılıyor..."

# 1. Kaynak kodları /usr/src dizinine yerleştir
SRC_DIR="/usr/src/tuxedo-drivers-monster-4.25.0"
BUNDLED_DIR="/usr/lib/Monster Control Center/_root_/usr/src/tuxedo-drivers-monster-4.25.0"

if [ -d "$BUNDLED_DIR" ] && [ ! -d "$SRC_DIR" ]; then
    mkdir -p "$SRC_DIR"
    cp -r "$BUNDLED_DIR"/* "$SRC_DIR/"
fi

# 2. DKMS'e sürücüyü kaydet ve derle
if command -v dkms &>/dev/null; then
    dkms add -m tuxedo-drivers-monster -v 4.25.0 || true
    dkms build -m tuxedo-drivers-monster -v 4.25.0 || true
    dkms install -m tuxedo-drivers-monster -v 4.25.0 --force || true
fi

# 2. Çekirdek modüllerini yükle
modprobe tuxedo_keyboard 2>/dev/null || true
modprobe uniwill_wmi 2>/dev/null || true
modprobe tuxedo_io 2>/dev/null || true

# 3. Udev kurallarını kur ve tetikle
cat << 'UDEV_EOF' > /etc/udev/rules.d/99-monster-control-center.rules
SUBSYSTEM=="leds", KERNEL=="*kbd_backlight*", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/brightness /sys%p/multi_intensity"
SUBSYSTEM=="platform", DRIVER=="tuxedo_keyboard", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/fn_lock"
KERNEL=="tuxedo_io", MODE="0666"
UDEV_EOF

udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --subsystem-match=leds 2>/dev/null || true
udevadm trigger --name-match=tuxedo_io 2>/dev/null || true

# Mevcut aygıt izinlerini anında uygula (+X klasör geçişini koruyarak)
chmod -R a+rX /sys/devices/platform/tuxedo_keyboard 2>/dev/null || true
chmod -R a+rw /sys/devices/platform/tuxedo_keyboard/leds 2>/dev/null || true
chmod 0666 /sys/devices/platform/tuxedo_keyboard/fn_lock 2>/dev/null || true
chmod 0666 /dev/tuxedo_io 2>/dev/null || true

echo "[+] Sürücü ve izinler başarıyla aktif edildi."
