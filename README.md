# Monster Control Center (Windows)

A clean-room Rust and Tauri v2 desktop application for hardware control on Uniwill / Tongfang chassis laptops (such as Monster Notebook Tulpar, Abra, and Semruk series) running Windows.

## Reverse Engineering Background

This implementation is grounded in the reverse engineering findings extracted from the official OEM software (`ControlCenter_5.8.49.19_Monster.exe`).

Key hardware facts established:
- Single-zone RGB keyboards on Tongfang chassis (e.g. Tulpar T7 V20.7) are controlled through Embedded Controller (EC) RAM offsets (`0x0769`–`0x076B`), trigger latch (`0x0767`), and a hardware re-arm toggle (`0x078C`).
- Hardware modes, battery charge thresholds, fan telemetry, and display calibration profiles are directly exposed via EC memory registers.

## Features

- **RGB Keyboard Lighting**:
  - Calibrated color presets (`red`, `green`, `blue`, `cyan`, `magenta`, `purple`, `orange`, `yellow`, `pink`, `white`, `off`).
  - Custom RGB hex / color picker (0..255 scaled to 0..50 hardware PWM).
  - 4-level hardware brightness adjustment.
- **Hardware & Power Profiles**: Office (Silent), Gaming (Standard), Turbo.
- **Battery Management**: Charge threshold modes (%80 Long Life, %100 Standard).
- **System Telemetry**: Real-time CPU, GPU, fan speed, memory, disk, and network stats.
- **Display & General Settings**: Color temperature, brightness calibration, and Windows key toggle.

## Building the Windows Executable (.exe)

### Option 1: Cross-Compilation on Linux

Requires the MinGW toolchain:

```bash
# Fedora
sudo dnf install -y mingw64-gcc mingw64-binutils

# Ubuntu / Debian
sudo apt-get install -y gcc-mingw-w64-x86-64

# Build standalone .exe
make build-exe
```

The resulting standalone executable will be generated at:
```
dist/monster-control-center-gui.exe
```

### Option 2: Native Build on Windows

Ensure Rust toolchain and Node.js are installed on Windows:

```bash
# Build standalone release executable
cargo build --release --bin monster-control-center-gui
```

Executable output:
```
target/release/monster-control-center-gui.exe
```

## Running the Application

Double-click `monster-control-center-gui.exe` on Windows. Microsoft Edge WebView2 runtime (pre-installed on Windows 10 and 11) hosts the desktop user interface.
