//! # Cooling Fan Control & Telemetry Module
//!
//! Interfaces with the Embedded Controller fan subsystem to inspect tachometer RPM,
//! PWM duty cycles, and switch active operating cooling modes (Auto, Turbo, CoolerBoost, Custom).

use crate::ec;

/// EC RAM offset for CPU fan speed in RPM (16-bit little-endian).
pub const ADDR_CPU_FAN_RPM: u16   = 0x0464;
/// EC RAM offset for GPU fan speed in RPM (16-bit little-endian).
pub const ADDR_GPU_FAN_RPM: u16   = 0x046B;
/// EC RAM offset for CPU real-time fan duty cycle (0..100%).
pub const ADDR_CPU_FAN_DUTY: u16  = 0x075B;
/// EC RAM offset for GPU real-time fan duty cycle (0..100%).
pub const ADDR_GPU_FAN_DUTY: u16  = 0x075C;
/// EC RAM offset for Master Fan Mode selector.
pub const ADDR_MAFAN_CONTROL: u16 = 0x0751;

/// Operating fan profile mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanMode {
    /// Automatic thermal control managed by laptop BIOS.
    Auto,
    /// Elevated cooling curve for sustained loads.
    Turbo,
    /// 100% full-speed hardware override (CoolerBoost).
    CoolerBoost,
    /// User piecewise curve loaded in EC RAM (RamFan table).
    Custom,
    /// Fixed manual PWM level (1 to 5).
    Fixed(u8),
}

impl FanMode {
    /// Converts the fan mode to its EC control register byte.
    pub fn to_byte(self) -> u8 {
        match self {
            FanMode::Auto => 0x00,
            FanMode::Turbo => 0x10,
            FanMode::CoolerBoost => 0x40,
            FanMode::Custom => 0x80,
            FanMode::Fixed(lvl) => 0x80 + lvl.clamp(1, 5),
        }
    }

    /// Parses an EC control register byte into a FanMode.
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x00 => FanMode::Auto,
            0x10 => FanMode::Turbo,
            0x40 => FanMode::CoolerBoost,
            0x80 => FanMode::Custom,
            lvl if (0x81..=0x85).contains(&lvl) => FanMode::Fixed(lvl - 0x80),
            _ => FanMode::Auto,
        }
    }

    /// User-friendly display label.
    pub fn as_str(self) -> &'static str {
        match self {
            FanMode::Auto => "Auto",
            FanMode::Turbo => "Turbo",
            FanMode::CoolerBoost => "CoolerBoost (100%)",
            FanMode::Custom => "Custom Curve",
            FanMode::Fixed(_) => "Fixed Level",
        }
    }
}

/// Real-time fan speed and telemetry readings.
pub struct FanStatus {
    pub cpu_rpm: u16,
    pub gpu_rpm: u16,
    pub cpu_duty: u8,
    pub gpu_duty: u8,
    pub mode: FanMode,
    pub raw_mode: u8,
}

/// Reads real-time tachometer RPM, PWM duty cycles, and the active fan profile mode from the EC.
pub fn get_status() -> Result<FanStatus, String> {
    let cpu_rpm = ec::read_u16_le(ADDR_CPU_FAN_RPM, ADDR_CPU_FAN_RPM + 1).unwrap_or(0);
    let gpu_rpm = ec::read_u16_le(ADDR_GPU_FAN_RPM, ADDR_GPU_FAN_RPM + 1).unwrap_or(0);
    let cpu_duty = ec::read_byte(ADDR_CPU_FAN_DUTY).unwrap_or(0);
    let gpu_duty = ec::read_byte(ADDR_GPU_FAN_DUTY).unwrap_or(0);
    let raw_mode = ec::read_byte(ADDR_MAFAN_CONTROL).unwrap_or(0);

    Ok(FanStatus {
        cpu_rpm,
        gpu_rpm,
        cpu_duty,
        gpu_duty,
        mode: FanMode::from_byte(raw_mode),
        raw_mode,
    })
}

/// Sets the active fan profile mode on the hardware EC.
///
/// # Arguments
/// * `mode` - Target [`FanMode`].
pub fn set_mode(mode: FanMode) -> Result<(), String> {
    ec::write_byte(ADDR_MAFAN_CONTROL, mode.to_byte())
}

