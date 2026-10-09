//! # Performance Management Module
//!
//! Controls hardware Embedded Controller (EC) performance profiles,
//! synchronizes Linux OS power profiles (tuned / D-Bus),
//! applies Intel RAPL power limits and CPU EPP policies,
//! and polls the hardware mode button with OSD desktop notifications.

use crate::ec;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub const ADDR_PERF_MODE: u16 = 0x07AB;
pub const ADDR_MAFAN_CONTROL: u16 = 0x0751;
pub const ADDR_PL1_CONTROL: u16 = 0x046A;
pub const ADDR_PL2_CONTROL: u16 = 0x046B;

static DAEMON_RUNNING: AtomicBool = AtomicBool::new(false);

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

/// Reads the current hardware performance mode from the EC.
/// Uses 0x0751 which directly controls physical LED state and fan profile.
pub fn get_hardware_mode() -> Result<u8, String> {
    let fan_byte = ec::read_byte(ADDR_MAFAN_CONTROL)?;
    if let Some(mode) = PerformanceMode::from_fan_byte(fan_byte) {
        Ok(mode as u8)
    } else {
        let mode_byte = ec::read_byte(ADDR_PERF_MODE).unwrap_or(0);
        Ok(PerformanceMode::from_u8(mode_byte).map(|m| m as u8).unwrap_or(0))
    }
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

/// Masaüstü OSD bildirimi gönderir (root veya kullanıcı oturumu fark etmeksizin).
pub fn send_notification(title: &str, body: &str) {
    let mut cmd = Command::new("notify-send");
    cmd.args(["-a", title, "-i", "preferences-system-power", title, body]);

    if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
        if Path::new("/run/user/1000/bus").exists() {
            cmd.env("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/user/1000/bus");
        }
        if Path::new("/run/user/1000").exists() {
            cmd.env("XDG_RUNTIME_DIR", "/run/user/1000");
        }
    }
    let _ = cmd.output();
}

/// Mod değişimini EC'ye, fan/LED moduna ve donanım PL limitlerine uygular.
pub fn apply_mode(mode: PerformanceMode) -> Result<(), String> {
    let (pl1, pl2, epp, fan_byte) = mode.config();

    // 1. EC Fan & LED Kontrol Register (0x0751: 0xA0 Kapalı, 0x20 Orta, 0x30 Tam)
    ec::write_byte(ADDR_MAFAN_CONTROL, fan_byte)?;

    // 2. EC Donanım PL1 / PL2 Register'ları (0x046A, 0x046B)
    let _ = ec::write_byte(ADDR_PL1_CONTROL, pl1);
    let _ = ec::write_byte(ADDR_PL2_CONTROL, pl2);

    // 3. EC Mod Index Register (0x07AB)
    let _ = ec::write_byte(ADDR_PERF_MODE, mode as u8);

    // 4. Linux Kernel RAPL
    if let Err(e) = apply_kernel_rapl(pl1 as u32, pl2 as u32) {
        eprintln!("[warn] Kernel RAPL: {}", e);
    }

    // 5. Linux CPU EPP
    set_cpu_epp(epp)?;

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

/// Klavyedeki fiziksel tuş basışlarını 0x0751 üzerinden yakalayan düşük maliyetli EC polling daemon'ı (~200ms).
pub fn start_polling_daemon() {
    if DAEMON_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        let mut last_fan_byte = ec::read_byte(ADDR_MAFAN_CONTROL).unwrap_or(0xA0);
        loop {
            thread::sleep(Duration::from_millis(200));
            if let Ok(curr_fan_byte) = ec::read_byte(ADDR_MAFAN_CONTROL) {
                if curr_fan_byte != last_fan_byte {
                    last_fan_byte = curr_fan_byte;
                    if let Some(perf_mode) = PerformanceMode::from_fan_byte(curr_fan_byte) {
                        let (pl1, pl2, epp, _) = perf_mode.config();
                        let _ = ec::write_byte(ADDR_PL1_CONTROL, pl1);
                        let _ = ec::write_byte(ADDR_PL2_CONTROL, pl2);
                        let _ = ec::write_byte(ADDR_PERF_MODE, perf_mode as u8);
                        let _ = apply_kernel_rapl(pl1 as u32, pl2 as u32);
                        let _ = set_cpu_epp(epp);
                        send_notification(
                            "Monster Kontrol Merkezi",
                            &format!("Performans Kipi: {}", perf_mode.name()),
                        );
                    }
                }
            }
        }
    });
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
}
