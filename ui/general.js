const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

const switchLanguage = document.getElementById("switch-language");
const langLabelTr = document.getElementById("lang-label-tr");
const langLabelEn = document.getElementById("lang-label-en");
const switchWinKey = document.getElementById("switch-winkey");
const switchTouchpad = document.getElementById("switch-touchpad");
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

function updateLangBadges(isEn) {
  if (langLabelTr) langLabelTr.classList.toggle("active", !isEn);
  if (langLabelEn) langLabelEn.classList.toggle("active", isEn);
}

if (switchLanguage) {
  const currentLang = typeof getLanguage === "function" ? getLanguage() : "tr";
  const isEn = currentLang === "en";
  switchLanguage.checked = isEn;
  updateLangBadges(isEn);

  switchLanguage.addEventListener("change", async (e) => {
    const isChecked = e.target.checked;
    const targetLang = isChecked ? "en" : "tr";
    updateLangBadges(isChecked);
    if (typeof setLanguage === "function") {
      await setLanguage(targetLang);
      showToast(t("toast_lang_changed"));
    }
  });
}

async function loadSettings() {
  try {
    const settings = await invoke("get_general_settings");
    if (!settings) return;
    if (switchWinKey) {
      switchWinKey.checked = !!settings.win_key_disabled;
    }
    if (switchTouchpad) {
      switchTouchpad.checked = !!settings.touchpad_disabled;
    }
  } catch (err) {
    console.error("General settings load error:", err);
  }
}

if (switchWinKey) {
  switchWinKey.addEventListener("change", async (e) => {
    const disabled = e.target.checked;
    try {
      await invoke("set_win_key_disabled", { disabled });
      showToast(disabled ? t("toast_winkey_disabled") : t("toast_winkey_enabled"));
    } catch (err) {
      showToast(`${t("error_prefix")} ${err}`);
      switchWinKey.checked = !disabled;
    }
  });
}

if (switchTouchpad) {
  switchTouchpad.addEventListener("change", async (e) => {
    const disabled = e.target.checked;
    try {
      await invoke("set_touchpad_disabled", { disabled });
      showToast(disabled ? t("toast_touchpad_disabled") : t("toast_touchpad_enabled"));
    } catch (err) {
      showToast(`${t("error_prefix")} ${err}`);
      switchTouchpad.checked = !disabled;
    }
  });
}

loadSettings();
setInterval(loadSettings, 500);
