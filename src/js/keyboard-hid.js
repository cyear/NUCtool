const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);
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
const keyGrid = document.querySelector(".kh-keyboard-art");
if (keyGrid) {
  keyGrid.replaceChildren();
  const keyButtons = [];
  for (let index = 0; index < extensionColors.length; index += 1) {
    const key = document.createElement("button");
    key.type = "button";
    key.className = "kh-key-cell";
    key.setAttribute("aria-label", `键位 ${index + 1}`);
    key.style.setProperty("--key-color", `rgb(${extensionColors[index].join(",")})`);
    key.addEventListener("click", () => {
      selectedKey = index;
      keyButtons.forEach((button) => button.classList.remove("selected"));
      key.classList.add("selected");
      $("kh-extended-color").value = `#${extensionColors[index].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
    });
    keyGrid.append(key);
    keyButtons.push(key);
  }
  keyButtons[0].classList.add("selected");
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
      keyButtons[index].style.setProperty("--key-color", `rgb(${color.join(",")})`);
    });
  });
  $("kh-extended-color").addEventListener("input", (event) => {
    extensionColors[selectedKey] = rgb(event.target.value);
    keyButtons[selectedKey].style.setProperty("--key-color", `rgb(${extensionColors[selectedKey].join(",")})`);
  });
}
let refreshing = false;
let lastInfo = null;
let lastNonZeroBrightness = 75;

function rgb(hex) {
  return [1, 3, 5].map((offset) => parseInt(hex.slice(offset, offset + 2), 16));
}

function message(text, error = false) {
  const toast = $("kh-toast");
  toast.textContent = text;
  toast.classList.toggle("error", error);
  toast.classList.add("visible");
  clearTimeout(message.timer);
  message.timer = setTimeout(() => toast.classList.remove("visible"), 2600);
}

async function refresh() {
  if (refreshing) return;
  refreshing = true;
  try {
    const info = await invoke("get_keyboard_hid_info");
    lastInfo = info;
    $("kh-connection").textContent = "HID 键盘已连接";
    $("kh-connection").classList.add("connected");
    $("kh-error").hidden = true;
    $("kh-firmware").textContent = info.firmware;
    $("kh-layout").textContent = info.layout;
    $("kh-current-effect").textContent = info.effect === "unknown"
      ? "回读不可用" : `${modeNames[info.effect] ?? info.effect} · ${info.effect_id}`;
    if (typeof info.power === "boolean") $("kh-power").checked = info.power;
    const currentBrightness = info.brightness_live ?? info.brightness_ac;
    if (typeof currentBrightness === "number") {
      $("kh-brightness").value = currentBrightness;
      $("kh-brightness-value").value = `${currentBrightness}%`;
      $("kh-effect-brightness").value = currentBrightness;
      $("kh-effect-brightness-value").value = `${currentBrightness}%`;
      if (currentBrightness > 0) lastNonZeroBrightness = currentBrightness;
    }
    if (modeNames[info.effect]) $("kh-effect").value = info.effect === "user" ? "mono" : info.effect;
    $("kh-speed").value = info.speed;
    $("kh-speed-value").value = info.speed;
    $("kh-direction").value = info.direction;
    if (Array.isArray(info.palette) && info.palette.length) {
      [...paletteRoot.querySelectorAll("input")].forEach((input, index) => {
        if (info.palette[index]) input.value = `#${info.palette[index].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
      });
    }
    $("kh-connection").title = info.effect_live
      ? `0x88: ${info.effect_live.map((v) => v.toString(16).padStart(2, "0")).join(" ")}`
      : "实时效果回读由此固件支持情况决定";
  } catch (error) {
    $("kh-connection").textContent = "未检测到兼容 HID 键盘";
    $("kh-connection").classList.remove("connected");
    $("kh-error").textContent = String(error);
    $("kh-error").hidden = false;
    $("kh-firmware").textContent = "—";
    $("kh-layout").textContent = "—";
    $("kh-current-effect").textContent = "—";
  } finally {
    refreshing = false;
  }
}

async function run(command, args, success) {
  try {
    await invoke(command, args);
    message(success);
    await refresh();
  } catch (error) {
    message(String(error), true);
  }
}

$("kh-refresh").addEventListener("click", refresh);
$("kh-power").addEventListener("change", (event) =>
  run("set_keyboard_hid_power", {
    enabled: event.target.checked,
    restoreLevel: Number($("kh-brightness").value) || lastNonZeroBrightness,
  }, "键盘灯开关已写入"));
$("kh-brightness").addEventListener("input", (event) => {
  $("kh-brightness-value").value = `${event.target.value}%`;
  if (Number(event.target.value) > 0) lastNonZeroBrightness = Number(event.target.value);
});
$("kh-brightness").addEventListener("change", (event) =>
  run("set_keyboard_hid_brightness", { level: Number(event.target.value) }, "亮度已写入"));
$("kh-speed").addEventListener("input", (event) => { $("kh-speed-value").value = event.target.value; });
$("kh-effect-brightness").addEventListener("input", (event) => { $("kh-effect-brightness-value").value = `${event.target.value}%`; });
$("kh-extended-brightness").addEventListener("input", (event) => { $("kh-extended-brightness-value").value = `${event.target.value}%`; });

$("kh-apply-effect").addEventListener("click", () => {
  const colors = [...paletteRoot.querySelectorAll("input")].map((input) => rgb(input.value));
  run("set_keyboard_hid_effect", {
    effect: $("kh-effect").value,
    speed: Number($("kh-speed").value),
    brightness: Number($("kh-effect-brightness").value),
    direction: $("kh-direction").value,
    colors,
    save: $("kh-save").checked,
  }, "内置灯效已发送");
});

$("kh-apply-extended").addEventListener("click", () =>
  run("set_keyboard_hid_extended", {
    colors: extensionColors,
    brightness: Number($("kh-extended-brightness").value),
    save: $("kh-save").checked,
  }, "126 键扩展灯效帧已发送"));

$("kh-timeout-apply").addEventListener("click", () => {
  const rawTime = Number($("kh-timeout-time").value);
  if (!Number.isInteger(rawTime) || rawTime < 0 || rawTime > 255) {
    message("固件时间值必须为 0–255", true);
    return;
  }
  run("set_keyboard_hid_timeout", {
    enabled: $("kh-timeout-enabled").checked,
    rawTime,
    save: $("kh-save").checked,
  }, "定时熄灭设置已写入");
});

refresh();
setInterval(() => {
  if ($("page-keyboard-hid")?.classList.contains("active")) refresh();
}, 4000);
