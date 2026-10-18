const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const statusIndicator = document.getElementById("status-indicator");
const statusText = document.getElementById("status-text");
const permBanner = document.getElementById("perm-banner");
const btnGrantPerm = document.getElementById("btn-grant-perm");
const searchInput = document.getElementById("menu-search");
const toast = document.getElementById("toast");

function showToast(msg) {
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2200);
}

// Search filter across tiles
if (searchInput) {
  searchInput.addEventListener("input", (e) => {
    const query = e.target.value.toLowerCase().trim();
    document.querySelectorAll(".menu-tile").forEach((tile) => {
      const text = tile.textContent.toLowerCase();
      tile.style.display = text.includes(query) ? "flex" : "none";
    });
  });
}

async function refreshStatus() {
  try {
    const data = await invoke("get_status");
    if (!data) return;

    if (!data.has_permission) {
      if (permBanner) permBanner.classList.remove("hidden");
      if (statusIndicator) statusIndicator.classList.add("error");
      if (statusText) statusText.textContent = typeof t === "function" ? t("status_perm_required") : "İzin Gerekli";
      return;
    }

    if (permBanner) permBanner.classList.add("hidden");
    if (statusIndicator) statusIndicator.classList.remove("error");
    if (statusText) statusText.textContent = typeof t === "function" ? t("status_connected") : "Bağlandı";
  } catch (err) {
    console.error("Status error:", err);
  }
}

if (btnGrantPerm) {
  btnGrantPerm.addEventListener("click", async () => {
    try {
      showToast(typeof t === "function" ? t("perm_requesting") : "pkexec ile izin isteniyor...");
      const msg = await invoke("grant_permissions");
      showToast(msg || (typeof t === "function" ? t("perm_granted") : "İzin verildi"));
      setTimeout(refreshStatus, 600);
    } catch (e) {
      showToast(`${typeof t === "function" ? t("perm_error") : "İzin hatası:"} ${e}`);
    }
  });
}

refreshStatus();

// Hero Logo Fan Speed Controller
// Continuous rotation without jumping or rewinding angle
const heroLogoContainer = document.querySelector(".hero-logo-container");
const heroFanArray = document.querySelector(".hero-fan-array");

if (heroLogoContainer && heroFanArray) {
  // Playback rates relative to base 12s period:
  // 1x = 12s, 3x = 4s, 12x = 1s, 0x = stopped
  const speedStages = [
    { rate: 2 },
    { rate: 6 },
    { rate: 12 },
    { rate: 0 }
  ];
  const STORAGE_KEY = "mcc_fan_speed_stage";

  const fanAnimation = heroFanArray.animate(
    [
      { transform: "rotate(0deg)" },
      { transform: "rotate(360deg)" }
    ],
    {
      duration: 12000,
      iterations: Infinity,
      easing: "linear"
    }
  );

  const applyStage = (index) => {
    const stage = speedStages[index];
    if (stage.rate === 0) {
      fanAnimation.pause();
    } else {
      if (fanAnimation.playState === "paused") {
        fanAnimation.play();
      }
      if (typeof fanAnimation.updatePlaybackRate === "function") {
        fanAnimation.updatePlaybackRate(stage.rate);
      } else {
        fanAnimation.playbackRate = stage.rate;
      }
    }
  };

  const savedIndex = parseInt(localStorage.getItem(STORAGE_KEY), 10);
  let currentStageIndex = !isNaN(savedIndex) && savedIndex >= 0 && savedIndex < speedStages.length ? savedIndex : 0;
  applyStage(currentStageIndex);

  heroLogoContainer.addEventListener("click", () => {
    currentStageIndex = (currentStageIndex + 1) % speedStages.length;
    localStorage.setItem(STORAGE_KEY, currentStageIndex);
    applyStage(currentStageIndex);
  });
}
