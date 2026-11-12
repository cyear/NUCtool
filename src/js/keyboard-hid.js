const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);
const setText = (id, value) => {
  const node = $(id);
  if (!node) { console.error(`[KeyboardHID] 页面控件不存在: #${id}`); return; }
  node.textContent = value;
};
const setValue = (id, value) => {
  const node = $(id);
  if (!node) { console.error(`[KeyboardHID] 页面控件不存在: #${id}`); return; }
  node.value = value;
};
const setChecked = (id, value) => {
  const node = $(id);
  if (!node) { console.error(`[KeyboardHID] 页面控件不存在: #${id}`); return; }
  node.checked = value;
};
const listen = (id, event, handler) => {
  const node = $(id);
  if (!node) { console.error(`[KeyboardHID] 页面控件不存在: #${id}`); return; }
  node.addEventListener(event, handler);
};
const paletteRoot = $("kh-palette");
if (paletteRoot) {
  for (let i = 0; i < 7; i += 1) {
    const input = document.createElement("input");
    input.type = "color";
    input.value = ["#3478f6", "#17c995", "#ff8a3d", "#bb70ff", "#f0506e", "#49c6df", "#f4ce46"][i];
    input.setAttribute("aria-label", `灯效颜色 ${i + 1}`);
    input.title = `颜色 ${i + 1}`;
    paletteRoot.append(input);
  }
}

const modeNames = {
  mono: "单色", breath: "呼吸", wave: "波浪", reactive: "响应", rainbow: "彩虹",
  ripple: "涟漪", raindrop: "雨滴", marquee: "跑马灯", aurora: "极光", spark: "火花",
  music: "音乐", user: "逐键", gaming: "游戏",
};
const extensionColors = Array.from({ length: 126 }, () => [52, 120, 246]);
let selectedKey = 0;
let keyButtons = [];
let keymapNames = [];
const fallbackKeymap = Array(126).fill(null);
const mapKeys = (name, slots) => slots.forEach((slot) => { fallbackKeymap[slot] = name; });
mapKeys("ESC", [0]);
for (let i = 1; i <= 12; i += 1) mapKeys(`F${i}`, [i]);
mapKeys("INSERT", [13]); mapKeys("SCRLK", [14]); mapKeys("DEL", [15]);
mapKeys("TILDE", [21]);
["ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN", "EIGHT", "NINE", "ZERO"]
  .forEach((name, index) => mapKeys(name, [22 + index]));
mapKeys("MINUS", [32]); mapKeys("EQUALS", [33]); mapKeys("BACKSPACE", [35]); mapKeys("HOME", [36]);
mapKeys("TAB", [42]);
["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "BRACKET_OPEN", "BRACKET_CLOSE", "BACKSLASH"]
  .forEach((name, index) => mapKeys(name, [44 + index]));
mapKeys("PAGE_UP", [57]); mapKeys("CAPS", [63]);
["A", "S", "D", "F", "G", "H", "J", "K", "L", "SEMICOLON", "QUOTE"]
  .forEach((name, index) => mapKeys(name, [65 + index]));
mapKeys("ENTER", [76, 77]); mapKeys("PAGE_DOWN", [78]);
mapKeys("SHIFT_LEFT", [84, 85, 86]);
["Z", "X", "C", "V", "B", "N", "M", "COMMA", "PERIOD", "SLASH"]
  .forEach((name, index) => mapKeys(name, [87 + index]));
mapKeys("SHIFT_RIGHT", [97]); mapKeys("ARROW_UP", [98]); mapKeys("END", [99]);
mapKeys("CTRL_LEFT", [105]); mapKeys("FN", [107]); mapKeys("WINDOWS", [108]); mapKeys("ALT_LEFT", [109]);
mapKeys("SPACE", [112]); mapKeys("ALT_RIGHT", [115]); mapKeys("MENU", [116]); mapKeys("CTRL_RIGHT", [117]);
mapKeys("ARROW_LEFT", [118]); mapKeys("ARROW_DOWN", [119]); mapKeys("ARROW_RIGHT", [120]);
const keyGrid = document.querySelector(".kh-keyboard-art");
function renderKeyGrid(names) {
  if (!keyGrid) return;
  keymapNames = Array.isArray(names) ? names : [];
  keyGrid.replaceChildren();
  keyGrid.removeAttribute("aria-hidden");
  keyGrid.setAttribute("role", "group");
  keyGrid.classList.add("kh-real-keyboard");
  keyButtons = Array(126).fill(null);
  const namesInOrder = [...new Set(keymapNames.filter(Boolean))];
  const keyLabel = {
    ONE: "1", TWO: "2", THREE: "3", FOUR: "4", FIVE: "5",
    SIX: "6", SEVEN: "7", EIGHT: "8", NINE: "9", ZERO: "0",
    ESC: "Esc", INSERT: "Ins", SCRLK: "ScrLk", DEL: "Del", TILDE: "~",
    BACKSPACE: "⌫", HOME: "Home", PAGE_UP: "PgUp", PAGE_DOWN: "PgDn",
    SHIFT_LEFT: "Shift", SHIFT_RIGHT: "Shift", ARROW_UP: "↑", ARROW_DOWN: "↓",
    ARROW_LEFT: "←", ARROW_RIGHT: "→", CTRL_LEFT: "Ctrl", CTRL_RIGHT: "Ctrl",
    ALT_LEFT: "Alt", ALT_RIGHT: "Alt", WINDOWS: "⊞", SPACE: "Space", MENU: "☰",
    BRACKET_OPEN: "[", BRACKET_CLOSE: "]", SEMICOLON: ";", QUOTE: "'", MINUS: "−",
    EQUALS: "=", COMMA: ",", PERIOD: ".", SLASH: "/", BACKSLASH: "\\",
  };
  for (let rowIndex = 0; rowIndex < 6; rowIndex += 1) {
    const row = document.createElement("div");
    row.className = "kh-key-row";
    const groups = namesInOrder.map((name) => ({
      name,
      slots: keymapNames.flatMap((item, slot) => item === name ? [slot] : []),
    })).filter((item) => Math.floor(item.slots[0] / 21) === rowIndex)
      .sort((a, b) => a.slots[0] - b.slots[0]);
    let cursor = rowIndex * 21;
    for (const { name, slots } of groups) {
      const first = slots[0];
      // The Backspace cap spans the otherwise-unused matrix slot before its
      // mapped LED, so draw it across that gap while retaining the real slot.
      const visualStart = name === "BACKSPACE" ? first - 1 : first;
      // The bottom row has no visual gaps between modifier keys on the X15;
      // reserved HID matrix slots there are protocol positions, not empty caps.
      const gap = rowIndex === 5 ? 0 : visualStart - cursor;
      if (gap > 0) {
        const spacer = document.createElement("span");
        spacer.className = "kh-key-spacer";
        spacer.style.flexGrow = String(gap * 0.65);
        row.append(spacer);
      }
      const physicalWidth = ({ TAB: 2, CAPS: 2, BACKSPACE: 2, SPACE: 6 })[name] ?? slots.length;
      const key = document.createElement("button");
      key.type = "button";
      key.className = "kh-key-cell";
      key.textContent = keyLabel[name] ?? name;
      key.style.flexGrow = String(Math.max(slots.length, physicalWidth));
      key.setAttribute("aria-label", `${name}，${slots.length} 个灯位`);
      key.style.setProperty("--key-color", `rgb(${extensionColors[first].join(",")})`);
      key.addEventListener("click", () => {
        selectedKey = first;
        keyGrid.querySelectorAll(".selected").forEach((button) => button.classList.remove("selected"));
        key.classList.add("selected");
        setValue("kh-extended-color", `#${extensionColors[first].map((v) => v.toString(16).padStart(2, "0")).join("")}`);
      });
      for (const slot of slots) keyButtons[slot] = key;
      row.append(key);
      cursor = visualStart + Math.max(slots.length, physicalWidth);
    }
    keyGrid.append(row);
  }
  if (keyButtons[selectedKey]) keyButtons[selectedKey].classList.add("selected");
}

function updateSelectedKeyColor(hex) {
  const name = keymapNames[selectedKey];
  if (!name) return;
  const color = rgb(hex);
  keymapNames.forEach((keyName, index) => {
    if (keyName === name) {
      extensionColors[index] = [...color];
      keyButtons[index]?.style.setProperty("--key-color", `rgb(${color.join(",")})`);
    }
  });
}

if (keyGrid) {
  const fillAll = document.createElement("button");
  fillAll.type = "button";
  fillAll.className = "secondary kh-fill-keys";
  fillAll.dataset.i18n = "keyboardHid.fillKeys";
  fillAll.textContent = "填充全部键位";
  keyGrid.parentElement.after(fillAll);
  fillAll.addEventListener("click", () => {
    const color = rgb($("kh-extended-color").value);
    extensionColors.forEach((_, index) => {
      extensionColors[index] = [...color];
      keyButtons[index]?.style.setProperty("--key-color", `rgb(${color.join(",")})`);
    });
  });
  listen("kh-extended-color", "input", (event) => updateSelectedKeyColor(event.target.value));
  renderKeyGrid(fallbackKeymap);
}
let refreshing = false;
let lastInfo = null;
let lastNonZeroBrightness = 75;
let activeLighting = null;
let timeoutExpired = false;
let checkingIdle = false;

const timeoutSettings = {
  enabled: $("kh-timeout-enabled"),
  duration: $("kh-timeout-time"),
  restore: $("kh-timeout-restore"),
};
try {
  const saved = JSON.parse(localStorage.getItem("keyboard-hid-idle-settings") || "null");
  if (saved) {
    if (timeoutSettings.enabled) timeoutSettings.enabled.checked = Boolean(saved.enabled);
    if (timeoutSettings.duration && [...timeoutSettings.duration.options].some((o) => Number(o.value) === saved.duration)) {
      timeoutSettings.duration.value = String(saved.duration);
    }
    if (timeoutSettings.restore) timeoutSettings.restore.checked = Boolean(saved.restore);
    if (saved.activeLighting && ["builtin", "extended"].includes(saved.activeLighting.type)) {
      activeLighting = saved.activeLighting;
    }
  }
} catch (error) {
  console.warn("[KeyboardHID] 无法读取空闲熄灯设置:", error);
}

function saveTimeoutSettings() {
  try {
    localStorage.setItem("keyboard-hid-idle-settings", JSON.stringify({
      enabled: Boolean(timeoutSettings.enabled?.checked),
      duration: Number(timeoutSettings.duration?.value || 300),
      restore: Boolean(timeoutSettings.restore?.checked),
      activeLighting,
    }));
  } catch (error) {
    console.warn("[KeyboardHID] 无法保存空闲熄灯设置:", error);
  }
}

function builtinLightingSnapshot() {
  return {
    type: "builtin",
    effect: $("kh-effect").value,
    speed: Number($("kh-speed").value),
    brightness: Number($("kh-effect-brightness").value),
    direction: $("kh-direction").value,
    colors: [...paletteRoot.querySelectorAll("input")].map((input) => rgb(input.value)),
  };
}

async function restoreLighting() {
  const lighting = activeLighting ?? builtinLightingSnapshot();
  if (lighting.type === "extended") {
    await invoke("set_keyboard_hid_extended", {
      colors: lighting.colors.map((color) => [...color]),
      brightness: lighting.brightness,
      save: false,
    });
  } else {
    const { type: _type, ...settings } = lighting.type === "builtin" ? lighting : builtinLightingSnapshot();
    await invoke("set_keyboard_hid_effect", { ...settings, save: false });
  }
  console.info("[KeyboardHID] 活动恢复，已重写灯效配置");
}

async function checkIdleShutdown() {
  if (!timeoutSettings.enabled?.checked || checkingIdle) return;
  checkingIdle = true;
  try {
    const idleSeconds = await invoke("get_keyboard_hid_idle_seconds");
    const duration = Number(timeoutSettings.duration?.value || 300);
    if (!timeoutExpired && idleSeconds >= duration) {
      timeoutExpired = true;
      await invoke("set_keyboard_hid_brightness", { level: 0 });
      console.info(`[KeyboardHID] 本机空闲 ${idleSeconds}s，已关闭键盘灯`);
      if ($("page-keyboard-hid")?.classList.contains("active")) refresh();
    } else if (timeoutExpired && idleSeconds < duration) {
      timeoutExpired = false;
      if (timeoutSettings.restore?.checked) {
        try {
          await restoreLighting();
          if ($("page-keyboard-hid")?.classList.contains("active")) refresh();
        } catch (error) {
          console.error("[KeyboardHID] 活动恢复时重写灯效失败:", error);
          message(`恢复灯效失败：${error}`, true);
        }
      }
    }
  } catch (error) {
    console.error("[KeyboardHID] 监测本机空闲时间失败:", error);
  } finally {
    checkingIdle = false;
  }
}

for (const control of Object.values(timeoutSettings)) {
  control?.addEventListener("change", () => {
    timeoutExpired = false;
    saveTimeoutSettings();
  });
}

function rgb(hex) {
  return [1, 3, 5].map((offset) => parseInt(hex.slice(offset, offset + 2), 16));
}

function message(text, error = false) {
  const toast = $("kh-toast");
  if (!toast) { console.error(`[KeyboardHID] ${text}`); return; }
  toast.textContent = text;
  toast.classList.toggle("error", error);
  toast.classList.add("visible");
  clearTimeout(message.timer);
  message.timer = setTimeout(() => toast.classList.remove("visible"), 2600);
}

async function refresh() {
  if (refreshing) return;
  refreshing = true;
  let info;
  try {
    info = await invoke("get_keyboard_hid_info");
    if (!info) return;
    lastInfo = info;
  } catch (error) {
    const detail = String(error);
    const found = !detail.includes("HID_OPEN:");
    setText("kh-connection", found ? "HID 已打开，读取设备信息失败" : "未检测到兼容 HID 键盘");
    const connection = $("kh-connection");
    if (connection) connection.classList.remove("connected");
    setText("kh-error", detail);
    const errorNode = $("kh-error");
    if (errorNode) errorNode.hidden = false;
    setText("kh-firmware", "—");
    setText("kh-layout", "—");
    setText("kh-current-effect", "—");
    console.error("[KeyboardHID] 状态读取失败:", error);
    return;
  } finally {
    refreshing = false;
  }

  try {
    setText("kh-connection", "HID 键盘已连接");
    const connection = $("kh-connection");
    if (connection) connection.classList.add("connected");
    const errorNode = $("kh-error");
    if (errorNode) errorNode.hidden = true;
    setText("kh-firmware", info.firmware ?? "—");
    setText("kh-layout", info.layout ?? "US");
    setText("kh-current-effect", info.effect === "unknown"
      ? "回读不可用" : `${modeNames[info.effect] ?? info.effect} · ${info.effect_id}`);
    if (typeof info.power === "boolean") setChecked("kh-power", info.power);
    const currentBrightness = info.brightness_live ?? info.brightness_ac;
    if (typeof currentBrightness === "number") {
      setValue("kh-brightness", currentBrightness);
      setValue("kh-brightness-value", `${currentBrightness}%`);
      setValue("kh-effect-brightness", currentBrightness);
      setValue("kh-effect-brightness-value", `${currentBrightness}%`);
      if (currentBrightness > 0) lastNonZeroBrightness = currentBrightness;
    }
    if (modeNames[info.effect]) setValue("kh-effect", info.effect === "user" ? "mono" : info.effect);
    setValue("kh-speed", info.speed);
    setValue("kh-speed-value", info.speed);
    setValue("kh-direction", info.direction);
    if (Array.isArray(info.keymap) && info.keymap.some(Boolean)) renderKeyGrid(info.keymap);
    if (Array.isArray(info.palette) && info.palette.length && paletteRoot) {
      [...paletteRoot.querySelectorAll("input")].forEach((input, index) => {
        if (info.palette[index]) input.value = `#${info.palette[index].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
      });
    }
    const connectionNode = $("kh-connection");
    if (connectionNode) connectionNode.title = info.effect_live
      ? `0x88: ${info.effect_live.map((v) => v.toString(16).padStart(2, "0")).join(" ")}`
      : "实时效果回读由此固件支持情况决定";
  } catch (error) {
    console.error("[KeyboardHID] 页面状态更新失败:", error);
    setText("kh-error", `页面控件更新失败：${error}`);
    const errorNode = $("kh-error");
    if (errorNode) errorNode.hidden = false;
  }
}

async function run(command, args, success) {
  try {
    await invoke(command, args);
    message(success);
    await refresh();
    return true;
  } catch (error) {
    message(String(error), true);
    return false;
  }
}

listen("kh-refresh", "click", refresh);
listen("kh-power", "change", async (event) => {
  const enabled = event.target.checked;
  const brightness = Number($("kh-brightness").value) || lastNonZeroBrightness;
  if (enabled) {
    activeLighting = { ...builtinLightingSnapshot(), brightness };
    saveTimeoutSettings();
  }
  await run("set_keyboard_hid_power", {
    enabled,
    restoreLevel: brightness,
  }, "键盘灯开关已写入");
});
listen("kh-brightness", "input", (event) => {
  setValue("kh-brightness-value", `${event.target.value}%`);
  if (Number(event.target.value) > 0) lastNonZeroBrightness = Number(event.target.value);
});
listen("kh-brightness", "change", async (event) => {
  const level = Number(event.target.value);
  if (activeLighting?.type === "extended") activeLighting.brightness = level;
  else activeLighting = { ...builtinLightingSnapshot(), brightness: level };
  saveTimeoutSettings();
  await run("set_keyboard_hid_brightness", { level }, "亮度已写入");
});
listen("kh-speed", "input", (event) => setValue("kh-speed-value", event.target.value));
listen("kh-effect-brightness", "input", (event) => setValue("kh-effect-brightness-value", `${event.target.value}%`));
listen("kh-extended-brightness", "input", (event) => setValue("kh-extended-brightness-value", `${event.target.value}%`));

listen("kh-apply-effect", "click", async () => {
  const colors = [...paletteRoot.querySelectorAll("input")].map((input) => rgb(input.value));
  const settings = {
    effect: $("kh-effect").value,
    speed: Number($("kh-speed").value),
    brightness: Number($("kh-effect-brightness").value),
    direction: $("kh-direction").value,
    colors,
    save: $("kh-save").checked,
  };
  if (await run("set_keyboard_hid_effect", settings, "内置灯效已发送")) {
    activeLighting = { type: "builtin", ...settings };
    delete activeLighting.save;
    saveTimeoutSettings();
  }
});

listen("kh-apply-extended", "click", async () => {
  const settings = {
    colors: extensionColors,
    brightness: Number($("kh-extended-brightness").value),
  };
  if (await run("set_keyboard_hid_extended", {
    ...settings,
    save: $("kh-save").checked,
  }, "126 键扩展灯效帧已发送")) {
    activeLighting = { type: "extended", colors: settings.colors.map((color) => [...color]), brightness: settings.brightness };
    saveTimeoutSettings();
  }
});

refresh();
setInterval(() => {
  if ($("page-keyboard-hid")?.classList.contains("active")) refresh();
}, 4000);
setInterval(checkIdleShutdown, 1000);
