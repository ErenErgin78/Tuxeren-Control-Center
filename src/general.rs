//! # General Laptop Settings Module
//!
//! Provides control for chassis-level general settings including
//! Windows Key toggle and Touchpad lock (direct switch & physical corner F24 button listener).

use evdev::{Device, EventSummary, KeyCode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const SETTINGS_FILE: &str = "/etc/monster-control-center/general_settings.json";
const TOUCHPAD_STATE_FILE: &str = "/dev/shm/touchpad_locker_state";

static TOUCHPAD_DEV: Mutex<Option<Device>> = Mutex::new(None);
static DAEMON_STARTED: AtomicBool = AtomicBool::new(false);

/// General settings state representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettings {
    /// Whether the Windows / Super key is disabled (locked).
    pub win_key_disabled: bool,
    /// Whether the touchpad is disabled (locked).
    #[serde(default)]
    pub touchpad_disabled: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            win_key_disabled: false,
            touchpad_disabled: false,
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

/// Sets whether the touchpad is disabled (locked).
pub fn set_touchpad_disabled(disabled: bool) -> Result<(), String> {
    let mut current = load_settings();
    current.touchpad_disabled = disabled;
    save_settings(&current);
    apply_touchpad_state(disabled)
}

fn user_settings_file() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config/monster-control-center/general_settings.json"))
}

fn load_settings() -> GeneralSettings {
    if let Ok(content) = fs::read_to_string(SETTINGS_FILE) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    if let Some(user_path) = user_settings_file() {
        if let Ok(content) = fs::read_to_string(user_path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    GeneralSettings::default()
}

fn save_settings(settings: &GeneralSettings) {
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        if let Some(parent) = Path::new(SETTINGS_FILE).parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(SETTINGS_FILE, &data).is_err() {
            if let Some(user_path) = user_settings_file() {
                if let Some(parent) = user_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(user_path, data);
            }
        }
    }
}

fn apply_win_key_state(disabled: bool) -> Result<(), String> {
    let overlay_val = if disabled { "''" } else { "'Super'" };
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.mutter", "overlay-key", overlay_val])
        .output();

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

fn find_device_by_keys(required_keys: &[KeyCode]) -> Option<Device> {
    if let Ok(entries) = fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.to_string_lossy().contains("event") {
                if let Ok(device) = Device::open(&path) {
                    if device.supported_keys().is_some_and(|keys| {
                        required_keys.iter().all(|&k| keys.contains(k))
                    }) {
                        return Some(device);
                    }
                }
            }
        }
    }
    None
}

fn find_touchpad_device() -> Option<Device> {
    find_device_by_keys(&[KeyCode::BTN_TOUCH, KeyCode::BTN_RIGHT])
}

fn find_f24_device() -> Option<Device> {
    find_device_by_keys(&[KeyCode::KEY_F24])
}

fn apply_touchpad_state(disabled: bool) -> Result<(), String> {
    let mut dev_lock = TOUCHPAD_DEV.lock().map_err(|e| e.to_string())?;
    if disabled {
        if dev_lock.is_none() {
            *dev_lock = find_touchpad_device();
        }
        let dev = dev_lock
            .as_mut()
            .ok_or_else(|| "Touchpad cihazı bulunamadı (/dev/input)".to_string())?;

        match dev.grab() {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(16) => {}
            Err(e) => {
                if let Some(mut new_dev) = find_touchpad_device() {
                    new_dev
                        .grab()
                        .map_err(|err| format!("Touchpad kilitlenemedi: {}", err))?;
                    *dev_lock = Some(new_dev);
                } else {
                    return Err(format!("Touchpad kilitlenemedi: {}", e));
                }
            }
        }
        let _ = fs::write(TOUCHPAD_STATE_FILE, "kapali");
    } else {
        if let Some(dev) = dev_lock.as_mut() {
            let _ = dev.ungrab();
        }
        let _ = fs::write(TOUCHPAD_STATE_FILE, "aktif");
    }
    Ok(())
}

/// Starts the passive background listener for physical double-tap corner button (F24).
pub fn start_touchpad_daemon() {
    if DAEMON_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        let initial_settings = load_settings();
        if initial_settings.touchpad_disabled {
            let _ = apply_touchpad_state(true);
        }

        loop {
            let mut f24_device = loop {
                if let Some(dev) = find_f24_device() {
                    break dev;
                }
                std::thread::sleep(Duration::from_secs(2));
            };

            while let Ok(events) = f24_device.fetch_events() {
                for event in events {
                    if let EventSummary::Key(_, KeyCode::KEY_F24, 1) = event.destructure() {
                        let new_disabled = !load_settings().touchpad_disabled;
                        if set_touchpad_disabled(new_disabled).is_ok() {
                            crate::performance::send_notification(
                                "Monster Kontrol Merkezi",
                                if new_disabled {
                                    "Touchpad Kilitlendi"
                                } else {
                                    "Touchpad Açıldı"
                                },
                            );
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_settings_defaults() {
        let settings = GeneralSettings::default();
        assert!(!settings.win_key_disabled);
        assert!(!settings.touchpad_disabled);
    }

    #[test]
    fn test_general_settings_serde() {
        let json = r#"{"win_key_disabled":true,"touchpad_disabled":true}"#;
        let s: GeneralSettings = serde_json::from_str(json).unwrap();
        assert!(s.win_key_disabled);
        assert!(s.touchpad_disabled);
    }
}
