#!/bin/bash
set -e

# 0. Mükerrer / eski desktop dosyalarını temizle
rm -f "/usr/share/applications/Tuxeren Control Center.desktop" \
      "/usr/share/applications/txectrl-gui.desktop" \
      "/usr/share/applications/org.txectrl.tcc.desktop" \
      "/usr/share/applications/monster-control-center-gui.desktop" \
      "/usr/share/applications/com.monster.controlcenter.desktop" 2>/dev/null || true

echo "[*] Tuxeren Control Center (TxeCtrl): Donanım sürücüsü yapılandırılıyor..."

# 1. Kaynak kodları /usr/src dizinine yerleştir
SRC_DIR="/usr/src/tuxedo-drivers-monster-4.25.0"
BUNDLED_DIR="/usr/lib/tuxeren-control-center/_root_/usr/src/tuxedo-drivers-monster-4.25.0"
if [ ! -d "$BUNDLED_DIR" ]; then
    BUNDLED_DIR="/usr/lib/Tuxeren Control Center/_root_/usr/src/tuxedo-drivers-monster-4.25.0"
fi

if [ -d "$BUNDLED_DIR" ]; then
    mkdir -p "$SRC_DIR"
    cp -rf "$BUNDLED_DIR"/* "$SRC_DIR/"
fi

# 2. DKMS'e sürücüyü kaydet ve derle
if command -v dkms &>/dev/null; then
    CURRENT_KERNEL="$(uname -r)"
    if dkms status -m tuxedo-drivers-monster -v 4.25.0 -k "$CURRENT_KERNEL" 2>/dev/null | grep -q "installed" && modinfo tuxedo_keyboard >/dev/null 2>&1; then
        echo "[*] DKMS sürücüsü (tuxedo-drivers-monster/4.25.0, $CURRENT_KERNEL için) zaten kurulu, derleme adımı atlanıyor."
    else
        echo "[*] DKMS sürücüsü (tuxedo-drivers-monster/4.25.0) yapılandırılıyor ve derleniyor..."
        dkms remove -m tuxedo-drivers-monster -v 4.25.0 --all 2>/dev/null || true
        dkms add -m tuxedo-drivers-monster -v 4.25.0 2>/dev/null || true
        dkms build -m tuxedo-drivers-monster -v 4.25.0 || true
        dkms install -m tuxedo-drivers-monster -v 4.25.0 --force || true
    fi
fi

# 3. Otomatik yükleme konfigürasyonu
mkdir -p /etc/modules-load.d
cat << 'MCONF' > /etc/modules-load.d/tuxedo.conf
tuxedo_keyboard
uniwill_wmi
tuxedo_io
MCONF

# 4. Çekirdek modüllerini yükle / yenile
modprobe -r tuxedo_nb02_nvidia_power_ctrl clevo_wmi uniwill_wmi tuxedo_io tuxedo_keyboard 2>/dev/null || true
modprobe tuxedo_keyboard 2>/dev/null || true
modprobe uniwill_wmi 2>/dev/null || true
modprobe tuxedo_io 2>/dev/null || true

# 5. Udev kurallarını kur ve tetikle
cat << 'UDEV_EOF' > /etc/udev/rules.d/99-tuxeren-control-center.rules
SUBSYSTEM=="leds", KERNEL=="*kbd_backlight*", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/brightness /sys%p/multi_intensity"
SUBSYSTEM=="platform", DRIVER=="tuxedo_keyboard", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/fn_lock"
SUBSYSTEM=="platform-profile", ACTION=="add", RUN+="/bin/chmod 666 /sys%p/profile /sys/firmware/acpi/platform_profile"
KERNEL=="tuxedo_io", MODE="0666"
UDEV_EOF

udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --subsystem-match=leds 2>/dev/null || true
udevadm trigger --subsystem-match=platform-profile 2>/dev/null || true
udevadm trigger --name-match=tuxedo_io 2>/dev/null || true

# Mevcut aygıt izinlerini anında uygula (+X klasör geçişini koruyarak)
chmod -R a+rX /sys/devices/platform/tuxedo_keyboard 2>/dev/null || true
chmod -R a+rw /sys/devices/platform/tuxedo_keyboard/leds 2>/dev/null || true
chmod 0666 /sys/devices/platform/tuxedo_keyboard/fn_lock 2>/dev/null || true
chmod 0666 /dev/tuxedo_io 2>/dev/null || true
chmod 0666 /sys/firmware/acpi/platform_profile* 2>/dev/null || true
chmod 0666 /sys/devices/platform/tuxedo_keyboard/platform-profile/platform-profile-*/profile 2>/dev/null || true
chmod 0666 /sys/class/hwmon/hwmon*/fan* 2>/dev/null || true

# 6. swcatalog ve app-info önbellek entegrasyonu
mkdir -p /usr/share/swcatalog/xml /usr/share/swcatalog/icons/tuxeren-control-center/64x64 /usr/share/swcatalog/icons/tuxeren-control-center/128x128 \
         /usr/share/app-info/xmls /usr/share/app-info/icons/tuxeren-control-center/64x64 /usr/share/app-info/icons/tuxeren-control-center/128x128 2>/dev/null || true

if [ -f "/usr/share/swcatalog/xml/tuxeren-control-center.xml" ]; then
    cp -f "/usr/share/swcatalog/xml/tuxeren-control-center.xml" "/usr/share/app-info/xmls/tuxeren-control-center.xml" 2>/dev/null || true
fi

if [ -f "/usr/share/icons/hicolor/64x64/apps/tuxeren-control-center.png" ]; then
    cp -f "/usr/share/icons/hicolor/64x64/apps/tuxeren-control-center.png" "/usr/share/swcatalog/icons/tuxeren-control-center/64x64/tuxeren-control-center.png" 2>/dev/null || true
    cp -f "/usr/share/icons/hicolor/64x64/apps/tuxeren-control-center.png" "/usr/share/app-info/icons/tuxeren-control-center/64x64/tuxeren-control-center.png" 2>/dev/null || true
fi

if [ -f "/usr/share/icons/hicolor/128x128/apps/tuxeren-control-center.png" ]; then
    cp -f "/usr/share/icons/hicolor/128x128/apps/tuxeren-control-center.png" "/usr/share/swcatalog/icons/tuxeren-control-center/128x128/tuxeren-control-center.png" 2>/dev/null || true
    cp -f "/usr/share/icons/hicolor/128x128/apps/tuxeren-control-center.png" "/usr/share/app-info/icons/tuxeren-control-center/128x128/tuxeren-control-center.png" 2>/dev/null || true
fi

# 7. Masaüstü, simge ve AppStream veritabanlarını güncelle
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
fi
if command -v appstreamcli &>/dev/null; then
    appstreamcli refresh-cache --force 2>/dev/null || true
fi

echo "[+] Sürücü ve izinler başarıyla aktif edildi."
