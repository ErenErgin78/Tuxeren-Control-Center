#!/usr/bin/env bash
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
DRIVER_SRC="$PROJECT_ROOT/driver/tuxedo-drivers-monster-4.25.0"
DKMS_DEST="/usr/src/tuxedo-drivers-monster-4.25.0"

if [ "$EUID" -ne 0 ]; then
    echo "[*] Elevating privileges via pkexec/sudo..."
    if command -v pkexec &>/dev/null; then
        exec pkexec bash "$0" "$@"
    else
        exec sudo bash "$0" "$@"
    fi
fi

echo "=== Deploying Monster Tuxedo Driver ==="

# 1. Sync source files to /usr/src
mkdir -p "$DKMS_DEST"
cp -r "$DRIVER_SRC"/* "$DKMS_DEST/"
chmod -R a+rX "$DKMS_DEST"

# 2. Rebuild and install via DKMS
echo "[*] Building and installing DKMS module tuxedo-drivers-monster/4.25.0..."
dkms remove tuxedo-drivers-monster/4.25.0 --all 2>/dev/null || true
dkms add tuxedo-drivers-monster/4.25.0 2>/dev/null || true
dkms build tuxedo-drivers-monster/4.25.0
dkms install tuxedo-drivers-monster/4.25.0 --force

# 3. Reload in-memory kernel modules
echo "[*] Reloading kernel modules..."
modprobe -r tuxedo_nb02_nvidia_power_ctrl clevo_wmi uniwill_wmi tuxedo_io tuxedo_keyboard 2>/dev/null || true
modprobe tuxedo_keyboard
modprobe uniwill_wmi
modprobe tuxedo_io

# 4. Configure module autoloading on boot
mkdir -p /etc/modules-load.d
cat << 'MCONF' > /etc/modules-load.d/tuxedo.conf
tuxedo_keyboard
uniwill_wmi
tuxedo_io
MCONF

# 5. Configure udev permissions
UDEV_RULE_FILE="/etc/udev/rules.d/99-monster-control-center.rules"
cat << 'UCONF' > "$UDEV_RULE_FILE"
SUBSYSTEM=="leds", KERNEL=="*kbd_backlight*", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/brightness /sys%p/multi_intensity"
SUBSYSTEM=="platform", DRIVER=="tuxedo_keyboard", ACTION=="add", RUN+="/bin/chmod -R a+rX /sys%p", RUN+="/bin/chmod 666 /sys%p/fn_lock"
SUBSYSTEM=="platform_profile", ACTION=="add", RUN+="/bin/chmod 666 /sys/firmware/acpi/platform_profile"
KERNEL=="tuxedo_io", MODE="0666"
UCONF

udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --subsystem-match=leds 2>/dev/null || true
udevadm trigger --name-match=tuxedo_io 2>/dev/null || true

# Apply permissions immediately to active nodes
chmod -R a+rX /sys/devices/platform/tuxedo_keyboard 2>/dev/null || true
chmod -R a+rw /sys/devices/platform/tuxedo_keyboard/leds 2>/dev/null || true
chmod 0666 /sys/devices/platform/tuxedo_keyboard/fn_lock 2>/dev/null || true
chmod 0666 /dev/tuxedo_io 2>/dev/null || true
chmod 0666 /sys/firmware/acpi/platform_profile* 2>/dev/null || true
chmod 0666 /sys/class/hwmon/hwmon*/fan* 2>/dev/null || true

echo "=== Driver Deployment Finished Successfully ==="
