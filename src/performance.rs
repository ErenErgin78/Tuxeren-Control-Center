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

static DAEMON_RUNNING: AtomicBool = AtomicBool::new(false);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceMode {
    Gaming       = 1,
    Office       = 2,
    BatterySaver = 3,
    Turbo        = 4,
}

impl PerformanceMode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(PerformanceMode::Gaming),
            2 => Some(PerformanceMode::Office),
            3 => Some(PerformanceMode::BatterySaver),
            4 => Some(PerformanceMode::Turbo),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PerformanceMode::Office => "Ofis (Sessiz)",
            PerformanceMode::Gaming => "Oyun (Standart)",
            PerformanceMode::BatterySaver => "Pil Tasarrufu (Eco)",
            PerformanceMode::Turbo => "Turbo (Canavar)",
        }
    }

    /// Returns (PL1 Watts, PL2 Watts, CPU EPP policy, Fedora OS Profile, Fan Byte 0x0751)
    pub fn config(&self) -> (u8, u8, &'static str, &'static str, u8) {
        match self {
            PerformanceMode::Office => (45, 65, "power", "power-saver", 0x00),
            PerformanceMode::Gaming => (75, 90, "balance_performance", "balanced", 0x00),
            PerformanceMode::BatterySaver => (35, 45, "power", "power-saver", 0x00),
            PerformanceMode::Turbo => (115, 140, "performance", "performance", 0x10),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceState {
    pub hardware_mode: u8,
    pub hardware_mode_name: String,
    pub os_profile: String,
}

/// Reads the current hardware/BIOS performance mode from the EC.
pub fn get_hardware_mode() -> Result<u8, String> {
    ec::read_byte(ADDR_PERF_MODE)
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

/// Mod değişimini EC'ye, fan moduna, çekirdeğe (RAPL/EPP) ve Fedora D-Bus'a uygular.
pub fn apply_mode(mode: PerformanceMode) -> Result<(), String> {
    let (pl1, pl2, epp, os_profile, fan_byte) = mode.config();

    // 1. EC Mod Register (0x07AB)
    ec::write_byte(ADDR_PERF_MODE, mode as u8)?;

    // 2. EC Fan Modu Register (0x0751: 0x10 Turbo, 0x00 Otomatik)
    ec::write_byte(ADDR_MAFAN_CONTROL, fan_byte)?;

    // 3. Linux Kernel RAPL (uyarı verir, çökmeyi engeller)
    if let Err(e) = apply_kernel_rapl(pl1 as u32, pl2 as u32) {
        eprintln!("[warn] Kernel RAPL: {}", e);
    }

    // 4. Linux CPU EPP
    set_cpu_epp(epp)?;

    // 5. Fedora OS Power Profile (D-Bus)
    set_os_profile(os_profile)?;

    Ok(())
}

/// Sets the hardware mode by raw u8 code (1=Gaming, 2=Office, 3=BatterySaver, 4=Turbo).
pub fn set_hardware_mode(mode: u8) -> Result<(), String> {
    if let Some(perf_mode) = PerformanceMode::from_u8(mode) {
        apply_mode(perf_mode)
    } else {
        Err(format!(
            "Geçersiz performans modu: {}. Geçerli: 1 (Oyun), 2 (Ofis), 3 (Pil), 4 (Turbo)",
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

/// Klavyedeki fiziksel tuş basışlarını yakalayan düşük maliyetli EC polling daemon'ı (~250ms).
pub fn start_polling_daemon() {
    if DAEMON_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        let mut last_mode = get_hardware_mode().unwrap_or(0);
        loop {
            thread::sleep(Duration::from_millis(250));
            if let Ok(current_mode) = get_hardware_mode() {
                if current_mode != last_mode {
                    last_mode = current_mode;
                    if let Some(perf_mode) = PerformanceMode::from_u8(current_mode) {
                        let _ = apply_mode(perf_mode);
                        send_notification("Monster Kontrol Merkezi", &format!("Performans Kipi: {}", perf_mode.name()));
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
        assert_eq!(PerformanceMode::from_u8(1), Some(PerformanceMode::Gaming));
        assert_eq!(PerformanceMode::from_u8(2), Some(PerformanceMode::Office));
        assert_eq!(PerformanceMode::from_u8(3), Some(PerformanceMode::BatterySaver));
        assert_eq!(PerformanceMode::from_u8(4), Some(PerformanceMode::Turbo));
        assert_eq!(PerformanceMode::from_u8(5), None);
    }
}
