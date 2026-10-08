//! # Monster Control Center CLI Binary
//!
//! Clean-room CLI for single-zone RGB keyboard lighting and raw EC access.

use monster_control_center::{device_info, ec, keyboard, get_chassis_project_id};

/// Prints CLI usage and available subcommands.
fn print_help() {
    println!(r#"monster-control-center: Keyboard & EC hardware control for Monster / Tongfang laptops

USAGE:
    monster-control-center <COMMAND> [OPTIONS]

COMMANDS:
    color <name> [1-4]          Set keyboard color preset with brightness (1..4)
    color rgb <R> <G> <B> [1-4] Set custom RGB color (0..255) with brightness (1..4)
    chassis                     Display chassis project ID
    info                        Display device and hardware specs
    read <hex_reg>              Read raw EC register byte (e.g. 0x0769)
    write <hex_reg> <hex_val>   Write raw EC register byte (e.g. 0x078C 0x01)

PRESETS:
    red, green, blue, cyan, ice_blue, magenta, purple, pink, yellow, orange, white, off
"#);
}

/// Parses CLI arguments and executes hardware requests.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = args[1].as_str();
    let res = match cmd {
        "chassis" => {
            match get_chassis_project_id() {
                Ok(id) => {
                    println!("Chassis Project ID: 0x{:02X}", id);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        "info" => {
            match device_info::get_device_info() {
                Ok(info) => {
                    println!("=== Monster Cihaz Bilgileri ===");
                    println!("\n[Panel 1: Cihaz & Kasa]");
                    println!("  Model Adı:        {}", info.model_name);
                    println!("  Üretici:          {}", info.vendor);
                    println!("  Kasa / Platform:  {}", info.chassis_code);
                    println!("  Anakart SKU/Rev:  {}", info.board_sku_rev);
                    println!("  Seri Numarası:    {}", info.serial_number);

                    println!("\n[Panel 2: Firmware & Denetleyici]");
                    println!("  BIOS Sürümü:      {}", info.bios_version);
                    println!("  BIOS Tarihi:      {}", info.bios_date);
                    println!("  EC Sürümü:        {}", info.ec_version);
                    println!("  Platform Sürücü:  {}", info.platform_driver);

                    println!("\n[Panel 3: Temel Donanım]");
                    println!("  İşlemci (CPU):    {}", info.cpu);
                    println!("  Dahili Grafik:    {}", info.igpu);
                    println!("  Harici Grafik:    {}", info.dgpu);
                    println!("  Sistem Belleği:   {}", info.ram);
                    println!("  Depolama:         {}", info.storage);

                    println!("\n[Panel 4: İşletim Sistemi & Sürücüler]");
                    println!("  İşletim Sistemi:  {}", info.os);
                    println!("  Masaüstü & Oturum:{}", info.desktop_protocol);
                    println!("  Linux Çekirdeği:  {}", info.kernel);
                    println!("  Grafik Sürücüsü:  {}", info.graphics_driver);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        "color" => {
            if args.len() < 3 {
                println!("Error: color preset name or 'rgb' required.");
                print_help();
                return;
            }
            if args[2] == "rgb" {
                if args.len() < 6 {
                    println!("Usage: monster-control-center color rgb <R> <G> <B> [brightness: 1-4]");
                    return;
                }
                let r: u8 = args[3].parse().expect("Invalid R value (0-255)");
                let g: u8 = args[4].parse().expect("Invalid G value (0-255)");
                let b: u8 = args[5].parse().expect("Invalid B value (0-255)");
                let brightness: u8 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(4);
                keyboard::set_rgb(r, g, b, brightness)
                    .map(|_| println!("[ok] Keyboard RGB set to ({}, {}, {}) [Brightness: {}/4]", r, g, b, brightness))
            } else {
                let name = &args[2];
                let brightness: u8 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(4);
                keyboard::set_preset(name, brightness)
                    .map(|_| println!("[ok] Keyboard color set to '{}' [Brightness: {}/4]", name, brightness))
            }
        }
        "read" => {
            if args.len() < 3 {
                println!("Usage: monster-control-center read <0xXXXX>");
                return;
            }
            let reg_str = args[2].strip_prefix("0x").unwrap_or(&args[2]);
            let reg = u16::from_str_radix(reg_str, 16).expect("Invalid hex register address");
            match ec::read_byte(reg) {
                Ok(val) => {
                    println!("[0x{:04X}] = 0x{:02X} ({})", reg, val, val);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        "write" => {
            if args.len() < 4 {
                println!("Usage: monster-control-center write <0xXXXX> <0xYY>");
                return;
            }
            let reg_str = args[2].strip_prefix("0x").unwrap_or(&args[2]);
            let val_str = args[3].strip_prefix("0x").unwrap_or(&args[3]);
            let reg = u16::from_str_radix(reg_str, 16).expect("Invalid hex register address");
            let val = u8::from_str_radix(val_str, 16).expect("Invalid hex value");
            ec::write_byte(reg, val).map(|_| println!("[ok] Wrote 0x{:02X} to 0x{:04X}", val, reg))
        }
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("Unknown command '{}'. Run with --help for usage.", other)),
    };

    if let Err(e) = res {
        eprintln!("[error] {}", e);
        std::process::exit(1);
    }
}




