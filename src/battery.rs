//! # Battery Management & Telemetry Module
//!
//! Provides battery charge state, health percentage, power source (AC vs Battery),
//! and estimated time to empty (when discharging) using Linux sysfs `/sys/class/power_supply`
//! with direct Embedded Controller (EC) register fallbacks for hardware cycle count.

use crate::ec;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BatteryInfo {
    /// Battery charge percentage (0..100)
    pub percentage: u8,
    /// Health percentage based on full vs design capacity (0..100)
    pub health_percent: Option<f32>,
    /// Raw state string: Charging, Discharging, Full, Not charging, Unknown
    pub status: String,
    /// Whether connected to AC mains power
    pub is_charging: bool,
    /// Whether running strictly on battery power
    pub on_battery: bool,
    /// Estimated remaining discharge duration in minutes (if discharging)
    pub estimated_minutes_left: Option<u32>,
    /// Human-friendly display string for estimated remaining time
    pub estimated_time_str: String,
    /// Current battery voltage in Volts (if available)
    pub voltage_volts: Option<f32>,
    /// Design capacity in mAh or mWh (if available)
    pub capacity_design: Option<u64>,
    /// Full charge capacity in mAh or mWh (if available)
    pub capacity_full: Option<u64>,
    /// Current charge or energy in mAh or mWh (if available)
    pub capacity_now: Option<u64>,
    /// Current power rate in Watts (if available)
    pub power_rate_watts: Option<f32>,
    /// Hardware battery cycle count (read directly from EC 0x04A6..0x04A7 or sysfs)
    pub cycle_count: Option<u32>,
    /// Model name or vendor reported by ACPI
    pub model_name: String,
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn read_u64(path: &Path) -> Option<u64> {
    read_trimmed(path)?.parse::<u64>().ok()
}

fn find_battery_path() -> Option<PathBuf> {
    let base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            let p_type = read_trimmed(&p.join("type")).unwrap_or_default();
            if p_type.eq_ignore_ascii_case("battery") {
                return Some(p);
            }
        }
    }
    None
}

fn check_ac_online() -> Option<bool> {
    let base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            let p_type = read_trimmed(&p.join("type")).unwrap_or_default();
            if p_type.eq_ignore_ascii_case("mains") {
                if let Some(online) = read_u64(&p.join("online")) {
                    return Some(online == 1);
                }
            }
        }
    }
    None
}

/// Reads the real battery cycle count from EC RAM (0x04A6 low, 0x04A7 high byte).
/// Linux ACPI battery driver often returns 0 because ACPI tables on Tongfang chassis do not expose it via _BST/_BIX.
fn read_ec_cycle_count() -> Option<u32> {
    let low = ec::read_byte(0x04A6).ok()?;
    let high = ec::read_byte(0x04A7).ok()?;
    let count = (low as u32) | ((high as u32) << 8);
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

/// Gathers comprehensive battery telemetry.
pub fn get_battery_info() -> Result<BatteryInfo, String> {
    let bat_dir = find_battery_path().ok_or_else(|| "Sistemde pil (BAT) donanımı bulunamadı.".to_string())?;

    // Status: Charging, Discharging, Full, Not charging, etc.
    let status_raw = read_trimmed(&bat_dir.join("status")).unwrap_or_else(|| "Unknown".to_string());

    // Percentage: sysfs `capacity`
    let percentage = read_u64(&bat_dir.join("capacity"))
        .map(|c| (c.min(100)) as u8)
        .unwrap_or(0);

    // Charge / Energy capacities
    let charge_full = read_u64(&bat_dir.join("charge_full"))
        .or_else(|| read_u64(&bat_dir.join("energy_full")));
    let charge_full_design = read_u64(&bat_dir.join("charge_full_design"))
        .or_else(|| read_u64(&bat_dir.join("energy_full_design")));
    let charge_now = read_u64(&bat_dir.join("charge_now"))
        .or_else(|| read_u64(&bat_dir.join("energy_now")));

    // Health: full / full_design
    let health_percent = match (charge_full, charge_full_design) {
        (Some(full), Some(design)) if design > 0 => {
            let pct = (full as f64 / design as f64) * 100.0;
            Some(pct.min(100.0).max(0.0) as f32)
        }
        _ => None,
    };

    // AC state
    let ac_online = check_ac_online();
    let is_charging = status_raw.eq_ignore_ascii_case("charging")
        || (status_raw.eq_ignore_ascii_case("full") && ac_online.unwrap_or(true));
    let on_battery = status_raw.eq_ignore_ascii_case("discharging")
        || (!is_charging && ac_online == Some(false));

    // Voltage (µV -> V)
    let voltage_now = read_u64(&bat_dir.join("voltage_now"));
    let voltage_volts = voltage_now.map(|v| (v as f32) / 1_000_000.0);

    // Rate: power_now (µW) or current_now (µA)
    let power_now = read_u64(&bat_dir.join("power_now"));
    let current_now = read_u64(&bat_dir.join("current_now"));

    let power_watts = if let Some(p) = power_now {
        Some((p as f32) / 1_000_000.0)
    } else if let (Some(cur), Some(volt)) = (current_now, voltage_now) {
        // P = V * I (µV * µA / 10^12 = W)
        Some(((cur as f64 * volt as f64) / 1_000_000_000_000.0) as f32)
    } else {
        None
    };

    // Remaining discharge estimation
    let mut estimated_minutes_left: Option<u32> = None;
    let mut estimated_time_str = if is_charging {
        if status_raw.eq_ignore_ascii_case("full") {
            "Tam Dolu (Şebekeye Bağlı)".to_string()
        } else {
            "Şarj Ediliyor".to_string()
        }
    } else if !on_battery {
        "Şarj Olmuyor (Şebeke Bağlı)".to_string()
    } else {
        "Hesaplanıyor...".to_string()
    };

    if on_battery {
        if let (Some(now), Some(cur)) = (charge_now, current_now) {
            if cur > 0 {
                let mins = ((now as u128 * 60) / cur as u128) as u32;
                estimated_minutes_left = Some(mins);
                let h = mins / 60;
                let m = mins % 60;
                estimated_time_str = if h > 0 {
                    format!("{} saat {} dakika", h, m)
                } else {
                    format!("{} dakika", m)
                };
            } else {
                estimated_time_str = "Tüketim tespit edilemiyor (0 mA)".to_string();
            }
        } else if let (Some(now_p), Some(p_w)) = (charge_now, power_watts) {
            if p_w > 0.05 {
                let mins = ((now_p as f64 / (p_w as f64 * 1_000_000.0)) * 60.0) as u32;
                estimated_minutes_left = Some(mins);
                let h = mins / 60;
                let m = mins % 60;
                estimated_time_str = if h > 0 {
                    format!("{} saat {} dakika", h, m)
                } else {
                    format!("{} dakika", m)
                };
            } else {
                estimated_time_str = "Tüketim hızı bekleniyor...".to_string();
            }
        } else {
            estimated_time_str = "Sensör verisi bekleniyor".to_string();
        }
    }

    // Cycle count: Try EC direct read first (Tongfang EC stores it at 0x04A6..0x04A7).
    // Sysfs cycle_count is hardcoded to 0 by the generic ACPI battery driver on many Linux kernels.
    let cycle_count = read_ec_cycle_count()
        .or_else(|| read_u64(&bat_dir.join("cycle_count")).filter(|&c| c > 0).map(|c| c as u32));

    // Model name
    let model = read_trimmed(&bat_dir.join("model_name")).unwrap_or_else(|| "Standart Batarya".to_string());

    Ok(BatteryInfo {
        percentage,
        health_percent,
        status: status_raw,
        is_charging,
        on_battery,
        estimated_minutes_left,
        estimated_time_str,
        voltage_volts,
        capacity_design: charge_full_design,
        capacity_full: charge_full,
        capacity_now: charge_now,
        power_rate_watts: power_watts,
        cycle_count,
        model_name: model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_info_fetch() {
        let res = get_battery_info();
        if let Ok(info) = res {
            assert!(info.percentage <= 100);
            if let Some(h) = info.health_percent {
                assert!(h >= 0.0 && h <= 100.0);
            }
        }
    }
}
