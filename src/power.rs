//! # Performance & Power Mode Module
//!
//! Controls platform operating power policies (Gaming, Office, Battery Saver, Turbo).
//! Maps directly to EC registers governing hardware CPU PL1/PL2 thresholds and GPU boost curves.

use crate::ec;

/// EC RAM offset for Operating Performance Mode index.
pub const ADDR_POWER_MODE: u16 = 0x07AB;

/// Hardware performance profiles supported by the Tongfang platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerProfile {
    /// High sustained power limits for gaming workloads (Mode 1).
    Gaming,
    /// Balanced acoustic and thermal profile for day-to-day productivity (Mode 2).
    Office,
    /// Minimal power draw and passive fan bias for extended battery life (Mode 3).
    BatterySaver,
    /// Peak boost allowances and aggressive fan curves for benchmarks (Mode 4).
    Turbo,
}

impl PowerProfile {
    /// Returns the integer code stored in the EC register for this profile.
    pub fn to_byte(self) -> u8 {
        match self {
            PowerProfile::Gaming => 1,
            PowerProfile::Office => 2,
            PowerProfile::BatterySaver => 3,
            PowerProfile::Turbo => 4,
        }
    }

    /// Converts an EC mode register value into a known PowerProfile.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(PowerProfile::Gaming),
            2 => Some(PowerProfile::Office),
            3 => Some(PowerProfile::BatterySaver),
            4 => Some(PowerProfile::Turbo),
            _ => None,
        }
    }

    /// Descriptive profile name.
    pub fn as_str(self) -> &'static str {
        match self {
            PowerProfile::Gaming => "Gaming (Performance)",
            PowerProfile::Office => "Office (Balanced)",
            PowerProfile::BatterySaver => "Battery Saver (Eco)",
            PowerProfile::Turbo => "Turbo (Benchmark)",
        }
    }
}

/// Reads the currently active performance mode from the EC.
///
/// Returns a tuple containing the optional recognized [`PowerProfile`] and the raw register byte.
pub fn get_mode() -> Result<(Option<PowerProfile>, u8), String> {
    let raw = ec::read_byte(ADDR_POWER_MODE)?;
    Ok((PowerProfile::from_byte(raw), raw))
}

/// Switches the platform performance profile mode in the EC.
///
/// # Arguments
/// * `profile` - The target [`PowerProfile`].
pub fn set_mode(profile: PowerProfile) -> Result<(), String> {
    ec::write_byte(ADDR_POWER_MODE, profile.to_byte())
}

