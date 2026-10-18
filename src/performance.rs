//! # Performance Management Module
//!
//! Controls hardware Embedded Controller (EC) performance profiles,
//! synchronizes Linux OS power profiles (tuned / D-Bus),
//! applies Intel RAPL power limits and CPU EPP policies,
//! and polls the hardware mode button.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

static PREV_CPU: Mutex<Option<(u64, u64)>> = Mutex::new(None);
static PREV_NET: Mutex<Option<(u64, u64, Instant)>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub ram_percent: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub cpu_percent: f32,
    pub cpu_temp: u32,
    pub gpu_percent: f32,
    pub gpu_temp: u32,
    pub cpu_fan_rpm: u16,
    pub cpu_fan_duty: u8,
    pub gpu_fan_rpm: u16,
    pub gpu_fan_duty: u8,
    pub disk_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub net_down_bytes: u64,
    pub net_up_bytes: u64,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceMode {
    Office = 0,
    Gaming = 1,
    Turbo  = 2,
}

impl PerformanceMode {
    /// Resolves PerformanceMode from EC fan/LED control byte (0x0751).
    pub fn from_fan_byte(val: u8) -> Option<Self> {
        match val {
            0xA0 => Some(PerformanceMode::Office),
            0x20 => Some(PerformanceMode::Gaming),
            0x30 | 0x10 => Some(PerformanceMode::Turbo),
            _ => None,
        }
    }

    /// Resolves PerformanceMode from index or UI code.
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 | 3 => Some(PerformanceMode::Office),
            1 => Some(PerformanceMode::Gaming),
            2 | 4 => Some(PerformanceMode::Turbo),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PerformanceMode::Office => "Ofis (Sessiz)",
            PerformanceMode::Gaming => "Oyun (Standart)",
            PerformanceMode::Turbo => "Turbo (Canavar)",
        }
    }

    /// Returns (PL1 Watts, PL2 Watts, CPU EPP policy, Fan/LED Byte 0x0751)
    pub fn config(&self) -> (u8, u8, &'static str, u8) {
        // ponytail: PL defaults (45/75/90W) match chassis targets; make user-configurable if multi-TDP models needed
        match self {
            PerformanceMode::Office => (45, 45, "power", 0xA0),
            PerformanceMode::Gaming => (75, 75, "balance_performance", 0x20),
            PerformanceMode::Turbo => (90, 90, "performance", 0x30),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceState {
    pub hardware_mode: u8,
    pub hardware_mode_name: String,
    pub os_profile: String,
}

/// Reads the current hardware performance mode from EC register 0x0751 or fallback.
pub fn get_hardware_mode() -> Result<u8, String> {
    if let Ok(fan_byte) = crate::ec::read_byte(0x0751) {
        if let Some(mode) = PerformanceMode::from_fan_byte(fan_byte) {
            return Ok(mode as u8);
        }
    }

    let profile_path = "/sys/firmware/acpi/platform_profile";
    if let Ok(content) = fs::read_to_string(profile_path) {
        let mode = match content.trim() {
            "performance" => PerformanceMode::Turbo,
            "balanced" => PerformanceMode::Gaming,
            "low-power" => PerformanceMode::Office,
            _ => PerformanceMode::Office,
        };
        return Ok(mode as u8);
    }

    Ok(PerformanceMode::Office as u8)
}

/// Linux çekirdeğine doğrudan Intel RAPL güç limitlerini uygular (microwatts).
pub fn apply_kernel_rapl(pl1_watts: u32, pl2_watts: u32) -> Result<(), String> {
    let rapl_path = "/sys/class/powercap/intel-rapl/intel-rapl:0";
    if !Path::new(rapl_path).exists() {
        return Ok(()); // Non-Intel or RAPL not supported
    }

    let pl0_file = format!("{rapl_path}/constraint_0_power_limit_uw");
    let pl1_file = format!("{rapl_path}/constraint_1_power_limit_uw");

    fs::write(&pl0_file, (pl1_watts * 1_000_000).to_string())
        .map_err(|e| format!("RAPL PL1 yazılamadı ({}): {}", pl0_file, e))?;

    fs::write(&pl1_file, (pl2_watts * 1_000_000).to_string())
        .map_err(|e| format!("RAPL PL2 yazılamadı ({}): {}", pl1_file, e))?;

    Ok(())
}

/// Linux CPU frekans ölçekleyici profilini (EPP) belirler.
pub fn set_cpu_epp(policy: &str) -> Result<(), String> {
    let base = Path::new("/sys/devices/system/cpu");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path().join("cpufreq/energy_performance_preference");
            if path.exists() {
                let _ = fs::write(path, policy);
            }
        }
    }
    Ok(())
}

/// Reads the active OS power profile from D-Bus (`net.hadess.PowerProfiles`).
pub fn get_os_profile() -> Result<String, String> {
    let out = Command::new("busctl")
        .args([
            "get-property",
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
            "net.hadess.PowerProfiles",
            "ActiveProfile",
        ])
        .output()
        .map_err(|e| format!("D-Bus erişim hatası: {}", e))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let profile = text
        .trim()
        .strip_prefix("s ")
        .unwrap_or(text.trim())
        .trim_matches('"')
        .to_string();

    Ok(profile)
}

/// Sets the OS power profile via D-Bus (`net.hadess.PowerProfiles`).
pub fn set_os_profile(profile: &str) -> Result<(), String> {
    let valid = ["power-saver", "balanced", "performance"];
    if !valid.contains(&profile) {
        return Err(format!(
            "Geçersiz OS güç kipi: '{}'. Geçerli: {:?}",
            profile, valid
        ));
    }

    let out = Command::new("busctl")
        .args([
            "set-property",
            "net.hadess.PowerProfiles",
            "/net/hadess/PowerProfiles",
            "net.hadess.PowerProfiles",
            "ActiveProfile",
            "s",
            profile,
        ])
        .output()
        .map_err(|e| format!("D-Bus profil ayarlama hatası: {}", e))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    Ok(())
}

/// Mod değişimini EC donanım kayıtlarına (0x0751, 0x046A, 0x046B, 0x07AB) ve Kernel RAPL limitlerine uygular.
/// OS güç kipi (D-Bus / tuned-ppd) ile çakışmaması için platform_profile dosyasına dokunmaz.
pub fn apply_mode(mode: PerformanceMode) -> Result<(), String> {
    let (pl1, pl2, epp, fan_byte) = mode.config();

    // 1. EC Donanım Düzeyi Kayıtları (Fan eğrisi, LED rengi ve PL sınırları)
    let _ = crate::ec::write_byte(0x0751, fan_byte);
    let _ = crate::ec::write_byte(0x046A, pl1);
    let _ = crate::ec::write_byte(0x046B, pl2);
    let _ = crate::ec::write_byte(0x07AB, mode as u8);

    // 2. Linux Kernel Intel RAPL Güç Sınırları
    if let Err(e) = apply_kernel_rapl(pl1 as u32, pl2 as u32) {
        eprintln!("[warn] Kernel RAPL: {}", e);
    }

    // 3. Linux CPU EPP Politikası
    let _ = set_cpu_epp(epp);

    Ok(())
}

/// Sets the hardware mode by raw u8 code (0=Office, 1=Gaming, 2=Turbo).
pub fn set_hardware_mode(mode: u8) -> Result<(), String> {
    if let Some(perf_mode) = PerformanceMode::from_u8(mode) {
        apply_mode(perf_mode)
    } else {
        Err(format!(
            "Geçersiz performans modu: {}. Geçerli: 0 (Ofis), 1 (Oyun), 2 (Turbo)",
            mode
        ))
    }
}

/// Retrieves complete performance state combining both hardware EC and OS D-Bus.
pub fn get_performance_state() -> Result<PerformanceState, String> {
    let hw = get_hardware_mode().unwrap_or(0);
    let hw_name = PerformanceMode::from_u8(hw)
        .map(|m| m.name().to_string())
        .unwrap_or_else(|| "Bilinmiyor".to_string());

    let os = get_os_profile().unwrap_or_else(|_| "Bilinmiyor".to_string());

    Ok(PerformanceState {
        hardware_mode: hw,
        hardware_mode_name: hw_name,
        os_profile: os,
    })
}

fn get_ram_metrics() -> (f32, f32, f32) {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0.0;
        let mut avail_kb = 0.0;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    avail_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            }
            if total_kb > 0.0 && avail_kb > 0.0 {
                break;
            }
        }
        if total_kb > 0.0 {
            let used_kb = total_kb - avail_kb;
            let total_gb = (total_kb / 1024.0 / 1024.0) as f32;
            let used_gb = (used_kb / 1024.0 / 1024.0) as f32;
            let percent = ((used_kb / total_kb) * 100.0) as f32;
            return (percent, used_gb, total_gb);
        }
    }
    (0.0, 0.0, 0.0)
}

fn get_cpu_usage() -> f32 {
    let stat = match fs::read_to_string("/proc/stat") {
        Ok(s) => s,
        Err(_) => return 0.0,
    };
    let first_line = match stat.lines().next() {
        Some(l) if l.starts_with("cpu ") => l,
        _ => return 0.0,
    };
    let parts: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    if parts.len() < 4 {
        return 0.0;
    }
    let idle = parts[3] + parts.get(4).copied().unwrap_or(0);
    let total: u64 = parts.iter().sum();

    let mut prev = PREV_CPU.lock().unwrap();
    match *prev {
        Some((p_idle, p_total)) => {
            *prev = Some((idle, total));
            let d_total = total.saturating_sub(p_total);
            let d_idle = idle.saturating_sub(p_idle);
            if d_total > 0 {
                let usage = ((d_total - d_idle) as f32 / d_total as f32) * 100.0;
                usage.clamp(0.0, 100.0)
            } else {
                0.0
            }
        }
        None => {
            *prev = Some((idle, total));
            0.0
        }
    }
}

fn get_cpu_temp() -> u32 {
    let base = Path::new("/sys/class/hwmon");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                if name.trim() == "coretemp" {
                    if let Ok(temp_str) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(millidegrees) = temp_str.trim().parse::<u32>() {
                            return millidegrees / 1000;
                        }
                    }
                }
            }
        }
    }
    if let Ok(temp_str) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(millidegrees) = temp_str.trim().parse::<u32>() {
            return millidegrees / 1000;
        }
    }
    0
}

fn get_gpu_metrics() -> (f32, u32) {
    if let Ok(out) = Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu,temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = text.trim().split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            let util = parts[0].parse::<f32>().unwrap_or(0.0);
            let temp = parts[1].parse::<u32>().unwrap_or(0);
            return (util, temp);
        }
    }
    (0.0, 0)
}

fn find_hwmon_fans() -> (Option<PathBuf>, Option<PathBuf>) {
    let hwmon_dir = Path::new("/sys/class/hwmon");
    let mut fan1 = None;
    let mut fan2 = None;

    if let Ok(entries) = hwmon_dir.read_dir() {
        for entry in entries.flatten() {
            let p = entry.path();
            let f1 = p.join("fan1_input");
            let f2 = p.join("fan2_input");
            if f1.exists() && fan1.is_none() {
                fan1 = Some(f1);
            }
            if f2.exists() && fan2.is_none() {
                fan2 = Some(f2);
            }
        }
    }
    (fan1, fan2)
}

fn get_fan_metrics() -> (u16, u8, u16, u8) {
    let (fan1_path, fan2_path) = find_hwmon_fans();

    let cpu_rpm = fan1_path
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(0);

    let gpu_rpm = fan2_path
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(0);

    let cpu_duty = ((cpu_rpm as f32 / 5500.0) * 100.0).round().clamp(0.0, 100.0) as u8;
    let gpu_duty = ((gpu_rpm as f32 / 5500.0) * 100.0).round().clamp(0.0, 100.0) as u8;

    (cpu_rpm, cpu_duty, gpu_rpm, gpu_duty)
}

fn get_disk_metrics() -> (f32, f32, f32) {
    if let Ok(output) = Command::new("df").args(["-k", "/"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().rev() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 5 {
                if let (Ok(total_kb), Ok(used_kb)) = (tokens[1].parse::<f64>(), tokens[2].parse::<f64>()) {
                    let total_gb = (total_kb / 1024.0 / 1024.0) as f32;
                    let used_gb = (used_kb / 1024.0 / 1024.0) as f32;
                    let percent = if total_gb > 0.0 { (used_gb / total_gb) * 100.0 } else { 0.0 };
                    return (percent, used_gb, total_gb);
                }
            }
        }
    }
    (0.0, 0.0, 0.0)
}

fn get_net_speed() -> (u64, u64) {
    let mut curr_rx = 0u64;
    let mut curr_tx = 0u64;

    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines() {
            if let Some((iface, data)) = line.split_once(':') {
                let name = iface.trim();
                if name == "lo" {
                    continue;
                }
                let parts: Vec<&str> = data.split_whitespace().collect();
                if parts.len() >= 9 {
                    let rx = parts[0].parse::<u64>().unwrap_or(0);
                    let tx = parts[8].parse::<u64>().unwrap_or(0);
                    curr_rx += rx;
                    curr_tx += tx;
                }
            }
        }
    }

    let now = Instant::now();
    let mut prev = PREV_NET.lock().unwrap();
    match *prev {
        Some((p_rx, p_tx, p_time)) => {
            let dt = now.duration_since(p_time).as_secs_f32();
            *prev = Some((curr_rx, curr_tx, now));
            if dt > 0.05 {
                let rx_speed = ((curr_rx.saturating_sub(p_rx)) as f32 / dt) as u64;
                let tx_speed = ((curr_tx.saturating_sub(p_tx)) as f32 / dt) as u64;
                (rx_speed, tx_speed)
            } else {
                (0, 0)
            }
        }
        None => {
            *prev = Some((curr_rx, curr_tx, now));
            (0, 0)
        }
    }
}

/// Gathers real-time live system and hardware telemetry metrics.
pub fn get_system_metrics() -> Result<SystemMetrics, String> {
    let (ram_percent, ram_used_gb, ram_total_gb) = get_ram_metrics();
    let cpu_percent = get_cpu_usage();
    let cpu_temp = get_cpu_temp();
    let (gpu_percent, gpu_temp) = get_gpu_metrics();
    let (cpu_fan_rpm, cpu_fan_duty, gpu_fan_rpm, gpu_fan_duty) = get_fan_metrics();
    let (disk_percent, disk_used_gb, disk_total_gb) = get_disk_metrics();
    let (net_down_bytes, net_up_bytes) = get_net_speed();

    Ok(SystemMetrics {
        ram_percent,
        ram_used_gb,
        ram_total_gb,
        cpu_percent,
        cpu_temp,
        gpu_percent,
        gpu_temp,
        cpu_fan_rpm,
        cpu_fan_duty,
        gpu_fan_rpm,
        gpu_fan_duty,
        disk_percent,
        disk_used_gb,
        disk_total_gb,
        net_down_bytes,
        net_up_bytes,
    })
}

/// No-op in modern driver mode: power profile switching is handled natively by Linux platform_profile and power-profiles-daemon.
pub fn start_polling_daemon() {
    // Deprecated: Linux kernel platform_profile handles profiles natively without polling.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_mappings() {
        assert_eq!(PerformanceMode::from_u8(0), Some(PerformanceMode::Office));
        assert_eq!(PerformanceMode::from_u8(1), Some(PerformanceMode::Gaming));
        assert_eq!(PerformanceMode::from_u8(2), Some(PerformanceMode::Turbo));
        assert_eq!(PerformanceMode::from_fan_byte(0xA0), Some(PerformanceMode::Office));
        assert_eq!(PerformanceMode::from_fan_byte(0x20), Some(PerformanceMode::Gaming));
        assert_eq!(PerformanceMode::from_fan_byte(0x30), Some(PerformanceMode::Turbo));
    }

    #[test]
    fn test_system_metrics() {
        let metrics = get_system_metrics().expect("Failed to retrieve system metrics");
        assert!(metrics.ram_total_gb > 0.0);
        assert!(metrics.ram_percent >= 0.0 && metrics.ram_percent <= 100.0);
        assert!(metrics.disk_total_gb > 0.0);
    }
}
