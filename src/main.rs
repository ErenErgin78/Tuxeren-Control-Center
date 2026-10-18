//! # Tuxeren Control Center CLI Binary
//!
//! Clean-room CLI for single-zone RGB keyboard lighting and raw EC access.

use std::os::unix::process::CommandExt;
use std::process::Command;

use txectrl::{device_info, ec, keyboard, get_chassis_project_id};

/// Prints CLI usage and available subcommands.
fn print_help() {
    println!(r#"txectrl: Keyboard & EC hardware control for Tuxeren Control Center (Uniwill / Tongfang laptops)

USAGE:
    txectrl <COMMAND> [OPTIONS]

COMMANDS:
    gui                         Launch graphical user interface (default)
    color <name> [1-4]          Set keyboard color preset with brightness (1..4)
    color rgb <R> <G> <B> [1-4] Set custom RGB color (0..255) with brightness (1..4)
    chassis                     Display chassis project ID
    info                        Display device and hardware specs
    perf [office|gaming|turbo]  Get or set hardware and OS performance modes
    perf os <saver|balanced|perf> Set OS power profile
    perf watch                  Watch keyboard mode button and sync OS with OSD notifications
    display                     Get active display settings
    display brightness <0-100>  Set display brightness
    display temp <1000-10000>   Set color temperature (Kelvin)
    display reset               Reset display to neutral calibration
    daemon                      Run low-power background service (touchpad F24 & performance)
    read <hex_reg>              Read raw EC register byte (e.g. 0x0769)
    write <hex_reg> <hex_val>   Write raw EC register byte (e.g. 0x078C 0x01)

PRESETS:
    red, green, blue, cyan, ice_blue, magenta, purple, pink, yellow, orange, white, off
"#);
}

/// Launches the GUI application binary.
fn launch_gui() {
    let has_display = std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
    if !has_display {
        print_help();
        return;
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let gui_path = parent.join("txectrl-gui");
            if gui_path.is_file() {
                let err = Command::new(gui_path).exec();
                eprintln!("[-] Failed to launch GUI binary: {}", err);
                return;
            }
        }
    }

    let err = Command::new("txectrl-gui").exec();
    eprintln!("[-] Failed to launch txectrl-gui: {}", err);
    print_help();
}

/// Parses CLI arguments and executes hardware requests.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        launch_gui();
        return;
    }

    let cmd = args[1].as_str();
    if cmd == "-h" || cmd == "--help" || cmd == "help" {
        print_help();
        return;
    }
    if cmd == "gui" {
        launch_gui();
        return;
    }

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
                    println!("=== Tuxeren Cihaz Bilgileri ===");
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
                    println!("Usage: txectrl color rgb <R> <G> <B> [brightness: 1-4]");
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
                println!("Usage: txectrl read <0xXXXX>");
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
                println!("Usage: txectrl write <0xXXXX> <0xYY>");
                return;
            }
            let reg_str = args[2].strip_prefix("0x").unwrap_or(&args[2]);
            let val_str = args[3].strip_prefix("0x").unwrap_or(&args[3]);
            let reg = u16::from_str_radix(reg_str, 16).expect("Invalid hex register address");
            let val = u8::from_str_radix(val_str, 16).expect("Invalid hex value");
            ec::write_byte(reg, val).map(|_| println!("[ok] Wrote 0x{:02X} to 0x{:04X}", val, reg))
        }
        "perf" => {
            if args.len() < 3 {
                match txectrl::performance::get_performance_state() {
                    Ok(state) => {
                        println!("Donanım (BIOS/EC) Modu: {} (Kod: {})", state.hardware_mode_name, state.hardware_mode);
                        println!("Sistem (OS/D-Bus) Kipi: {}", state.os_profile);
                        if let Ok(m) = txectrl::performance::get_system_metrics() {
                            println!("CPU: {:.1}% ({}°C) | GPU: {:.1}% ({}°C)", m.cpu_percent, m.cpu_temp, m.gpu_percent, m.gpu_temp);
                            println!("RAM: {:.1}% ({:.1} GB / {:.1} GB) | Disk: {:.1}% ({:.1} GB / {:.1} GB)", m.ram_percent, m.ram_used_gb, m.ram_total_gb, m.disk_percent, m.disk_used_gb, m.disk_total_gb);
                            println!("Fanlar: CPU {} RPM ({}%) | GPU {} RPM ({}%)", m.cpu_fan_rpm, m.cpu_fan_duty, m.gpu_fan_rpm, m.gpu_fan_duty);
                            let fmt_speed = |b: u64| {
                                if b >= 1024 * 1024 {
                                    format!("{:.1} MB/s", b as f64 / 1024.0 / 1024.0)
                                } else {
                                    format!("{:.1} KB/s", b as f64 / 1024.0)
                                }
                            };
                            println!("Ağ: İndirme {} | Yükleme {}", fmt_speed(m.net_down_bytes), fmt_speed(m.net_up_bytes));
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                match args[2].to_lowercase().as_str() {
                    "office" | "ofis" => txectrl::performance::set_hardware_mode(0)
                        .map(|_| println!("[ok] Donanım modu 'Ofis' (0) olarak ayarlandı.")),
                    "gaming" | "oyun" => txectrl::performance::set_hardware_mode(1)
                        .map(|_| println!("[ok] Donanım modu 'Oyun' (1) olarak ayarlandı.")),
                    "battery" | "pil" | "eco" => txectrl::performance::set_hardware_mode(3)
                        .map(|_| println!("[ok] Donanım modu 'Pil Tasarrufu' (3) olarak ayarlandı.")),
                    "turbo" => txectrl::performance::set_hardware_mode(4)
                        .map(|_| println!("[ok] Donanım modu 'Turbo' (4) olarak ayarlandı.")),
                    "os" => {
                        if args.len() < 4 {
                            println!("Usage: txectrl perf os <power-saver|balanced|performance>");
                            return;
                        }
                        txectrl::performance::set_os_profile(&args[3])
                            .map(|_| println!("[ok] Sistem güç kipi '{}' olarak ayarlandı.", args[3]))
                    }
                    "watch" | "daemon" => {
                        println!("[*] EC Donanım Buton İzleyici başlatıldı (~250ms). Çıkmak için Ctrl+C.");
                        txectrl::performance::start_polling_daemon();
                        loop {
                            std::thread::sleep(std::time::Duration::from_secs(3600));
                        }
                    }
                    _ => Err(format!("Bilinmeyen perf parametresi '{}'. Geçerli: office, gaming, turbo, os, watch", args[2])),
                }
            }
        }
        "display" => {
            if args.len() == 2 {
                let s = txectrl::display::get_display_settings();
                println!("=== Görüntü Ayarları ===");
                println!("  Parlaklık (Brightness):       %{}", s.brightness);
                println!("  Renk Sıcaklığı (Color Temp):  {} K", s.color_temp);
                println!("  Doygunluk (Saturation):       {:.2}", s.saturation);
                println!("  Gamma Değeri:                 {:.2}", s.gamma);
                println!("  Kanal Kırmızı (Red):          {}", s.red);
                println!("  Kanal Yeşil (Green):          {}", s.green);
                println!("  Kanal Mavi (Blue):            {}", s.blue);
                Ok(())
            } else {
                match args[2].as_str() {
                    "reset" => {
                        txectrl::display::reset_display_settings()
                            .map(|_| println!("[ok] Görüntü ayarları sıfırlandı."))
                    }
                    "brightness" => {
                        if args.len() < 4 {
                            Err("Parlaklık yüzdesi belirtilmelidir (0..100).".to_string())
                        } else {
                            match args[3].parse::<u32>() {
                                Ok(val) => {
                                    let mut s = txectrl::display::get_display_settings();
                                    s.brightness = val;
                                    txectrl::display::apply_display_settings(s)
                                        .map(|_| println!("[ok] Ekran parlaklığı %{} olarak ayarlandı.", val))
                                }
                                Err(_) => Err("Geçersiz parlaklık değeri.".to_string()),
                            }
                        }
                    }
                    "temp" => {
                        if args.len() < 4 {
                            Err("Renk sıcaklığı belirtilmelidir (Kelvin, örn. 6500).".to_string())
                        } else {
                            match args[3].parse::<u32>() {
                                Ok(val) => {
                                    let mut s = txectrl::display::get_display_settings();
                                    s.color_temp = val;
                                    txectrl::display::apply_display_settings(s)
                                        .map(|_| println!("[ok] Renk sıcaklığı {} K olarak ayarlandı.", val))
                                }
                                Err(_) => Err("Geçersiz renk sıcaklığı değeri.".to_string()),
                            }
                        }
                    }
                    _ => Err(format!("Bilinmeyen display alt komutu '{}'.", args[2])),
                }
            }
        }
        "daemon" | "service" => {
            println!("[*] Tuxeren Control Center (TxeCtrl) arkaplan servisi başlatıldı.");
            txectrl::performance::start_polling_daemon();
            txectrl::general::start_touchpad_daemon();
            txectrl::general::run_daemon_server();
            Ok(())
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




