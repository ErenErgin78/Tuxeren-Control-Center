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
const presetButtons = document.querySelectorAll(".profile-btn");
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

// Preset definitions reverse-engineered from DisplayTable.json
const PRESETS = {
  default: {
    brightness: 100,
    color_temp: 6500,
    saturation: 1.0,
    gamma: 1.0,
    red: 128,
    green: 128,
    blue: 128,
  },
  vibrant: {
    brightness: 90,
    color_temp: 6500,
    saturation: 1.5,
    gamma: 1.0,
    red: 128,
    green: 128,
    blue: 128,
  },
  reading: {
    brightness: 50,
    color_temp: 4500,
    saturation: 0.9,
    gamma: 1.1,
    red: 128,
    green: 110,
    blue: 85,
  },
  cinema: {
    brightness: 75,
    color_temp: 6000,
    saturation: 1.15,
    gamma: 0.95,
    red: 107,
    green: 119,
    blue: 116,
  },
  photo: {
    brightness: 80,
    color_temp: 6500,
    saturation: 1.05,
    gamma: 1.0,
    red: 98,
    green: 129,
    blue: 114,
  },
};

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
    presetButtons.forEach(btn => btn.classList.remove("active"));
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

presetButtons.forEach(btn => {
  btn.addEventListener("click", async () => {
    const key = btn.dataset.preset;
    const target = PRESETS[key];
    if (!target) return;

    presetButtons.forEach(b => b.classList.remove("active"));
    btn.classList.add("active");

    updateUI(target);
    try {
      await invoke("apply_display_settings", { settings: target });
      showToast(`Profil uygulandı: ${btn.querySelector("span").textContent}`);
    } catch (err) {
      showToast(`Hata: ${err}`);
    }
  });
});

if (btnReset) {
  btnReset.addEventListener("click", async () => {
    try {
      const def = await invoke("reset_display_settings");
      updateUI(def);
      presetButtons.forEach(b => {
        b.classList.toggle("active", b.dataset.preset === "default");
      });
      showToast("Görüntü ayarları sıfırlandı");
    } catch (err) {
      showToast(`Hata: ${err}`);
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
