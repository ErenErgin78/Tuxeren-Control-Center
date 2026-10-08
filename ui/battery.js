const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

function formatCapacity(val) {
  if (val === null || val === undefined) return "-";
  // Usually reported in micro-amp-hours (µAh) or micro-watt-hours (µWh)
  if (val > 100000) {
    const mah = (val / 1000).toFixed(0);
    return `${mah} mAh`;
  }
  return `${val}`;
}

async function updateBatteryUI() {
  try {
    const data = await invoke("get_battery_info");
    if (!data) return;

    // 1. Percentage
    const percentEl = document.getElementById("bat-percent");
    const progressBar = document.getElementById("bat-progress-bar");
    if (percentEl) percentEl.textContent = data.percentage;
    if (progressBar) {
      progressBar.style.width = `${data.percentage}%`;
      progressBar.classList.remove("warn", "crit");
      if (data.percentage <= 15) {
        progressBar.classList.add("crit");
      } else if (data.percentage <= 30) {
        progressBar.classList.add("warn");
      }
    }

    // 2. Charging Status & Badge
    const statusBadge = document.getElementById("bat-status-badge");
    const powerSourceText = document.getElementById("bat-power-source");
    const statusText = document.getElementById("bat-status-text");

    let sourceLabel = "Bilinmiyor";
    if (data.is_charging) {
      sourceLabel = "Şarj Ediliyor (Şebeke)";
      if (statusBadge) statusBadge.classList.remove("discharging");
    } else if (data.on_battery) {
      sourceLabel = "Pilde Çalışıyor (Deşarj)";
      if (statusBadge) statusBadge.classList.add("discharging");
    } else {
      sourceLabel = "Şebeke Gücü (Dolu/Beklemede)";
      if (statusBadge) statusBadge.classList.remove("discharging");
    }

    if (powerSourceText) powerSourceText.textContent = sourceLabel;
    if (statusText) statusText.textContent = data.status;

    // 3. Time remaining / Estimated shutdown
    const timeEl = document.getElementById("bat-time-remaining");
    if (timeEl) {
      timeEl.textContent = data.estimated_time_str;
    }

    // 4. Battery Health
    const healthVal = document.getElementById("bat-health-val");
    const healthBar = document.getElementById("health-progress-bar");

    if (data.health_percent !== null && data.health_percent !== undefined) {
      const h = data.health_percent.toFixed(1);
      if (healthVal) healthVal.textContent = `%${h}`;
      if (healthBar) {
        healthBar.style.width = `${Math.min(data.health_percent, 100)}%`;
        healthBar.classList.remove("warn", "crit");
        if (data.health_percent < 50) {
          healthBar.classList.add("crit");
        } else if (data.health_percent < 75) {
          healthBar.classList.add("warn");
        }
      }
    } else {
      if (healthVal) healthVal.textContent = "Okunamadı";
    }

    // 5. Details
    const setElem = (id, text) => {
      const el = document.getElementById(id);
      if (el) el.textContent = text || "-";
    };

    setElem("bat-cap-design", formatCapacity(data.capacity_design));
    setElem("bat-cap-full", formatCapacity(data.capacity_full));
    setElem("val-voltage", data.voltage_volts !== null ? `${data.voltage_volts.toFixed(3)} V` : "-");
    setElem(
      "val-power-rate",
      data.power_rate_watts !== null ? `${data.power_rate_watts.toFixed(2)} W` : "0 W / Ölçülemedi"
    );
    setElem("val-source-type", data.is_charging ? "AC Adaptör (Şebeke)" : (data.on_battery ? "Dahili Batarya" : "AC Şebeke Gücü"));
    setElem("val-design-cap", formatCapacity(data.capacity_design));
    setElem("val-full-cap", formatCapacity(data.capacity_full));
    setElem("val-now-cap", formatCapacity(data.capacity_now));
    setElem("val-cycle-count", data.cycle_count !== null && data.cycle_count !== undefined ? `${data.cycle_count}` : "Desteklenmiyor / 0");

  } catch (err) {
    console.error("Failed to update battery UI:", err);
  }
}

// Initial update & 3-second auto refresh interval
updateBatteryUI();
setInterval(updateBatteryUI, 3000);
