// NUCtool 主界面 — 风扇曲线控制
const invoke = window.__TAURI__.core.invoke;
const { listen } = window.__TAURI__.event;

document.addEventListener('contextmenu', (e) => e.preventDefault());

/* ---- 设计令牌(与 app.css 保持一致) ---- */
const COLOR_CPU = '#3987e5';
const COLOR_GPU = '#d95926';
const INK_MUTED = '#898781';
const GRID = 'rgba(255, 255, 255, 0.08)';

/* ---- Chart.js 全局默认: 暗色、克制的网格与文字 ---- */
Chart.defaults.color = INK_MUTED;
Chart.defaults.borderColor = GRID;
Chart.defaults.font.family = '"Segoe UI", "Microsoft YaHei", system-ui, sans-serif';
Chart.defaults.font.size = 11;
Chart.defaults.animation = false;
Chart.defaults.plugins.legend.labels.boxWidth = 8;
Chart.defaults.plugins.legend.labels.boxHeight = 8;
Chart.defaults.plugins.legend.labels.usePointStyle = true;
Chart.defaults.plugins.legend.labels.pointStyle = 'circle';

/* 温度刻度: 30-100°C, 步进 5, 共 15 个节点 */
const CURVE_TEMPS = Array.from({ length: 15 }, (_, i) => 30 + i * 5);
/* 实时窗口: 21 个采样点, 约 2.5s 一个 */
const WINDOW_LEN = 21;

/* ---- 可拖动的风扇曲线图 ---- */
function createCurveChart(canvasId, color) {
    return new Chart(document.getElementById(canvasId), {
        type: 'line',
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
            }]
        },
        options: {
            maintainAspectRatio: false,
            cubicInterpolationMode: 'monotone',
            plugins: {
                legend: { display: false }, // 单一系列: 标题即图例
                tooltip: {
                    displayColors: false,
                    callbacks: {
                        title: (items) => `${items[0].label} °C`,
                        label: (item) => `${item.formattedValue} %`,
                    }
                },
                dragData: {
                    round: 0,
                    dragX: false,
                    onDragStart: () => true,
                },
            },
            scales: {
                x: {
                    grid: { display: false },
                    ticks: { maxRotation: 0, callback: (v, i) => (i % 2 === 0 ? `${CURVE_TEMPS[i]}°` : '') },
                },
                y: {
                    min: 0, max: 100,
                    grid: { color: GRID },
                    border: { display: false },
                    ticks: { stepSize: 25, callback: (v) => `${v}%` },
                }
            },
        }
    });
}

/* ---- 实时滚动图(转速 / 温度) ---- */
function createLiveChart(canvasId, { max, cpuLabel, gpuLabel }) {
    return new Chart(document.getElementById(canvasId), {
        type: 'line',
        data: {
            labels: Array.from({ length: WINDOW_LEN }, (_, i) =>
                Math.round((WINDOW_LEN - 1 - i) * 2.5)),
            datasets: [
                { label: cpuLabel, data: Array(WINDOW_LEN).fill(null), borderColor: COLOR_CPU, borderWidth: 2, pointRadius: 0, fill: false },
                { label: gpuLabel, data: Array(WINDOW_LEN).fill(null), borderColor: COLOR_GPU, borderWidth: 2, pointRadius: 0, fill: false },
            ]
        },
        options: {
            maintainAspectRatio: false,
            cubicInterpolationMode: 'monotone',
            interaction: { mode: 'index', intersect: false },
            plugins: {
                legend: { position: 'top', align: 'end' },
            },
            scales: {
                x: {
                    grid: { display: false },
                    ticks: { maxRotation: 0, callback: (v, i) => (i % 4 === 0 ? `-${Math.round((WINDOW_LEN - 1 - i) * 2.5)}s` : '') },
                },
                y: {
                    min: 0, max,
                    grid: { color: GRID },
                    border: { display: false },
                    ticks: { maxTicksLimit: 5 },
                }
            },
        }
    });
}

const leftFanCurve = createCurveChart('leftFanCurve', COLOR_CPU);
const rightFanCurve = createCurveChart('rightFanCurve', COLOR_GPU);
const rpmChart = createLiveChart('rpmChart', { max: 6000, cpuLabel: 'CPU 风扇', gpuLabel: 'GPU 风扇' });
const tempChart = createLiveChart('tempChart', { max: 100, cpuLabel: 'CPU', gpuLabel: 'GPU' });

/* ---- 实时数据推送 ---- */
function pushSample(chart, a, b) {
    chart.data.datasets[0].data.push(a);
    chart.data.datasets[0].data.shift();
    chart.data.datasets[1].data.push(b);
    chart.data.datasets[1].data.shift();
    chart.update('none');
}

function onFanSpeeds({ left_fan_speed, right_fan_speed, left_temp, right_temp }) {
    // 过滤异常值(读取失败为负数)
    if (left_fan_speed < 0 || right_fan_speed < 0 || left_fan_speed > 7000 || right_fan_speed > 7000) return;
    if (left_temp < 0 || right_temp < 0 || left_temp > 100 || right_temp > 100) return;
    pushSample(rpmChart, left_fan_speed, right_fan_speed);
    pushSample(tempChart, left_temp, right_temp);
}

/* ---- 曲线数据 <-> 后端 ---- */
function getFanCurveData() {
    const pick = (chart) => chart.data.labels.map((temp, i) => ({
        temperature: temp,
        speed: chart.data.datasets[0].data[i],
    }));
    return { left_fan: pick(leftFanCurve), right_fan: pick(rightFanCurve) };
}

function applyCurve(chart, points) {
    // 后端配置以温度为键对齐到当前刻度, 兼容旧配置文件
    const byTemp = new Map(points.map((p) => [Math.round(p.temperature), p.speed]));
    chart.data.datasets[0].data = chart.data.labels.map(
        (t, i) => byTemp.get(t) ?? chart.data.datasets[0].data[i]
    );
    chart.update();
}

async function loadConfig() {
    try {
        const fanData = await invoke('load_fan_config');
        applyCurve(leftFanCurve, fanData.left_fan);
        applyCurve(rightFanCurve, fanData.right_fan);
        return true;
    } catch (e) {
        console.warn('加载配置失败:', e);
        return false;
    }
}

/* ---- 控制状态 ---- */
const startStopButton = document.getElementById('startStopButton');
const loadConfigButton = document.getElementById('loadConfigButton');
const saveConfigButton = document.getElementById('saveConfigButton');
const statusPill = document.getElementById('statusPill');
const statusText = document.getElementById('statusText');

let isRunning = false;

function renderRunState() {
    startStopButton.textContent = isRunning ? '停止控制' : '启动控制';
    startStopButton.classList.toggle('btn-danger', isRunning);
    startStopButton.classList.toggle('btn-primary', !isRunning);
    statusPill.classList.toggle('on', isRunning);
    statusText.textContent = isRunning ? '控制运行中' : '未运行';
}

async function startControl() {
    try {
        await invoke('start_fan_control', { fanData: getFanCurveData() });
        isRunning = true;
    } catch (e) {
        console.warn('启动风扇控制失败:', e);
    }
    renderRunState();
}

async function stopControl() {
    try {
        await invoke('stop_fan_control');
        isRunning = false;
    } catch (e) {
        console.warn('停止风扇控制失败:', e);
    }
    renderRunState();
}

startStopButton.addEventListener('click', () => (isRunning ? stopControl() : startControl()));
loadConfigButton.addEventListener('click', loadConfig);

saveConfigButton.addEventListener('click', async () => {
    try {
        await invoke('save_fan_config', { fanData: getFanCurveData() });
        saveConfigButton.textContent = '已保存 ✓';
        setTimeout(() => (saveConfigButton.textContent = '保存配置'), 1200);
    } catch (e) {
        console.warn('保存配置失败:', e);
    }
});

/* ---- 初始化 ---- */
(async () => {
    await listen('get-fan-speeds', (event) => onFanSpeeds(event.payload));
    invoke('get_fan_speeds'); // 让后端开始推送(后端保证只启动一个推送线程)

    // 开机自启动场景: 自动加载配置并开始控制
    try {
        const autostartEnabled = await invoke('plugin:autostart|is_enabled');
        if (autostartEnabled) {
            console.log('自启动模式: 自动加载配置并启动控制');
            // 旧版此处直接连点两个按钮, 配置尚未加载完就开始控制; 现在等待加载完成
            if (await loadConfig()) {
                await startControl();
            }
        }
    } catch (e) {
        console.warn('自启动检查失败:', e);
    }
})();
