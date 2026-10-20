<div align="center">

<img src="ui/assets/tuxeren_fan_animated.svg" alt="Tuxeren Control Center Logo" width="140" height="140" />

# Tuxeren Control Center (TxeCtrl)

**Hardware Control Center and CLI for Uniwill / Tongfang Chassis Laptops on Linux**

[![Latest Release](https://img.shields.io/github/v/release/ErenErgin78/Monster-Control-Center-Linux?color=00c853&label=Release&style=for-the-badge)](https://github.com/ErenErgin78/Monster-Control-Center-Linux/releases)
[![License](https://img.shields.io/badge/License-GPL--3.0-orange.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20x86__64-purple.svg?style=for-the-badge)](#)

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#launching-the-gui">Launching the GUI</a> •
  <a href="#cli-usage">CLI Usage</a> •
  <a href="#technical-architecture">Technical Architecture</a>
</p>

</div>

---

## Features

Tuxeren Control Center (`txectrl`) is a Rust-based hardware control suite designed for Tongfang and Uniwill chassis laptops (such as Monster Notebook and Tuxedo devices) running Linux. It provides both a native Tauri v2 desktop interface and a standalone command-line tool.

### Keyboard Backlight Control
- 11 color presets (`cyan`, `ice_blue`, `blue`, `purple`, `magenta`, `pink`, `red`, `orange`, `yellow`, `green`, `white`) and quick off state.
- Custom color picker with interactive 2D HSV canvas, hue slider, and a 5-color recent history.
- 4-level hardware brightness scaling (0 to 4).
- Direct kernel LED integration via non-root `udev` permissions.

### Performance Management and Telemetry
- **Hardware Power Modes**: Switch between Office, Gaming, and Turbo hardware profiles backed by driver interfaces, EC fallbacks, and Intel RAPL limits.
- **OS Power Profiles**: Bi-directional integration with Linux `power-profiles-daemon` via D-Bus (`power-saver`, `balanced`, `performance`).
- **Physical Mode Button Sync**: Hardware listener thread for physical chassis mode buttons with desktop OSD notification sync.
- **System Telemetry Dashboard**: Real-time polling for CPU/GPU usage and temperatures, RAM and disk usage, fan RPMs, and network throughput.

### General Laptop Settings & Interface
- **Multi-Language Support (i18n)**: Full interface localization with instant switching between English and Turkish.
- **Windows Key Lock**: Hardware-level lock toggled via platform sysfs attributes.
- **Touchpad Lock and Hardware Switch**: Software disable toggle with low-level `evdev` monitoring for corner double-tap (F24) hardware events.
- **One-Click Driver Permission Grants**: GUI banner with `pkexec` privilege elevation to configure driver modules (`uniwill_wmi`, `tuxedo_keyboard`, `tuxedo_io`) and `udev` access rules without manual terminal intervention.

### Display Calibration
- Screen backlight brightness adjustment.
- Color temperature calibration (Kelvin scale, 3000K to 9000K).
- Saturation, gamma correction, and per-channel RGB balance adjustments.
- Factory reset option for neutral calibration.

### Battery Health and Power Monitoring
- Charge percentage, charging state (Charging, Discharging, Full), and power source.
- Estimated discharge time remaining.
- Battery health calculations (Full Charge Capacity vs Design Capacity) and wear level.
- Electrical metrics: Voltage, instantaneous power draw rate (Watts), and cycle count.

### Device and Chassis Information
- Platform identification: Chassis family code (e.g. GM7, PF4/PF5), motherboard SKU, and serial number.
- BIOS and EC firmware version reporting.
- Component breakdown: CPU, integrated graphics (iGPU), dedicated GPU (dGPU), memory, kernel version, and display server (Wayland / X11).

---

## Installation

### Pre-built Packages (Recommended)

Pre-compiled packages are available on the [Releases](https://github.com/ErenErgin78/Monster-Control-Center-Linux/releases) page:

- **Fedora / RHEL / openSUSE (`.rpm`)**:
  ```bash
  sudo dnf install ./tuxeren-control-center-*.rpm
  ```
- **Ubuntu / Debian (`.deb`)**:
  ```bash
  sudo apt install ./tuxeren-control-center-*.deb
  ```
- **Universal Linux (`.AppImage`)**:
  ```bash
  chmod +x tuxeren-control-center-*.AppImage
  ./tuxeren-control-center-*.AppImage
  ```

---

### Building from Source

Clone the repository:

```bash
git clone https://github.com/ErenErgin78/Monster-Control-Center-Linux.git
cd Monster-Control-Center-Linux
```

#### Using Makefile

- **Build and install RPM locally (Fedora / RHEL)**:
  ```bash
  make install-rpm
  ```
- **Build and install DEB locally (Ubuntu / Debian)**:
  ```bash
  make install-deb
  ```
- **Local user installation** (`~/.local/bin`):
  ```bash
  make install
  # or directly: ./install.sh
  ```

Ensure `~/.local/bin` is in your `PATH` if using user-level installation. Re-logging or reloading `udev` rules may be required for non-root hardware access to take immediate effect.

---

## Launching the GUI

Once installed, **Tuxeren Control Center** integrates with your desktop environment and appears in your system's application menu/launcher (e.g. GNOME Dash, KDE Kickoff, Application Drawer) alongside your regular apps. You can launch it by searching for **Tuxeren Control Center** or running:

```bash
txectrl-gui
```

---

## CLI Usage

Hardware controls can be managed directly via the standalone `txectrl` command-line utility:

```bash
# Display system and hardware specifications
txectrl info

# Show chassis project identifier
txectrl chassis

# Keyboard backlight presets
txectrl color cyan 4
txectrl color off

# Custom RGB color and brightness (R G B Brightness)
txectrl color rgb 255 60 0 4

# Performance modes and monitoring
txectrl perf
txectrl perf turbo
txectrl perf os performance
txectrl perf watch

# Display settings and calibration
txectrl display
txectrl display brightness 80
txectrl display temp 6500
txectrl display reset

# Background daemon (touchpad F24 switch and mode button sync)
txectrl daemon

# Low-level EC register access (read / write)
txectrl read 0x0769
txectrl write 0x078C 0x01
```

---

## Technical Architecture

The suite interacts with laptop hardware via standard Linux kernel subsystems and `tuxedo-drivers`:

- **Keyboard Backlight**: Managed through the Linux LED class (`/sys/devices/platform/tuxedo_keyboard/leds/rgb:kbd_backlight/`) utilizing `multi_intensity` and `brightness` attributes.
- **Hardware I/O & Fan Telemetry**: Interfaced through the `/dev/tuxedo_io` character device and platform sysfs nodes.
- **Power & Thermal Controls**: Integrated with Linux ACPI `platform_profile`, D-Bus `power-profiles-daemon`, Intel RAPL, and CPU EPP sysfs nodes.
- **Input & Fn-Lock**: Managed via `/sys/devices/platform/tuxedo_keyboard/fn_lock` and low-level `evdev` event monitoring.
- **EC Interface**: Hardware abstraction layer in `src/ec.rs` prioritizing sysfs endpoints and `/dev/tuxedo_io`, with debugfs EC RAM inspection (`/sys/kernel/debug/ec/ec0/io`) available for low-level diagnostics.
- **dGPU Power-State Optimization**: Checks NVIDIA dGPU runtime power state via sysfs before querying `nvidia-smi` to prevent PCIe D3cold wake latency, with cached static specs via `std::sync::OnceLock`.

---

<div align="center">

<sub>Disclaimer: This project is an independent, community-driven open-source tool and is not affiliated with, endorsed by, or associated with Tongfang Co., Ltd., Tuxedo Computers, Monster Notebook, or any of their subsidiaries. All trademarks belong to their respective owners.</sub>

</div>


