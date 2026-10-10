//! # General Laptop Settings Module
//!
//! Provides control for chassis-level general settings including
//! Windows Key toggle and Touchpad lock (direct switch & physical corner F24 button listener).

#[cfg(target_os = "linux")]
use evdev::{Device, EventSummary, KeyCode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "linux")]
use std::sync::Mutex;
use std::time::Duration;

const SETTINGS_FILE: &str = "/etc/monster-control-center/general_settings.json";
const TOUCHPAD_STATE_FILE: &str = "/dev/shm/touchpad_locker_state";

#[cfg(target_os = "linux")]
static TOUCHPAD_DEVS: Mutex<Vec<Device>> = Mutex::new(Vec::new());
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
        PathBuf::from(runtime).join("monster_control.sock")
    } else {
        PathBuf::from("/dev/shm/monster_control.sock")
    }
}

/// Sends a command to the background daemon via Unix domain socket and reads one line response.
#[cfg(unix)]
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

#[cfg(not(unix))]
pub fn send_daemon_command(_cmd: &str) -> Result<String, String> {
    Err("Daemon IPC is only supported on Unix systems".to_string())
}

/// Checks if the background daemon is alive and responding.
pub fn is_daemon_alive() -> bool {
    send_daemon_command("ping").map(|r| r == "pong").unwrap_or(false)
}

/// Retrieves the current status of general settings, querying daemon if running.
pub fn get_general_settings() -> GeneralSettings {
    let mut settings = load_settings();

    // In client (GUI): live lock state is tracked by the daemon
    if !IS_DAEMON_SERVER.load(Ordering::SeqCst) {
        if let Ok(resp) = send_daemon_command("get") {
            settings.touchpad_disabled = resp == "1";
            return settings;
        }
    }

    // In daemon or standalone: use atomic memory state
    settings.touchpad_disabled = IS_TOUCHPAD_DISABLED.load(Ordering::SeqCst);
    settings
}

/// Sets whether the Windows key is disabled.
pub fn set_win_key_disabled(disabled: bool) -> Result<(), String> {
    let mut current = load_settings();
    current.win_key_disabled = disabled;
    save_settings(&current);
    apply_win_key_state(disabled)
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
    let new_state = !IS_TOUCHPAD_DISABLED.load(Ordering::SeqCst);
    set_touchpad_disabled_local(new_state)?;
    Ok(new_state)
}

/// Ensures background daemon is running via systemd user service or process spawn.
pub fn ensure_daemon_running() {
    #[cfg(unix)]
    {
        if is_daemon_alive() {
            return;
        }

        let _ = Command::new("systemctl")
            .args(["--user", "start", "monster-control-center.service"])
            .status();

        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(80));
            if is_daemon_alive() {
                return;
            }
        }

        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("monster-control-center")));

        let bin = exe.filter(|p| p.exists()).unwrap_or_else(|| {
            if let Ok(home) = std::env::var("HOME") {
                let p = PathBuf::from(home).join(".local/bin/monster-control-center");
                if p.exists() {
                    return p;
                }
            }
            PathBuf::from("monster-control-center")
        });

        let _ = Command::new(bin).arg("daemon").spawn();
    }
}

/// Runs the IPC server for the standalone background daemon.
#[cfg(unix)]
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
                    let is_dis = IS_TOUCHPAD_DISABLED.load(Ordering::SeqCst);
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
                _ => {
                    let _ = writer.write_all(b"unknown\n");
                }
            }
            let _ = writer.flush();
        }
    }
}

#[cfg(not(unix))]
pub fn run_daemon_server() {}

fn user_settings_file() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(PathBuf::from(appdata).join("MonsterControlCenter").join("general_settings.json"));
        }
    }
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
    #[cfg(target_os = "linux")]
    {
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
    }
    let _ = disabled;
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_desktop_touchpad_toggle(disabled: bool) {
    let mode = if disabled { "disabled" } else { "enabled" };
    let mut cmd = Command::new("gsettings");
    cmd.args(["set", "org.gnome.desktop.peripherals.touchpad", "send-events", mode]);

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
    let _ = cmd.output();
}

#[cfg(target_os = "linux")]
fn find_touchpad_devices() -> Vec<Device> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.to_string_lossy().contains("event") {
                if let Ok(device) = Device::open(&path) {
                    let name = device.name().unwrap_or("").to_lowercase();
                    if name.contains("virtual") || name.contains("uinput") {
                        continue;
                    }
                    let is_touchpad = name.contains("touchpad")
                        || (name.contains("uniw") && name.contains("mouse"))
                        || device.supported_keys().is_some_and(|keys| keys.contains(KeyCode::BTN_TOUCH));

                    if is_touchpad {
                        devices.push(device);
                    }
                }
            }
        }
    }
    devices
}

#[cfg(target_os = "linux")]
fn find_f24_device() -> Option<Device> {
    if let Ok(entries) = fs::read_dir("/dev/input") {
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.to_string_lossy().contains("event") {
                if let Ok(device) = Device::open(&path) {
                    let name = device.name().unwrap_or("").to_lowercase();
                    if name.contains("virtual") || name.contains("uinput") {
                        continue;
                    }
                    if device.supported_keys().is_some_and(|keys| keys.contains(KeyCode::KEY_F24)) {
                        candidates.push((name, device));
                    }
                }
            }
        }
        // Prefer internal AT keyboard / serio keyboard
        if let Some(pos) = candidates.iter().position(|(name, _)| name.contains("at translated") || name.contains("keyboard")) {
            return Some(candidates.swap_remove(pos).1);
        }
        if let Some((_, dev)) = candidates.pop() {
            return Some(dev);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn apply_touchpad_state(disabled: bool) -> Result<(), String> {
    // 1. Wayland / GNOME desktop-level touchpad disable/enable
    run_desktop_touchpad_toggle(disabled);

    // 2. evdev hardware grab on all touchpad / UNIW mouse devices
    let mut devs_lock = TOUCHPAD_DEVS.lock().map_err(|e| e.to_string())?;
    if disabled {
        if devs_lock.is_empty() {
            *devs_lock = find_touchpad_devices();
        }
        for dev in devs_lock.iter_mut() {
            let _ = dev.grab();
        }
        IS_TOUCHPAD_DISABLED.store(true, Ordering::SeqCst);
        let _ = fs::write(TOUCHPAD_STATE_FILE, "kapali");
    } else {
        for mut dev in devs_lock.drain(..) {
            let _ = dev.ungrab();
        }
        IS_TOUCHPAD_DISABLED.store(false, Ordering::SeqCst);
        let _ = fs::write(TOUCHPAD_STATE_FILE, "aktif");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn apply_touchpad_state(disabled: bool) -> Result<(), String> {
    IS_TOUCHPAD_DISABLED.store(disabled, Ordering::SeqCst);
    Ok(())
}

/// Starts the passive background listener for physical double-tap corner button (F24).
pub fn start_touchpad_daemon() {
    #[cfg(target_os = "linux")]
    {
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
                        if let Some(name) = dev.name() {
                            eprintln!("[touchpad_daemon] F24 buton dinleyici bağlandı: {}", name);
                        }
                        break dev;
                    }
                    std::thread::sleep(Duration::from_secs(2));
                };

                while let Ok(events) = f24_device.fetch_events() {
                    for event in events {
                        if let EventSummary::Key(_, KeyCode::KEY_F24, 1) = event.destructure() {
                            eprintln!("[touchpad_daemon] F24 tuşu algılandı, touchpad durumu değiştiriliyor...");
                            let _ = toggle_touchpad();
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        });
    }
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
