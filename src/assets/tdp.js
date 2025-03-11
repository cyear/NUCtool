const gpu1 = document.getElementById("gpu1");
const gpu2 = document.getElementById("gpu2");
const cpu1 = document.getElementById("cpu1");
const cpu2 = document.getElementById("cpu2");
const tcc = document.getElementById("tcc");
// console.log(gpu1, gpu2, tcc);
const a = document.getElementById("a");
const b = document.getElementById('b');
const rInput = document.getElementById("rgb_r");
const gInput = document.getElementById("rgb_g");
const bInput = document.getElementById("rgb_b");
const rVal = document.getElementById("r_val");
const gVal = document.getElementById("g_val");
const bVal = document.getElementById("b_val");
const colorPreview = document.getElementById("colorPreview");
const rgbButton = document.getElementById("rgb_button");
const toggle = document.getElementById("rgb_toggle");

async function get() {
    // console.log(window.__TAURI__);
    const [c1, c2, g1, g2, cc] = await window.__TAURI__.core.invoke('get_tdp');
    console.log(c1, c2, g1, g2, cc);
    // console.log("cpu l1:" + cpu1.value, "\ncpu l2:" + cpu2.value, "\ngpu l1:" + gpu1.value, "\ngpu l2:" + gpu2.value, "\ntcc:" + tcc.value);
    cpu1.value = c1;
    cpu2.value = c2;
    gpu1.value = g1;
    gpu2.value = g2;
    tcc.value = cc;
}

b.addEventListener('click', async () => {
    const t = {
        cpu1: parseInt(cpu1.value),
        cpu2: parseInt(cpu2.value),
        gpu1: parseInt(gpu1.value),
        gpu2: parseInt(gpu2.value),
        tcc: parseInt(tcc.value)
    };
    console.log(t);
    window.__TAURI__.core.invoke('set_tdp', {t});
    // location.reload();
    get();
});
a.addEventListener('click', async () => {
    get();
})

async function updateColorPreview() {
    const r = Math.round(rInput.value * 5.1);
    const g = Math.round(gInput.value * 5.1);
    const b = Math.round(bInput.value * 5.1);
    // 当彩色模式未开启时，预览框显示滑条颜色转换后的值
    if (!toggle.classList.contains('active')) {
        colorPreview.style.backgroundColor = `rgb(${r}, ${g}, ${b})`;
    }
    rVal.textContent = rInput.value;
    gVal.textContent = gInput.value;
    bVal.textContent = bInput.value;
}

[rInput, gInput, bInput].forEach(input => {
    input.addEventListener('input', updateColorPreview);
    input.addEventListener('change', updateColorPreview);
});

// 彩色模式切换
toggle.addEventListener('click', () => {
    toggle.classList.toggle('active');
    const disabled = toggle.classList.contains('active');
    rInput.disabled = disabled;
    gInput.disabled = disabled;
    bInput.disabled = disabled;
    rgbButton.disabled = disabled;
    if (disabled) {
        console.log("打开彩色模式");
        // 彩色模式打开时，预览框显示灰色，滑条和应用按钮锁定为灰色
        rgbButton.style.background = 'gray';
        colorPreview.style.backgroundColor = 'gray';
        window.__TAURI__.core.invoke('set_rgb_color_y');
        // window.__TAURI__.core.invoke('set_rgb', { r: rInput, g: gInput, b: bInput });
    } else {
        console.log("关闭彩色模式");
        window.__TAURI__.core.invoke('set_rgb_color_n');
        rgbButton.style.background = 'red';
        const rgb = window.__TAURI__.core.invoke('get_rgb');
        rVal.value = rgb.r;
        gVal.value = rgb.g;
        bVal.value = rgb.b;
        updateColorPreview();
    }
});

rgbButton.addEventListener('click', async () => {
    if (rgbButton.disabled) return;
    const r = rVal.value;
    const g = gVal.value;
    const b = bVal.value;
    // const r = Math.round(rInput.value * 5.1);
    // const g = Math.round(gInput.value * 5.1);
    // const b = Math.round(bInput.value * 5.1);
    await window.__TAURI__.core.invoke('set_rgb', {r, g, b});
});

document.addEventListener("DOMContentLoaded", async () => {
  console.log("DOMContentLoaded");
  const c = await window.__TAURI__.core.invoke('get_rgb_color');
  console.log(c);
  if (c) {
    // toggle.classList.toggle('active');
    toggle.click();
    toggle.classList.add('active');
  }
  await updateColorPreview();
});
await get();
