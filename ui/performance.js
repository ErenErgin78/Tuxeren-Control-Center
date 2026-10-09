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

// Initial fetch and light polling (500ms) to reflect physical button changes
refreshPerformance();
setInterval(refreshPerformance, 500);
