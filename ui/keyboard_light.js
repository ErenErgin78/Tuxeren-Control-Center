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

const kbdBrightness = document.getElementById("kbd-brightness");
const brightnessVal = document.getElementById("brightness-val");
const activeSwatch = document.getElementById("active-swatch");
const activeHexLabel = document.getElementById("active-hex-label");
const btnOpenPalette = document.getElementById("btn-open-palette");
const recentChipsList = document.getElementById("recent-chips-list");
const colorChips = document.querySelectorAll(".color-chip");
const toast = document.getElementById("toast");

// Inline Palette Modal Elements
const inlinePalettePanel = document.getElementById("inline-palette-panel");
const btnClosePalette = document.getElementById("btn-close-palette");
const spectrumCanvas = document.getElementById("color-spectrum-canvas");
const canvasCursor = document.getElementById("canvas-cursor");
const hueSlider = document.getElementById("hue-slider");
const modalPreviewDot = document.getElementById("modal-preview-dot");
const modalHexText = document.getElementById("modal-hex-text");
const btnPaletteSelect = document.getElementById("btn-palette-select");

let currentColorHex = localStorage.getItem("mcc_last_color") || "#00FFFF";
let recentColors = JSON.parse(localStorage.getItem("mcc_recent_colors") || '["#00FFFF", "#0077FF", "#00E676", "#FF0000", "#FFD600"]');
let lastActiveBrightness = 4;

// HSV State for Palette
let curHue = 180;
let curSat = 1.0;
let curVal = 1.0;
let isDraggingSpectrum = false;

function showToast(msg) {
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2200);
}

function hsvToRgb(h, s, v) {
  let f = (n, k = (n + h / 60) % 6) => v - v * s * Math.max(Math.min(k, 4 - k, 1), 0);
  let r = Math.round(f(5) * 255);
  let g = Math.round(f(3) * 255);
  let b = Math.round(f(1) * 255);
  return { r, g, b };
}

function rgbToHex(r, g, b) {
  return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1).toUpperCase()}`;
}

function hexToRgb(hex) {
  return {
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16)
  };
}

function hexToHsv(hex) {
  const { r, g, b } = hexToRgb(hex);
  let rNorm = r / 255, gNorm = g / 255, bNorm = b / 255;
  let max = Math.max(rNorm, gNorm, bNorm), min = Math.min(rNorm, gNorm, bNorm);
  let d = max - min;
  let h = 0, s = max === 0 ? 0 : d / max, v = max;

  if (max !== min) {
    switch (max) {
      case rNorm: h = (gNorm - bNorm) / d + (gNorm < bNorm ? 6 : 0); break;
      case gNorm: h = (bNorm - rNorm) / d + 2; break;
      case bNorm: h = (rNorm - gNorm) / d + 4; break;
    }
    h *= 60;
  }
  return { h, s, v };
}

function drawSpectrum(hue) {
  const ctx = spectrumCanvas.getContext("2d");
  const width = spectrumCanvas.width;
  const height = spectrumCanvas.height;

  // Base hue
  ctx.fillStyle = `hsl(${hue}, 100%, 50%)`;
  ctx.fillRect(0, 0, width, height);

  // White gradient (left to right)
  const whiteGrad = ctx.createLinearGradient(0, 0, width, 0);
  whiteGrad.addColorStop(0, "rgba(255,255,255,1)");
  whiteGrad.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = whiteGrad;
  ctx.fillRect(0, 0, width, height);

  // Black gradient (top to bottom)
  const blackGrad = ctx.createLinearGradient(0, 0, 0, height);
  blackGrad.addColorStop(0, "rgba(0,0,0,0)");
  blackGrad.addColorStop(1, "rgba(0,0,0,1)");
  ctx.fillStyle = blackGrad;
  ctx.fillRect(0, 0, width, height);
}

function updateModalPreview() {
  const { r, g, b } = hsvToRgb(curHue, curSat, curVal);
  const hex = rgbToHex(r, g, b);
  modalPreviewDot.style.backgroundColor = hex;
  modalHexText.textContent = hex;
  return hex;
}

function setCursorPos(s, v) {
  const x = s * spectrumCanvas.width;
  const y = (1 - v) * spectrumCanvas.height;
  canvasCursor.style.left = `${x}px`;
  canvasCursor.style.top = `${y}px`;
}

function updateActiveSwatchUI(hex) {
  currentColorHex = hex.toUpperCase();
  activeSwatch.style.backgroundColor = currentColorHex;
  activeHexLabel.textContent = currentColorHex;
  localStorage.setItem("mcc_last_color", currentColorHex);
}

function addRecentColor(hex) {
  const norm = hex.toUpperCase();
  if (norm === "#000000") return;
  recentColors = [norm, ...recentColors.filter((c) => c !== norm)].slice(0, 5);
  localStorage.setItem("mcc_recent_colors", JSON.stringify(recentColors));
  renderRecentColors();
}

function renderRecentColors() {
  recentChipsList.innerHTML = "";
  recentColors.forEach((hex) => {
    const chip = document.createElement("button");
    chip.className = "recent-color-chip";
    chip.style.backgroundColor = hex;
    chip.title = hex;
    chip.addEventListener("click", () => {
      updateActiveSwatchUI(hex);
      applyHexColor(hex);
      highlightActivePreset(hex);
    });
    recentChipsList.appendChild(chip);
  });
}

function highlightActivePreset(hex) {
  colorChips.forEach((chip) => {
    if (chip.getAttribute("data-hex")?.toUpperCase() === hex.toUpperCase()) {
      chip.classList.add("active");
    } else {
      chip.classList.remove("active");
    }
  });
}

async function applyHexColor(hex) {
  let brightness = parseInt(kbdBrightness.value, 10);
  if (brightness === 0) {
    brightness = lastActiveBrightness || 4;
    kbdBrightness.value = brightness.toString();
    brightnessVal.textContent = brightness.toString();
  }

  const { r, g, b } = hexToRgb(hex);
  try {
    await invoke("set_keyboard_rgb", { r, g, b, brightness });
    showToast(`RGB: (${r}, ${g}, ${b})`);
    addRecentColor(hex);
  } catch (e) {
    showToast(`Hata: ${e}`);
  }
}

// Preset Color Chips
colorChips.forEach((chip) => {
  chip.addEventListener("click", async () => {
    colorChips.forEach((c) => c.classList.remove("active"));
    chip.classList.add("active");

    const preset = chip.getAttribute("data-color");
    const hex = chip.getAttribute("data-hex");

    if (preset === "off") {
      kbdBrightness.value = "0";
      brightnessVal.textContent = "0";
      try {
        await invoke("set_keyboard_preset", { preset: "off", brightness: 0 });
        showToast("Kapatıldı");
      } catch (e) {
        showToast(`Hata: ${e}`);
      }
      return;
    }

    updateActiveSwatchUI(hex);
    let brightness = parseInt(kbdBrightness.value, 10);
    if (brightness === 0) {
      brightness = lastActiveBrightness || 4;
      kbdBrightness.value = brightness.toString();
      brightnessVal.textContent = brightness.toString();
    }

    try {
      await invoke("set_keyboard_preset", { preset, brightness });
      showToast(`Renk: ${preset.toUpperCase()}`);
      addRecentColor(hex);
    } catch (e) {
      showToast(`Hata: ${e}`);
    }
  });
});

// Tıklayınca son rengi klavyeye gönderen preview swatch
activeSwatch.addEventListener("click", () => {
  applyHexColor(currentColorHex);
  highlightActivePreset(currentColorHex);
});

// Palet Butonu -> Doğrudan tam renk paletini aç
btnOpenPalette.addEventListener("click", () => {
  const hsv = hexToHsv(currentColorHex);
  curHue = hsv.h;
  curSat = hsv.s;
  curVal = hsv.v;
  hueSlider.value = Math.round(curHue);
  drawSpectrum(curHue);
  setCursorPos(curSat, curVal);
  updateModalPreview();
  inlinePalettePanel.classList.remove("hidden");
});

btnClosePalette.addEventListener("click", () => {
  inlinePalettePanel.classList.add("hidden");
});

// Spectrum Canvas Drag / Click
function handleSpectrumCoords(e) {
  const rect = spectrumCanvas.getBoundingClientRect();
  const x = Math.max(0, Math.min(rect.width, e.clientX - rect.left));
  const y = Math.max(0, Math.min(rect.height, e.clientY - rect.top));
  curSat = x / rect.width;
  curVal = 1 - (y / rect.height);
  setCursorPos(curSat, curVal);
  updateModalPreview();
}

spectrumCanvas.addEventListener("mousedown", (e) => {
  isDraggingSpectrum = true;
  handleSpectrumCoords(e);
});

window.addEventListener("mousemove", (e) => {
  if (isDraggingSpectrum) {
    handleSpectrumCoords(e);
  }
});

window.addEventListener("mouseup", () => {
  isDraggingSpectrum = false;
});

// Hue Slider
hueSlider.addEventListener("input", (e) => {
  curHue = parseFloat(e.target.value);
  drawSpectrum(curHue);
  updateModalPreview();
});

btnPaletteSelect.addEventListener("click", () => {
  const hex = updateModalPreview();
  updateActiveSwatchUI(hex);
  applyHexColor(hex);
  highlightActivePreset(hex);
  inlinePalettePanel.classList.add("hidden");
});

// Brightness Slider
kbdBrightness.addEventListener("input", (e) => {
  const val = parseInt(e.target.value, 10);
  brightnessVal.textContent = val;
  if (val > 0) {
    lastActiveBrightness = val;
  }
});

kbdBrightness.addEventListener("change", async (e) => {
  const val = parseInt(e.target.value, 10);
  if (val === 0) {
    try {
      await invoke("set_keyboard_preset", { preset: "off", brightness: 0 });
      showToast("Kapatıldı");
    } catch (err) {}
  } else {
    applyHexColor(currentColorHex);
  }
});

async function refreshStatus() {
  try {
    const data = await invoke("get_status");
    if (!data) return;

    if (!data.has_permission) {
      if (permBanner) permBanner.classList.remove("hidden");
      return;
    }

    if (permBanner) permBanner.classList.add("hidden");
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

// Initial setup
updateActiveSwatchUI(currentColorHex);
renderRecentColors();
highlightActivePreset(currentColorHex);
refreshStatus();
