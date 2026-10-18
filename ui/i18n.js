const translations = {
  tr: {
    // Navigation & Common
    nav_main_menu: "Ana Menü",
    nav_back: "Geri",
    perm_banner_text: "Donanım erişimi için sürücü izni gerekiyor (<code>tuxedo-drivers</code>).",
    perm_btn_grant: "İzin Ver",
    perm_requesting: "pkexec ile izin isteniyor...",
    perm_granted: "İzin verildi",
    perm_error: "İzin hatası:",
    error_prefix: "Hata:",
    status_perm_required: "İzin Gerekli",
    status_connected: "Bağlandı",
    modal_close: "Kapat",

    // Lobby (index.html)
    lobby_title: "Tuxeren Control Center (TxeCtrl)",
    hero_title: "Tuxeren Kontrol Merkezi",
    menu_general: "Genel Ayarlar",
    menu_general_desc: "Kolay Ayar, Windows Aygıtları, Oyun içi Hızlı Bilgi Ekranı, Klavye Haritalandırması",
    menu_perf: "Performans",
    menu_perf_desc: "Donanım & Güç Kipi Seçimi",
    menu_light: "Aydınlatma Ayarları",
    menu_light_desc: "Klavye Aydınlatması",
    menu_battery: "Pil",
    menu_battery_desc: "Pil Modu, Pil Durumu",
    menu_display: "Görüntü Ayarları",
    menu_display_desc: "Ekran Fonksiyonları",
    menu_device: "Cihaz Bilgileri",
    menu_device_desc: "Cihaz Bilgileri, Hakkında",

    // General Settings (general.html)
    general_title: "Genel Ayarlar - Tuxeren Control Center",
    page_general_heading: "Genel Ayarlar",
    setting_language_title: "Uygulama Dili",
    setting_language_desc: "Arayüzün görüntüleneceği dili seçin",
    setting_winkey_title: "Windows Tuşunu Devre Dışı Bırak",
    setting_winkey_desc: "Oyun esnasında yanlışlıkla basılmasını önler",
    setting_touchpad_title: "Touchpad'i Devre Dışı Bırak",
    setting_touchpad_desc: "Dokunmatik yüzeyi ve tıklama algılamasını kilitler",
    toast_winkey_disabled: "Windows tuşu devre dışı bırakıldı",
    toast_winkey_enabled: "Windows tuşu etkinleştirildi",
    toast_touchpad_disabled: "Touchpad kilitlendi",
    toast_touchpad_enabled: "Touchpad etkinleştirildi",
    toast_lang_changed: "Dil Türkçe olarak ayarlandı",

    // Performance (performance.html)
    perf_title: "Performans Yönetimi - Tuxeren Control Center",
    page_perf_heading: "Performans Yönetimi",
    telemetry_title: "Sistem İzleme",
    telemetry_cpu: "İŞLEMCİ (CPU)",
    telemetry_gpu: "EKRAN KARTI (GPU)",
    telemetry_ram: "BELLEK (RAM)",
    telemetry_disk: "DEPOLAMA (DİSK)",
    telemetry_fans: "SOĞUTMA FANLARI",
    telemetry_fan_cpu: "CPU FAN",
    telemetry_fan_gpu: "GPU FAN",
    telemetry_net: "AĞ TRAFİĞİ",
    telemetry_net_down: "İNDİRME",
    telemetry_net_up: "YÜKLEME",
    card_bios_title: "BIOS Güç Kipi",
    mode_office_title: "Ofis Modu",
    mode_office_desc: "45W CPU Limiti • Düşük Fan Hızı",
    mode_game_title: "Oyun Modu",
    mode_game_desc: "75W CPU Limiti • Dengeli Fan Profili",
    mode_turbo_title: "Turbo Modu",
    mode_turbo_desc: "90W CPU Limiti • Turbo Fan",
    card_os_title: "İşletim Sistemi Güç Kipi",
    os_powersaver_title: "Güç Tasarrufu",
    os_powersaver_desc: "Pil ömrü odaklı, düşük CPU saat frekansı",
    os_balanced_title: "Dengeli",
    os_balanced_desc: "Standart kullanım, dinamik frekans ölçekleme",
    os_perf_title: "Yüksek Performans",
    os_perf_desc: "Maksimum çekirdek saat frekansları",
    toast_hw_mode: "Donanım modu değiştirildi (Kod: {mode})",
    toast_os_mode: "İşletim sistemi güç kipi '{profile}' olarak ayarlandı",

    // Battery (battery.html)
    battery_title: "Pil & Güç Durumu - Tuxeren Control Center",
    page_battery_heading: "Pil Durumu",
    bat_card_level: "Pil Doluluk & Güç Durumu",
    bat_est_time: "Tahmini Kalan Süre:",
    bat_source_mode: "Güç Kaynağı & Mod:",
    bat_hw_status: "Donanım Durumu:",
    bat_card_health: "Pil Sağlığı",
    bat_full_design_cap: "Tam / Tasarım Kapasitesi:",
    bat_card_electrical: "Elektriksel Ölçümler",
    bat_voltage: "Çalışma Gerilimi:",
    bat_power_rate: "Anlık Güç Tüketimi:",
    bat_power_input: "Aktif Güç Girişi:",
    bat_card_details: "Donanım & Döngü Bilgileri",
    bat_energy_now: "Pildeki Mevcut Enerji Miktarı:",
    bat_cycle_count: "Donanım Şarj Döngüsü:",
    bat_charging_ac: "Şarj Ediliyor (Şebeke)",
    bat_on_battery: "Pilde Çalışıyor (Deşarj)",
    bat_full_idle: "Şebeke Gücü (Dolu/Beklemede)",
    bat_source_ac: "AC Adaptör (Şebeke)",
    bat_source_internal: "Dahili Batarya",
    bat_source_ac_power: "AC Şebeke Gücü",
    bat_not_readable: "Okunamadı",
    bat_not_measured: "0 W / Ölçülemedi",
    bat_not_supported: "Desteklenmiyor / 0",
    bat_unknown: "Bilinmiyor",

    // Display (display.html)
    display_title: "Görüntü Ayarları - Tuxeren Control Center",
    page_display_heading: "Görüntü Ayarları",
    btn_reset_defaults: "Varsayılana Sıfırla",
    btn_reset_title: "Tüm görüntü ve renk ayarlarını fabrika ayarlarına sıfırla",
    card_general_color: "Genel Parlaklık & Renk",
    disp_brightness: "Parlaklık (Brightness)",
    disp_color_temp: "Renk Sıcaklığı (Color Temp)",
    disp_temp_warm: "3000K (Sıcak)",
    disp_temp_cool: "9000K (Soğuk)",
    disp_saturation: "Renk Doygunluğu (Saturation)",
    disp_sat_bw: "0.0 (S/B)",
    disp_sat_std: "1.0 (Standart)",
    disp_sat_vivid: "2.0 (Canlı)",
    disp_gamma: "Gamma Düzeltmesi",
    disp_gamma_light: "0.5 (Açık)",
    disp_gamma_dark: "2.0 (Koyu)",
    card_rgb_channels: "RGB Renk Kanalları",
    disp_channel_red: "Kırmızı Kanal (Red)",
    disp_channel_green: "Yeşil Kanal (Green)",
    disp_channel_blue: "Mavi Kanal (Blue)",
    toast_display_reset: "Görüntü ayarları sıfırlandı",

    // Device Info (device_info.html)
    device_info_title: "Cihaz Bilgileri - Tuxeren Control Center",
    page_device_heading: "Cihaz Bilgileri",
    panel_device_chassis: "Cihaz & Kasa",
    info_model_name: "Model Adı:",
    info_vendor: "Üretici:",
    info_chassis_code: "Kasa / Platform Kodu:",
    info_board_sku: "Anakart SKU / Revizyon:",
    info_serial_number: "Seri Numarası:",
    panel_firmware_ec: "Firmware & Denetleyici",
    info_bios_version: "BIOS Sürümü:",
    info_bios_date: "BIOS Tarihi:",
    info_ec_version: "EC Sürümü:",
    info_platform_driver: "Platform Sürücüsü:",
    panel_core_hw: "Temel Donanım",
    info_cpu: "İşlemci (CPU):",
    info_igpu: "Dahili Grafik (iGPU):",
    info_dgpu: "Harici Grafik (dGPU):",
    info_ram: "Sistem Belleği (RAM):",
    info_storage: "Depolama:",
    panel_os_drivers: "İşletim Sistemi & Sürücüler",
    info_os: "İşletim Sistemi:",
    info_desktop_protocol: "Masaüstü & Protokol:",
    info_kernel: "Linux Çekirdeği:",
    info_graphics_driver: "Grafik Sürücüsü:",

    // Keyboard Light (keyboard_light.html)
    kbd_title: "Aydınlatma Ayarları - Tuxeren Control Center",
    page_kbd_heading: "Aydınlatma Ayarları",
    section_main_colors: "Ana Renkler",
    section_custom_color: "Özel Renk",
    btn_open_palette: "Palet",
    recent_colors_label: "Son Kullanılanlar",
    palette_modal_title: "Renk Paleti",
    btn_palette_select: "Rengi Seç",
    section_brightness: "Işık Şiddeti",
    brightness_off: "0 (Kapalı)",
    brightness_max: "4 (Maksimum)",
    color_off: "Kapat",
    color_cyan: "Cyan",
    color_ice_blue: "Buz Mavisi",
    color_blue: "Mavi",
    color_purple: "Mor",
    color_magenta: "Macenta",
    color_pink: "Pembe",
    color_red: "Kırmızı",
    color_orange: "Turuncu",
    color_yellow: "Sarı",
    color_green: "Yeşil",
    color_white: "Beyaz",
    apply_custom_color: "Seçili rengi klavyeye uygula",
    toast_preset_set: "{preset} rengi uygulandı",
    toast_custom_rgb_set: "Özel RGB ({hex}) uygulandı",
    toast_brightness_set: "Parlaklık seviyesi: {level}"
  },

  en: {
    // Navigation & Common
    nav_main_menu: "Main Menu",
    nav_back: "Back",
    perm_banner_text: "Driver permission required for hardware access (<code>tuxedo-drivers</code>).",
    perm_btn_grant: "Grant Permission",
    perm_requesting: "Requesting permission via pkexec...",
    perm_granted: "Permission granted",
    perm_error: "Permission error:",
    error_prefix: "Error:",
    status_perm_required: "Permission Required",
    status_connected: "Connected",
    modal_close: "Close",

    // Lobby (index.html)
    lobby_title: "Tuxeren Control Center (TxeCtrl)",
    hero_title: "Tuxeren Control Center",
    menu_general: "General Settings",
    menu_general_desc: "Quick Settings, Windows Devices, In-game OSD, Keyboard Mapping",
    menu_perf: "Performance",
    menu_perf_desc: "Hardware & Power Profile Selection",
    menu_light: "Lighting Settings",
    menu_light_desc: "Keyboard Backlight",
    menu_battery: "Battery",
    menu_battery_desc: "Battery Mode, Battery Status",
    menu_display: "Display Settings",
    menu_display_desc: "Display Functions",
    menu_device: "Device Information",
    menu_device_desc: "Device Details, About",

    // General Settings (general.html)
    general_title: "General Settings - Tuxeren Control Center",
    page_general_heading: "General Settings",
    setting_language_title: "Application Language",
    setting_language_desc: "Select the display language for the interface",
    setting_winkey_title: "Disable Windows Key",
    setting_winkey_desc: "Prevents accidental press during gaming",
    setting_touchpad_title: "Disable Touchpad",
    setting_touchpad_desc: "Locks touch surface and tap detection",
    toast_winkey_disabled: "Windows key disabled",
    toast_winkey_enabled: "Windows key enabled",
    toast_touchpad_disabled: "Touchpad locked",
    toast_touchpad_enabled: "Touchpad enabled",
    toast_lang_changed: "Language set to English",

    // Performance (performance.html)
    perf_title: "Performance Management - Tuxeren Control Center",
    page_perf_heading: "Performance Management",
    telemetry_title: "System Monitoring",
    telemetry_cpu: "PROCESSOR (CPU)",
    telemetry_gpu: "GRAPHICS (GPU)",
    telemetry_ram: "MEMORY (RAM)",
    telemetry_disk: "STORAGE (DISK)",
    telemetry_fans: "COOLING FANS",
    telemetry_fan_cpu: "CPU FAN",
    telemetry_fan_gpu: "GPU FAN",
    telemetry_net: "NETWORK TRAFFIC",
    telemetry_net_down: "DOWNLOAD",
    telemetry_net_up: "UPLOAD",
    card_bios_title: "BIOS Power Mode",
    mode_office_title: "Office Mode",
    mode_office_desc: "45W CPU Limit • Low Fan Speed",
    mode_game_title: "Gaming Mode",
    mode_game_desc: "75W CPU Limit • Balanced Fan Profile",
    mode_turbo_title: "Turbo Mode",
    mode_turbo_desc: "90W CPU Limit • Turbo Fan",
    card_os_title: "OS Power Profile",
    os_powersaver_title: "Power Saver",
    os_powersaver_desc: "Battery life focused, lower CPU clock",
    os_balanced_title: "Balanced",
    os_balanced_desc: "Standard usage, dynamic frequency scaling",
    os_perf_title: "High Performance",
    os_perf_desc: "Maximum core clock frequencies",
    toast_hw_mode: "Hardware mode changed (Code: {mode})",
    toast_os_mode: "OS power profile set to '{profile}'",

    // Battery (battery.html)
    battery_title: "Battery & Power Status - Tuxeren Control Center",
    page_battery_heading: "Battery Status",
    bat_card_level: "Battery Level & Power Status",
    bat_est_time: "Estimated Remaining Time:",
    bat_source_mode: "Power Source & Mode:",
    bat_hw_status: "Hardware Status:",
    bat_card_health: "Battery Health",
    bat_full_design_cap: "Full / Design Capacity:",
    bat_card_electrical: "Electrical Measurements",
    bat_voltage: "Operating Voltage:",
    bat_power_rate: "Current Power Draw:",
    bat_power_input: "Active Power Input:",
    bat_card_details: "Hardware & Cycle Info",
    bat_energy_now: "Current Energy in Battery:",
    bat_cycle_count: "Hardware Charge Cycles:",
    bat_charging_ac: "Charging (AC Power)",
    bat_on_battery: "On Battery (Discharging)",
    bat_full_idle: "AC Power (Full/Standby)",
    bat_source_ac: "AC Adapter (Mains)",
    bat_source_internal: "Internal Battery",
    bat_source_ac_power: "AC Mains Power",
    bat_not_readable: "Unreadable",
    bat_not_measured: "0 W / Unmeasured",
    bat_not_supported: "Unsupported / 0",
    bat_unknown: "Unknown",

    // Display (display.html)
    display_title: "Display Settings - Tuxeren Control Center",
    page_display_heading: "Display Settings",
    btn_reset_defaults: "Reset to Defaults",
    btn_reset_title: "Reset all display and color settings to factory defaults",
    card_general_color: "General Brightness & Color",
    disp_brightness: "Brightness",
    disp_color_temp: "Color Temperature",
    disp_temp_warm: "3000K (Warm)",
    disp_temp_cool: "9000K (Cool)",
    disp_saturation: "Color Saturation",
    disp_sat_bw: "0.0 (B/W)",
    disp_sat_std: "1.0 (Standard)",
    disp_sat_vivid: "2.0 (Vivid)",
    disp_gamma: "Gamma Correction",
    disp_gamma_light: "0.5 (Light)",
    disp_gamma_dark: "2.0 (Dark)",
    card_rgb_channels: "RGB Color Channels",
    disp_channel_red: "Red Channel",
    disp_channel_green: "Green Channel",
    disp_channel_blue: "Blue Channel",
    toast_display_reset: "Display settings reset to defaults",

    // Device Info (device_info.html)
    device_info_title: "Device Information - Tuxeren Control Center",
    page_device_heading: "Device Information",
    panel_device_chassis: "Device & Chassis",
    info_model_name: "Model Name:",
    info_vendor: "Vendor:",
    info_chassis_code: "Chassis / Platform Code:",
    info_board_sku: "Motherboard SKU / Rev:",
    info_serial_number: "Serial Number:",
    panel_firmware_ec: "Firmware & Controller",
    info_bios_version: "BIOS Version:",
    info_bios_date: "BIOS Date:",
    info_ec_version: "EC Version:",
    info_platform_driver: "Platform Driver:",
    panel_core_hw: "Core Hardware",
    info_cpu: "Processor (CPU):",
    info_igpu: "Integrated Graphics (iGPU):",
    info_dgpu: "Dedicated Graphics (dGPU):",
    info_ram: "System Memory (RAM):",
    info_storage: "Storage:",
    panel_os_drivers: "Operating System & Drivers",
    info_os: "Operating System:",
    info_desktop_protocol: "Desktop & Protocol:",
    info_kernel: "Linux Kernel:",
    info_graphics_driver: "Graphics Driver:",

    // Keyboard Light (keyboard_light.html)
    kbd_title: "Lighting Settings - Tuxeren Control Center",
    page_kbd_heading: "Lighting Settings",
    section_main_colors: "Primary Colors",
    section_custom_color: "Custom Color",
    btn_open_palette: "Palette",
    recent_colors_label: "Recent Colors",
    palette_modal_title: "Color Palette",
    btn_palette_select: "Select Color",
    section_brightness: "Brightness Level",
    brightness_off: "0 (Off)",
    brightness_max: "4 (Maximum)",
    color_off: "Off",
    color_cyan: "Cyan",
    color_ice_blue: "Ice Blue",
    color_blue: "Blue",
    color_purple: "Purple",
    color_magenta: "Magenta",
    color_pink: "Pink",
    color_red: "Red",
    color_orange: "Orange",
    color_yellow: "Yellow",
    color_green: "Green",
    color_white: "White",
    apply_custom_color: "Apply selected color to keyboard",
    toast_preset_set: "{preset} color applied",
    toast_custom_rgb_set: "Custom RGB ({hex}) applied",
    toast_brightness_set: "Brightness level: {level}"
  }
};

let currentLanguage = localStorage.getItem("mcc_lang") || "tr";

function t(key, fallback = "", params = null) {
  const dict = translations[currentLanguage] || translations.tr;
  let text = dict[key] !== undefined ? dict[key] : (translations.tr[key] !== undefined ? translations.tr[key] : fallback);
  if (params && typeof params === "object") {
    Object.keys(params).forEach((paramKey) => {
      text = text.replace(new RegExp(`\\{${paramKey}\\}`, "g"), params[paramKey]);
    });
  }
  return text;
}

function getLanguage() {
  return currentLanguage;
}

function applyTranslations(lang) {
  if (lang && translations[lang]) {
    currentLanguage = lang;
  }
  document.documentElement.lang = currentLanguage;

  // 1. Text content or innerHTML for elements with data-i18n
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    const val = t(key);
    if (val) {
      if (val.includes("<") && val.includes(">")) {
        el.innerHTML = val;
      } else {
        el.textContent = val;
      }
    }
  });

  // 2. Titles (tooltips)
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    const key = el.getAttribute("data-i18n-title");
    const val = t(key);
    if (val) el.setAttribute("title", val);
  });

  // 3. Placeholders
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const key = el.getAttribute("data-i18n-placeholder");
    const val = t(key);
    if (val) el.setAttribute("placeholder", val);
  });

  // 4. Document title if data-i18n-page is on body or root
  const pageKey = document.body?.getAttribute("data-i18n-page") || document.documentElement?.getAttribute("data-i18n-page");
  if (pageKey) {
    const val = t(pageKey);
    if (val) document.title = val;
  }
}

function setLanguage(lang) {
  if (!translations[lang]) return;
  currentLanguage = lang;
  localStorage.setItem("mcc_lang", lang);
  applyTranslations(lang);
  window.dispatchEvent(new CustomEvent("mcc_language_changed", { detail: { language: lang } }));
}

// Auto-run applyTranslations immediately and on DOMContentLoaded
applyTranslations();
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => applyTranslations());
}
