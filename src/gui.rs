//! # Monster Control Center GUI Application
//!
//! Tauri v2 desktop application for single-zone RGB keyboard control and EC management.

use monster_control_center::{battery, device_info, display, ec, keyboard, get_chassis_project_id};
use serde::Serialize;
use std::fs::OpenOptions;
use std::process::Command;

#[derive(Serialize)]
pub struct AppStatus {
    pub has_permission: bool,
    pub chassis_id: u8,
}

fn check_permission() -> bool {
    #[cfg(target_os = "linux")]
    {
        OpenOptions::new().write(true).open(ec::ACPI_CALL_PATH).is_ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

#[tauri::command]
fn get_status() -> Result<AppStatus, String> {
    let has_permission = check_permission();
    let chassis_id = if has_permission {
        get_chassis_project_id().unwrap_or(0)
    } else {
        0
    };
    Ok(AppStatus {
        has_permission,
        chassis_id,
    })
}

#[tauri::command]
fn set_keyboard_preset(preset: String, brightness: u8) -> Result<(), String> {
    keyboard::set_preset(&preset, brightness)
}

#[tauri::command]
fn set_keyboard_rgb(r: u8, g: u8, b: u8, brightness: u8) -> Result<(), String> {
    keyboard::set_rgb(r, g, b, brightness)
}

#[tauri::command]
fn grant_permissions() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let script = r#"
            modprobe acpi_call 2>/dev/null || true
            echo acpi_call > /etc/modules-load.d/acpi_call.conf 2>/dev/null || true
            mkdir -p /etc/tmpfiles.d 2>/dev/null || true
            echo 'z /proc/acpi/call 0666 root root -' > /etc/tmpfiles.d/acpi_call.conf 2>/dev/null || true
            chmod 666 /proc/acpi/call 2>/dev/null || true
            echo 'z /sys/class/powercap/intel-rapl/intel-rapl:0/constraint_*_power_limit_uw 0666 root root -' > /etc/tmpfiles.d/monster_rapl.conf 2>/dev/null || true
            chmod 666 /sys/class/powercap/intel-rapl/intel-rapl:0/constraint_*_power_limit_uw 2>/dev/null || true
            test -w /proc/acpi/call
        "#;
        let output = Command::new("pkexec")
            .arg("bash")
            .arg("-c")
            .arg(script)
            .output()
            .map_err(|e| format!("pkexec çalıştırılamadı: {}", e))?;

        if output.status.success() {
            Ok("Permissions granted and acpi_call module loaded successfully".into())
        } else {
            Err(format!(
                "pkexec failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok("Hardware permissions are managed by system kernel drivers on Windows".into())
    }
}

#[tauri::command]
fn get_device_info() -> Result<device_info::DeviceInfo, String> {
    device_info::get_device_info()
}

#[tauri::command]
fn get_battery_info() -> Result<battery::BatteryInfo, String> {
    battery::get_battery_info()
}

#[tauri::command]
fn get_performance_state() -> Result<monster_control_center::performance::PerformanceState, String> {
    monster_control_center::performance::get_performance_state()
}

#[tauri::command]
fn get_system_metrics() -> Result<monster_control_center::performance::SystemMetrics, String> {
    monster_control_center::performance::get_system_metrics()
}

#[tauri::command]
fn set_hardware_mode(mode: u8) -> Result<(), String> {
    monster_control_center::performance::set_hardware_mode(mode)
}

#[tauri::command]
fn set_os_power_profile(profile: String) -> Result<(), String> {
    monster_control_center::performance::set_os_profile(&profile)
}

#[tauri::command]
fn get_general_settings() -> Result<monster_control_center::general::GeneralSettings, String> {
    Ok(monster_control_center::general::get_general_settings())
}

#[tauri::command]
fn set_win_key_disabled(disabled: bool) -> Result<(), String> {
    monster_control_center::general::set_win_key_disabled(disabled)
}

#[tauri::command]
fn set_touchpad_disabled(disabled: bool) -> Result<(), String> {
    monster_control_center::general::set_touchpad_disabled(disabled)
}

#[tauri::command]
fn get_display_settings() -> Result<display::DisplaySettings, String> {
    Ok(display::get_display_settings())
}

#[tauri::command]
fn apply_display_settings(settings: display::DisplaySettings) -> Result<(), String> {
    display::apply_display_settings(settings)
}

#[tauri::command]
fn reset_display_settings() -> Result<display::DisplaySettings, String> {
    display::reset_display_settings()
}

fn main() {
    monster_control_center::general::ensure_daemon_running();
    monster_control_center::performance::start_polling_daemon();
    if !monster_control_center::general::is_daemon_alive() {
        monster_control_center::general::start_touchpad_daemon();
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_keyboard_preset,
            set_keyboard_rgb,
            grant_permissions,
            get_device_info,
            get_battery_info,
            get_performance_state,
            get_system_metrics,
            set_hardware_mode,
            set_os_power_profile,
            get_general_settings,
            set_win_key_disabled,
            set_touchpad_disabled,
            get_display_settings,
            apply_display_settings,
            reset_display_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
