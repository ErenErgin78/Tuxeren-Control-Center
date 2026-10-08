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
      if (statusText) statusText.textContent = "İzin Gerekli";
      return;
    }

    if (permBanner) permBanner.classList.add("hidden");
    if (statusIndicator) statusIndicator.classList.remove("error");
    if (statusText) statusText.textContent = "Bağlandı";
  } catch (err) {
    console.error("Status error:", err);
  }
}

if (btnGrantPerm) {
  btnGrantPerm.addEventListener("click", async () => {
    try {
      showToast("pkexec ile izin isteniyor...");
      const msg = await invoke("grant_permissions");
      showToast(msg || "İzin verildi");
      setTimeout(refreshStatus, 600);
    } catch (e) {
      showToast(`İzin hatası: ${e}`);
    }
  });
}

refreshStatus();
