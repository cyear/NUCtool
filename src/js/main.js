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

  const keyboardLedToggle = document.getElementById("keyboard-led-toggle");

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

/*
 * AC / DC 分开保存。
 *
 * AC:
 *   State: 0x7EA
 *   R:     0x769
 *   G:     0x76A
 *   B:     0x76B
 *
 * DC:
 *   State: 0x7EB
 *   R:     0x7EC
 *   G:     0x7ED
 *   B:     0x7EE
 *
 * Rust:
 *   get_keyboard_led(ac)
 *   set_keyboard_enabled(enable, ac)
 *   set_keyboard_brightness(brightness, ac)
 *   set_keyboard_rainbow(rainbow, ac)
 *   set_keyboard_red(red, ac)
 *   set_keyboard_green(green, ac)
 *   set_keyboard_blue(blue, ac)
 */


let keyboardPowerMode = "ac";


/*
 * 初始化的默认值。
 *
 * 页面启动以后会立即从 EC 读取 AC / DC，
 * 所以这里的值只是防止 UI 在读取完成之前出现 undefined。
 */

const keyboardProfiles = {

  ac: {
    loaded: false,
    enabled: false,
    brightness: 0,
    rainbow: false,
    red: 0,
    green: 0,
    blue: 0,
  },

  dc: {
    loaded: false,
    enabled: false,
    brightness: 0,
    rainbow: false,
    red: 0,
    green: 0,
    blue: 0,
  },

};


/* =========================================================
   工具：获取当前 Profile
   ========================================================= */

function getKeyboardProfile() {

  return keyboardProfiles[
    keyboardPowerMode
  ];

}


/* =========================================================
   EC RGB 0..50
   ->
   CSS RGB 0..255
   ========================================================= */

function keyboardEcToCss(value) {

  const number =
    Number(value) || 0;

  const clamped =
    Math.max(
      0,
      Math.min(
        50,
        number
      )
    );

  return Math.round(
    clamped * 255 / 50
  );

}


/* =========================================================
   CSS RGB 0..255
   ->
   EC RGB 0..50
   ========================================================= */

function keyboardCssToEc(value) {

  const number =
    Number(value) || 0;

  const clamped =
    Math.max(
      0,
      Math.min(
        255,
        number
      )
    );

  return Math.round(
    clamped * 50 / 255
  );

}


/* =========================================================
   RGB -> HEX
   ========================================================= */

function keyboardRgbToHex(
  red,
  green,
  blue
) {

  const toHex = (value) => {

    return Number(value)
      .toString(16)
      .padStart(2, "0");

  };


  return (
    "#"
    + toHex(red)
    + toHex(green)
    + toHex(blue)
  );

}


/* =========================================================
   HEX -> RGB
   ========================================================= */

function keyboardHexToRgb(hex) {

  const value =
    String(hex)
      .replace("#", "")
      .trim();


  if (value.length !== 6) {

    return {
      r: 255,
      g: 255,
      b: 255,
    };

  }


  return {

    r: parseInt(
      value.substring(0, 2),
      16
    ),

    g: parseInt(
      value.substring(2, 4),
      16
    ),

    b: parseInt(
      value.substring(4, 6),
      16
    ),

  };

}


/* =========================================================
   更新 RGB 预览
   ========================================================= */

function updateKeyboardColorPreview() {

  const redInput =
    document.getElementById(
      "keyboard-red"
    );

  const greenInput =
    document.getElementById(
      "keyboard-green"
    );

  const blueInput =
    document.getElementById(
      "keyboard-blue"
    );

  const redValue =
    document.getElementById(
      "keyboard-red-value"
    );

  const greenValue =
    document.getElementById(
      "keyboard-green-value"
    );

  const blueValue =
    document.getElementById(
      "keyboard-blue-value"
    );

  const preview =
    document.getElementById(
      "keyboard-color-preview"
    );

  const picker =
    document.getElementById(
      "keyboard-color-picker"
    );


  if (
    !redInput ||
    !greenInput ||
    !blueInput
  ) {

    return;

  }


  const red =
    Number(redInput.value);

  const green =
    Number(greenInput.value);

  const blue =
    Number(blueInput.value);


  /*
   * 更新数字
   */

  if (redValue) {
    redValue.textContent =
      red;
  }

  if (greenValue) {
    greenValue.textContent =
      green;
  }

  if (blueValue) {
    blueValue.textContent =
      blue;
  }


  /*
   * EC 0..50
   *
   * ->
   *
   * CSS 0..255
   */

  const cssRed =
    keyboardEcToCss(red);

  const cssGreen =
    keyboardEcToCss(green);

  const cssBlue =
    keyboardEcToCss(blue);


  /*
   * 实时更新颜色预览
   */

  if (preview) {

    preview.style.backgroundColor =
      `rgb(${cssRed}, ${cssGreen}, ${cssBlue})`;

  }


  /*
   * 同步 color picker
   */

  if (picker) {

    picker.value =
      keyboardRgbToHex(
        cssRed,
        cssGreen,
        cssBlue
      );

  }


  /*
   * 同步当前内存 Profile
   */

  const profile =
    getKeyboardProfile();

  if (profile) {

    profile.red =
      red;

    profile.green =
      green;

    profile.blue =
      blue;

  }

}


/* =========================================================
   更新整个键盘 UI
   ========================================================= */

function updateKeyboardLedUI(
  enabled
) {

  const toggle =
    document.getElementById(
      "keyboard-led-toggle"
    );

  if (!toggle) {
    return;
  }


  /*
   * 如果只是语言变化，
   * enabled 参数来自当前 UI。
   *
   * 这里不重新读取 EC。
   */

  toggle.checked =
    Boolean(enabled);


  updateKeyboardLedSettingsUI();

}


/* =========================================================
   更新键盘设置区域
   ========================================================= */

function updateKeyboardLedSettingsUI() {

  const profile =
    getKeyboardProfile();

  if (!profile) {
    return;
  }


  const toggle =
    document.getElementById(
      "keyboard-led-toggle"
    );

  const brightness =
    document.getElementById(
      "keyboard-brightness"
    );

  const brightnessValue =
    document.getElementById(
      "keyboard-brightness-value"
    );

  const staticMode =
    document.getElementById(
      "keyboard-mode-static"
    );

  const rainbowMode =
    document.getElementById(
      "keyboard-mode-rainbow"
    );

  const red =
    document.getElementById(
      "keyboard-red"
    );

  const green =
    document.getElementById(
      "keyboard-green"
    );

  const blue =
    document.getElementById(
      "keyboard-blue"
    );

  const redValue =
    document.getElementById(
      "keyboard-red-value"
    );

  const greenValue =
    document.getElementById(
      "keyboard-green-value"
    );

  const blueValue =
    document.getElementById(
      "keyboard-blue-value"
    );

  const picker =
    document.getElementById(
      "keyboard-color-picker"
    );

  const preview =
    document.getElementById(
      "keyboard-color-preview"
    );

  const settings =
    document.getElementById(
      "keyboard-led-settings"
    );


  /*
   * Enable
   */

  if (toggle) {

    toggle.checked =
      Boolean(profile.enabled);

  }


  /*
   * 亮度
   *
   * 0 = 0%
   * 1 = 25%
   * 2 = 50%
   * 3 = 75%
   * 4 = 100%
   */

  if (brightness) {

    brightness.value =
      profile.brightness;

  }

  if (brightnessValue) {

    brightnessValue.textContent =
      `${Number(profile.brightness) * 25}%`;

  }


  /*
   * 模式
   */

  if (staticMode) {

    staticMode.checked =
      !profile.rainbow;

  }

  if (rainbowMode) {

    rainbowMode.checked =
      Boolean(profile.rainbow);

  }


  /*
   * RGB
   */

  if (red) {

    red.value =
      profile.red;

  }

  if (green) {

    green.value =
      profile.green;

  }

  if (blue) {

    blue.value =
      profile.blue;

  }


  if (redValue) {

    redValue.textContent =
      profile.red;

  }

  if (greenValue) {

    greenValue.textContent =
      profile.green;

  }

  if (blueValue) {

    blueValue.textContent =
      profile.blue;

  }


  /*
   * RGB 预览
   */

  const cssRed =
    keyboardEcToCss(
      profile.red
    );

  const cssGreen =
    keyboardEcToCss(
      profile.green
    );

  const cssBlue =
    keyboardEcToCss(
      profile.blue
    );


  if (preview) {

    preview.style.backgroundColor =
      `rgb(${cssRed}, ${cssGreen}, ${cssBlue})`;

  }


  if (picker) {

    picker.value =
      keyboardRgbToHex(
        cssRed,
        cssGreen,
        cssBlue
      );

  }


  /*
   * 设置区域状态
   */

  if (settings) {

    settings.classList.toggle(
      "disabled",
      !profile.enabled
    );

    settings.classList.toggle(
      "rainbow-mode",
      Boolean(profile.rainbow)
    );

  }

}


/* =========================================================
   更新 AC / DC Tab
   ========================================================= */

function updateKeyboardPowerTabs() {

  const acButton =
    document.getElementById(
      "keyboard-power-ac"
    );

  const dcButton =
    document.getElementById(
      "keyboard-power-dc"
    );


  if (acButton) {

    acButton.classList.toggle(
      "active",
      keyboardPowerMode === "ac"
    );

  }


  if (dcButton) {

    dcButton.classList.toggle(
      "active",
      keyboardPowerMode === "dc"
    );

  }

}


/* =========================================================
   从 Rust 获取 AC / DC
   ========================================================= */

async function loadKeyboardProfile(
  powerMode
) {

  const ac =
    powerMode === "ac";


  try {

    const result =
      await invoke(
        "get_keyboard_led",
        {
          ac: ac,
        }
      );


    /*
     * 读取并限制 EC 数据范围
     */

    const brightness =
      Math.max(
        0,
        Math.min(
          4,
          Number(result.brightness) || 0
        )
      );


    const red =
      Math.max(
        0,
        Math.min(
          50,
          Number(result.red) || 0
        )
      );


    const green =
      Math.max(
        0,
        Math.min(
          50,
          Number(result.green) || 0
        )
      );


    const blue =
      Math.max(
        0,
        Math.min(
          50,
          Number(result.blue) || 0
        )
      );


    /*
     * 保存 Profile
     */

    keyboardProfiles[powerMode] = {

      loaded: true,

      enabled:
        Boolean(result.enabled),

      brightness:
        brightness,

      rainbow:
        Boolean(result.rainbow),

      red:
        red,

      green:
        green,

      blue:
        blue,

    };


    console.log(
      `Keyboard ${powerMode.toUpperCase()} profile:`,
      keyboardProfiles[powerMode]
    );


    /*
     * 如果当前就是这个 Profile，
     * 更新界面
     */

    if (
      keyboardPowerMode ===
      powerMode
    ) {

      updateKeyboardLedSettingsUI();

    }


    return true;

  } catch (error) {

    console.error(
      `读取键盘 ${powerMode.toUpperCase()} 配置失败:`,
      error
    );

    return false;

  }

}


/* =========================================================
   初始化：同时读取 AC + DC
   ========================================================= */

async function loadKeyboardProfiles() {

  /*
   * 两个 profile 同时读取。
   *
   * 这里不是共用一个结果。
   *
   * AC -> get_keyboard_led({ ac: true })
   * DC -> get_keyboard_led({ ac: false })
   */

  const results =
    await Promise.all([
      loadKeyboardProfile("ac"),
      loadKeyboardProfile("dc"),
    ]);


  /*
   * 默认显示 AC
   */

  keyboardPowerMode =
    "ac";


  updateKeyboardPowerTabs();

  updateKeyboardLedSettingsUI();


  console.log(
    "Keyboard profiles loaded:",
    keyboardProfiles
  );


  return results.every(
    Boolean
  );

}


/* =========================================================
   切换 AC / DC
   ========================================================= */

async function switchKeyboardPowerMode(
  mode
) {

  if (
    mode !== "ac" &&
    mode !== "dc"
  ) {

    return;

  }


  keyboardPowerMode =
    mode;


  updateKeyboardPowerTabs();


  /*
   * 如果初始化时已经读取过，
   * 直接使用缓存。
   */

  if (
    keyboardProfiles[mode].loaded
  ) {

    updateKeyboardLedSettingsUI();

    return;

  }


  /*
   * 如果之前没有成功读取，
   * 再读取一次。
   */

  await loadKeyboardProfile(
    mode
  );

}


/* =========================================================
   设置 Enable
   ========================================================= */

async function setKeyboardEnabled(
  enabled
) {

  const profile =
    getKeyboardProfile();

  if (!profile) {
    return;
  }


  const ac =
    keyboardPowerMode === "ac";


  /*
   * 先更新 UI，
   * 让用户立即看到状态。
   */

  profile.enabled =
    Boolean(enabled);

  updateKeyboardLedSettingsUI();


  try {

    await invoke(
      "set_keyboard_enabled",
      {
        enable:
          Boolean(enabled),

        ac:
          ac,
      }
    );


    console.log(
      `键盘 ${keyboardPowerMode.toUpperCase()} LED:`,
      enabled
        ? "开启"
        : "关闭"
    );

  } catch (error) {

    console.error(
      "设置键盘 LED 开关失败:",
      error
    );


    /*
     * 写入失败后重新读取真实状态。
     */

    await loadKeyboardProfile(
      keyboardPowerMode
    );

  }

}


/* =========================================================
   设置亮度
   ========================================================= */

async function setKeyboardBrightness(
  brightness
) {

  const profile =
    getKeyboardProfile();

  if (!profile) {
    return;
  }


  const value =
    Math.max(
      0,
      Math.min(
        4,
        Number(brightness)
      )
    );


  const ac =
    keyboardPowerMode === "ac";


  /*
   * 先更新 UI。
   */

  profile.brightness =
    value;


  updateKeyboardLedSettingsUI();


  try {

    await invoke(
      "set_keyboard_brightness",
      {
        brightness:
          value,

        ac:
          ac,
      }
    );


    console.log(
      `键盘 ${keyboardPowerMode.toUpperCase()} 亮度:`,
      value
    );

  } catch (error) {

    console.error(
      "设置键盘亮度失败:",
      error
    );


    await loadKeyboardProfile(
      keyboardPowerMode
    );

  }

}


/* =========================================================
   设置 Rainbow
   ========================================================= */

async function setKeyboardRainbow(
  rainbow
) {

  const profile =
    getKeyboardProfile();

  if (!profile) {
    return;
  }


  const ac =
    keyboardPowerMode === "ac";


  profile.rainbow =
    Boolean(rainbow);


  updateKeyboardLedSettingsUI();


  try {

    await invoke(
      "set_keyboard_rainbow",
      {
        rainbow:
          Boolean(rainbow),

        ac:
          ac,
      }
    );


    console.log(
      `键盘 ${keyboardPowerMode.toUpperCase()} 彩虹模式:`,
      rainbow
    );

  } catch (error) {

    console.error(
      "设置键盘彩虹模式失败:",
      error
    );


    await loadKeyboardProfile(
      keyboardPowerMode
    );

  }

}


/* =========================================================
   设置 RGB
   ========================================================= */

async function setKeyboardRGB(
  red,
  green,
  blue
) {

  const profile =
    getKeyboardProfile();

  if (!profile) {
    return;
  }


  const r =
    Math.max(
      0,
      Math.min(
        50,
        Number(red)
      )
    );


  const g =
    Math.max(
      0,
      Math.min(
        50,
        Number(green)
      )
    );


  const b =
    Math.max(
      0,
      Math.min(
        50,
        Number(blue)
      )
    );


  const ac =
    keyboardPowerMode === "ac";


  try {

    /*
     * 不使用 Promise.all。
     *
     * 你的目标是通过 ACPIDriver 操作 EC，
     * 这里串行写三个寄存器更稳妥。
     */

    await invoke(
      "set_keyboard_red",
      {
        red:
          r,

        ac:
          ac,
      }
    );


    await invoke(
      "set_keyboard_green",
      {
        green:
          g,

        ac:
          ac,
      }
    );


    await invoke(
      "set_keyboard_blue",
      {
        blue:
          b,

        ac:
          ac,
      }
    );


    /*
     * 三个写入全部成功后更新 Profile。
     */

    profile.red =
      r;

    profile.green =
      g;

    profile.blue =
      b;


    /*
     * 更新 UI。
     */

    updateKeyboardLedSettingsUI();


    console.log(
      `键盘 ${keyboardPowerMode.toUpperCase()} RGB:`,
      r,
      g,
      b
    );

  } catch (error) {

    console.error(
      "设置键盘 RGB 失败:",
      error
    );


    /*
     * 任意一个写入失败，
     * 重新从 EC 获取真实状态。
     */

    await loadKeyboardProfile(
      keyboardPowerMode
    );

  }

}


/* =========================================================
   Color Picker -> RGB
   ========================================================= */

function updateKeyboardFromColorPicker() {

  const picker =
    document.getElementById(
      "keyboard-color-picker"
    );

  if (!picker) {
    return null;
  }


  const rgb =
    keyboardHexToRgb(
      picker.value
    );


  const red =
    keyboardCssToEc(
      rgb.r
    );

  const green =
    keyboardCssToEc(
      rgb.g
    );

  const blue =
    keyboardCssToEc(
      rgb.b
    );


  /*
   * 更新滑块。
   */

  const redInput =
    document.getElementById(
      "keyboard-red"
    );

  const greenInput =
    document.getElementById(
      "keyboard-green"
    );

  const blueInput =
    document.getElementById(
      "keyboard-blue"
    );


  if (redInput) {
    redInput.value =
      red;
  }

  if (greenInput) {
    greenInput.value =
      green;
  }

  if (blueInput) {
    blueInput.value =
      blue;
  }


  /*
   * 立即更新预览。
   */

  updateKeyboardColorPreview();


  return {
    red,
    green,
    blue,
  };

}


/* =========================================================
   初始化键盘事件
   ========================================================= */

function bindKeyboardLedEvents() {

  const toggle =
    document.getElementById(
      "keyboard-led-toggle"
    );

  const brightness =
    document.getElementById(
      "keyboard-brightness"
    );

  const staticMode =
    document.getElementById(
      "keyboard-mode-static"
    );

  const rainbowMode =
    document.getElementById(
      "keyboard-mode-rainbow"
    );

  const red =
    document.getElementById(
      "keyboard-red"
    );

  const green =
    document.getElementById(
      "keyboard-green"
    );

  const blue =
    document.getElementById(
      "keyboard-blue"
    );

  const picker =
    document.getElementById(
      "keyboard-color-picker"
    );

  const acButton =
    document.getElementById(
      "keyboard-power-ac"
    );

  const dcButton =
    document.getElementById(
      "keyboard-power-dc"
    );


  /*
   * 页面不存在键盘控件。
   */

  if (
    !toggle ||
    !brightness ||
    !staticMode ||
    !rainbowMode ||
    !red ||
    !green ||
    !blue
  ) {

    console.warn(
      "键盘 LED 控件不存在"
    );

    return;

  }


  /* =======================================================
     AC
     ======================================================= */

  if (acButton) {

    acButton.addEventListener(
      "click",
      async () => {

        await switchKeyboardPowerMode(
          "ac"
        );

      }
    );

  }


  /* =======================================================
     DC
     ======================================================= */

  if (dcButton) {

    dcButton.addEventListener(
      "click",
      async () => {

        await switchKeyboardPowerMode(
          "dc"
        );

      }
    );

  }


  /* =======================================================
     Enable
     ======================================================= */

  toggle.addEventListener(
    "change",
    async () => {

      await setKeyboardEnabled(
        toggle.checked
      );

    }
  );


  /* =======================================================
     Brightness
     ======================================================= */

  /*
   * 拖动过程中只更新 UI，
   * 不连续写 EC。
   */

  brightness.addEventListener(
    "input",
    () => {

      const profile =
        getKeyboardProfile();

      if (!profile) {
        return;
      }


      const value =
        Number(
          brightness.value
        );


      profile.brightness =
        value;


      const brightnessValue =
        document.getElementById(
          "keyboard-brightness-value"
        );


      if (brightnessValue) {

        brightnessValue.textContent =
          `${value * 25}%`;

      }

    }
  );


  /*
   * 松开滑块后写 EC。
   */

  brightness.addEventListener(
    "change",
    async () => {

      await setKeyboardBrightness(
        Number(
          brightness.value
        )
      );

    }
  );


  /* =======================================================
     Static
     ======================================================= */

  staticMode.addEventListener(
    "change",
    async () => {

      if (!staticMode.checked) {
        return;
      }


      await setKeyboardRainbow(
        false
      );

    }
  );


  /* =======================================================
     Rainbow
     ======================================================= */

  rainbowMode.addEventListener(
    "change",
    async () => {

      if (!rainbowMode.checked) {
        return;
      }


      await setKeyboardRainbow(
        true
      );

    }
  );


  /* =======================================================
     RGB - 实时预览
     ======================================================= */

  red.addEventListener(
    "input",
    () => {

      updateKeyboardColorPreview();

    }
  );


  green.addEventListener(
    "input",
    () => {

      updateKeyboardColorPreview();

    }
  );


  blue.addEventListener(
    "input",
    () => {

      updateKeyboardColorPreview();

    }
  );


  /* =======================================================
     RGB - 写入 EC
     ======================================================= */

  red.addEventListener(
    "change",
    async () => {

      const profile =
        getKeyboardProfile();

      if (!profile) {
        return;
      }


      await setKeyboardRGB(
        Number(red.value),
        profile.green,
        profile.blue
      );

    }
  );


  green.addEventListener(
    "change",
    async () => {

      const profile =
        getKeyboardProfile();

      if (!profile) {
        return;
      }


      await setKeyboardRGB(
        profile.red,
        Number(green.value),
        profile.blue
      );

    }
  );


  blue.addEventListener(
    "change",
    async () => {

      const profile =
        getKeyboardProfile();

      if (!profile) {
        return;
      }


      await setKeyboardRGB(
        profile.red,
        profile.green,
        Number(blue.value)
      );

    }
  );


  /* =======================================================
     Color Picker
     ======================================================= */

  if (picker) {

    /*
     * 点击/拖动系统颜色选择器时，
     * 立即更新 RGB 滑块和预览。
     *
     * 此时不写 EC。
     */

    picker.addEventListener(
      "input",
      () => {

        updateKeyboardFromColorPicker();

      }
    );


    /*
     * 颜色选择完成后写 EC。
     */

    picker.addEventListener(
      "change",
      async () => {

        const rgb =
          updateKeyboardFromColorPicker();


        if (!rgb) {
          return;
        }


        await setKeyboardRGB(
          rgb.red,
          rgb.green,
          rgb.blue
        );

      }
    );

  }

}


/* =========================================================
   初始化键盘 LED
   ========================================================= */

async function initKeyboardLed() {

  /*
   * 先绑定事件。
   *
   * 这样即使读取 EC 比较慢，
   * 页面也不会出现没有事件的问题。
   */

  bindKeyboardLedEvents();


  /*
   * 默认显示 AC。
   */

  keyboardPowerMode =
    "ac";


  updateKeyboardPowerTabs();


  /*
   * 同时读取 AC + DC。
   *
   * 这里会调用：
   *
   * get_keyboard_led({ ac: true })
   * get_keyboard_led({ ac: false })
   */

  await loadKeyboardProfiles();


  /*
   * 再次确保当前 AC UI 与 EC 同步。
   */

  updateKeyboardPowerTabs();

  updateKeyboardLedSettingsUI();

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
   检查更新
   ========================================================= */

async function checkGithubVersion() {
  const versionElement = document.getElementById("app-version");
  const updateBadge = document.getElementById("update-badge");

  if (!versionElement || !updateBadge) {
    return;
  }
  try {
    const currentVersion = versionElement.textContent
      .trim()
      .replace(/^v/i, "");
    const response = await fetch(
      "https://api.github.com/repos/cyear/NUCtool/releases/latest",
      {
        headers: {
          Accept: "application/vnd.github+json"
        }
      }
    );
    if (!response.ok) {
      throw new Error(
        `GitHub API returned ${response.status}`
      );
    }
    const release = await response.json();
    const tagName = String(release.tag_name || "").trim();
    const match = tagName.match(/v(\d+(?:\.\d+)+)$/i);
    if (!match) {
      console.warn(
        "无法解析 GitHub 版本号:",
        tagName
      );
      return;
    }
    const latestVersion = match[1];
    console.log(
      `当前版本: ${currentVersion}, GitHub 最新版本: ${latestVersion}`
    );
    if (
      compareVersions(
        latestVersion,
        currentVersion
      ) > 0
    ) {
      updateBadge.textContent =
        `NEW v${latestVersion}`;

      updateBadge.style.display =
        "inline-flex";
      console.log(release.html_url);
      updateBadge.onclick = async () => {
        if (release.html_url) {
          try {
              if (window.__TAURI__.opener?.openUrl) {
                  await window.__TAURI__.opener.openUrl(release.html_url);
              } else {
                  window.open(url, '_blank');
              }
          } catch (e) {
              console.error('[UPDATE] failed to open URL:', e);
          }
        }
      };
    } else {
      updateBadge.style.display =
        "none";
    }
  } catch (error) {
    // 更新检查失败不影响应用正常运行
    console.warn(
      "检查 GitHub 最新版本失败:",
      error
    );
    updateBadge.style.display =
      "none";
  }
}

function compareVersions(a, b) {
  const parseVersion = (version) => {
    return version
      .split(".")
      .map((part) => {
        const match = part.match(/^\d+/);
        return match
          ? Number(match[0])
          : 0;
      });
  };

  const av = parseVersion(a);
  const bv = parseVersion(b);

  const length = Math.max(
    av.length,
    bv.length
  );

  for (let i = 0; i < length; i++) {
    const x = av[i] || 0;
    const y = bv[i] || 0;

    if (x > y) {
      return 1;
    }

    if (x < y) {
      return -1;
    }
  }

  return 0;
}


// =========================================================
// New Fan 状态实时推送
// =========================================================

/* =========================================================
 * New Fan 写控制状态
 *
 * independent / fan / duty
 * 三项全部 false：
 *     → 启动控制
 *
 * 任意一项 true：
 *     → 退出控制
 *
 * mode 不参与判断
 * ========================================================= */

let newFanWriteRunning = false;


/* =========================================================
 * 根据 EC 状态更新按钮
 * ========================================================= */

function updateNewFanWriteControlState(status) {
  const startButton =
    document.getElementById(
      "newfan-start-button"
    );

  if (!startButton) {
    return;
  }

  const independent =
    Boolean(status?.independent);

  const fan =
    Boolean(status?.fan);

  const duty =
    Boolean(status?.duty);

  /*
   * 三项全部 false：
   * 控制未启动
   */
  newFanWriteRunning =
    independent ||
    fan ||
    duty;

  if (newFanWriteRunning) {
    startButton.textContent =
      t("newfan.stopControl");

    startButton.classList.remove(
      "primary"
    );

    startButton.classList.add(
      "danger"
    );
  } else {
    startButton.textContent =
      t("newfan.startControl");

    startButton.classList.remove(
      "danger"
    );

    startButton.classList.add(
      "primary"
    );
  }
}


/* =========================================================
 * 启动 New Fan 写控制
 * ========================================================= */

async function startNewFanWrite() {
  try {
    const fandata = {
      left_fan: getNewFanCurve(newFanMainCurve),
      right_fan: getNewFanCurve(newFanSecondaryCurve),
    };

    console.log(
      "[NewFan] 启动写控制，发送配置:",
      fandata
    );

    await invoke(
      "start_newfan_write",
      {
        fandata,
      }
    );

    console.log(
      "[NewFan] 写控制已启动"
    );

  } catch (error) {
    console.error(
      "[NewFan] 启动写控制失败:",
      error
    );
  }
}


/* =========================================================
 * 停止 New Fan 写控制
 * ========================================================= */

async function stopNewFanWrite() {
  try {
    await invoke(
      "stop_newfan_write"
    );

    console.log(
      "[NewFan] 写控制已停止"
    );

  } catch (error) {
    console.error(
      "[NewFan] 停止写控制失败:",
      error
    );
  }
}


/* =========================================================
 * 按钮点击
 * ========================================================= */

function initNewFanWriteControl() {
  const startButton =
    document.getElementById(
      "newfan-start-button"
    );

  if (!startButton) {
    return;
  }

  startButton.addEventListener(
    "click",
    async () => {
      if (newFanWriteRunning) {
        await stopNewFanWrite();
      } else {
        await startNewFanWrite();
      }
    }
  );
}

function updateNewFanOverallStatus(status) {
  const statusDot = document.getElementById("newfan-status-dot");
  const statusText = document.getElementById("newfan-status-text");

  if (!statusDot || !statusText) return;

  const mode = Boolean(status?.mode);
  const independent = Boolean(status?.independent);
  const fan = Boolean(status?.fan);
  const duty = Boolean(status?.duty);

  const allControlOff =
    !independent &&
    !fan &&
    !duty;

  const allRunning =
    mode &&
    independent &&
    fan &&
    duty;

  // 清除所有状态颜色
  statusDot.classList.remove("active", "error", "warning");

  if (allRunning) {
    // Mode + 三项控制全部开启
    statusDot.classList.add("active");
    statusText.textContent = t("status.running");
  } else if (allControlOff) {
    // 三项控制全部关闭
    // 即使 mode = true，也属于未运行
    statusText.textContent = t("status.stopped");
  } else {
    // 三项控制出现部分开启
    // 说明状态不一致
    statusDot.classList.add("error");
    statusText.textContent = t("status.error");
  }
}

/* =========================================================
 * New Fan 状态监听
 * ========================================================= */

async function initNewFanStatusListener() {
  try {
    await listen(
      "newfan-status",
      (event) => {
        const status =
          event.payload || {};

        console.log(
          "[NewFan] status:",
          status
        );

        /*
         * 更新三个状态指示灯
         */

        updateNewFanStatus(
          "newfan-mode-indicator",
          "newfan-mode-value",
          status.mode
        );

        updateNewFanStatus(
          "newfan-independent-indicator",
          "newfan-independent-value",
          status.independent
        );

        updateNewFanStatus(
          "newfan-fan-indicator",
          "newfan-fan-value",
          status.fan
        );

        updateNewFanStatus(
          "newfan-duty-indicator",
          "newfan-duty-value",
          status.duty
        );
        updateNewFanOverallStatus(status);
        /*
         * 根据 independent / fan / duty
         * 更新启动/退出控制按钮
         */
        updateNewFanWriteControlState(
          status
        );
      }
    );

    await invoke(
      "start_newfan_monitor"
    );

    console.log(
      "[NewFan] monitor started"
    );

  } catch (error) {
    console.error(
      "[NewFan] 初始化状态监听失败:",
      error
    );
  }
}


// =========================================================
// New Fan 状态显示
// =========================================================

function updateNewFanStatus(indicatorId, valueId, status) {
    const indicator = document.getElementById(indicatorId);
    const value = document.getElementById(valueId);
    if (!indicator || !value) {
        return;
    }
    const ok = Boolean(status);
    // 指示灯状态
    indicator.classList.toggle("active", ok);
    indicator.classList.toggle("error", !ok);
    // 状态文字
    value.textContent = ok ? "YES" : "NOT";
}


/* =========================================================
 * New Fan 曲线
 * ========================================================= */

const NEWFAN_COLOR_MAIN = "#3987e5";
const NEWFAN_COLOR_SECONDARY = "#d95926";
const NEWFAN_GRID = "rgba(255, 255, 255, 0.08)";
const NEWFAN_DEFAULT_TEMPERATURES = [
  30,
  35,
  40,
  45,
  50,
  55,
  60,
  65,
  70,
  75,
  80,
  85,
  90,
  95,
  100,
];

let newFanMainCurve = null;
let newFanSecondaryCurve = null;

function createNewFanDefaultConfig() {
  const createCurve = () =>
    NEWFAN_DEFAULT_TEMPERATURES.map(
      (temperature) => ({
        temperature,
        speed: 50,
      })
    );

  return {
    left_fan: createCurve(),
    right_fan: createCurve(),
  };
}
function createNewFanCurve(id, color, points) {
  const canvas = document.getElementById(id);

  if (!canvas) {
    console.warn(`[NewFan] Canvas not found: ${id}`);
    return null;
  }

  if (!Array.isArray(points) || points.length === 0) {
    console.warn(
      `[NewFan] No curve data for ${id}`
    );
    return null;
  }

  const labels = points.map(
    (point) => Number(point.temperature)
  );

  const data = points.map(
    (point) => Number(point.speed)
  );

  return new Chart(canvas, {
    type: "line",

    data: {
      labels,

      datasets: [
        {
          data,

          borderColor: color,
          borderWidth: 2,

          pointRadius: 4,
          pointHoverRadius: 6,

          pointBackgroundColor: color,

          fill: false,
        },
      ],
    },

    options: {
      maintainAspectRatio: false,

      cubicInterpolationMode: "monotone",

      plugins: {
        legend: {
          display: false,
        },

        tooltip: {
          displayColors: false,

          callbacks: {
            title: (items) =>
              `${items[0].label} °C`,

            label: (item) =>
              `${item.formattedValue} %`,
          },
        },

        dragData: {
          round: 0,

          dragX: false,

          onDrag: (
            event,
            datasetIndex,
            index,
            value
          ) => {
            return Math.max(
              0,
              Math.min(100, value)
            );
          },
        },
      },

      scales: {
        x: {
          grid: {
            display: false,
          },

          ticks: {
            maxRotation: 0,

            callback: (value, index) => {
              const temperature =
                labels[index];

              return temperature !== undefined
                ? `${temperature}°`
                : "";
            },
          },
        },

        y: {
          min: 0,

          max: 100,

          grid: {
            color: NEWFAN_GRID,
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
  });
}

function getNewFanCurve(chart) {
  if (!chart) {
    return [];
  }

  return chart.data.labels.map(
    (temperature, index) => ({
      temperature: Number(temperature),

      speed: Number(
        chart.data.datasets[0].data[index]
      ),
    })
  );
}

async function loadNewFanConfig() {
  let data = null;

  try {
    data = await invoke("load_fan_config");

    console.log(
      "[NewFan] loaded config:",
      data
    );

  } catch (error) {
    console.warn(
      "[NewFan] 加载配置失败，使用默认 50% 曲线:",
      error
    );
  }

  /*
   * 没有配置：
   *
   * Main      → 全部 50%
   * Secondary → 全部 50%
   */
  if (
    !data ||
    !Array.isArray(data.left_fan) ||
    !Array.isArray(data.right_fan)
  ) {
    console.warn(
      "[NewFan] 没有有效配置，使用默认 50% 曲线"
    );

    data = createNewFanDefaultConfig();
  }

  /*
   * 销毁旧图表
   */
  if (newFanMainCurve) {
    newFanMainCurve.destroy();
    newFanMainCurve = null;
  }

  if (newFanSecondaryCurve) {
    newFanSecondaryCurve.destroy();
    newFanSecondaryCurve = null;
  }

  /*
   * 根据 Rust 返回的实际节点创建图表
   */
  newFanMainCurve =
    createNewFanCurve(
      "newfan-main-curve",
      NEWFAN_COLOR_MAIN,
      data.left_fan
    );

  newFanSecondaryCurve =
    createNewFanCurve(
      "newfan-secondary-curve",
      NEWFAN_COLOR_SECONDARY,
      data.right_fan
    );

  const success =
    newFanMainCurve !== null &&
    newFanSecondaryCurve !== null;

  if (!success) {
    console.error(
      "[NewFan] 创建风扇曲线失败"
    );
  }

  return success;
}

async function saveNewFanConfig() {
  try {
    const fanData = {
      left_fan:
        getNewFanCurve(
          newFanMainCurve
        ),

      right_fan:
        getNewFanCurve(
          newFanSecondaryCurve
        ),
    };

    console.log(
      "[NewFan] saving config:",
      fanData
    );

    await invoke(
      "save_fan_config",
      {
        fanData,
      }
    );

    const button =
      document.getElementById(
        "newfan-save-button"
      );

    if (button) {
      button.textContent =
        t("common.saved");

      setTimeout(() => {
        button.textContent =
          t("newfan.saveConfig");
      }, 1200);
    }

    return true;

  } catch (error) {
    console.error(
      "[NewFan] 保存配置失败:",
      error
    );

    return false;
  }
}


/* =========================================================
 * Load / Save 按钮
 * ========================================================= */

const newFanLoadButton =
  document.getElementById(
    "newfan-load-button"
  );

const newFanSaveButton =
  document.getElementById(
    "newfan-save-button"
  );


if (newFanLoadButton) {
  newFanLoadButton.addEventListener(
    "click",
    loadNewFanConfig
  );
}


if (newFanSaveButton) {
  newFanSaveButton.addEventListener(
    "click",
    saveNewFanConfig
  );
}

window.loadNewFanConfig =
  loadNewFanConfig;

window.saveNewFanConfig =
  saveNewFanConfig;

async function initNewFanConfig() {
  await loadNewFanConfig();
}

window.initNewFanConfig =
  initNewFanConfig;


/* =========================================================
   初始化
   ========================================================= */

async function init() {
  // 初始化语言
  initI18n();
  await loadConfig();
  await initNewFanConfig();
  await initNewFanStatusListener();
  initNewFanWriteControl();
  // 初始化风扇控制状态
  await initFanControlStatus();
  updateControlState();
  loadPerformanceMode();
  await loadBatteryHealthMode();
  loadFanMode();
  loadAutostartState();
  await initGscCheck();
  await initKeyboardLed();
  if (model != "LAPAC71H" && model != "LAPAC71G") {
    await initLightbar();
  }
  checkGithubVersion();
}

init();