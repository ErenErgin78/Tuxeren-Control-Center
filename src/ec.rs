//! # Embedded Controller (EC) Communication Module
//!
//! Provides direct byte-level read and write access to the Embedded Controller
//! on Uniwill / Tongfang laptops via the ACPI subsystem interface (`\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`).

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// Path to the `acpi_call` kernel module character device interface.
pub const ACPI_CALL_PATH: &str = "/proc/acpi/call";

/// Writes a single byte to an EC RAM register via `\_SB.INOU.ECRW`.
///
/// # Arguments
/// * `reg` - 16-bit EC register address (e.g. `0x0751`).
/// * `val` - 8-bit value to write.
pub fn write_byte(reg: u16, val: u8) -> Result<(), String> {
    if !Path::new(ACPI_CALL_PATH).exists() {
        return Err(format!("{} not found. Ensure acpi_call kernel module is loaded.", ACPI_CALL_PATH));
    }
    let mut f = OpenOptions::new()
        .write(true)
        .open(ACPI_CALL_PATH)
        .map_err(|e| format!("Failed to open {}: {} (root required)", ACPI_CALL_PATH, e))?;
    let cmd = format!("\\_SB.INOU.ECRW 0x{:04X} 0x{:02X}\n", reg, val);
    f.write_all(cmd.as_bytes())
        .map_err(|e| format!("Write failed: {}", e))?;
    Ok(())
}

/// Reads a single byte from an EC RAM register via `\_SB.INOU.ECRR`.
///
/// # Arguments
/// * `reg` - 16-bit EC register address.
pub fn read_byte(reg: u16) -> Result<u8, String> {
    if !Path::new(ACPI_CALL_PATH).exists() {
        return Err(format!("{} not found. Ensure acpi_call kernel module is loaded.", ACPI_CALL_PATH));
    }
    {
        let mut f = OpenOptions::new()
            .write(true)
            .open(ACPI_CALL_PATH)
            .map_err(|e| format!("Failed to open {}: {} (root required)", ACPI_CALL_PATH, e))?;
        let cmd = format!("\\_SB.INOU.ECRR 0x{:04X}\n", reg);
        f.write_all(cmd.as_bytes())
            .map_err(|e| format!("Write query failed: {}", e))?;
    }
    let mut resp = String::new();
    File::open(ACPI_CALL_PATH)
        .and_then(|mut f| f.read_to_string(&mut resp))
        .map_err(|e| format!("Read response failed: {}", e))?;

    let trimmed = resp.trim().trim_matches('\0').trim();
    let num_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    u8::from_str_radix(num_str, 16)
        .map_err(|e| format!("Failed to parse EC return '{}': {}", trimmed, e))
}

/// Reads a 16-bit little-endian value from two consecutive or designated EC registers.
///
/// # Arguments
/// * `reg_low` - Address containing the lower 8 bits.
/// * `reg_high` - Address containing the upper 8 bits.
pub fn read_u16_le(reg_low: u16, reg_high: u16) -> Result<u16, String> {
    let low = read_byte(reg_low)? as u16;
    let high = read_byte(reg_high)? as u16;
    Ok(low | (high << 8))
}

