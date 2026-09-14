const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

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
  if (e.key === "F12") {
    e.preventDefault();
    e.stopPropagation();
    return;
  }

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

// ========== 更新监控数据 ==========
function updateUI(data) {
  document.getElementById("cpu-temp").textContent = data.cpu_temp ?? "--";
  document.getElementById("gpu-temp").textContent = data.gpu_temp ?? "--";
  document.getElementById("fan1-rpm").textContent = data.fan1_rpm ?? "--";
  document.getElementById("fan2-rpm").textContent = data.fan2_rpm ?? "--";

  // 风扇控制页同步显示当前转速
  document.getElementById("fan1-current").textContent = data.fan1_rpm ?? "--";
  document.getElementById("fan2-current").textContent = data.fan2_rpm ?? "--";
}

// ========== 监听 Rust 推送 ==========
async function startListen() {
  const statusEl = document.getElementById("status");

  try {
    await invoke("start_sensor_loop");

    await listen("sensor-update", (event) => {
      updateUI(event.payload);
      statusEl.textContent = "实时监控中";
      statusEl.className = "status ok";
    });

    statusEl.textContent = "已连接";
    statusEl.className = "status ok";
  } catch (e) {
    console.error(e);
    statusEl.textContent = "连接失败";
    statusEl.className = "status err";
  }
}

startListen();



/* =========================================================
   页面切换
   ========================================================= */

document.querySelectorAll(".nav-btn").forEach((button) => {
  button.addEventListener("click", () => {
    const page = button.dataset.page;

    document.querySelectorAll(".nav-btn")
      .forEach((btn) => btn.classList.remove("active"));

    document.querySelectorAll(".page")
      .forEach((el) => el.classList.remove("active"));

    button.classList.add("active");
    document.getElementById(`page-${page}`)?.classList.add("active");
  });
});


/* =========================================================
   Chart.js
   ========================================================= */

const COLOR_CPU = "#3987e5";
const COLOR_GPU = "#d95926";

const GRID = "rgba(255, 255, 255, 0.08)";

Chart.defaults.color = "#898781";
Chart.defaults.borderColor = GRID;
Chart.defaults.font.family =
  '"Segoe UI", "Microsoft YaHei", system-ui, sans-serif';
Chart.defaults.font.size = 11;
Chart.defaults.animation = false;


/* 温度节点：30°C ~ 100°C，每 5°C 一个节点 */
const CURVE_TEMPS =
  Array.from({ length: 15 }, (_, i) => 30 + i * 5);


/* =========================================================
   创建风扇曲线
   ========================================================= */

function createCurveChart(id, color) {
  return new Chart(document.getElementById(id), {
    type: "line",

    data: {
      labels: CURVE_TEMPS,

      datasets: [{
        data: Array(CURVE_TEMPS.length).fill(50),

        borderColor: color,
        borderWidth: 2,

        pointRadius: 4,
        pointHoverRadius: 6,

        pointBackgroundColor: color,

        fill: false,
      }],
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

            callback: (value, index) =>
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
            callback: (value) => `${value}%`,
          },
        },
      },
    },
  });
}


const leftFanCurve =
  createCurveChart("leftFanCurve", COLOR_CPU);

const rightFanCurve =
  createCurveChart("rightFanCurve", COLOR_GPU);


/* =========================================================
   曲线数据
   ========================================================= */

function getFanCurveData() {
  const getCurve = (chart) =>
    chart.data.labels.map((temperature, index) => ({
      temperature,
      speed: chart.data.datasets[0].data[index],
    }));

  return {
    left_fan: getCurve(leftFanCurve),
    right_fan: getCurve(rightFanCurve),
  };
}


/* 将后端配置应用到曲线 */
function applyCurve(chart, points) {
  if (!Array.isArray(points)) return;

  const map = new Map(
    points.map((point) => [
      Math.round(point.temperature),
      point.speed,
    ])
  );

  chart.data.datasets[0].data =
    chart.data.labels.map((temperature, index) =>
      map.get(temperature) ??
      chart.data.datasets[0].data[index]
    );

  chart.update();
}


/* =========================================================
   配置
   ========================================================= */

async function loadConfig() {
  try {
    const data = await invoke("load_fan_config");

    applyCurve(leftFanCurve, data.left_fan);
    applyCurve(rightFanCurve, data.right_fan);

    return true;
  } catch (error) {
    console.error("加载配置失败:", error);
    return false;
  }
}


async function saveConfig() {
  try {
    await invoke("save_fan_config", {
      fanData: getFanCurveData(),
    });

    saveConfigButton.textContent = "已保存 ✓";

    setTimeout(() => {
      saveConfigButton.textContent = "保存配置";
    }, 1200);

  } catch (error) {
    console.error("保存配置失败:", error);
  }
}


/* =========================================================
   风扇控制
   ========================================================= */

const startStopButton =
  document.getElementById("startStopButton");

const loadConfigButton =
  document.getElementById("loadConfigButton");

const saveConfigButton =
  document.getElementById("saveConfigButton");

const fanStatus =
  document.getElementById("fan-status");


let isRunning = false;


function updateControlState() {
  startStopButton.textContent =
    isRunning ? "停止控制" : "启动控制";

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
      ? "控制运行中"
      : "未运行";

  fanStatus.className =
    isRunning
      ? "status ok"
      : "status";
}


async function startControl() {
  try {
    await invoke("start_fan_control", {
      fanData: getFanCurveData(),
    });

    isRunning = true;
    updateControlState();

  } catch (error) {
    console.error("启动风扇控制失败:", error);
  }
}


async function stopControl() {
  try {
    await invoke("stop_fan_control");

    isRunning = false;
    updateControlState();

  } catch (error) {
    console.error("停止风扇控制失败:", error);
  }
}


/* =========================================================
   按钮
   ========================================================= */

startStopButton.addEventListener("click", () => {
  isRunning
    ? stopControl()
    : startControl();
});

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

const tuningButtons = document.querySelectorAll(".tuning-btn");

tuningButtons.forEach((button) => {

  button.addEventListener("click", async () => {

    const mode = button.dataset.mode;

    // 更新选中状态
    tuningButtons.forEach((btn) => {
      btn.classList.remove("active");
    });

    button.classList.add("active");

    console.log("模式:", mode);
    await invoke("set_performance_mode", {
      mode: mode
    });

  });

});


// =====================================================
// 功耗设置
// =====================================================

const tdpRefreshButton = document.getElementById("tdpRefreshButton");
const tdpSetButtons = document.querySelectorAll(".tdp-set-btn");


// -----------------------------------------------------
// 读取功耗配置
// -----------------------------------------------------

async function loadTdp() {

  try {

    const tdp = await invoke("get_tdp");

    console.log("TDP:", tdp);

    document.getElementById("cpu-pl1").value = tdp.cpu_pl1;
    document.getElementById("cpu-pl2").value = tdp.cpu_pl2;
    document.getElementById("cpu-pl4").value = tdp.cpu_pl4;

    document.getElementById("gpu-pl1").value = tdp.gpu_pl1;
    document.getElementById("gpu-pl2").value = tdp.gpu_pl2;

  } catch (error) {

    console.error("读取功耗配置失败:", error);

  }

}


// -----------------------------------------------------
// 写入功耗
// -----------------------------------------------------

async function setTdp(type) {

  const input = document.getElementById(type);

  if (!input) {
    return;
  }

  const value = Number(input.value);

  if (!Number.isFinite(value) || value < 0) {

    console.error("无效的功耗值:", value);

    return;
  }


  try {
    
    await invoke("set_tdp", {
        tdpType: type,
        value: value
    });
    console.log(`设置 ${type}: ${value} W`);

  } catch (error) {

    console.error(`设置 ${type} 失败:`, error);

  }

}


// -----------------------------------------------------
// 读取按钮
// -----------------------------------------------------

if (tdpRefreshButton) {

  tdpRefreshButton.addEventListener("click", () => {
    loadTdp();
  });

}


// -----------------------------------------------------
// 写入按钮
// -----------------------------------------------------

tdpSetButtons.forEach((button) => {

  button.addEventListener("click", () => {

    const type = button.dataset.tdp;

    setTdp(type);

  });

});


/* =========================================================
   初始化
   ========================================================= */

async function init() {
  await loadConfig();
  updateControlState();
}

init();