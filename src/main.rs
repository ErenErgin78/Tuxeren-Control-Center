//! # Monster Control Center CLI Binary
//!
//! Entry point and command-line parser for `monster-control-center`.
//! Dispatches commands for telemetry status, keyboard lighting, fan mode,
//! performance profiles, battery charging threshold, and direct EC memory I/O.

use monster_control_center::{battery, ec, fan, keyboard, power, get_chassis_project_id};

/// Displays real-time hardware telemetry: chassis ID, performance profile, fan RPM/duty, and battery metrics.
fn print_status() -> Result<(), String> {
    let fan_stat = fan::get_status()?;
    let (pwr_profile, pwr_raw) = power::get_mode().unwrap_or((None, 0));
    let bat_stat = battery::get_status()?;
    let proj_id = get_chassis_project_id().unwrap_or(0);

    let pwr_name = pwr_profile
        .map(|p| p.as_str())
        .unwrap_or("Unknown");

    println!("--- Monster / Tongfang Hardware Status ---");
    println!("Chassis Project ID : 0x{:02X}", proj_id);
    println!("Performance Mode   : {} (0x{:02X})", pwr_name, pwr_raw);
    println!("Fan Mode           : {} (0x{:02X})", fan_stat.mode.as_str(), fan_stat.raw_mode);
    println!("CPU Fan            : {} RPM (Duty: {}%)", fan_stat.cpu_rpm, fan_stat.cpu_duty);
    println!("GPU Fan            : {} RPM (Duty: {}%)", fan_stat.gpu_rpm, fan_stat.gpu_duty);
    println!("Battery State      : {}% (Temp: {}°C, Cycles: {})", bat_stat.rsoc_percent, bat_stat.temp_c, bat_stat.cycles);
    println!("Battery Threshold  : {}%", bat_stat.charge_limit);
    Ok(())
}

/// Prints CLI usage and available subcommands.
fn print_help() {
    println!(r#"monster-control-center: Clean-room Linux CLI for Monster / Tongfang laptops

USAGE:
    monster-control-center <COMMAND> [OPTIONS]

COMMANDS:
    status                   Show real-time fan RPM, temperatures, power & battery telemetry
    color <name> [1-4]       Set keyboard color from calibrated presets with brightness (1..4)
    color rgb <R> <G> <B> [1-4] Set custom RGB color (0..255) with brightness (1..4)
    fan <mode>               Set fan mode: auto | turbo | boost | custom
    mode <profile>           Set performance mode: gaming | office | eco | turbo
    limit <50-100>           Set battery charge limit percentage
    read <hex_reg>           Read raw EC register byte (e.g. 0x0751)
    write <hex_reg> <hex_val> Write raw EC register byte (e.g. 0x0751 0x40)

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
        "status" => print_status(),
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
        "fan" => {
            if args.len() < 3 {
                println!("Usage: monster-control-center fan <auto|turbo|boost|custom>");
                return;
            }
            let mode = match args[2].to_lowercase().as_str() {
                "auto" => fan::FanMode::Auto,
                "turbo" => fan::FanMode::Turbo,
                "boost" | "coolerboost" => fan::FanMode::CoolerBoost,
                "custom" => fan::FanMode::Custom,
                other => {
                    eprintln!("Unknown fan mode '{}'. Use: auto, turbo, boost, custom", other);
                    std::process::exit(1);
                }
            };
            fan::set_mode(mode).map(|_| println!("[ok] Fan mode set to: {}", mode.as_str()))
        }
        "mode" => {
            if args.len() < 3 {
                println!("Usage: monster-control-center mode <gaming|office|eco|turbo>");
                return;
            }
            let profile = match args[2].to_lowercase().as_str() {
                "gaming" => power::PowerProfile::Gaming,
                "office" => power::PowerProfile::Office,
                "eco" | "battery" => power::PowerProfile::BatterySaver,
                "turbo" => power::PowerProfile::Turbo,
                other => {
                    eprintln!("Unknown profile '{}'. Use: gaming, office, eco, turbo", other);
                    std::process::exit(1);
                }
            };
            power::set_mode(profile).map(|_| println!("[ok] Performance mode set to: {}", profile.as_str()))
        }
        "limit" => {
            if args.len() < 3 {
                println!("Usage: monster-control-center limit <50-100>");
                return;
            }
            let lim: u8 = args[2].parse().expect("Invalid charge limit (50-100)");
            battery::set_charge_limit(lim)
                .map(|_| println!("[ok] Battery charge threshold set to: {}%", lim))
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



