//! # General Laptop Settings Module
//!
//! Provides control for chassis-level general settings including
//! Windows Key toggle (disable/enable).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

const SETTINGS_FILE: &str = "/etc/monster-control-center/general_settings.json";

/// General settings state representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettings {
    /// Whether the Windows / Super key is disabled (locked).
    pub win_key_disabled: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            win_key_disabled: false,
        }
    }
}

/// Retrieves the current status of general settings.
pub fn get_general_settings() -> GeneralSettings {
    load_settings()
}

/// Sets whether the Windows key is disabled.
pub fn set_win_key_disabled(disabled: bool) -> Result<(), String> {
    let mut current = load_settings();
    current.win_key_disabled = disabled;
    save_settings(&current);
    apply_win_key_state(disabled)
}

fn load_settings() -> GeneralSettings {
    if let Ok(content) = fs::read_to_string(SETTINGS_FILE) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    GeneralSettings::default()
}

fn save_settings(settings: &GeneralSettings) {
    if let Some(parent) = Path::new(SETTINGS_FILE).parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(SETTINGS_FILE, data);
    }
}

fn apply_win_key_state(disabled: bool) -> Result<(), String> {
    // 1. GNOME Wayland / Mutter overlay key management
    let overlay_val = if disabled { "''" } else { "'Super'" };
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.mutter", "overlay-key", overlay_val])
        .output();

    // 2. X11 / Xwayland Super_L key remapping
    let xmodmap_cmd = if disabled {
        "keycode 133 = NoSymbol"
    } else {
        "keycode 133 = Super_L"
    };
    let _ = Command::new("xmodmap")
        .args(["-e", xmodmap_cmd])
        .output();

    Ok(())
}
