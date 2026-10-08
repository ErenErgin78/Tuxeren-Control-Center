# Monster Control Center (Linux)

A clean-room, zero-dependency Rust CLI and library for hardware control on Uniwill / Tongfang chassis laptops (such as Monster Notebook Tulpar, Abra, and Semruk series) running Linux.

## Reverse Engineering Background

This implementation is grounded in the reverse engineering findings documented in [`../Report.md`](../Report.md), extracted from the official Windows OEM software (`ControlCenter_5.8.49.19_Monster.exe`).

Key hardware facts established:
- Hardware communication bypasses vendor Windows drivers by directly invoking the ACPI subsystem (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`) via the Linux `acpi_call` kernel module.
- Single-zone RGB keyboards on Tongfang GM7 chassis (e.g. Tulpar T7 V20.7) are controlled through Embedded Controller (EC) RAM offsets (`0x0769`–`0x076B`), trigger latch (`0x0767`), and a hardware re-arm toggle (`0x078C`).

## What Is Implemented

The codebase is organized into modular Rust components with a standalone CLI:

- **EC Communication Layer (`ec.rs`)**: Direct byte-level read and write primitives communicating with `acpi_call`.
- **Keyboard RGB Backlight (`keyboard.rs`)**:
  - Optically balanced color presets (`red`, `green`, `blue`, `cyan`, `magenta`, `purple`, `orange`, `yellow`, `pink`, `white`, `off`).
  - Raw RGB mapping (0..255) scaled to the hardware 0..50 PWM range.
  - Hardware brightness levels (1..4).
  - 40ms power re-arm pulse for instant color updates without requiring Fn key triggers.
- **Fan Control & Telemetry (`fan.rs`)**:
  - Real-time tachometer readings (CPU/GPU fan RPM).
  - PWM duty cycle telemetry.
  - Mode switching (`auto`, `turbo`, `boost` / CoolerBoost 100%, `custom`).
- **Power Profiles (`power.rs`)**:
  - Platform performance profile switching (`gaming`, `office`, `eco`, `turbo`).
- **Battery Health (`battery.rs`)**:
  - Live gas gauge telemetry (State of Charge %, pack temperature, cycle count).
  - Upper charge threshold limiter with 5% hysteresis.
- **Raw EC Access (`read` / `write`)**:
  - Direct byte inspection and manual EC register override for debugging.

## Usage

Requires the `acpi_call` kernel module loaded and root privileges (`sudo`):

```bash
# Build
cargo build --release

# View hardware status
sudo ./target/release/monster-control-center status

# Keyboard lighting
sudo ./target/release/monster-control-center color cyan 4
sudo ./target/release/monster-control-center color rgb 255 60 0 4
sudo ./target/release/monster-control-center color off

# Fan modes
sudo ./target/release/monster-control-center fan boost
sudo ./target/release/monster-control-center fan auto

# Performance profile
sudo ./target/release/monster-control-center mode gaming

# Battery charge limit (50-100%)
sudo ./target/release/monster-control-center limit 80
```

## Scope & Future Development

This project serves as the foundational hardware control core. There are plans to extend the scope into a full-featured Linux control center.
