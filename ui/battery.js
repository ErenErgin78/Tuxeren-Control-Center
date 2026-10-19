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

    let sourceLabel = typeof t === "function" ? t("bat_unknown") : "Bilinmiyor";
    if (data.is_charging) {
      sourceLabel = typeof t === "function" ? t("bat_charging_ac") : "Şarj Ediliyor (Şebeke)";
      if (statusBadge) statusBadge.classList.remove("discharging");
    } else if (data.on_battery) {
      sourceLabel = typeof t === "function" ? t("bat_on_battery") : "Pilde Çalışıyor (Deşarj)";
      if (statusBadge) statusBadge.classList.add("discharging");
    } else {
      sourceLabel = typeof t === "function" ? t("bat_full_idle") : "Şebeke Gücü (Dolu/Beklemede)";
      if (statusBadge) statusBadge.classList.remove("discharging");
    }

    if (powerSourceText) powerSourceText.textContent = sourceLabel;
    if (statusText) statusText.textContent = data.status;

    // 3. Time remaining / Estimated shutdown
    const timeEl = document.getElementById("bat-time-remaining");
    if (timeEl) {
      let timeText = data.estimated_time_str;
      if (typeof t === "function") {
        if (data.is_charging) {
          if (data.status && data.status.toLowerCase() === "full") {
            timeText = t("bat_status_full_mains");
          } else {
            timeText = t("bat_status_charging");
          }
        } else if (!data.on_battery) {
          timeText = t("bat_status_not_charging");
        } else if (data.estimated_minutes_left !== null && data.estimated_minutes_left !== undefined) {
          const minsTotal = data.estimated_minutes_left;
          const h = Math.floor(minsTotal / 60);
          const m = minsTotal % 60;
          if (h > 0) {
            timeText = t("bat_time_hours_mins", { hours: h, mins: m });
          } else {
            timeText = t("bat_time_mins", { mins: m });
          }
        } else if (data.estimated_time_str === "No draw detected (0 mA)") {
          timeText = t("bat_status_no_draw");
        } else if (data.estimated_time_str === "Waiting for discharge rate...") {
          timeText = t("bat_status_waiting_rate");
        } else if (data.estimated_time_str === "Waiting for sensor data") {
          timeText = t("bat_status_waiting_sensor");
        } else {
          timeText = t("bat_status_calculating");
        }
      }
      timeEl.textContent = timeText;
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
      if (healthVal) healthVal.textContent = typeof t === "function" ? t("bat_not_readable") : "Okunamadı";
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
      data.power_rate_watts !== null ? `${data.power_rate_watts.toFixed(2)} W` : (typeof t === "function" ? t("bat_not_measured") : "0 W / Ölçülemedi")
    );
    setElem("val-source-type", data.is_charging ? (typeof t === "function" ? t("bat_source_ac") : "AC Adaptör (Şebeke)") : (data.on_battery ? (typeof t === "function" ? t("bat_source_internal") : "Dahili Batarya") : (typeof t === "function" ? t("bat_source_ac_power") : "AC Şebeke Gücü")));
    setElem("val-design-cap", formatCapacity(data.capacity_design));
    setElem("val-full-cap", formatCapacity(data.capacity_full));
    setElem("val-now-cap", formatCapacity(data.capacity_now));
    setElem("val-cycle-count", data.cycle_count !== null && data.cycle_count !== undefined ? `${data.cycle_count}` : (typeof t === "function" ? t("bat_not_supported") : "Desteklenmiyor / 0"));

  } catch (err) {
    console.error("Failed to update battery UI:", err);
  }
}

// Initial update & 3-second auto refresh interval
updateBatteryUI();
setInterval(updateBatteryUI, 3000);
