const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const sliderBrightness = document.getElementById("slider-brightness");
const sliderTemp = document.getElementById("slider-temp");
const sliderSaturation = document.getElementById("slider-saturation");
const sliderGamma = document.getElementById("slider-gamma");
const sliderRed = document.getElementById("slider-red");
const sliderGreen = document.getElementById("slider-green");
const sliderBlue = document.getElementById("slider-blue");

const valBrightness = document.getElementById("val-brightness");
const valTemp = document.getElementById("val-temp");
const valSaturation = document.getElementById("val-saturation");
const valGamma = document.getElementById("val-gamma");
const valRed = document.getElementById("val-red");
const valGreen = document.getElementById("val-green");
const valBlue = document.getElementById("val-blue");

const btnReset = document.getElementById("btn-reset");
const toast = document.getElementById("toast");

function showToast(msg) {
  if (!toast) return;
  toast.textContent = msg;
  toast.classList.remove("hidden");
  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.add("hidden");
  }, 2000);
}

function updateUI(settings) {
  if (!settings) return;

  if (sliderBrightness) {
    sliderBrightness.value = settings.brightness;
    valBrightness.textContent = `${settings.brightness}%`;
  }
  if (sliderTemp) {
    sliderTemp.value = settings.color_temp;
    valTemp.textContent = `${settings.color_temp} K`;
  }
  if (sliderSaturation) {
    sliderSaturation.value = settings.saturation;
    valSaturation.textContent = parseFloat(settings.saturation).toFixed(2);
  }
  if (sliderGamma) {
    sliderGamma.value = settings.gamma;
    valGamma.textContent = parseFloat(settings.gamma).toFixed(2);
  }
  if (sliderRed) {
    sliderRed.value = settings.red;
    valRed.textContent = settings.red;
  }
  if (sliderGreen) {
    sliderGreen.value = settings.green;
    valGreen.textContent = settings.green;
  }
  if (sliderBlue) {
    sliderBlue.value = settings.blue;
    valBlue.textContent = settings.blue;
  }
}

function getCurrentState() {
  return {
    brightness: parseInt(sliderBrightness.value, 10),
    color_temp: parseInt(sliderTemp.value, 10),
    saturation: parseFloat(sliderSaturation.value),
    gamma: parseFloat(sliderGamma.value),
    red: Math.max(15, parseInt(sliderRed.value, 10)),
    green: Math.max(15, parseInt(sliderGreen.value, 10)),
    blue: Math.max(15, parseInt(sliderBlue.value, 10)),
  };
}

let applyTimeout = null;
function scheduleApply() {
  clearTimeout(applyTimeout);
  applyTimeout = setTimeout(async () => {
    try {
      const state = getCurrentState();
      await invoke("apply_display_settings", { settings: state });
    } catch (err) {
      console.error("Display apply error:", err);
    }
  }, 40);
}

function attachSliderListener(slider, labelElem, suffix, fmt) {
  if (!slider) return;
  slider.addEventListener("input", (e) => {
    const val = fmt ? fmt(e.target.value) : e.target.value;
    labelElem.textContent = `${val}${suffix || ""}`;
    scheduleApply();
  });
}

attachSliderListener(sliderBrightness, valBrightness, "%");
attachSliderListener(sliderTemp, valTemp, " K");
attachSliderListener(sliderSaturation, valSaturation, "", v => parseFloat(v).toFixed(2));
attachSliderListener(sliderGamma, valGamma, "", v => parseFloat(v).toFixed(2));
attachSliderListener(sliderRed, valRed);
attachSliderListener(sliderGreen, valGreen);
attachSliderListener(sliderBlue, valBlue);

if (btnReset) {
  btnReset.addEventListener("click", async () => {
    try {
      const def = await invoke("reset_display_settings");
      updateUI(def);
      showToast(typeof t === "function" ? t("toast_display_reset") : "Görüntü ayarları sıfırlandı");
    } catch (err) {
      showToast(`${typeof t === "function" ? t("error_prefix") : "Hata:"} ${err}`);
    }
  });
}

async function init() {
  try {
    const initial = await invoke("get_display_settings");
    if (initial) {
      updateUI(initial);
    }
  } catch (err) {
    console.error("Display settings init error:", err);
  }
}

init();
