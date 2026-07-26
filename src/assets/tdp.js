// NUCtool 调试模式 — 功耗墙 / 键盘 LED
const invoke = window.__TAURI__.core.invoke;

document.addEventListener('contextmenu', (e) => e.preventDefault());

/* ---- 元素 ---- */
const fields = ['cpu1', 'cpu2', 'gpu1', 'gpu2', 'tcc'].map((id) => document.getElementById(id));
const [cpu1, cpu2, gpu1, gpu2, tcc] = fields;
const readTdpBtn = document.getElementById('readTdp');
const applyTdpBtn = document.getElementById('applyTdp');
const tdpMsg = document.getElementById('tdpMsg');

const rInput = document.getElementById('rgb_r');
const gInput = document.getElementById('rgb_g');
const bInput = document.getElementById('rgb_b');
const rVal = document.getElementById('r_val');
const gVal = document.getElementById('g_val');
const bVal = document.getElementById('b_val');
const colorPreview = document.getElementById('colorPreview');
const applyRgbBtn = document.getElementById('applyRgb');
const toggle = document.getElementById('rgbToggle');

function flash(el, text) {
    el.textContent = text;
    setTimeout(() => (el.textContent = ''), 2000);
}

/* ---- TDP ---- */
async function readTdp() {
    try {
        // 后端返回 (cpu1, cpu2, gpu1, gpu2, tcc)
        const [c1, c2, g1, g2, cc] = await invoke('get_tdp');
        cpu1.value = c1;
        cpu2.value = c2;
        gpu1.value = g1;
        gpu2.value = g2;
        tcc.value = cc;
    } catch (e) {
        console.warn('读取 TDP 失败:', e);
        flash(tdpMsg, '读取失败');
    }
}

applyTdpBtn.addEventListener('click', async () => {
    const t = {
        cpu1: parseInt(cpu1.value, 10) || 0,
        cpu2: parseInt(cpu2.value, 10) || 0,
        gpu1: parseInt(gpu1.value, 10) || 0,
        gpu2: parseInt(gpu2.value, 10) || 0,
        tcc: parseInt(tcc.value, 10) || 0,
    };
    applyTdpBtn.disabled = true;
    flash(tdpMsg, '写入中(约 3 秒)...');
    try {
        await invoke('set_tdp', { t });
        flash(tdpMsg, '已应用 ✓');
    } catch (e) {
        console.warn('应用 TDP 失败:', e);
        flash(tdpMsg, '应用失败');
    }
    applyTdpBtn.disabled = false;
    readTdp();
});

readTdpBtn.addEventListener('click', readTdp);

/* ---- 键盘 LED ---- */
function isColorMode() {
    return toggle.classList.contains('on');
}

function renderLed() {
    const on = isColorMode();
    toggle.setAttribute('aria-checked', String(on));
    // 彩色模式下滑条与应用按钮锁定
    [rInput, gInput, bInput].forEach((el) => (el.disabled = on));
    applyRgbBtn.disabled = on;
    if (on) {
        colorPreview.style.background =
            'linear-gradient(135deg, #e34948, #eda100, #1baf7a, #3987e5, #9085e9)';
    } else {
        updatePreview();
    }
}

function updatePreview() {
    // 滑条 0-50 对应标准 RGB 0-255
    const r = Math.round(rInput.value * 5.1);
    const g = Math.round(gInput.value * 5.1);
    const b = Math.round(bInput.value * 5.1);
    if (!isColorMode()) {
        colorPreview.style.background = `rgb(${r}, ${g}, ${b})`;
    }
    rVal.textContent = rInput.value;
    gVal.textContent = gInput.value;
    bVal.textContent = bInput.value;
}

[rInput, gInput, bInput].forEach((el) => el.addEventListener('input', updatePreview));

async function toggleColorMode() {
    const turnOn = !isColorMode();
    toggle.classList.toggle('on', turnOn);
    renderLed();
    try {
        await invoke(turnOn ? 'set_rgb_color_y' : 'set_rgb_color_n');
        if (!turnOn) {
            // 旧版此处缺 await 且把数值写进 <span>.value, 彩色关闭后状态错乱
            const rgb = await invoke('get_rgb');
            rInput.value = rgb.r;
            gInput.value = rgb.g;
            bInput.value = rgb.b;
            updatePreview();
        }
    } catch (e) {
        console.warn('切换彩色模式失败:', e);
    }
}

toggle.addEventListener('click', toggleColorMode);
toggle.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        toggleColorMode();
    }
});

applyRgbBtn.addEventListener('click', async () => {
    if (applyRgbBtn.disabled) return;
    const rgb = {
        r: parseInt(rInput.value, 10),
        g: parseInt(gInput.value, 10),
        b: parseInt(bInput.value, 10),
    };
    try {
        // 旧版参数名不匹配(缺 rgb 包装), 调用必然失败; 后端当前为占位实现
        await invoke('set_rgb', { rgb });
    } catch (e) {
        console.warn('应用 RGB 失败:', e);
    }
});

/* ---- 初始化 ---- */
(async () => {
    updatePreview();
    try {
        const colorOn = await invoke('get_rgb_color');
        toggle.classList.toggle('on', colorOn);
    } catch (e) {
        console.warn('读取彩色模式状态失败:', e);
    }
    renderLed();
    await readTdp();
})();
