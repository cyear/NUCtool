const tauri = window.__TAURI__;

if (!tauri) {
    throw new Error("Tauri global API is unavailable");
}

const { invoke } = tauri.core;
const { listen } = tauri.event;
const { getCurrentWindow } = tauri.window;

const appWindow = getCurrentWindow();

const osd = document.getElementById("osd");
const titleElement = document.getElementById("osd-title");
const subtitleElement = document.getElementById("osd-subtitle");

let hideTimer = null;

// 每次显示 OSD 都会递增。
// 旧的隐藏任务发现版本不一致后，就不会再执行 hide()。
let osdGeneration = 0;

async function hideOsd(generation) {
    osd.classList.remove("visible");

    // 等待 CSS 淡出动画完成
    await new Promise((resolve) => {
        setTimeout(resolve, 250);
    });

    // 如果期间重新显示过 OSD，
    // 当前隐藏任务已经失效，不允许把新的 OSD 隐藏掉。
    if (generation !== osdGeneration) {
        return;
    }

    await appWindow.hide();
}

function showOsd(title, subtitle = "") {
    // 新的一次显示，使之前所有正在等待的 hideOsd() 失效
    osdGeneration++;

    const generation = osdGeneration;

    titleElement.textContent = title ?? "";
    subtitleElement.textContent = subtitle ?? "";

    if (hideTimer !== null) {
        clearTimeout(hideTimer);
        hideTimer = null;
    }

    // 重新显示原生窗口
    void appWindow.show();

    // 强制重新计算布局，确保 CSS 动画重新触发
    void osd.offsetWidth;

    osd.classList.add("visible");

    hideTimer = setTimeout(() => {
        hideTimer = null;

        void hideOsd(generation);
    }, 1000);
}

await listen("osd-show", (event) => {
    const payload = event.payload;

    if (!payload) {
        return;
    }

    showOsd(
        payload.title ?? "",
        payload.subtitle ?? ""
    );
});

await invoke("osd_ready");