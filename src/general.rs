//! # General Laptop Settings Module
//!
//! Provides control for chassis-level general settings including
//! Windows Key toggle and Touchpad lock (direct switch & physical corner F24 button listener).

use evdev::{Device, EventSummary, KeyCode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const SETTINGS_FILE: &str = "/etc/txectrl/general_settings.json";

static DAEMON_STARTED: AtomicBool = AtomicBool::new(false);
static IS_DAEMON_SERVER: AtomicBool = AtomicBool::new(false);
static IS_TOUCHPAD_DISABLED: AtomicBool = AtomicBool::new(false);

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

/// Retrieves the socket path for the background daemon IPC.
pub fn socket_path() -> PathBuf {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(runtime);
        if p.is_dir() {
            return p.join("txectrl.sock");
        }
    }
    PathBuf::from("/dev/shm/txectrl.sock")
}

/// Sends a command to the background daemon via Unix domain socket and reads one line response.
pub fn send_daemon_command(cmd: &str) -> Result<String, String> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .map_err(|e| e.to_string())?;
    stream
        .write_all(format!("{}\n", cmd).as_bytes())
        .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).map_err(|e| e.to_string())?;
    Ok(response.trim().to_string())
}

/// Checks if the background daemon is alive and responding.
pub fn is_daemon_alive() -> bool {
    send_daemon_command("ping").map(|r| r == "pong").unwrap_or(false)
}

/// Queries the desktop environment (GNOME/mutter) directly as the single source of truth.
pub fn is_touchpad_disabled_system() -> bool {
    let mut cmd = Command::new("gsettings");
    cmd.args(["get", "org.gnome.desktop.peripherals.touchpad", "send-events"]);
    setup_desktop_env(&mut cmd);
    if let Ok(output) = cmd.output() {
        let text = String::from_utf8_lossy(&output.stdout);
        return text.contains("disabled");
    }
    IS_TOUCHPAD_DISABLED.load(Ordering::SeqCst)
}

/// Retrieves the current status of general settings, reading live desktop state.
pub fn get_general_settings() -> GeneralSettings {
    let mut settings = load_settings();
    settings.win_key_disabled = is_win_key_disabled_system();
    settings.touchpad_disabled = is_touchpad_disabled_system();
    settings
}

/// Applies Windows key disabled state locally within this process.
pub fn set_win_key_disabled_local(disabled: bool) -> Result<(), String> {
    let mut current = load_settings();
    current.win_key_disabled = disabled;
    save_settings(&current);
    apply_win_key_state(disabled)
}

/// Sets whether the Windows key is disabled. Delegates to daemon if running, else applies locally.
pub fn set_win_key_disabled(disabled: bool) -> Result<(), String> {
    if !IS_DAEMON_SERVER.load(Ordering::SeqCst) {
        let cmd = if disabled { "set_win:1" } else { "set_win:0" };
        if let Ok(resp) = send_daemon_command(cmd) {
            if resp == "ok" {
                return Ok(());
            }
        }
    }

    set_win_key_disabled_local(disabled)
}

/// Applies touchpad state locally within this process.
pub fn set_touchpad_disabled_local(disabled: bool) -> Result<(), String> {
    let mut current = load_settings();
    current.touchpad_disabled = disabled;
    save_settings(&current);
    apply_touchpad_state(disabled)
}

/// Sets whether the touchpad is disabled. Delegates to daemon if running, else applies locally.
pub fn set_touchpad_disabled(disabled: bool) -> Result<(), String> {
    if !IS_DAEMON_SERVER.load(Ordering::SeqCst) {
        let cmd = if disabled { "set:1" } else { "set:0" };
        if let Ok(resp) = send_daemon_command(cmd) {
            if resp == "ok" {
                return Ok(());
            }
        }
    }

    set_touchpad_disabled_local(disabled)
}

/// Toggles touchpad disabled state.
pub fn toggle_touchpad() -> Result<bool, String> {
    let current_dis = get_general_settings().touchpad_disabled;
    let new_state = !current_dis;
    set_touchpad_disabled(new_state)?;
    Ok(new_state)
}

/// Ensures background daemon is running via systemd user service or process spawn.
pub fn ensure_daemon_running() {
    if is_daemon_alive() {
        return;
    }

    let _ = Command::new("systemctl")
        .args(["--user", "start", "txectrl.service"])
        .status();

    for _ in 0..5 {
        std::thread::sleep(Duration::from_millis(80));
        if is_daemon_alive() {
            return;
        }
    }

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("txectrl")));

    let bin = exe.filter(|p| p.exists()).unwrap_or_else(|| {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".local/bin/txectrl");
            if p.exists() {
                return p;
            }
        }
        PathBuf::from("txectrl")
    });

    let _ = Command::new(bin).arg("daemon").spawn();
}

/// Runs the IPC server for the standalone background daemon.
pub fn run_daemon_server() {
    IS_DAEMON_SERVER.store(true, Ordering::SeqCst);
    let path = socket_path();
    let _ = fs::remove_file(&path);

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[daemon] UnixListener bind error on {}: {}", path.display(), e);
            return;
        }
    };

    println!("[daemon] IPC socket dinleniyor: {}", path.display());

    for stream in listener.incoming().flatten() {
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() {
            let cmd = line.trim();
            let mut writer = &stream;
            match cmd {
                "ping" => {
                    let _ = writer.write_all(b"pong\n");
                }
                "get" => {
                    let is_dis = is_touchpad_disabled_system();
                    let resp = if is_dis { b"1\n" } else { b"0\n" };
                    let _ = writer.write_all(resp);
                }
                "set:1" => {
                    let res = set_touchpad_disabled_local(true);
                    let resp = if res.is_ok() { "ok\n" } else { "err\n" };
                    let _ = writer.write_all(resp.as_bytes());
                }
                "set:0" => {
                    let res = set_touchpad_disabled_local(false);
                    let resp = if res.is_ok() { "ok\n" } else { "err\n" };
                    let _ = writer.write_all(resp.as_bytes());
                }
                "toggle" => {
                    let res = toggle_touchpad();
                    let resp = if res.is_ok() { "ok\n" } else { "err\n" };
                    let _ = writer.write_all(resp.as_bytes());
                }
                "set_win:1" => {
                    let res = set_win_key_disabled_local(true);
                    let resp = if res.is_ok() { "ok\n" } else { "err\n" };
                    let _ = writer.write_all(resp.as_bytes());
                }
                "set_win:0" => {
                    let res = set_win_key_disabled_local(false);
                    let resp = if res.is_ok() { "ok\n" } else { "err\n" };
                    let _ = writer.write_all(resp.as_bytes());
                }
                _ => {
                    let _ = writer.write_all(b"unknown\n");
                }
            }
            let _ = writer.flush();
        }
    }
}

fn user_settings_file() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config/txectrl/general_settings.json"))
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

/// Queries whether the Windows (Super) key is disabled across supported desktop environments or hardware.
pub fn is_win_key_disabled_system() -> bool {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_uppercase();

    if desktop.contains("KDE") {
        for reader in ["kreadconfig6", "kreadconfig5"] {
            let mut kde_cmd = Command::new(reader);
            kde_cmd.args(["--file", "kwinrc", "--group", "ModifierOnlyShortcuts", "--key", "Meta"]);
            setup_desktop_env(&mut kde_cmd);
            if let Ok(output) = kde_cmd.output() {
                if output.status.success() {
                    return String::from_utf8_lossy(&output.stdout).trim().is_empty();
                }
            }
        }
    } else if desktop.contains("GNOME") {
        let mut cmd = Command::new("gsettings");
        cmd.args(["get", "org.gnome.mutter", "overlay-key"]);
        setup_desktop_env(&mut cmd);
        if let Ok(output) = cmd.output() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.contains("''") || text == "''" {
                return true;
            }
            if text.contains("Super") {
                return false;
            }
        }
    }

    let fn_lock_path = "/sys/devices/platform/tuxedo_keyboard/fn_lock";
    if let Ok(content) = fs::read_to_string(fn_lock_path) {
        return content.trim() == "1";
    }

    false
}

fn apply_win_key_state(disabled: bool) -> Result<(), String> {
    let fn_lock_path = "/sys/devices/platform/tuxedo_keyboard/fn_lock";
    if Path::new(fn_lock_path).exists() {
        let val = if disabled { "1" } else { "0" };
        let _ = fs::write(fn_lock_path, val);
    }

    // 1. GNOME / Mutter (Wayland & X11)
    let overlay_val = if disabled { "''" } else { "'Super'" };
    let mut gnome_cmd = Command::new("gsettings");
    gnome_cmd.args(["set", "org.gnome.mutter", "overlay-key", overlay_val]);
    setup_desktop_env(&mut gnome_cmd);
    let _ = gnome_cmd.output();

    // 2. KDE Plasma (Wayland & X11)
    let kde_writers = ["kwriteconfig6", "kwriteconfig5"];
    let meta_val = if disabled { "" } else { "org.kde.plasmashell,/PlasmaShell,org.kde.PlasmaShell,activateLauncherMenu" };
    for writer in kde_writers {
        let mut kcmd = Command::new(writer);
        kcmd.args(["--file", "kwinrc", "--group", "ModifierOnlyShortcuts", "--key", "Meta", meta_val]);
        setup_desktop_env(&mut kcmd);
        if let Ok(status) = kcmd.status() {
            if status.success() {
                let mut reconf = Command::new("qdbus");
                reconf.args(["org.kde.KWin", "/KWin", "reconfigure"]);
                setup_desktop_env(&mut reconf);
                let _ = reconf.status();
                break;
            }
        }
    }

    // 3. Standalone X11 environments (XFCE, i3, etc.)
    if std::env::var("DISPLAY").is_ok() && std::env::var("WAYLAND_DISPLAY").is_err() {
        let key_mapping = if disabled { "keycode 133 = " } else { "keycode 133 = Super_L NoSymbol Super_L" };
        let mut xmod = Command::new("xmodmap");
        xmod.args(["-e", key_mapping]);
        let _ = xmod.output();
    }

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

fn setup_desktop_env(cmd: &mut Command) {
    if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        let bus_path = format!("/run/user/{}/bus", uid);
        let runtime_path = format!("/run/user/{}", uid);
        if Path::new(&bus_path).exists() {
            cmd.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", bus_path));
        }
        if Path::new(&runtime_path).exists() {
            cmd.env("XDG_RUNTIME_DIR", runtime_path);
        }
    }
}

fn run_desktop_touchpad_toggle(disabled: bool) {
    let mode = if disabled { "disabled" } else { "enabled" };
    let mut cmd = Command::new("gsettings");
    cmd.args(["set", "org.gnome.desktop.peripherals.touchpad", "send-events", mode]);
    setup_desktop_env(&mut cmd);
    let _ = cmd.output();
}

fn find_f24_device() -> Option<Device> {
    find_device_by_keys(&[KeyCode::KEY_F24])
}

fn apply_touchpad_state(disabled: bool) -> Result<(), String> {
    run_desktop_touchpad_toggle(disabled);
    IS_TOUCHPAD_DISABLED.store(disabled, Ordering::SeqCst);
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
                        let _ = toggle_touchpad();
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
