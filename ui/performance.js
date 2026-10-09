const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const hwActiveBadge = document.getElementById("hw-active-badge");
const osActiveBadge = document.getElementById("os-active-badge");
const toast = document.getElementById("toast");

let currentHwMode = null;
let currentOsProfile = null;

function showToast(msg) {
  if (!toast) return;
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2200);
}

async function refreshPerformance() {
  try {
    const state = await invoke("get_performance_state");
    if (!state) return;

    currentHwMode = state.hardware_mode;
    currentOsProfile = state.os_profile;

    // 1. Update Hardware UI
    document.querySelectorAll("[data-hw]").forEach((el) => {
      const modeVal = parseInt(el.getAttribute("data-hw"), 10);
      if (modeVal === currentHwMode) {
        el.classList.add("active");
      } else {
        el.classList.remove("active");
      }
    });

    if (hwActiveBadge) {
      hwActiveBadge.textContent = state.hardware_mode_name;
    }

    // 2. Update OS UI
    document.querySelectorAll("[data-os]").forEach((el) => {
      const osVal = el.getAttribute("data-os");
      if (osVal === currentOsProfile) {
        el.classList.add("active");
      } else {
        el.classList.remove("active");
      }
    });

    if (osActiveBadge) {
      osActiveBadge.textContent = state.os_profile;
    }
  } catch (err) {
    console.error("Failed to load performance state:", err);
  }
}

// Hardware mode click handlers
document.querySelectorAll("[data-hw]").forEach((el) => {
  el.addEventListener("click", async () => {
    const mode = parseInt(el.getAttribute("data-hw"), 10);
    try {
      await invoke("set_hardware_mode", { mode });
      showToast(`Donanım modu değiştirildi (Kod: ${mode})`);
      await refreshPerformance();
    } catch (e) {
      showToast(`Hata: ${e}`);
    }
  });
});

// OS mode click handlers
document.querySelectorAll("[data-os]").forEach((el) => {
  el.addEventListener("click", async () => {
    const profile = el.getAttribute("data-os");
    try {
      await invoke("set_os_power_profile", { profile });
      showToast(`Fedora güç kipi '${profile}' olarak ayarlandı`);
      await refreshPerformance();
    } catch (e) {
      showToast(`Hata: ${e}`);
    }
  });
});

function updateTempBadge(el, temp) {
  if (!el) return;
  el.classList.remove("temp-cool", "temp-warm", "temp-hot");
  if (temp >= 80) {
    el.classList.add("temp-hot");
  } else if (temp >= 65) {
    el.classList.add("temp-warm");
  } else {
    el.classList.add("temp-cool");
  }
}

async function refreshMetrics() {
  try {
    const m = await invoke("get_system_metrics");
    if (!m) return;

    // CPU
    const cpuUsageEl = document.getElementById("cpu-usage");
    const cpuBarEl = document.getElementById("cpu-bar");
    const cpuTempEl = document.getElementById("cpu-temp");
    if (cpuUsageEl) cpuUsageEl.textContent = Math.round(m.cpu_percent);
    if (cpuBarEl) cpuBarEl.style.width = `${Math.min(100, Math.max(0, m.cpu_percent))}%`;
    if (cpuTempEl) {
      cpuTempEl.textContent = `${m.cpu_temp}°C`;
      updateTempBadge(cpuTempEl, m.cpu_temp);
    }

    // GPU
    const gpuUsageEl = document.getElementById("gpu-usage");
    const gpuBarEl = document.getElementById("gpu-bar");
    const gpuTempEl = document.getElementById("gpu-temp");
    if (gpuUsageEl) gpuUsageEl.textContent = Math.round(m.gpu_percent);
    if (gpuBarEl) gpuBarEl.style.width = `${Math.min(100, Math.max(0, m.gpu_percent))}%`;
    if (gpuTempEl) {
      gpuTempEl.textContent = `${m.gpu_temp}°C`;
      updateTempBadge(gpuTempEl, m.gpu_temp);
    }

    // RAM
    const ramUsageEl = document.getElementById("ram-usage");
    const ramBarEl = document.getElementById("ram-bar");
    const ramSubEl = document.getElementById("ram-sub");
    if (ramUsageEl) ramUsageEl.textContent = Math.round(m.ram_percent);
    if (ramBarEl) ramBarEl.style.width = `${Math.min(100, Math.max(0, m.ram_percent))}%`;
    if (ramSubEl) ramSubEl.textContent = `${m.ram_used_gb.toFixed(1)} GB / ${m.ram_total_gb.toFixed(1)} GB`;

    // Disk
    const diskUsageEl = document.getElementById("disk-usage");
    const diskBarEl = document.getElementById("disk-bar");
    const diskSubEl = document.getElementById("disk-sub");
    if (diskUsageEl) diskUsageEl.textContent = Math.round(m.disk_percent);
    if (diskBarEl) diskBarEl.style.width = `${Math.min(100, Math.max(0, m.disk_percent))}%`;
    if (diskSubEl) diskSubEl.textContent = `${Math.round(m.disk_used_gb)} GB / ${Math.round(m.disk_total_gb)} GB`;

    // Fans
    const fanCpuRpmEl = document.getElementById("fan-cpu-rpm");
    const fanCpuBarEl = document.getElementById("fan-cpu-bar");
    const fanGpuRpmEl = document.getElementById("fan-gpu-rpm");
    const fanGpuBarEl = document.getElementById("fan-gpu-bar");
    const fanIconEl = document.querySelector(".fan-telemetry-icon");

    const cpuDuty = Math.min(100, Math.max(0, Math.round(m.cpu_fan_duty)));
    const gpuDuty = Math.min(100, Math.max(0, Math.round(m.gpu_fan_duty)));

    if (fanCpuRpmEl) fanCpuRpmEl.textContent = `${m.cpu_fan_rpm} RPM (${cpuDuty}%)`;
    if (fanCpuBarEl) fanCpuBarEl.style.width = `${cpuDuty}%`;
    if (fanGpuRpmEl) fanGpuRpmEl.textContent = `${m.gpu_fan_rpm} RPM (${gpuDuty}%)`;
    if (fanGpuBarEl) fanGpuBarEl.style.width = `${gpuDuty}%`;

    if (fanIconEl) {
      const maxRpm = Math.max(m.cpu_fan_rpm, m.gpu_fan_rpm);
      if (maxRpm > 0) {
        fanIconEl.classList.add("spinning");
        const speed = Math.max(0.4, 2.5 - (maxRpm / 3000));
        fanIconEl.style.animationDuration = `${speed.toFixed(2)}s`;
      } else {
        fanIconEl.classList.remove("spinning");
      }
    }

    // Network Speeds
    const netDownSpeedEl = document.getElementById("net-down-speed");
    const netDownBarEl = document.getElementById("net-down-bar");
    const netUpSpeedEl = document.getElementById("net-up-speed");
    const netUpBarEl = document.getElementById("net-up-bar");

    if (netDownSpeedEl) netDownSpeedEl.textContent = formatSpeed(m.net_down_bytes);
    if (netUpSpeedEl) netUpSpeedEl.textContent = formatSpeed(m.net_up_bytes);

    if (netDownBarEl) {
      const downPct = Math.min(100, (m.net_down_bytes / (10 * 1024 * 1024)) * 100);
      netDownBarEl.style.width = `${Math.max(m.net_down_bytes > 0 ? 5 : 0, downPct)}%`;
    }
    if (netUpBarEl) {
      const upPct = Math.min(100, (m.net_up_bytes / (5 * 1024 * 1024)) * 100);
      netUpBarEl.style.width = `${Math.max(m.net_up_bytes > 0 ? 5 : 0, upPct)}%`;
    }
  } catch (err) {
    console.error("Failed to load telemetry metrics:", err);
  }
}

function formatSpeed(bytesPerSec) {
  if (bytesPerSec >= 1024 * 1024) {
    return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
  } else if (bytesPerSec >= 1024) {
    return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  } else {
    return `${Math.round(bytesPerSec)} B/s`;
  }
}

// Initial fetch and polling
refreshPerformance();
setInterval(refreshPerformance, 500);

refreshMetrics();
setInterval(refreshMetrics, 1000);
