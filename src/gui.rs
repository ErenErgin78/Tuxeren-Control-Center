//! # Monster Control Center GUI Application
//!
//! Tauri v2 desktop application for single-zone RGB keyboard control and EC management.

use monster_control_center::{battery, device_info, ec, keyboard, get_chassis_project_id};
use serde::Serialize;
use std::fs::OpenOptions;
use std::process::Command;

#[derive(Serialize)]
pub struct AppStatus {
    pub has_permission: bool,
    pub chassis_id: u8,
}

fn check_permission() -> bool {
    OpenOptions::new().write(true).open(ec::ACPI_CALL_PATH).is_ok()
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
    let script = r#"
        modprobe acpi_call 2>/dev/null || insmod /home/eren/Depo/ReverseEngineering/py_deneme/acpi_call/acpi_call.ko 2>/dev/null || true
        mkdir -p /lib/modules/$(uname -r)/extra 2>/dev/null || true
        if [ -f /home/eren/Depo/ReverseEngineering/py_deneme/acpi_call/acpi_call.ko ]; then
            cp -f /home/eren/Depo/ReverseEngineering/py_deneme/acpi_call/acpi_call.ko /lib/modules/$(uname -r)/extra/ 2>/dev/null || true
            depmod -a 2>/dev/null || true
        fi
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
        .map_err(|e| format!("Failed to run pkexec: {}", e))?;

    if output.status.success() {
        Ok("Permissions granted and acpi_call module loaded successfully".into())
    } else {
        Err(format!(
            "pkexec failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
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
fn set_hardware_mode(mode: u8) -> Result<(), String> {
    monster_control_center::performance::set_hardware_mode(mode)
}

#[tauri::command]
fn set_os_power_profile(profile: String) -> Result<(), String> {
    monster_control_center::performance::set_os_profile(&profile)
}

fn main() {
    monster_control_center::performance::start_polling_daemon();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_keyboard_preset,
            set_keyboard_rgb,
            grant_permissions,
            get_device_info,
            get_battery_info,
            get_performance_state,
            set_hardware_mode,
            set_os_power_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
