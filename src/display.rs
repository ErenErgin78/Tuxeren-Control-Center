//! # Display Settings Module
//!
//! Controls display brightness, color temperature, saturation, gamma,
//! and per-channel RGB levels on Linux.
//! Integrates with GNOME DisplayConfig / Color daemon, dconf extension shaders,
//! and sysfs backlight interfaces.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SETTINGS_FILE: &str = "/etc/monster-control-center/display_settings.json";
const BACKLIGHT_PATH: &str = "/sys/class/backlight/intel_backlight";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplaySettings {
    /// Brightness level in percentage (0..100)
    pub brightness: u32,
    /// Color temperature in Kelvin (1000..10000, default 6500)
    pub color_temp: u32,
    /// Color saturation factor (0.0..2.0, default 1.0)
    pub saturation: f32,
    /// Gamma correction factor (0.5..2.5, default 1.0)
    pub gamma: f32,
    /// Red channel level (0..255, default 128)
    pub red: u8,
    /// Green channel level (0..255, default 128)
    pub green: u8,
    /// Blue channel level (0..255, default 128)
    pub blue: u8,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            brightness: 100,
            color_temp: 6500,
            saturation: 1.0,
            gamma: 1.0,
            red: 128,
            green: 128,
            blue: 128,
        }
    }
}

fn config_path() -> PathBuf {
    let p = Path::new(SETTINGS_FILE);
    if p.exists() || (p.parent().is_some() && fs::create_dir_all(p.parent().unwrap()).is_ok()) {
        return p.to_path_buf();
    }
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".config/monster-control-center/display_settings.json")
    } else {
        PathBuf::from("/dev/shm/monster_display_settings.json")
    }
}

pub fn load_settings() -> DisplaySettings {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str::<DisplaySettings>(&data) {
            return settings;
        }
    }
    DisplaySettings::default()
}

pub fn save_settings(settings: &DisplaySettings) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(&path, data);
    }
}

/// Reads current hardware backlight brightness as a percentage (0..100).
pub fn get_brightness() -> u32 {
    let path = Path::new(BACKLIGHT_PATH);
    let cur_str = fs::read_to_string(path.join("brightness")).unwrap_or_default();
    let max_str = fs::read_to_string(path.join("max_brightness")).unwrap_or_default();

    let cur = cur_str.trim().parse::<u64>().unwrap_or(0);
    let max = max_str.trim().parse::<u64>().unwrap_or(0);

    if max > 0 {
        ((cur as f64 / max as f64) * 100.0).round() as u32
    } else {
        100
    }
}

/// Sets hardware backlight brightness (0..100%).
pub fn set_brightness(percent: u32) -> Result<(), String> {
    let pct = percent.clamp(1, 100);

    // 1. Try GNOME Mutter DisplayConfig SetBacklight via busctl
    let output = Command::new("busctl")
        .args([
            "--user",
            "get-property",
            "org.gnome.Mutter.DisplayConfig",
            "/org/gnome/Mutter/DisplayConfig",
            "org.gnome.Mutter.DisplayConfig",
            "Backlight",
        ])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(serial) = parts[1].parse::<u32>() {
                let max_b = parts.windows(3).find_map(|w| {
                    if w[0] == "\"max\"" && w[1] == "i" {
                        w[2].parse::<i32>().ok()
                    } else {
                        None
                    }
                }).unwrap_or(96000);

                let min_b = parts.windows(3).find_map(|w| {
                    if w[0] == "\"min\"" && w[1] == "i" {
                        w[2].parse::<i32>().ok()
                    } else {
                        None
                    }
                }).unwrap_or(960);

                let connector = parts.windows(3).find_map(|w| {
                    if w[0] == "\"connector\"" && w[1] == "s" {
                        Some(w[2].trim_matches('"'))
                    } else {
                        None
                    }
                }).unwrap_or("eDP-1");

                let target = min_b + ((max_b - min_b) as f64 * (pct as f64 / 100.0)).round() as i32;

                let set_res = Command::new("busctl")
                    .args([
                        "--user",
                        "call",
                        "org.gnome.Mutter.DisplayConfig",
                        "/org/gnome/Mutter/DisplayConfig",
                        "org.gnome.Mutter.DisplayConfig",
                        "SetBacklight",
                        "usi",
                        &serial.to_string(),
                        connector,
                        &target.to_string(),
                    ])
                    .output();

                if let Ok(r) = set_res {
                    if r.status.success() {
                        return Ok(());
                    }
                }
            }
        }
    }

    // 2. Direct sysfs fallback
    let path = Path::new(BACKLIGHT_PATH);
    let max_str = fs::read_to_string(path.join("max_brightness"))
        .map_err(|e| format!("Parlaklık okunamadı: {}", e))?;
    let max = max_str.trim().parse::<u64>().map_err(|e| e.to_string())?;
    let raw = ((max as f64) * (pct as f64 / 100.0)).round() as u64;

    fs::write(path.join("brightness"), raw.to_string())
        .map_err(|e| format!("Parlaklık yazılamadı: {}", e))
}

/// Applies color temperature in Kelvin (1000..10000).
pub fn set_color_temperature(temp: u32) -> Result<(), String> {
    let clamped = temp.clamp(1000, 10000);
    if clamped == 6500 {
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.settings-daemon.plugins.color", "night-light-enabled", "false"])
            .output();
    } else {
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.settings-daemon.plugins.color", "night-light-enabled", "true"])
            .output();
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.settings-daemon.plugins.color",
                "night-light-temperature",
                &clamped.to_string(),
            ])
            .output();
    }
    Ok(())
}

/// Applies color correction factors (RGB, saturation, gamma) via display color shaders.
pub fn apply_color_factors(r: u8, g: u8, b: u8, saturation: f32, gamma: f32) {
    // Normalization: 128 is neutral 1.0; gamma applies inverse curve.
    // Clamp each channel to at least 15 to prevent the display from going completely black.
    let safe_r = r.max(15);
    let safe_g = g.max(15);
    let safe_b = b.max(15);

    let inv_gamma = if gamma > 0.01 { 1.0 / gamma as f64 } else { 1.0 };
    let r_factor = (safe_r as f64 / 128.0) * inv_gamma;
    let g_factor = (safe_g as f64 / 128.0) * inv_gamma;
    let b_factor = (safe_b as f64 / 128.0) * inv_gamma;

    let sat = saturation.clamp(0.0, 2.0) as f64;

    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/red-factor",
            &format!("{:.3}", r_factor),
        ])
        .output();
    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/green-factor",
            &format!("{:.3}", g_factor),
        ])
        .output();
    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/blue-factor",
            &format!("{:.3}", b_factor),
        ])
        .output();
    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/red-saturation",
            &format!("{:.3}", sat),
        ])
        .output();
    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/green-saturation",
            &format!("{:.3}", sat),
        ])
        .output();
    let _ = Command::new("dconf")
        .args([
            "write",
            "/org/gnome/shell/extensions/display-color-correct/blue-saturation",
            &format!("{:.3}", sat),
        ])
        .output();
}

/// Retrieves all active display settings.
pub fn get_display_settings() -> DisplaySettings {
    let mut current = load_settings();
    current.brightness = get_brightness();
    current
}

/// Sets and applies full display configuration.
pub fn apply_display_settings(mut settings: DisplaySettings) -> Result<(), String> {
    settings.red = settings.red.max(15);
    settings.green = settings.green.max(15);
    settings.blue = settings.blue.max(15);

    set_brightness(settings.brightness)?;
    set_color_temperature(settings.color_temp)?;
    apply_color_factors(
        settings.red,
        settings.green,
        settings.blue,
        settings.saturation,
        settings.gamma,
    );
    save_settings(&settings);
    Ok(())
}

/// Resets display settings back to factory neutral calibration.
pub fn reset_display_settings() -> Result<DisplaySettings, String> {
    let default = DisplaySettings::default();
    apply_display_settings(default.clone())?;
    Ok(default)
}
