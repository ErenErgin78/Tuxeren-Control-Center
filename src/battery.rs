//! # Battery Gas Gauge & Health Control Module
//!
//! Provides telemetry for battery state of charge, temperature, and cycle count,
//! as well as setting upper and lower charge thresholds to preserve battery lifespan.

use crate::ec;

/// EC RAM offset for upper battery charge limit threshold (e.g. 80 = stop at 80%).
pub const ADDR_BATTERY_LIMIT_UP: u16   = 0x07B9;
/// EC RAM offset for lower battery charge limit threshold (e.g. 75 = resume at 75%).
pub const ADDR_BATTERY_LIMIT_DOWN: u16 = 0x07D0;
/// EC RAM offset for relative state of charge percentage (0..100%).
pub const ADDR_BATTERY_RSOC: u16       = 0x04AB;
/// EC RAM offset for battery temperature in °C.
pub const ADDR_BATTERY_TEMP: u16       = 0x04A2;
/// EC RAM offset for battery charge cycle count (16-bit little-endian).
pub const ADDR_BATTERY_CYCLES: u16     = 0x04A6;

/// Battery state and lifespan telemetry readings.
pub struct BatteryStatus {
    pub rsoc_percent: u8,
    pub temp_c: u8,
    pub cycles: u16,
    pub charge_limit: u8,
}

/// Reads current battery telemetry and configured charge limit threshold.
pub fn get_status() -> Result<BatteryStatus, String> {
    let rsoc_percent = ec::read_byte(ADDR_BATTERY_RSOC).unwrap_or(0);
    let temp_c = ec::read_byte(ADDR_BATTERY_TEMP).unwrap_or(0);
    let cycles = ec::read_u16_le(ADDR_BATTERY_CYCLES, ADDR_BATTERY_CYCLES + 1).unwrap_or(0);
    let charge_limit = ec::read_byte(ADDR_BATTERY_LIMIT_UP).unwrap_or(100);

    Ok(BatteryStatus {
        rsoc_percent,
        temp_c,
        cycles,
        charge_limit,
    })
}

/// Configures battery charging threshold to prevent battery overcharging wear.
///
/// Automatically establishes a 5% hysteresis band below the specified ceiling.
///
/// # Arguments
/// * `limit` - Percentage cap between 50 and 100.
pub fn set_charge_limit(limit: u8) -> Result<(), String> {
    if !(50..=100).contains(&limit) {
        return Err("Charge limit must be between 50 and 100".to_string());
    }
    let lower = limit.saturating_sub(5).max(45);
    ec::write_byte(ADDR_BATTERY_LIMIT_UP, limit)?;
    ec::write_byte(ADDR_BATTERY_LIMIT_DOWN, lower)?;
    Ok(())
}

