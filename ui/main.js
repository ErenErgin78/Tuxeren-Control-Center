const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const chassisLabel = document.getElementById("chassis-label");
const statusIndicator = document.getElementById("status-indicator");
const statusText = document.getElementById("status-text");
const permBanner = document.getElementById("perm-banner");
const btnGrantPerm = document.getElementById("btn-grant-perm");

const kbdBrightness = document.getElementById("kbd-brightness");
const brightnessVal = document.getElementById("brightness-val");
const customColorPicker = document.getElementById("custom-color-picker");
const rgbHexLabel = document.getElementById("rgb-hex-label");
const btnApplyCustomRgb = document.getElementById("btn-apply-custom-rgb");

const toast = document.getElementById("toast");

function showToast(msg) {
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2200);
}

async function refreshStatus() {
  try {
    const data = await invoke("get_status");
    if (!data) return;

    if (!data.has_permission) {
      permBanner.classList.remove("hidden");
      statusIndicator.classList.add("error");
      statusText.textContent = "Permission Required";
      return;
    }

    permBanner.classList.add("hidden");
    statusIndicator.classList.remove("error");
    statusText.textContent = "Connected";

    if (data.chassis_id) {
      chassisLabel.textContent = `Chassis ID: 0x${data.chassis_id.toString(16).toUpperCase()}`;
    }
  } catch (err) {
    console.error("Status error:", err);
  }
}

// Brightness Slider
kbdBrightness.addEventListener("input", (e) => {
  brightnessVal.textContent = e.target.value;
});

// Color Presets
document.querySelectorAll(".color-dot").forEach((dot) => {
  dot.addEventListener("click", async () => {
    const preset = dot.getAttribute("data-color");
    const brightness = parseInt(kbdBrightness.value, 10);
    try {
      await invoke("set_keyboard_preset", { preset, brightness });
      showToast(`Preset: ${preset.toUpperCase()}`);
    } catch (e) {
      showToast(`Error: ${e}`);
    }
  });
});

// Custom Color Picker
customColorPicker.addEventListener("input", (e) => {
  rgbHexLabel.textContent = e.target.value.toUpperCase();
});

btnApplyCustomRgb.addEventListener("click", async () => {
  const hex = customColorPicker.value;
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const brightness = parseInt(kbdBrightness.value, 10);
  try {
    await invoke("set_keyboard_rgb", { r, g, b, brightness });
    showToast(`RGB: (${r}, ${g}, ${b})`);
  } catch (e) {
    showToast(`Error: ${e}`);
  }
});

// Grant Permission Button
btnGrantPerm.addEventListener("click", async () => {
  try {
    showToast("Authenticating via pkexec...");
    const msg = await invoke("grant_permissions");
    showToast(msg || "Permissions granted");
    setTimeout(refreshStatus, 600);
  } catch (e) {
    showToast(`Permission error: ${e}`);
  }
});

refreshStatus();
