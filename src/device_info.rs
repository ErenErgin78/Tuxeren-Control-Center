//! # Device & Hardware Information Module
//!
//! Gathers system, firmware, hardware, and operating system information dynamically
//! from DMI, sysfs, /proc, and the Embedded Controller on Linux.

use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    // Panel 1: Cihaz & Kasa (Sol Üst)
    pub model_name: String,
    pub vendor: String,
    pub chassis_code: String,
    pub board_sku_rev: String,
    pub serial_number: String,

    // Panel 2: Firmware & Denetleyici (Sağ Üst)
    pub bios_version: String,
    pub bios_date: String,
    pub ec_version: String,
    pub platform_driver: String,

    // Panel 3: Temel Donanım (Sol Alt)
    pub cpu: String,
    pub igpu: String,
    pub dgpu: String,
    pub ram: String,
    pub storage: String,

    // Panel 4: İşletim Sistemi & Sürücüler (Sağ Alt)
    pub os: String,
    pub desktop_protocol: String,
    pub kernel: String,
    pub graphics_driver: String,

    // Auxiliary status
    pub acpi_call_available: bool,
}

fn read_sys_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Maps the EC chassis project ID (register 0x0740) to known Tongfang chassis families.
pub fn chassis_family_name(project_id: u8) -> &'static str {
    match project_id {
        0x00 => "Legacy Uniwill",
        0x01 => "GIxKN",
        0x02 => "GJxKN",
        0x03 => "GKxCN",
        0x04 => "GIxCN",
        0x05 => "GJxCN",
        0x06 => "GK5CN_X",
        0x07 => "GK7CN_S",
        0x08 => "GK7CPCS / GK5CQ7Z",
        0x0B => "IDP",
        0x0C => "ID6Y",
        0x0D => "ID7Y",
        0x0E => "PF4MU / PF4MN / PF5MU",
        0x0F => "CML_Gaming",
        0x10 => "GK7NXXR",
        0x11 => "GM5MU1Y",
        _ => "Tongfang / Uniwill Chassis",
    }
}

fn get_cpu_info() -> String {
    let mut model = "Bilinmiyor".to_string();
    let mut physical_cores = 0;
    let mut threads = 0;

    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") && model == "Bilinmiyor" {
                if let Some((_, val)) = line.split_once(':') {
                    model = val.trim().to_string();
                }
            } else if line.starts_with("cpu cores") && physical_cores == 0 {
                if let Some((_, val)) = line.split_once(':') {
                    physical_cores = val.trim().parse::<u32>().unwrap_or(0);
                }
            } else if line.starts_with("processor") {
                threads += 1;
            }
        }
    }

    if physical_cores > 0 && threads > 0 {
        format!("{} ({} Çekirdek, {} İş Parçacığı)", model, physical_cores, threads)
    } else {
        model
    }
}

fn get_gpu_info() -> (String, String) {
    let mut igpu = "Bilinmiyor".to_string();
    let mut dgpu = "Yok".to_string();

    if let Ok(output) = Command::new("lspci").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("VGA compatible controller") || line.contains("3D controller") {
                if let Some((_, dev)) = line.split_once(": ") {
                    let cleaned = dev.trim();
                    if cleaned.contains("Intel") || (cleaned.contains("AMD") && !cleaned.contains("Radeon RX") && !cleaned.contains("Discrete")) {
                        if igpu == "Bilinmiyor" {
                            igpu = cleaned.to_string();
                        }
                    } else if cleaned.contains("NVIDIA") || cleaned.contains("GeForce") || cleaned.contains("Radeon RX") || cleaned.contains("Arc") {
                        dgpu = cleaned.to_string();
                    }
                }
            }
        }
    }

    if dgpu.contains("NVIDIA") || dgpu.contains("GeForce") {
        if let Ok(out) = Command::new("nvidia-smi")
            .args(["--query-gpu=power.default_limit", "--format=csv,noheader,nounits"])
            .output()
        {
            let pwr = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(watts) = pwr.parse::<f32>() {
                dgpu = format!("{} ({:.0}W TGP)", dgpu, watts);
            }
        }
    }

    (igpu, dgpu)
}

fn get_ram_info() -> String {
    let mut total_str = "Bilinmiyor".to_string();
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        let gb = kb / 1024.0 / 1024.0;
                        total_str = format!("{:.1} GB", gb);
                    }
                }
                break;
            }
        }
    }

    // Inspect dmidecode if permissions allow
    if let Ok(out) = Command::new("dmidecode").args(["-t", "memory"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut mem_type = "";
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Type:") && !trimmed.contains("Error") && !trimmed.contains("Unknown") {
                if let Some((_, val)) = trimmed.split_once(':') {
                    let t = val.trim();
                    if t.starts_with("DDR") || t.starts_with("LPDDR") {
                        mem_type = t;
                        break;
                    }
                }
            }
        }
        if !mem_type.is_empty() {
            return format!("{} ({} / Çift Kanal)", total_str, mem_type);
        }
    }

    format!("{} Kullanılabilir", total_str)
}

fn get_storage_info() -> String {
    // 1. Determine disk backing the root filesystem "/"
    let mut root_disk = String::new();
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "/" {
                let dev_name = parts[0].trim_start_matches("/dev/");
                let trimmed = dev_name.trim_end_matches(|c: char| c.is_ascii_digit());
                let base = trimmed.trim_end_matches('p');
                root_disk = base.to_string();
                break;
            }
        }
    }

    let candidate_disks = if !root_disk.is_empty() {
        vec![root_disk.clone(), "nvme0n1".into(), "sda".into()]
    } else {
        vec!["nvme0n1".into(), "sda".into(), "vda".into()]
    };

    for disk in candidate_disks {
        let model_path = format!("/sys/block/{}/device/model", disk);
        let size_path = format!("/sys/block/{}/size", disk);
        if let Some(model) = read_sys_file(&model_path) {
            let size_str = read_sys_file(&size_path).unwrap_or_default();
            if let Ok(sectors) = size_str.parse::<u64>() {
                let bytes = sectors * 512;
                let gb = bytes / (1000 * 1000 * 1000);
                let cap = if gb >= 900 && gb <= 1050 {
                    "1 TB".to_string()
                } else if gb >= 1800 && gb <= 2100 {
                    "2 TB".to_string()
                } else if gb >= 450 && gb <= 550 {
                    "512 GB".to_string()
                } else if gb >= 220 && gb <= 270 {
                    "256 GB".to_string()
                } else {
                    format!("{} GB", gb)
                };
                let disk_type = if disk.starts_with("nvme") {
                    "NVMe SSD"
                } else if disk.starts_with("sd") {
                    "SATA SSD / HDD"
                } else {
                    "Disk"
                };
                return format!("{} ({} {})", model, cap, disk_type);
            }
            return model;
        }
    }
    "Bilinmiyor".to_string()
}

fn get_os_info() -> String {
    let mut name = "Linux".to_string();
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let val = line.strip_prefix("PRETTY_NAME=").unwrap_or("");
                name = val.trim_matches('"').to_string();
                break;
            }
        }
    }
    format!("{} ({})", name, std::env::consts::ARCH)
}

fn get_desktop_and_protocol() -> String {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Bilinmiyor".to_string());
    let session = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_else(|_| "Bilinmiyor".to_string());

    let session_cap = match session.to_lowercase().as_str() {
        "wayland" => "Wayland".to_string(),
        "x11" => "X11".to_string(),
        other => other.to_string(),
    };

    format!("{} / {}", desktop, session_cap)
}

fn get_graphics_driver() -> String {
    if let Some(ver) = read_sys_file("/sys/module/nvidia/version") {
        if let Ok(proc_ver) = fs::read_to_string("/proc/driver/nvidia/version") {
            if proc_ver.contains("Open Kernel") {
                return format!("NVIDIA Open Kernel Sürücüsü {}", ver);
            }
        }
        return format!("NVIDIA Sürücüsü {}", ver);
    }
    if Path::new("/sys/module/nouveau").exists() {
        return "Nouveau (Açık Kaynak)".to_string();
    }
    if Path::new("/sys/module/amdgpu").exists() {
        return "AMDGPU (Açık Kaynak)".to_string();
    }
    "Mesa / DRM Açık Kaynak".to_string()
}

pub fn get_device_info() -> Result<DeviceInfo, String> {
    // Panel 1: Cihaz & Kasa
    let model_name = read_sys_file("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|| "Bilinmiyor".to_string());
    let vendor = read_sys_file("/sys/class/dmi/id/sys_vendor")
        .unwrap_or_else(|| "Bilinmiyor".to_string());

    let chassis_project_id = crate::get_chassis_project_id().unwrap_or(0);
    let chassis_family = chassis_family_name(chassis_project_id);
    let chassis_code = if chassis_project_id > 0 {
        format!("{} [Proje ID: 0x{:02X}]", chassis_family, chassis_project_id)
    } else {
        chassis_family.to_string()
    };

    let board_name = read_sys_file("/sys/class/dmi/id/board_name").unwrap_or_else(|| "-".to_string());
    let board_ver = read_sys_file("/sys/class/dmi/id/board_version");
    let board_sku_rev = match board_ver {
        Some(ver) if !ver.is_empty() && ver != "-" => format!("{} (Rev: {})", board_name, ver),
        _ => board_name,
    };

    let serial_number = read_sys_file("/sys/class/dmi/id/product_serial")
        .or_else(|| read_sys_file("/sys/class/dmi/id/product_uuid"))
        .or_else(|| read_sys_file("/etc/machine-id"))
        .unwrap_or_else(|| "Bilinmiyor".to_string());

    // Panel 2: Firmware & Denetleyici
    let bios_version = read_sys_file("/sys/class/dmi/id/bios_version").unwrap_or_else(|| "-".to_string());
    let bios_date = read_sys_file("/sys/class/dmi/id/bios_date").unwrap_or_else(|| "-".to_string());
    let ec_version = read_sys_file("/sys/class/dmi/id/ec_firmware_release").unwrap_or_else(|| "-".to_string());
    let acpi_call_available = Path::new(crate::ec::ACPI_CALL_PATH).exists();
    let platform_driver = if acpi_call_available {
        "acpi_call (\\_SB.INOU)".to_string()
    } else {
        "acpi_call (Yüklü Değil)".to_string()
    };

    // Panel 3: Temel Donanım
    let cpu = get_cpu_info();
    let (igpu, dgpu) = get_gpu_info();
    let ram = get_ram_info();
    let storage = get_storage_info();

    // Panel 4: İşletim Sistemi & Sürücüler
    let os = get_os_info();
    let desktop_protocol = get_desktop_and_protocol();
    let kernel = read_sys_file("/proc/sys/kernel/osrelease").unwrap_or_else(|| "-".to_string());
    let graphics_driver = get_graphics_driver();

    Ok(DeviceInfo {
        model_name,
        vendor,
        chassis_code,
        board_sku_rev,
        serial_number,
        bios_version,
        bios_date,
        ec_version,
        platform_driver,
        cpu,
        igpu,
        dgpu,
        ram,
        storage,
        os,
        desktop_protocol,
        kernel,
        graphics_driver,
        acpi_call_available,
    })
}
