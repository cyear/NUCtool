const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

import {
  initI18n,
  setLanguage,
  getLanguageSetting,
  t,
} from "./i18n.js";

import { initGscCheck } from "./gsc-check.js";

// ==============================================
// 禁止右键菜单
document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
});

// 禁止选中文字
document.addEventListener("selectstart", (e) => {
  e.preventDefault();
});

// 禁止拖拽
document.addEventListener("dragstart", (e) => {
  e.preventDefault();
});

// 禁止常见开发者工具快捷键
document.addEventListener("keydown", (e) => {
  // F12
  // if (e.key === "F12") {
  //   e.preventDefault();
  //   e.stopPropagation();
  //   return;
  // }

  // Ctrl + Shift + I
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "i") {
    e.preventDefault();
    e.stopPropagation();
    return;
  }

  // Ctrl + Shift + J
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "j") {
    e.preventDefault();
    e.stopPropagation();
    return;
  }

  // Ctrl + U
  if (e.ctrlKey && e.key.toLowerCase() === "u") {
    e.preventDefault();
    e.stopPropagation();
    return;
  }
});

// ==============================================
// 多语言状态
// ==============================================

let monitorStatusKey = "status.connecting";


// ==============================================
// 更新动态多语言文字
// ==============================================

function updateDynamicTranslations() {

  // --------------------------------------------
  // 监控状态
  // --------------------------------------------

  const statusEl = document.getElementById("status");

  if (statusEl && monitorStatusKey) {
    statusEl.textContent = t(monitorStatusKey);
  }


  // --------------------------------------------
  // 风扇控制
  // --------------------------------------------

  if (typeof updateControlState === "function") {
    updateControlState();
  }


  // --------------------------------------------
  // 开机自启动
  // --------------------------------------------

  if (autostartToggle) {
    updateAutostartUI(autostartToggle.checked);
  }


  // --------------------------------------------
  // 键盘 LED
  // --------------------------------------------

  if (keyboardLedToggle) {
    updateKeyboardLedUI(keyboardLedToggle.checked);
  }
}


// ==============================================
// 监听语言变化
// ==============================================

window.addEventListener(
  "nuctool-language-changed",
  () => {
    updateDynamicTranslations();
  }
);


// ========== 更新监控数据 ==========

function updateUI(data) {
  document.getElementById("cpu-temp").textContent =
    data.cpu_temp ?? "--";

  document.getElementById("gpu-temp").textContent =
    data.gpu_temp ?? "--";

  document.getElementById("fan1-rpm").textContent =
    data.fan1_rpm ?? "--";

  document.getElementById("fan2-rpm").textContent =
    data.fan2_rpm ?? "--";

  document.getElementById("system_power").textContent =
    data.system_power ?? "--";

  document.getElementById("bat_mah_percent").textContent =
    data.bat_mah_percent ?? "--";

  // 风扇控制页同步显示当前转速
  document.getElementById("fan1-current").textContent =
    data.fan1_rpm ?? "--";

  document.getElementById("fan2-current").textContent =
    data.fan2_rpm ?? "--";
}


// ========== 监听 Rust 推送 ==========

async function startListen() {

  const statusEl =
    document.getElementById("status");

  try {

    await invoke("start_sensor_loop");

    await listen(
      "sensor-update",
      (event) => {

        updateUI(event.payload);

        monitorStatusKey =
          "status.connected";

        statusEl.textContent =
          t(monitorStatusKey);

        statusEl.className =
          "status ok";
      }
    );


    monitorStatusKey =
      "status.connected";

    statusEl.textContent =
      t(monitorStatusKey);

    statusEl.className =
      "status ok";

  } catch (e) {

    console.error(e);

    monitorStatusKey =
      "status.disconnected";

    statusEl.textContent =
      t(monitorStatusKey);

    statusEl.className =
      "status err";
  }
}

startListen();



/* =========================================================
   页面切换
   ========================================================= */

document.querySelectorAll(".nav-btn").forEach((button) => {

  button.addEventListener("click", () => {

    const page =
      button.dataset.page;

    document.querySelectorAll(".nav-btn")
      .forEach((btn) =>
        btn.classList.remove("active")
      );

    document.querySelectorAll(".page")
      .forEach((el) =>
        el.classList.remove("active")
      );

    button.classList.add("active");

    document
      .getElementById(`page-${page}`)
      ?.classList.add("active");
  });

});



/* =========================================================
   Chart.js
   ========================================================= */

const COLOR_CPU = "#3987e5";
const COLOR_GPU = "#d95926";

const GRID =
  "rgba(255, 255, 255, 0.08)";

Chart.defaults.color =
  "#898781";

Chart.defaults.borderColor =
  GRID;

Chart.defaults.font.family =
  '"Segoe UI", "Microsoft YaHei", system-ui, sans-serif';

Chart.defaults.font.size = 11;

Chart.defaults.animation = false;


/* 温度节点：30°C ~ 100°C，每 5°C 一个节点 */

const CURVE_TEMPS =
  Array.from(
    { length: 15 },
    (_, i) => 30 + i * 5
  );



/* =========================================================
   创建风扇曲线
   ========================================================= */

function createCurveChart(id, color) {

  return new Chart(
    document.getElementById(id),
    {
      type: "line",

      data: {

        labels: CURVE_TEMPS,

        datasets: [
          {
            data:
              Array(
                CURVE_TEMPS.length
              ).fill(50),

            borderColor:
              color,

            borderWidth: 2,

            pointRadius: 4,

            pointHoverRadius: 6,

            pointBackgroundColor:
              color,

            fill: false,
          },
        ],
      },

      options: {

        maintainAspectRatio:
          false,

        cubicInterpolationMode:
          "monotone",

        plugins: {

          legend: {
            display: false,
          },

          tooltip: {

            displayColors:
              false,

            callbacks: {

              title: (items) =>
                `${items[0].label} °C`,

              label: (item) =>
                `${item.formattedValue} %`,
            },
          },

          // 允许拖动曲线节点
          dragData: {
            round: 0,
            dragX: false,
          },
        },

        scales: {

          x: {

            grid: {
              display: false,
            },

            ticks: {

              maxRotation: 0,

              callback: (
                value,
                index
              ) =>
                index % 2 === 0
                  ? `${CURVE_TEMPS[index]}°`
                  : "",
            },
          },

          y: {

            min: 0,

            max: 100,

            grid: {
              color: GRID,
            },

            border: {
              display: false,
            },

            ticks: {

              stepSize: 25,

              callback: (value) =>
                `${value}%`,
            },
          },
        },
      },
    }
  );
}


const leftFanCurve =
  createCurveChart(
    "leftFanCurve",
    COLOR_CPU
  );

const rightFanCurve =
  createCurveChart(
    "rightFanCurve",
    COLOR_GPU
  );


/* =========================================================
   曲线数据
   ========================================================= */

function getFanCurveData() {

  const getCurve = (chart) =>
    chart.data.labels.map(
      (temperature, index) => ({
        temperature,
        speed:
          chart.data.datasets[0]
            .data[index],
      })
    );

  return {
    left_fan:
      getCurve(leftFanCurve),

    right_fan:
      getCurve(rightFanCurve),
  };
}


/* 将后端配置应用到曲线 */

function applyCurve(chart, points) {

  if (!Array.isArray(points)) {
    return;
  }

  const map =
    new Map(
      points.map((point) => [
        Math.round(
          point.temperature
        ),
        point.speed,
      ])
    );

  chart.data.datasets[0].data =
    chart.data.labels.map(
      (temperature, index) =>
        map.get(temperature) ??
        chart.data.datasets[0]
          .data[index]
    );

  chart.update();
}


/* =========================================================
   配置
   ========================================================= */

async function loadConfig() {

  try {

    const data =
      await invoke(
        "load_fan_config"
      );

    applyCurve(
      leftFanCurve,
      data.left_fan
    );

    applyCurve(
      rightFanCurve,
      data.right_fan
    );

    return true;

  } catch (error) {

    console.error(
      "加载配置失败:",
      error
    );

    return false;
  }
}


async function saveConfig() {

  try {

    await invoke(
      "save_fan_config",
      {
        fanData:
          getFanCurveData(),
      }
    );

    saveConfigButton.textContent =
      t("common.saved");

    setTimeout(() => {

      saveConfigButton.textContent =
        t("fan.saveConfig");

    }, 1200);

  } catch (error) {

    console.error(
      "保存配置失败:",
      error
    );
  }
}



/* =========================================================
   风扇控制
   ========================================================= */

const startStopButton =
  document.getElementById(
    "startStopButton"
  );

const loadConfigButton =
  document.getElementById(
    "loadConfigButton"
  );

const saveConfigButton =
  document.getElementById(
    "saveConfigButton"
  );

const fanStatus =
  document.getElementById(
    "fan-status"
  );

let isRunning = false;



// ==============================================
// 监听 Rust 风扇控制状态
// ==============================================

async function initFanControlStatus() {

  try {

    // Rust 主动推送状态
    await listen(
      "fan-control-status",
      (event) => {

        isRunning =
          Boolean(event.payload);

        console.log(
          "风扇控制状态:",
          isRunning
            ? "运行中"
            : "已停止"
        );

        updateControlState();
      }
    );


    // 页面加载后主动查询一次
    const status =
      await invoke(
        "get_fan_control_status"
      );

    isRunning =
      Boolean(status);

    updateControlState();

  } catch (error) {

    console.error(
      "初始化风扇控制状态失败:",
      error
    );
  }
}



// ==============================================
// 更新 UI
// ==============================================

function updateControlState() {

  startStopButton.textContent =
    isRunning
      ? t("fan.stopControl")
      : t("fan.startControl");


  startStopButton.classList.toggle(
    "primary",
    !isRunning
  );

  startStopButton.classList.toggle(
    "danger",
    isRunning
  );


  fanStatus.textContent =
    isRunning
      ? t("status.running")
      : t("status.stopped");


  fanStatus.className =
    isRunning
      ? "status ok"
      : "status";
}



// ==============================================
// 启动风扇控制
// ==============================================

async function startControl() {

  try {

    await invoke(
      "start_fan_control",
      {
        fanData:
          getFanCurveData(),
      }
    );

  } catch (error) {

    console.error(
      "启动风扇控制失败:",
      error
    );
  }
}



// ==============================================
// 停止风扇控制
// ==============================================

async function stopControl() {

  try {

    await invoke(
      "stop_fan_control"
    );

  } catch (error) {

    console.error(
      "停止风扇控制失败:",
      error
    );
  }
}



// ==============================================
// 按钮
// ==============================================

startStopButton.addEventListener(
  "click",
  () => {

    if (isRunning) {

      stopControl();

    } else {

      startControl();

    }

  }
);


loadConfigButton.addEventListener(
  "click",
  loadConfig
);


saveConfigButton.addEventListener(
  "click",
  saveConfig
);



// =====================================================
// 性能调优
// =====================================================

const tuningButtons =
  document.querySelectorAll(
    ".tuning-btn"
  );


tuningButtons.forEach(
  (button) => {

    button.addEventListener(
      "click",
      async () => {
        const mode = button.dataset.mode;
        tuningButtons.forEach((btn) => { btn.classList.remove(); });
        button.classList.add("active");
        if (mode === "unspecified") {
          console.log("性能模式: 默认");
          return;
        }
        try {
          await invoke("set_performance_mode", { mode: mode });
          console.log("性能模式:", mode);
        } catch (error) {
          console.error("设置性能模式失败:", error);
        }
        loadPerformanceMode();
      }
    );

  }
);

// 获取性能模式
async function loadPerformanceMode() {
  try {
    let mode;

    do {
      mode = await invoke("get_performance_mode");

      if (mode === 0) {
        await new Promise((resolve) => {
          setTimeout(resolve, 3000);
        });
      }
    } while (mode === 0);

    const modeMap = {
      1: "performance",
      2: "balanced",
      3: "power-saving",
      5: "benchmark-on"
    };

    tuningButtons.forEach((btn) => {
      btn.classList.remove("active");
    });

    const modeName = modeMap[mode];

    const activeButton = document.querySelector(
      `.tuning-btn[data-mode="${modeName}"]`
    );

    if (activeButton) {
      activeButton.classList.add("active");
    }
  } catch (error) {
    console.error("获取性能模式失败:", error);
  }
}



// =====================================================
// 电源计划
// =====================================================

const powerPlanButtons =
  document.querySelectorAll(
    ".power-plan-btn"
  );


powerPlanButtons.forEach(
  (button) => {

    button.addEventListener(
      "click",
      async () => {

        const plan =
          button.dataset.powerPlan;

        // 更新选中状态
        powerPlanButtons.forEach(
          (btn) => {
            btn.classList.remove(
              "active"
            );
          }
        );

        button.classList.add(
          "active"
        );


        // 默认：不修改电源计划
        if (
          plan === "unspecified"
        ) {

          console.log(
            "电源计划: 默认"
          );

          return;
        }


        const value =
          Number(plan);


        if (
          !Number.isInteger(value)
        ) {

          console.error(
            "无效的电源计划:",
            plan
          );

          return;
        }


        try {

          await invoke(
            "set_power_plan",
            {
              mode: value
            }
          );

          console.log(
            "电源计划:",
            value
          );

        } catch (error) {

          console.error(
            "设置电源计划失败:",
            error
          );
        }

      }
    );

  }
);



// =====================================================
// 功耗设置
// =====================================================

const tdpRefreshButton =
  document.getElementById(
    "tdpRefreshButton"
  );

const tdpSetButtons =
  document.querySelectorAll(
    ".tdp-set-btn"
  );



// -----------------------------------------------------
// 读取功耗配置
// -----------------------------------------------------

async function loadTdp() {

  try {

    const tdp =
      await invoke(
        "get_tdp"
      );

    console.log(
      "TDP:",
      tdp
    );


    document.getElementById(
      "cpu-pl1"
    ).value =
      tdp.cpu_pl1;

    document.getElementById(
      "cpu-pl2"
    ).value =
      tdp.cpu_pl2;

    document.getElementById(
      "cpu-pl4"
    ).value =
      tdp.cpu_pl4;


    document.getElementById(
      "gpu-pl1"
    ).value =
      tdp.gpu_pl1;

    document.getElementById(
      "gpu-pl2"
    ).value =
      tdp.gpu_pl2;

    document.getElementById(
      "psys_pl1"
    ).value =
      tdp.psys_pl1;

  } catch (error) {

    console.error(
      "读取功耗配置失败:",
      error
    );
  }
}



// -----------------------------------------------------
// 写入功耗
// -----------------------------------------------------

async function setTdp(type) {

  const input =
    document.getElementById(
      type
    );

  if (!input) {
    return;
  }


  const value =
    Number(input.value);


  if (
    !Number.isFinite(value) ||
    value < 0
  ) {

    console.error(
      "无效的功耗值:",
      value
    );

    return;
  }


  try {

    await invoke(
      "set_tdp",
      {
        tdpType: type,
        value: value
      }
    );

    console.log(
      `设置 ${type}: ${value} W`
    );

  } catch (error) {

    console.error(
      `设置 ${type} 失败:`,
      error
    );
  }
}



// -----------------------------------------------------
// 读取按钮
// -----------------------------------------------------

if (tdpRefreshButton) {

  tdpRefreshButton.addEventListener(
    "click",
    () => {
      loadTdp();
    }
  );

}



// -----------------------------------------------------
// 写入按钮
// -----------------------------------------------------

tdpSetButtons.forEach(
  (button) => {

    button.addEventListener(
      "click",
      () => {

        const type =
          button.dataset.tdp;

        setTdp(type);

      }
    );

  }
);



// =====================================================
// 开机自启动
// =====================================================

const autostartToggle =
  document.getElementById(
    "autostart-toggle"
  );

const autostartStatus =
  document.getElementById(
    "autostart-status"
  );

const autostartDescription =
  document.getElementById(
    "autostart-description"
  );



/**
 * 更新开机自启动 UI
 */

function updateAutostartUI(
  enabled
) {

  if (
    !autostartToggle ||
    !autostartStatus
  ) {
    return;
  }


  autostartToggle.checked =
    enabled;


  if (enabled) {

    autostartStatus.textContent =
      t("settings.enabled");

    autostartStatus.classList.remove(
      "disabled",
      "error"
    );

    autostartStatus.classList.add(
      "enabled"
    );


    if (autostartDescription) {

      autostartDescription.textContent =
        t(
          "settings.startupEnabledDescription"
        );
    }

  } else {

    autostartStatus.textContent =
      t("settings.disabled");

    autostartStatus.classList.remove(
      "enabled",
      "error"
    );

    autostartStatus.classList.add(
      "disabled"
    );


    if (autostartDescription) {

      autostartDescription.textContent =
        t(
          "settings.startupDisabledDescription"
        );
    }
  }
}



/**
 * 读取当前开机自启动状态
 */

async function loadAutostartState() {

  if (
    !autostartToggle ||
    !autostartStatus
  ) {
    return;
  }


  try {

    autostartToggle.disabled =
      true;


    autostartStatus.textContent =
      t(
        "settings.autostartChecking"
      );


    const enabled =
      await invoke(
        "get_autostart"
      );


    updateAutostartUI(
      enabled
    );

  } catch (error) {

    console.error(
      "读取开机自启动状态失败:",
      error
    );


    autostartStatus.textContent =
      t(
        "settings.readFailed"
      );


    autostartStatus.classList.remove(
      "enabled",
      "disabled"
    );

    autostartStatus.classList.add(
      "error"
    );


    autostartToggle.checked =
      false;

  } finally {

    autostartToggle.disabled =
      false;
  }
}



/**
 * 设置开机自启动
 */

async function setAutostart(
  enabled
) {

  if (!autostartToggle) {
    return;
  }


  try {

    autostartToggle.disabled =
      true;


    await invoke(
      "set_autostart",
      {
        enabled:
          enabled
      }
    );


    updateAutostartUI(
      enabled
    );

  } catch (error) {

    console.error(
      "设置开机自启动失败:",
      error
    );


    // 操作失败，重新读取真实状态
    await loadAutostartState();

  } finally {

    autostartToggle.disabled =
      false;
  }
}



/**
 * 开机自启动开关
 */

if (autostartToggle) {

  autostartToggle.addEventListener(
    "change",
    async () => {

      const enabled =
        autostartToggle.checked;

      await setAutostart(
        enabled
      );

    }
  );

}



// =====================================================
// 显示设置
// =====================================================

const displayButtons =
  document.querySelectorAll(
    ".display-btn"
  );


displayButtons.forEach(
  (button) => {

    button.addEventListener(
      "click",
      async () => {

        const mode =
          button.dataset.display;


        // 先更新选中状态
        displayButtons.forEach(
          (btn) => {
            btn.classList.remove(
              "active"
            );
          }
        );


        button.classList.add(
          "active"
        );


        // 默认：不修改显示模式
        // if (mode === "5") {
        //   console.log("显示模式: 关闭");
        //   return;
        // }


        const value =
          Number(mode);


        if (
          !Number.isInteger(value)
        ) {

          console.error(
            "无效的显示模式:",
            mode
          );

          return;
        }


        try {

          await invoke(
            "set_display_mode",
            {
              mode: value
            }
          );


          console.log(
            "显示模式:",
            value
          );

        } catch (error) {

          console.error(
            "设置显示模式失败:",
            error
          );
        }

      }
    );

  }
);



// =====================================================
// 键盘设置
// =====================================================

const keyboardLedToggle =
  document.getElementById(
    "keyboard-led-toggle"
  );

const keyboardLedDescription =
  document.getElementById(
    "keyboard-led-description"
  );



// -----------------------------------------------------
// 更新键盘 LED UI
// -----------------------------------------------------

function updateKeyboardLedUI(
  enabled
) {

  if (!keyboardLedToggle) {
    return;
  }


  keyboardLedToggle.checked =
    enabled;


  if (keyboardLedDescription) {

    keyboardLedDescription.textContent =
      enabled
        ? t(
          "keyboard.ledEnabled"
        )
        : t(
          "keyboard.ledDisabled"
        );
  }
}



// -----------------------------------------------------
// 获取键盘 LED 状态
// -----------------------------------------------------

async function loadKeyboardLedState() {

  if (!keyboardLedToggle) {
    return;
  }


  try {

    keyboardLedToggle.disabled =
      true;


    if (keyboardLedDescription) {

      keyboardLedDescription.textContent =
        t(
          "keyboard.reading"
        );
    }


    const enabled =
      await invoke(
        "get_keyboard_led"
      );


    updateKeyboardLedUI(
      Boolean(enabled)
    );

  } catch (error) {

    console.error(
      "读取键盘 LED 状态失败:",
      error
    );


    if (keyboardLedDescription) {

      keyboardLedDescription.textContent =
        t(
          "keyboard.readFailed"
        );
    }

  } finally {

    keyboardLedToggle.disabled =
      false;
  }
}



// -----------------------------------------------------
// 修改键盘 LED 状态
// -----------------------------------------------------

async function setKeyboardLed(
  enabled
) {

  if (!keyboardLedToggle) {
    return;
  }


  try {

    keyboardLedToggle.disabled =
      true;


    await invoke(
      "set_keyboard_led",
      {
        enabled:
          enabled
      }
    );


    updateKeyboardLedUI(
      enabled
    );


    console.log(
      "键盘 LED:",
      enabled
        ? "开启"
        : "关闭"
    );

  } catch (error) {

    console.error(
      "设置键盘 LED 失败:",
      error
    );


    // 设置失败，恢复实际状态
    await loadKeyboardLedState();

  } finally {

    keyboardLedToggle.disabled =
      false;
  }
}



// -----------------------------------------------------
// 开关事件
// -----------------------------------------------------

if (keyboardLedToggle) {

  keyboardLedToggle.addEventListener(
    "change",
    async () => {

      const enabled =
        keyboardLedToggle.checked;

      await setKeyboardLed(
        enabled
      );

    }
  );

}



// =====================================================
// 语言设置
// =====================================================

const languageSelect =
  document.getElementById(
    "language-select"
  );


if (languageSelect) {

  languageSelect.value =
    getLanguageSetting();


  languageSelect.addEventListener(
    "change",
    () => {

      setLanguage(
        languageSelect.value
      );

    }
  );
}


const fanModeSelect = document.getElementById(
  "fan-mode-select"
);


async function loadFanMode() {

  const mode = await invoke(
    "get_fan_mode"
  );

  fanModeSelect.value = String(mode);
}


fanModeSelect.addEventListener(
  "change",
  async () => {

    await invoke(
      "set_fan_mode",
      {
        mode: Number(fanModeSelect.value)
      }
    );

  }
);


/* =========================================================
   灯条设置
   ========================================================= */

let lightbarProfile = null;


/* =========================================================
   获取当前灯条设置
   ========================================================= */

async function loadLightbarProfile() {

  try {

    const profile =
      await invoke(
        "get_lightbar_profile"
      );

    lightbarProfile =
      profile;

    console.log(
      "Lightbar Profile:",
      lightbarProfile
    );

    updateLightbarUI();

  } catch (error) {

    console.error(
      "读取灯条配置失败:",
      error
    );

  }

}


/* =========================================================
   更新整个灯条 UI
   ========================================================= */

function updateLightbarUI() {

  if (!lightbarProfile) {
    return;
  }


  /* =====================================================
     AC
     ===================================================== */

  updateLightbarSettingUI(
    "ac",
    lightbarProfile.ac
  );


  /* =====================================================
     DC
     ===================================================== */

  updateLightbarSettingUI(
    "dc",
    lightbarProfile.dc
  );


  /* =====================================================
     灯效
     ===================================================== */

  updateLightbarEffectUI(
    "ac",
    lightbarProfile.ac.effect
  );

  updateLightbarEffectUI(
    "dc",
    lightbarProfile.dc.effect
  );


  /* =====================================================
     呼吸
     ===================================================== */

  updateLightbarBreathingUI(
    lightbarProfile.breathing_enable
  );

}


/* =========================================================
   更新 AC / DC
   ========================================================= */

function updateLightbarSettingUI(
  power,
  setting
) {

  if (!setting) {
    return;
  }


  const blue =
    document.getElementById(
      `lightbar-${power}-blue`
    );

  const green =
    document.getElementById(
      `lightbar-${power}-green`
    );

  const red =
    document.getElementById(
      `lightbar-${power}-red`
    );


  /* =====================================================
     更新滑块
     ===================================================== */

  if (blue) {

    blue.value =
      setting.blue_brightness;

  }

  if (green) {

    green.value =
      setting.green_brightness;

  }

  if (red) {

    red.value =
      setting.red_brightness;

  }


  /* =====================================================
     更新数字
     ===================================================== */

  updateLightbarValue(
    power,
    "blue",
    setting.blue_brightness
  );

  updateLightbarValue(
    power,
    "green",
    setting.green_brightness
  );

  updateLightbarValue(
    power,
    "red",
    setting.red_brightness
  );


  /* =====================================================
     更新预览
     ===================================================== */

  updateLightbarPreview(
    power,
    setting
  );

}


/* =========================================================
   更新数值
   ========================================================= */

function updateLightbarValue(
  power,
  color,
  value
) {

  const element =
    document.getElementById(
      `lightbar-${power}-${color}-value`
    );

  if (!element) {
    return;
  }


  element.textContent =
    value;

}


/* =========================================================
   亮度百分比 → CSS RGB
   ========================================================= */

function lightbarBrightnessToRgb(
  value
) {

  const brightness =
    Number(value);


  if (!Number.isFinite(
    brightness
  )) {

    return 0;

  }


  return Math.round(
    Math.max(
      0,
      Math.min(
        100,
        brightness
      )
    ) * 2.55
  );

}


/* =========================================================
   更新灯条预览
   ========================================================= */

function updateLightbarPreview(
  power,
  setting
) {

  const preview =
    document.getElementById(
      `lightbar-${power}-preview`
    );

  if (!preview) {
    return;
  }


  const effect =
    Number(
      setting.effect
    );


  /* =====================================================
     彩虹模式
     ===================================================== */

  if (effect === 1) {

    preview.classList.add(
      "rainbow"
    );


    /*
     * 清除单色模式的行内样式，
     * 让 CSS .rainbow 接管显示
     */

    preview.style.background =
      "";

    preview.style.backgroundColor =
      "";

    preview.style.boxShadow =
      "";

    return;
  }


  /* =====================================================
     单色模式
     ===================================================== */

  preview.classList.remove(
    "rainbow"
  );


  /*
   * 硬件亮度是 0~100，
   * CSS RGB 是 0~255
   */

  const red =
    lightbarBrightnessToRgb(
      setting.red_brightness
    );

  const green =
    lightbarBrightnessToRgb(
      setting.green_brightness
    );

  const blue =
    lightbarBrightnessToRgb(
      setting.blue_brightness
    );


  /* =====================================================
     背景颜色
     ===================================================== */

  preview.style.background =
    `rgb(${red}, ${green}, ${blue})`;


  preview.style.backgroundColor =
    "";


  /* =====================================================
     发光效果
     ===================================================== */

  preview.style.boxShadow =
    `
      0 0 18px rgba(
        ${red},
        ${green},
        ${blue},
        0.50
      ),
      0 0 36px rgba(
        ${red},
        ${green},
        ${blue},
        0.22
      )
    `;

}


/* =========================================================
   更新灯效按钮
   ========================================================= */

function updateLightbarEffectUI(
  power,
  effect
) {

  document
    .querySelectorAll(
      `.lightbar-effect-btn[data-power="${power}"]`
    )
    .forEach(
      (button) => {

        const buttonEffect =
          Number(
            button.dataset.lightbarEffect
          );


        button.classList.toggle(
          "active",
          buttonEffect ===
          Number(effect)
        );

      }
    );

}


/* =========================================================
   更新呼吸效果
   ========================================================= */

function updateLightbarBreathingUI(
  enabled
) {

  const checkbox =
    document.getElementById(
      "lightbar-breathing"
    );

  if (!checkbox) {
    return;
  }


  checkbox.checked =
    Number(enabled) !== 0;

}


/* =========================================================
   修改呼吸效果
   ========================================================= */

function bindLightbarBreathing() {

  const checkbox =
    document.getElementById(
      "lightbar-breathing"
    );

  if (!checkbox) {
    return;
  }


  checkbox.addEventListener(
    "change",
    async () => {

      if (!lightbarProfile) {
        return;
      }


      lightbarProfile.breathing_enable =
        checkbox.checked
          ? 1
          : 0;


      await saveLightbarProfile();

    }
  );

}


/* =========================================================
   写入完整 Lightbar Profile
   ========================================================= */

async function saveLightbarProfile() {

  if (!lightbarProfile) {
    return;
  }


  try {

    await invoke(
      "set_lightbar_profile",
      {
        profile:
          lightbarProfile
      }
    );


    console.log(
      "Lightbar Profile 已写入:",
      lightbarProfile
    );

  } catch (error) {

    console.error(
      "写入灯条配置失败:",
      error
    );

  }

}


/* =========================================================
   修改 RGB
   ========================================================= */

function bindLightbarSlider(
  power,
  color
) {

  const slider =
    document.getElementById(
      `lightbar-${power}-${color}`
    );

  if (!slider) {
    return;
  }


  /* =====================================================
     拖动时
     ===================================================== */

  slider.addEventListener(
    "input",
    (event) => {

      if (!lightbarProfile) {
        return;
      }


      const value =
        Number(
          event.target.value
        );


      /*
       * 修改 Profile
       */

      lightbarProfile[
        power
      ][
        `${color}_brightness`
      ] = value;


      /*
       * 更新数字
       */

      updateLightbarValue(
        power,
        color,
        value
      );


      /*
       * 更新预览
       */

      updateLightbarPreview(
        power,
        lightbarProfile[power]
      );

    }
  );


  /* =====================================================
     松开滑块后写入
     ===================================================== */

  slider.addEventListener(
    "change",
    async () => {

      if (!lightbarProfile) {
        return;
      }


      await saveLightbarProfile();

    }
  );

}


/* =========================================================
   修改灯效
   ========================================================= */

function bindLightbarEffect(
  power
) {

  document
    .querySelectorAll(
      `.lightbar-effect-btn[data-power="${power}"]`
    )
    .forEach(
      (button) => {

        button.addEventListener(
          "click",
          async () => {

            if (!lightbarProfile) {
              return;
            }


            const effect =
              Number(
                button.dataset.lightbarEffect
              );


            /* =================================================
               修改 Profile
               ================================================= */

            lightbarProfile[
              power
            ].effect =
              effect;


            /* =================================================
               更新按钮
               ================================================= */

            updateLightbarEffectUI(
              power,
              effect
            );


            /* =================================================
               更新预览
               ================================================= */

            updateLightbarPreview(
              power,
              lightbarProfile[power]
            );


            /* =================================================
               写入完整 Profile
               ================================================= */

            await saveLightbarProfile();

          }
        );

      }
    );

}


function bindLightbarQuickOff(power) {
  const button = document.getElementById(`lightbar-${power}-quick-off`);
  if (!button) return;

  button.addEventListener("click", async () => {
    if (!lightbarProfile || !lightbarProfile[power]) return;

    const setting = lightbarProfile[power];

    // 快速关闭灯条
    setting.red_brightness = 0;
    setting.green_brightness = 0;
    setting.blue_brightness = 0;
    setting.effect = 0;

    // 更新界面
    updateLightbarSettingUI(power, setting);
    updateLightbarEffectUI(power, 0);

    // 写入硬件
    await saveLightbarProfile();
  });
}


/* =========================================================
   初始化灯条
   ========================================================= */

async function initLightbar() {
  bindLightbarSlider("ac", "blue");
  bindLightbarSlider("ac", "green");
  bindLightbarSlider("ac", "red");

  bindLightbarSlider("dc", "blue");
  bindLightbarSlider("dc", "green");
  bindLightbarSlider("dc", "red");

  bindLightbarEffect("ac");
  bindLightbarEffect("dc");

  bindLightbarBreathing();

  bindLightbarQuickOff("ac");
  bindLightbarQuickOff("dc");

  await loadLightbarProfile();
}


const model = await invoke("get_sys_model");
if (model === "LAPAC71H" || model == "LAPAC71G") {
  const lightbarNav = document.querySelector(
    '.nav-btn[data-page="lightbar"]'
  );

  if (lightbarNav) {
    lightbarNav.remove();
  }
}


// =====================================================
// 电池健康优化器
// =====================================================

const batteryHealthModeSelect =
  document.getElementById(
    "battery-health-mode"
  );

const batteryCustomLimit =
  document.getElementById(
    "battery-custom-limit"
  );

const batteryChargingLevelInput =
  document.getElementById(
    "battery_charglimit"
  );

const batteryChargingLimitSetButton =
  document.getElementById(
    "battery-charging-limit-set"
  );


// -----------------------------------------------------
// 更新电池健康 UI
// -----------------------------------------------------

function updateBatteryHealthUI(mode) {

  if (!batteryHealthModeSelect) {
    return;
  }

  const batteryMode = Number(mode);

  batteryHealthModeSelect.value =
    String(batteryMode);


  // 只有“定制”模式显示充电限制
  if (batteryCustomLimit) {

    if (batteryMode === 1) {
      batteryCustomLimit.style.display = "";
    } else {
      batteryCustomLimit.style.display = "none";
    }

  }

}


// -----------------------------------------------------
// 读取充电限制
// -----------------------------------------------------

async function loadBatteryChargingLevel() {

  if (!batteryChargingLevelInput) {
    return;
  }

  try {

    const level =
      await invoke(
        "get_battery_charging_level"
      );


    console.log(
      "电池充电限制:",
      level
    );


    batteryChargingLevelInput.value =
      Number(level);

  } catch (error) {

    console.error(
      "读取电池充电限制失败:",
      error
    );

  }

}


// -----------------------------------------------------
// 设置充电限制
// -----------------------------------------------------

async function setBatteryChargingLevel() {

  if (!batteryChargingLevelInput) {
    return;
  }


  const level =
    Number(
      batteryChargingLevelInput.value
    );


  if (
    !Number.isInteger(level) ||
    level < 0 ||
    level > 100
  ) {

    console.error(
      "无效的电池充电限制:",
      level
    );

    return;
  }


  try {

    await invoke(
      "set_battery_charging_level",
      {
        level: level
      }
    );


    console.log(
      "电池充电限制设置为:",
      level
    );

  } catch (error) {

    console.error(
      "设置电池充电限制失败:",
      error
    );


    // 设置失败，重新读取实际值
    await loadBatteryChargingLevel();

  }

}


// -----------------------------------------------------
// 读取电池健康模式
// -----------------------------------------------------

async function loadBatteryHealthMode() {

  if (!batteryHealthModeSelect) {
    return;
  }

  try {

    const mode =
      Number(
        await invoke("get_battery_mode")
      );

    console.log(
      "电池健康模式:",
      mode
    );

    updateBatteryHealthUI(mode);

    // 只有定制模式读取充电限制
    if (mode === 1) {
      await loadBatteryChargingLevel();
    }

  } catch (error) {

    console.error(
      "读取电池健康模式失败:",
      error
    );

  }

}


// -----------------------------------------------------
// 设置电池健康模式
// -----------------------------------------------------

async function setBatteryHealthMode(mode) {

  if (!batteryHealthModeSelect) {
    return;
  }

  const batteryMode = Number(mode);

  try {

    await invoke("set_battery_mode", {
      mode: batteryMode
    });

    console.log(
      "电池健康模式:",
      batteryMode
    );

    updateBatteryHealthUI(batteryMode);

    // 只有切换到定制模式才读取充电限制
    if (batteryMode === 1) {
      await loadBatteryChargingLevel();
    }

  } catch (error) {

    console.error(
      "设置电池健康模式失败:",
      error
    );

    // 设置失败，恢复真实状态
    await loadBatteryHealthMode();

  }

}


// -----------------------------------------------------
// 电池健康模式下拉框
// -----------------------------------------------------

if (batteryHealthModeSelect) {

  batteryHealthModeSelect.addEventListener(
    "change",
    async () => {

      const mode =
        Number(
          batteryHealthModeSelect.value
        );


      if (
        !Number.isInteger(mode) ||
        mode < 0 ||
        mode > 2
      ) {

        console.error(
          "无效的电池健康模式:",
          mode
        );

        return;
      }


      await setBatteryHealthMode(
        mode
      );

    }
  );

}


// -----------------------------------------------------
// 充电限制写入按钮
// -----------------------------------------------------

if (batteryChargingLimitSetButton) {

  batteryChargingLimitSetButton.addEventListener(
    "click",
    async () => {

      await setBatteryChargingLevel();

    }
  );

}


/* =========================================================
   初始化
   ========================================================= */

async function init() {
  // 初始化语言
  initI18n();
  await loadConfig();
  // 初始化风扇控制状态
  await initFanControlStatus();
  updateControlState();
  loadPerformanceMode();
  loadKeyboardLedState();
  await loadBatteryHealthMode();
  loadFanMode();
  loadAutostartState();
  await initGscCheck();
  if (model != "LAPAC71H" && model != "LAPAC71G") {
    await initLightbar();
  }
}

init();