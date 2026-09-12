const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

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