const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const permBanner = document.getElementById("perm-banner");
const btnGrantPerm = document.getElementById("btn-grant-perm");
const toast = document.getElementById("toast");

function showToast(msg) {
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2200);
}

async function loadDeviceInfo() {
  try {
    const info = await invoke("get_device_info");
    if (!info) return;

    if (info.acpi_call_available) {
      if (permBanner) permBanner.classList.add("hidden");
    } else {
      if (permBanner) permBanner.classList.remove("hidden");
    }

    const setVal = (id, val) => {
      const el = document.getElementById(id);
      if (el) el.textContent = val || "-";
    };

    // Panel 1: Cihaz & Kasa (Sol Üst)
    setVal("val-model-name", info.model_name);
    setVal("val-vendor", info.vendor);
    setVal("val-chassis-code", info.chassis_code);
    setVal("val-board-sku", info.board_sku_rev);
    setVal("val-serial-number", info.serial_number);

    // Panel 2: Firmware & Denetleyici (Sağ Üst)
    setVal("val-bios-version", info.bios_version);
    setVal("val-bios-date", info.bios_date);
    setVal("val-ec-version", info.ec_version);
    setVal("val-platform-driver", info.platform_driver);

    // Panel 3: Temel Donanım (Sol Alt)
    setVal("val-cpu", info.cpu);
    setVal("val-igpu", info.igpu);
    setVal("val-dgpu", info.dgpu);
    setVal("val-ram", info.ram);
    setVal("val-storage", info.storage);

    // Panel 4: İşletim Sistemi & Sürücüler (Sağ Alt)
    setVal("val-os", info.os);
    setVal("val-desktop-protocol", info.desktop_protocol);
    setVal("val-kernel", info.kernel);
    setVal("val-graphics-driver", info.graphics_driver);
  } catch (err) {
    console.error("Device info load error:", err);
    showToast(`${typeof t === "function" ? t("error_prefix") : "Hata:"} ${err}`);
  }
}

if (btnGrantPerm) {
  btnGrantPerm.addEventListener("click", async () => {
    try {
      showToast(typeof t === "function" ? t("perm_requesting") : "pkexec ile izin isteniyor...");
      const msg = await invoke("grant_permissions");
      showToast(msg || (typeof t === "function" ? t("perm_granted") : "İzin verildi"));
      setTimeout(loadDeviceInfo, 600);
    } catch (e) {
      showToast(`${typeof t === "function" ? t("perm_error") : "İzin hatası:"} ${e}`);
    }
  });
}

loadDeviceInfo();
