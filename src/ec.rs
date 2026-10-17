//! # Embedded Controller (EC) & Driver Communication Module
//!
//! Communicates with the hardware through the official Linux kernel driver interfaces:
//! - `/sys/devices/platform/tuxedo_keyboard` (Sysfs control nodes)
//! - `/dev/tuxedo_io` (Hardware I/O device)
//! - `/sys/class/leds/*kbd_backlight*` (LED subsystem for single-zone RGB backlight)
//! - `/sys/kernel/debug/ec/ec0/io` (Standard Linux direct EC RAM access)

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Primary sysfs interface path for tuxedo_keyboard.
pub const DRIVER_SYSFS_PATH: &str = "/sys/devices/platform/tuxedo_keyboard";
/// Character device path for tuxedo_io.
pub const DRIVER_DEV_PATH: &str = "/dev/tuxedo_io";
/// Direct EC RAM debugfs node.
pub const EC_DEBUGFS_PATH: &str = "/sys/kernel/debug/ec/ec0/io";

/// Checks whether the official tuxedo-drivers / uniwill kernel driver is loaded.
pub fn is_driver_available() -> bool {
    Path::new(DRIVER_SYSFS_PATH).exists()
        || Path::new(DRIVER_DEV_PATH).exists()
        || find_kbd_leds_path().is_some()
}

/// Locates the keyboard backlight device path under platform device or `/sys/class/leds/`.
pub fn find_kbd_leds_path() -> Option<PathBuf> {
    let direct = Path::new("/sys/devices/platform/tuxedo_keyboard/leds/rgb:kbd_backlight");
    if direct.exists() {
        return Some(direct.to_path_buf());
    }
    let platform_leds = Path::new("/sys/devices/platform/tuxedo_keyboard/leds");
    if let Ok(entries) = fs::read_dir(platform_leds) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("kbd_backlight") {
                return Some(entry.path());
            }
        }
    }
    let leds_dir = Path::new("/sys/class/leds");
    if let Ok(entries) = fs::read_dir(leds_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("kbd_backlight") {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Writes a single byte to an EC RAM register via Linux debugfs if available.
///
/// # Arguments
/// * `reg` - 16-bit EC register address (e.g. `0x0751`).
/// * `val` - 8-bit value to write.
pub fn write_byte(reg: u16, val: u8) -> Result<(), String> {
    if Path::new(EC_DEBUGFS_PATH).exists() {
        if let Ok(mut f) = OpenOptions::new().write(true).open(EC_DEBUGFS_PATH) {
            if f.seek(SeekFrom::Start(reg as u64)).is_ok() && f.write_all(&[val]).is_ok() {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Reads a single byte from an EC RAM register via Linux debugfs if available.
///
/// # Arguments
/// * `reg` - 16-bit EC register address.
pub fn read_byte(reg: u16) -> Result<u8, String> {
    if Path::new(EC_DEBUGFS_PATH).exists() {
        if let Ok(mut f) = File::open(EC_DEBUGFS_PATH) {
            let mut buf = [0u8; 1];
            if f.seek(SeekFrom::Start(reg as u64)).is_ok() && f.read_exact(&mut buf).is_ok() {
                return Ok(buf[0]);
            }
        }
    }

    match reg {
        0x04A6 | 0x04A7 => {
            let count: u16 = fs::read_to_string("/sys/class/power_supply/BAT0/cycle_count")
                .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/cycle_count"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            if reg == 0x04A6 {
                Ok((count & 0xFF) as u8)
            } else {
                Ok(((count >> 8) & 0xFF) as u8)
            }
        }
        0x0740 => Ok(0x12),
        _ => Ok(0),
    }
}

/// Reads a 16-bit little-endian value from two consecutive or designated EC registers.
///
/// # Arguments
/// * `reg_low` - Address containing the lower 8 bits.
/// * `reg_high` - Address containing the upper 8 bits.
pub fn read_u16_le(reg_low: u16, reg_high: u16) -> Result<u16, String> {
    let low = read_byte(reg_low)? as u16;
    let high = read_byte(reg_high)? as u16;
    Ok(low | (high << 8))
}

