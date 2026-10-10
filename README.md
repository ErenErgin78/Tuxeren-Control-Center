# Monster Control Center (Linux)

A clean-room, zero-dependency Rust CLI and Tauri desktop application for hardware control on Uniwill / Tongfang chassis laptops (such as Monster Notebook Tulpar, Abra, and Semruk series) running Linux.

## Reverse Engineering Background

This implementation is grounded in the reverse engineering findings documented in [`../Report.md`](../Report.md), extracted from the official Windows OEM software (`ControlCenter_5.8.49.19_Monster.exe`).

Key hardware facts established:
- Hardware communication bypasses vendor Windows drivers by directly invoking the ACPI subsystem (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`) via the Linux `acpi_call` kernel module.
- Single-zone RGB keyboards on Tongfang GM7 chassis (e.g. Tulpar T7 V20.7) are controlled through Embedded Controller (EC) RAM offsets (`0x0769`–`0x076B`), trigger latch (`0x0767`), and a hardware re-arm toggle (`0x078C`).

## What Is Implemented

The codebase is organized into modular Rust components with both a standalone CLI and a Tauri v2 desktop application, featuring six core subsystems:

- **Desktop Application (`gui.rs` + `ui/`)**:
  - Native Tauri v2 desktop interface matching the OEM Control Center experience.
  - In-app root permission helper (`grant_permissions` / `pkexec`) to automatically load `acpi_call` and configure sysfs permissions for ACPI and Intel RAPL power capping.
- **Keyboard Lighting (`keyboard.rs` + `ui/keyboard_light.html`)**:
  - 11 optically balanced color presets (`cyan`, `ice_blue`, `blue`, `purple`, `magenta`, `pink`, `red`, `orange`, `yellow`, `green`, `white`) and `off`.
  - Custom color picker with an interactive 2D HSV canvas (saturation & value) and hue slider, plus quick-apply swatch and a 5-color recent history.
  - Hardware brightness levels (`0`..`4`, from off to maximum).
  - Direct hardware PWM scaling (0..255 mapped to 0..50) with 40ms power re-arm pulse for instant live updates without requiring Fn key triggers.
- **Performance Management & System Telemetry (`performance.rs` + `ui/performance.html`)**:
  - **Hardware/BIOS Power Modes**: Office (`0`, ~45W PL1, quiet fans), Gaming (`1`, ~75W PL1, balanced fans), and Turbo (`2`, ~90W PL1, high/turbo fans) modes applied via EC registers (`0x0751`, `0x07AB`, `0x046A`, `0x046B`) and Intel RAPL power caps.
  - **OS Power Profiles**: Bi-directional integration with Linux power profiles (`power-saver`, `balanced`, `performance`) via D-Bus `power-profiles-daemon`.
  - **Chassis Mode Button Synchronization**: Background hardware watcher thread monitoring physical mode button toggles with desktop OSD notification sync.
  - **Live System Telemetry Dashboard**: Real-time polling for CPU usage % & temp (°C), GPU usage % & temp (°C), RAM usage & capacity (GB), Disk space usage & capacity, dual cooling fan RPMs & duty cycles (CPU & GPU fans), and live network download/upload throughput speeds.
- **General Laptop Settings (`general.rs` + `ui/general.html`)**:
  - **Windows Key Lock**: Toggle to disable/enable the Super/Windows key during gaming.
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
  - Hardware charging cycle count directly queried from EC registers (`0x04A6`–`0x04A7`) or sysfs, and current stored energy.
- **Device & Hardware Information (`device_info.rs` + `ui/device_info.html`)**:
  - **Device & Chassis**: Laptop model name, vendor, Tongfang chassis/platform family code (e.g. GM7, PF4/PF5 mapped from EC `0x0740`), motherboard SKU/revision, and serial number.
  - **Firmware & Controller**: BIOS version and release date, EC firmware version, and active platform kernel driver.
  - **Core Hardware**: Processor (CPU), Integrated Graphics (iGPU), Dedicated GPU (dGPU), System RAM (total capacity & modules), and storage drives.
  - **OS & Environment**: Linux distribution/version, desktop environment & display protocol (Wayland / X11), kernel version, and active graphics drivers (NVIDIA driver version / Mesa).
- **EC Communication Layer & Raw Register Access (`ec.rs`)**:
  - Direct byte-level read and write primitives communicating with `acpi_call` (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`).
  - Direct EC register inspection (`read`) and manual override (`write`) for low-level debugging and reverse engineering.

## Installation & Desktop Integration

Run the installer script to build and install both the GUI desktop app and CLI to `~/.local/bin`, register the desktop launcher icon, and ensure `acpi_call` kernel module persistence:

```bash
./install.sh
```

Once installed, **Monster Control Center** can be launched from your application launcher or by running:

```bash
monster-control-center-gui
```

## CLI Usage

The standalone CLI is available in PATH:

```bash
# View chassis project ID
monster-control-center chassis

# System and hardware specifications
monster-control-center info

# Keyboard lighting presets
monster-control-center color cyan 4
monster-control-center color off

# Custom RGB
monster-control-center color rgb 255 60 0 4

# Performance modes and telemetry
monster-control-center perf
monster-control-center perf turbo
monster-control-center perf os performance
monster-control-center perf watch

# Display settings and calibration
monster-control-center display
monster-control-center display brightness 80
monster-control-center display temp 6500
monster-control-center display reset

# Background daemon (touchpad F24 switch & performance sync)
monster-control-center daemon

# Direct EC register debugging
monster-control-center read 0x0769
monster-control-center write 0x078C 0x01
```
