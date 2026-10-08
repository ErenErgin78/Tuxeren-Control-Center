//! # Keyboard Backlight & RGB Subsystem Module
//!
//! Controls the single-zone RGB keyboard backlight integrated into the Embedded Controller.
//! Supports calibrated color presets, manual RGB mapping (0..255 to 0..50 PWM scale),
//! hardware brightness levels (1..4), and instant refresh re-arming.

use crate::ec;
use std::thread::sleep;
use std::time::Duration;

/// EC RAM offset for Red channel duty cycle (0..50).
pub const ADDR_RGB_R: u16      = 0x0769;
/// EC RAM offset for Green channel duty cycle (0..50).
pub const ADDR_RGB_G: u16      = 0x076A;
/// EC RAM offset for Blue channel duty cycle (0..50).
pub const ADDR_RGB_B: u16      = 0x076B;
/// EC RAM offset for Keyboard backlight power switch (0x01 = On, 0x00 = Off).
pub const ADDR_KBL_ENABLE: u16 = 0x078C;
/// EC RAM offset for Latch trigger register (Bit 0x20 notifies EC to apply changes).
pub const ADDR_TRIGGER: u16    = 0x0767;

/// Optically balanced color presets adjusted for LED lumen efficiency curves.
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

/// Sets the keyboard RGB backlight color and brightness.
///
/// Converts 0..255 RGB inputs to 0..50 EC hardware PWM steps, scales by brightness (1..4),
/// triggers the latch register, and issues a 40ms power re-arm pulse to bypass FN+F6 requirement.
///
/// # Arguments
/// * `r` - Red channel (0..255).
/// * `g` - Green channel (0..255).
/// * `b` - Blue channel (0..255).
/// * `brightness` - Brightness level (0..4).
pub fn set_rgb(r: u8, g: u8, b: u8, brightness: u8) -> Result<(), String> {
    if brightness == 0 || (r == 0 && g == 0 && b == 0) {
        ec::write_byte(ADDR_KBL_ENABLE, 0x00)?;
        ec::write_byte(ADDR_TRIGGER, 0x20)?;
        return Ok(());
    }

    let b_clamped = brightness.clamp(1, 4);
    let scale = b_clamped as f32 / 4.0;
    let ec_r = (((r as f32 / 255.0) * 50.0 * scale).round() as u8).min(50);
    let ec_g = (((g as f32 / 255.0) * 50.0 * scale).round() as u8).min(50);
    let ec_b = (((b as f32 / 255.0) * 50.0 * scale).round() as u8).min(50);

    ec::write_byte(ADDR_RGB_R, ec_r)?;
    ec::write_byte(ADDR_RGB_G, ec_g)?;
    ec::write_byte(ADDR_RGB_B, ec_b)?;
    ec::write_byte(ADDR_TRIGGER, 0x20)?;

    // Hardware re-arm pulse (0x00 -> 40ms -> 0x01)
    ec::write_byte(ADDR_KBL_ENABLE, 0x00)?;
    sleep(Duration::from_millis(40));
    ec::write_byte(ADDR_KBL_ENABLE, 0x01)?;
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
            "Unknown color preset '{}'. Available: {}",
            name,
            PRESETS.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
        ))
    }
}

