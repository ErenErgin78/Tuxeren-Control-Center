# Monster Control Center (Linux)

A clean-room, zero-dependency Rust CLI and Tauri desktop application for hardware control on Uniwill / Tongfang chassis laptops (such as Monster Notebook Tulpar, Abra, and Semruk series) running Linux.

## Reverse Engineering Background

This implementation is grounded in the reverse engineering findings documented in [`../Report.md`](../Report.md), extracted from the official Windows OEM software (`ControlCenter_5.8.49.19_Monster.exe`).

Key hardware facts established:
- Hardware communication bypasses vendor Windows drivers by directly invoking the ACPI subsystem (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`) via the Linux `acpi_call` kernel module.
- Single-zone RGB keyboards on Tongfang GM7 chassis (e.g. Tulpar T7 V20.7) are controlled through Embedded Controller (EC) RAM offsets (`0x0769`–`0x076B`), trigger latch (`0x0767`), and a hardware re-arm toggle (`0x078C`).

## What Is Implemented

The codebase is organized into modular Rust components with both a standalone CLI and a Tauri v2 desktop application:

- **Desktop Application (`gui.rs` + `ui/`)**: A native Tauri GUI for single-zone RGB keyboard control, brightness adjustment, preset switching, and custom hex color picking.
- **EC Communication Layer (`ec.rs`)**: Direct byte-level read and write primitives communicating with `acpi_call`.
- **Keyboard RGB Backlight (`keyboard.rs`)**:
  - Optically balanced color presets (`red`, `green`, `blue`, `cyan`, `magenta`, `purple`, `orange`, `yellow`, `pink`, `white`, `off`).
  - Raw RGB mapping (0..255) scaled to the hardware 0..50 PWM range.
  - Hardware brightness levels (1..4).
  - 40ms power re-arm pulse for instant color updates without requiring Fn key triggers.
- **Raw EC Access (`read` / `write`)**:
  - Direct byte inspection and manual EC register override for debugging.

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

# Keyboard lighting presets
monster-control-center color cyan 4
monster-control-center color off

# Custom RGB
monster-control-center color rgb 255 60 0 4

# Direct EC register debugging
monster-control-center read 0x0769
monster-control-center write 0x078C 0x01
```
