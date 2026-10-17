//! # Keyboard Backlight & RGB Subsystem Module
//!
//! Controls the single-zone RGB keyboard backlight via the official Linux LED subsystem (`/sys/class/leds/*kbd_backlight*`).

use std::fs;
use std::path::{Path, PathBuf};

/// Optically balanced color presets.
pub const PRESETS: &[(&str, (u8, u8, u8))] = &[
    ("red", (255, 0, 0)),
    ("green", (0, 255, 0)),
    ("blue", (0, 0, 255)),
    ("cyan", (0, 90, 255)),
    ("ice_blue", (0, 50, 255)),
    ("magenta", (140, 0, 255)),
    ("purple", (70, 0, 255)),
    ("pink", (255, 20, 90)),
    ("yellow", (255, 180, 0)),
    ("orange", (255, 45, 0)),
    ("white", (200, 255, 255)),
    ("off", (0, 0, 0)),
];

/// Locates the keyboard backlight device path under `/sys/class/leds/`.
pub fn find_kbd_led_device() -> Option<PathBuf> {
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

/// Sets the keyboard RGB backlight color and brightness via the Linux LED class.
///
/// # Arguments
/// * `r` - Red channel (0..255).
/// * `g` - Green channel (0..255).
/// * `b` - Blue channel (0..255).
/// * `brightness` - Brightness level (0..4).
pub fn set_rgb(r: u8, g: u8, b: u8, brightness: u8) -> Result<(), String> {
    let dev = find_kbd_led_device().ok_or_else(|| {
        "Klavye LED aygıtı bulunamadı (/sys/class/leds/*kbd_backlight*). tuxedo_keyboard sürücüsünü yükleyin.".to_string()
    })?;

    let max_brightness = fs::read_to_string(dev.join("max_brightness"))
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(4);

    let target_brightness = if r == 0 && g == 0 && b == 0 {
        0
    } else {
        brightness.clamp(0, max_brightness)
    };

    let brightness_path = dev.join("brightness");
    fs::write(&brightness_path, target_brightness.to_string())
        .map_err(|e| format!("Parlaklık yazılamadı: {}", e))?;

    if target_brightness > 0 {
        let multi_path = dev.join("multi_intensity");
        if multi_path.exists() {
            fs::write(&multi_path, format!("{} {} {}", r, g, b))
                .map_err(|e| format!("RGB rengi yazılamadı: {}", e))?;
        } else {
            let color_path = dev.join("color");
            if color_path.exists() {
                let _ = fs::write(color_path, format!("{:02X}{:02X}{:02X}", r, g, b));
            }
        }
    }

    Ok(())
}

/// Sets the keyboard backlight to a pre-calibrated color preset name.
///
/// # Arguments
/// * `name` - Preset identifier (e.g. "cyan", "purple", "white").
/// * `brightness` - Brightness level (1..4).
pub fn set_preset(name: &str, brightness: u8) -> Result<(), String> {
    if let Some(&(_, (r, g, b))) = PRESETS.iter().find(|(k, _)| *k == name.to_lowercase()) {
        set_rgb(r, g, b, brightness)
    } else {
        Err(format!(
            "Bilinmeyen renk önayarı '{}'. Kullanılabilir: {}",
            name,
            PRESETS.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
        ))
    }
}

