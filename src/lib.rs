//! # Monster / Tongfang Hardware Control Library (`monster_control_center`)
//!
//! A clean-room hardware control library for Uniwill / Tongfang ODM laptop chassis
//! (Monster Notebook Tulpar, Abra, Semruk) running on Linux.
//! Interfaces with the Embedded Controller (EC) via ACPI methods `\_SB.INOU.ECRR` and `\_SB.INOU.ECRW`.

pub mod ec;
pub mod keyboard;

/// EC RAM offset for chassis project / model family identifier.
pub const ADDR_PROJECT_ID: u16 = 0x0740;

/// Reads the hardware chassis project code from the EC (e.g. 0x0E for PF4/PF5, 0x11 for GM5).
pub fn get_chassis_project_id() -> Result<u8, String> {
    ec::read_byte(ADDR_PROJECT_ID)
}

