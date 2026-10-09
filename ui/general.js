const getInvoke = () => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  return async (cmd, args) => {
    console.warn("Tauri API not detected:", cmd, args);
    return null;
  };
};

const invoke = getInvoke();

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

async function loadSettings() {
  try {
    const settings = await invoke("get_general_settings");
    if (!settings) return;
    if (switchWinKey && switchWinKey !== document.activeElement) {
      switchWinKey.checked = !!settings.win_key_disabled;
    }
    if (switchTouchpad && switchTouchpad !== document.activeElement) {
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
      showToast(disabled ? "Windows tuşu devre dışı bırakıldı" : "Windows tuşu etkinleştirildi");
    } catch (err) {
      showToast(`Hata: ${err}`);
      switchWinKey.checked = !disabled;
    }
  });
}

if (switchTouchpad) {
  switchTouchpad.addEventListener("change", async (e) => {
    const disabled = e.target.checked;
    try {
      await invoke("set_touchpad_disabled", { disabled });
      showToast(disabled ? "Touchpad kilitlendi" : "Touchpad etkinleştirildi");
    } catch (err) {
      showToast(`Hata: ${err}`);
      switchTouchpad.checked = !disabled;
    }
  });
}

loadSettings();
setInterval(loadSettings, 500);
