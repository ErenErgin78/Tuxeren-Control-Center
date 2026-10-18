# Tuxeren Control Center (TxeCtrl) - Linux

A clean-room, Rust CLI and Tauri desktop application for hardware control on Uniwill / Tongfang chassis laptops running Linux.

## Kernel Driver Architecture: Transition from `acpi_call` to `tuxedo-drivers`

Historically, this project relied on the third-party `acpi_call` kernel module to execute raw AML methods (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`). While functional for initial reverse engineering, `acpi_call` posed major drawbacks:
- It was blocked under UEFI Secure Boot and Linux Kernel Lockdown.
- It required elevated permissions (`/proc/acpi/call` rw access) and raw byte parsing in userspace.

The project has transitioned to the official **`tuxedo-drivers` (Uniwill / Tongfang WMI and platform driver stack)** and standard Linux kernel subsystems:
- **Keyboard Lighting**: Managed via the Linux LED class (`/sys/devices/platform/tuxedo_keyboard/leds/rgb:kbd_backlight/`) utilizing `multi_intensity` and `brightness` attributes.
- **Hardware I/O & Fan Telemetry**: Controlled through the character device `/dev/tuxedo_io` and native sysfs platform nodes.
- **Power & Thermal Management**: Integrated with Linux ACPI `platform_profile` and D-Bus `power-profiles-daemon`, backed by Intel RAPL and CPU EPP sysfs knobs.
- **Input & Fn-Lock**: Controlled natively via `/sys/devices/platform/tuxedo_keyboard/fn_lock` and low-level `evdev` event monitoring.

## What Is Implemented

The codebase is organized into modular Rust components with both a standalone CLI and a Tauri v2 desktop application, featuring six core subsystems:

- **Desktop Application (`gui.rs` + `ui/`)**:
  - Native Tauri v2 desktop interface matching the OEM Control Center experience.
  - Automated `udev` rules configuration granting non-root user access to keyboard backlight nodes and `/dev/tuxedo_io`.
- **Keyboard Lighting (`keyboard.rs` + `ui/keyboard_light.html`)**:
  - 11 optically balanced color presets (`cyan`, `ice_blue`, `blue`, `purple`, `magenta`, `pink`, `red`, `orange`, `yellow`, `green`, `white`) and `off`.
  - Custom color picker with an interactive 2D HSV canvas (saturation & value) and hue slider, plus quick-apply swatch and a 5-color recent history.
  - Hardware brightness levels (`0`..`4`, from off to maximum).
  - Direct kernel multicolor LED integration without requiring root privileges or manual Fn-key toggling.
- **Performance Management & System Telemetry (`performance.rs` + `ui/performance.html`)**:
  - **Hardware/BIOS Power Modes**: Office, Gaming, and Turbo modes applied via driver interfaces, EC fallbacks, and Intel RAPL power caps.
  - **OS Power Profiles**: Bi-directional integration with Linux power profiles (`power-saver`, `balanced`, `performance`) via D-Bus `power-profiles-daemon`.
  - **Chassis Mode Button Synchronization**: Background hardware watcher thread monitoring physical mode button toggles with desktop OSD notification sync.
  - **Live System Telemetry Dashboard**: Real-time polling for CPU usage % & temp (°C), GPU usage % & temp (°C), RAM usage & capacity (GB), Disk space usage & capacity, cooling fan RPMs, and live network throughput.
- **General Laptop Settings (`general.rs` + `ui/general.html`)**:
  - **Windows Key Lock**: Hardware-level toggle via `tuxedo_keyboard/fn_lock` (eliminates environment-specific `gsettings` or `xmodmap` hacks).
  - **Touchpad Lock & Hardware Switch**: Software touchpad disable toggle, integrated with low-level `evdev` listening for the physical touchpad double-tap corner switch (F24 key event).
  - **Background Daemon & IPC (`daemon`)**: Lightweight background service with Unix domain socket IPC managing input locks and hardware event listening.
- **Display Calibration (`display.rs` + `ui/display.html`)**:
  - Screen brightness adjustment via sysfs backlight interface.
  - Color temperature calibration (Kelvin scale, 3000K to 9000K).
  - Saturation and gamma curve correction.
  - Per-channel RGB calibration (Red, Green, Blue sliders).
  - Factory reset function to restore neutral calibration.
- **Battery Health & Power Telemetry (`battery.rs` + `ui/battery.html`)**:
  - Real-time battery charge percentage, charging state (Charging, Discharging, Full), and power source (AC adapter vs Battery).
  - Estimated discharge duration remaining.
  - Battery health and wear level calculation (Full Charge Capacity vs Design Capacity).
  - Electrical telemetry: Operating voltage, instantaneous power draw rate (Watts), and power source type.
  - Hardware charging cycle count queried via sysfs power supply class with EC fallbacks.
- **Device & Hardware Information (`device_info.rs` + `ui/device_info.html`)**:
  - **Device & Chassis**: Laptop model name, vendor, Tongfang chassis/platform family code (e.g. GM7, PF4/PF5), motherboard SKU/revision, and serial number.
  - **Firmware & Controller**: BIOS version and release date, EC firmware version, and active platform kernel driver (`tuxedo-drivers (uniwill_wmi)`).
  - **Core Hardware**: Processor (CPU), Integrated Graphics (iGPU), Dedicated GPU (dGPU), System RAM, and storage drives.
  - **OS & Environment**: Linux distribution/version, desktop environment & display protocol (Wayland / X11), kernel version, and active graphics drivers.
- **Driver & Hardware Communication Layer (`ec.rs`)**:
  - Abstracted hardware interface prioritizing official kernel sysfs endpoints and `/dev/tuxedo_io`.
  - Direct debugfs EC RAM inspection (`/sys/kernel/debug/ec/ec0/io`) for low-level diagnostic analysis.

## Installation & Desktop Integration

Run the installer script or make target to build and install both the GUI desktop app and CLI to `~/.local/bin`, register the desktop launcher icon, and set up non-root `udev` permissions:

```bash
./install.sh
```

Once installed, **Tuxeren Control Center** can be launched from your application launcher or by running:

```bash
txectrl-gui
```

## CLI Usage

The standalone CLI is available in PATH:

```bash
# View chassis project ID
txectrl chassis

# System and hardware specifications
txectrl info

# Keyboard lighting presets
txectrl color cyan 4
txectrl color off

# Custom RGB
txectrl color rgb 255 60 0 4

# Performance modes and telemetry
txectrl perf
txectrl perf turbo
txectrl perf os performance
txectrl perf watch

# Display settings and calibration
txectrl display
txectrl display brightness 80
txectrl display temp 6500
txectrl display reset

# Background daemon (touchpad F24 switch & performance sync)
txectrl daemon

# Direct EC register debugging
txectrl read 0x0769
txectrl write 0x078C 0x01
```

Disclaimer: This project is an independent, community-driven open-source tool and is not affiliated with, endorsed by, or associated with Tongfang Co., Ltd., Tuxedo Computers or any of their subsidiaries. All trademarks and brand names belong to their respective owners.
